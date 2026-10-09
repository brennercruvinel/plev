//! the plev and scripted arms: the workbench served through the agent
//! port, recorded by the flight recorder on the workbench's clock, with
//! the scripted person on the port's side, who may also answer what an
//! agent asks her (`crate::answer`). `finish` tells the run at the end
//! (`crate::finish`).

use std::cell::RefCell;
use std::sync::mpsc::RecvTimeoutError;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use accord::{Millis, Rejection, SemId, Stage};
use record::{Limits, Recorder};
use socket::{AgentPort, Config};

use crate::answer::Answers;
use crate::args::{Args, Arm, Until};
use crate::arms::scripted::{self, Scripted};
use crate::host::{ActSeen, Costs, LabHost, Log, Person, Served};
use crate::repo::TASK;
use crate::report::ERROR;
use crate::subject::{Bench, Wake};

/// how long the loop sleeps between two looks at the timers.
const TICK: Duration = Duration::from_millis(10);

/// the messages the server answered: what a drain serves. `hello` is not
/// among them, so the first message the laboratory sees is the agent's
/// first observation, one round trip after it connected.
fn served_messages(t: &socket::Traffic) -> u64 {
    t.observes_snapshot + t.observes_delta + t.acts
}

pub fn run(args: &Args) -> i32 {
    match Lab::open(args.clone()).and_then(Lab::run) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("agenteval: {e}");
            ERROR
        }
    }
}

/// what the loop timed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Timing {
    pub drain: Duration,
    pub drains: u64,
    /// drains that served at least one message.
    pub drain_busy: Duration,
    pub drains_busy: u64,
    /// messages the drains served (observations and acts, which the
    /// server counts as it answers them).
    pub messages: u64,
    pub snapshot: Duration,
    pub snapshots: u64,
    pub rec_snapshot: Duration,
    pub rec_snapshots: u64,
    pub rec_frame: Duration,
    pub rec_frames: u64,
    pub build: Duration,
    pub builds: u64,
    pub draw: Duration,
    pub draws: u64,
}

/// a stale rejection of `commit.submit`, with HEAD at that instant and
/// who acted on the state it rested on.
pub struct StaleSeen {
    pub t: Millis,
    pub request: accord::RequestId,
    pub head: String,
    pub changes: Vec<accord::Change>,
    pub acted: Vec<accord::Acted>,
}

pub struct Lab {
    pub args: Args,
    pub app: Bench,
    pub rec: Recorder,
    pub log: Log,
    pub port: AgentPort,
    pub person: Person,
    pub served: Vec<Served>,
    pub acts: Vec<ActSeen>,
    pub costs: Costs,
    pub projection_error: RefCell<Option<String>>,
    pub timing: Timing,
    pub started: Instant,
    pub head_before: String,
    pub first_message: Option<Millis>,
    pub submit_completed: Option<Millis>,
    pub stale: Vec<StaleSeen>,
    /// the first state recorded after the intervention.
    pub intervention_recorded_at: Option<Millis>,
    pub frame_error: Option<String>,
    pub scripted: Option<JoinHandle<Result<Scripted, String>>>,
}

impl Lab {
    fn open(args: Args) -> Result<Lab, String> {
        std::fs::create_dir_all(&args.out).map_err(|e| format!("{}: {e}", args.out.display()))?;
        let mut app = Bench::open(true)?;
        let head_before = app.repo.head()?;
        // the whole run must fit the ring: nothing may be evicted.
        let limits = Limits {
            max_age: u64::try_from(args.timeout.as_millis()).unwrap_or(u64::MAX) + 60_000,
            max_bytes: 1 << 30,
            ..Limits::default()
        };
        let rec = Recorder::new(limits);
        let log = Log::default();
        app.bench.add_sink(Box::new(rec.sink()));
        app.bench.add_sink(Box::new(log.clone()));
        let wake = app.tx.clone();
        let port = AgentPort::open_with(Config::new(&args.app), move || {
            let _ = wake.send(Wake::Port);
        })
        .map_err(|e| format!("the agent port: {e}"))?;
        let person = Person::new(
            args.human.clone(),
            Answers::new(args.answer, args.answer_after),
        );
        Ok(Lab {
            args,
            app,
            rec,
            log,
            port,
            person,
            served: Vec::new(),
            acts: Vec::new(),
            costs: Costs::default(),
            projection_error: RefCell::new(None),
            timing: Timing::default(),
            started: Instant::now(),
            head_before,
            first_message: None,
            submit_completed: None,
            stale: Vec::new(),
            intervention_recorded_at: None,
            frame_error: None,
            scripted: None,
        })
    }

    fn run(mut self) -> Result<i32, String> {
        println!(
            "agenteval: arm {}, app {}",
            self.args.arm.name(),
            self.args.app
        );
        println!("task: {TASK}");
        println!("person: {}", self.args.human.describe());
        println!(
            "the person answers: {}, {} ms after a request waits for her",
            self.args.answer.name(),
            self.args.answer_after.as_millis()
        );
        println!("agent port: {}", self.port.path().display());
        println!(
            "until: {}, timeout {} s, grace {} s",
            match self.args.until {
                Until::Commit => "commit",
                Until::Timeout => "timeout",
            },
            self.args.timeout.as_secs_f64(),
            self.args.grace.as_secs_f64()
        );
        self.record();
        if self.args.arm == Arm::Scripted {
            let path = self.port.path().to_owned();
            self.scripted = Some(scripted::spawn(path, self.args.script));
        }
        let ended_by = self.serve();
        self.finish(ended_by)
    }

    /// the loop: wait for a wake, apply the git events, drain the port,
    /// record what changed, let the person answer what waits for her,
    /// watch the receipts, until the end.
    fn serve(&mut self) -> &'static str {
        let deadline = self.started + self.args.timeout;
        let mut ending: Option<(Instant, &'static str)> = None;
        loop {
            let now = Instant::now();
            let wait = TICK.min(deadline.saturating_duration_since(now));
            let wait = self
                .person
                .answers
                .next_due()
                .map_or(wait, |due| wait.min(due.saturating_duration_since(now)));
            let mut changed = false;
            match self.app.rx.recv_timeout(wait) {
                Ok(Wake::Git(event)) => {
                    self.app.bench.apply(event);
                    changed = true;
                }
                Ok(Wake::Port) | Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return "the git worker is gone",
            }
            while let Ok(wake) = self.app.rx.try_recv() {
                if let Wake::Git(event) = wake {
                    self.app.bench.apply(event);
                    changed = true;
                }
            }
            let fired_before = self.person.fired.is_some();
            let acted = self.drain();
            let fired_now = self.person.fired.is_some() && !fired_before;
            if acted || changed || fired_now {
                self.record();
                if fired_now {
                    self.intervention_recorded_at = Some(self.app.bench.now());
                }
            }
            self.answer();
            self.settle();
            if ending.is_none() {
                if self.args.until == Until::Commit && self.submit_completed.is_some() {
                    ending = Some((Instant::now() + self.args.grace, "commit"));
                } else if self.scripted.as_ref().is_some_and(JoinHandle::is_finished)
                    && self.args.until == Until::Commit
                {
                    ending = Some((Instant::now() + self.args.grace, "the scripted agent ended"));
                }
            }
            if let Some((at, why)) = ending
                && Instant::now() >= at
            {
                return why;
            }
            if Instant::now() >= deadline {
                return "timeout";
            }
        }
    }

    /// one drain of the port through the workbench, timed.
    fn drain(&mut self) -> bool {
        let before = served_messages(&self.port.traffic());
        let looked = self.costs.look.get();
        let Lab {
            port,
            app,
            rec,
            person,
            served,
            acts,
            costs,
            projection_error,
            ..
        } = self;
        let mut host = LabHost {
            bench: &mut app.bench,
            rec,
            repo: &app.repo,
            person,
            served,
            acts,
            costs,
            projection_error,
        };
        let at = Instant::now();
        let acted = port.drain(&mut host);
        // the laboratory's look at a submit is not the port's work.
        let took = at
            .elapsed()
            .saturating_sub(self.costs.look.get().saturating_sub(looked));
        self.timing.drain += took;
        self.timing.drains += 1;
        let served = served_messages(&self.port.traffic());
        if served != before {
            self.timing.drain_busy += took;
            self.timing.drains_busy += 1;
            self.timing.messages += served - before;
            if self.first_message.is_none() {
                self.first_message = Some(self.app.bench.now());
            }
        }
        acted
    }

    /// the state as projected now and the frame as drawn now, both
    /// recorded at the workbench's clock.
    pub fn record(&mut self) {
        let now = self.app.bench.now();
        let at = Instant::now();
        let snap = self.app.bench.snapshot();
        self.timing.snapshot += at.elapsed();
        self.timing.snapshots += 1;
        match snap {
            Ok(snap) => {
                let at = Instant::now();
                self.rec.snapshot(now, &snap);
                self.timing.rec_snapshot += at.elapsed();
                self.timing.rec_snapshots += 1;
            }
            Err(errors) => {
                let mut first = self.projection_error.borrow_mut();
                if first.is_none() {
                    *first = Some(errors.to_string());
                }
            }
        }
        let timed = self.app.frame();
        self.timing.build += timed.build;
        self.timing.builds += 1;
        if let Some(draw) = timed.draw {
            self.timing.draw += draw;
            self.timing.draws += 1;
        }
        let at = Instant::now();
        if let Err(e) = self.rec.frame(now, &self.app.compositor) {
            self.frame_error.get_or_insert_with(|| e.to_string());
        }
        self.timing.rec_frame += at.elapsed();
        self.timing.rec_frames += 1;
    }

    /// reads the receipts of `commit.submit`: a stale rejection is noted
    /// with HEAD at that instant, the first completion ends the run.
    fn settle(&mut self) {
        let Ok(submit) = SemId::parse("commit.submit") else {
            return;
        };
        for r in self.log.receipts() {
            if r.request.target != submit {
                continue;
            }
            let Some((stage, t)) = r.stages.last() else {
                continue;
            };
            match stage {
                Stage::Rejected(Rejection::Stale { changes, acted }) => {
                    if !self.stale.iter().any(|s| s.request == r.id) {
                        self.stale.push(StaleSeen {
                            t: *t,
                            request: r.id,
                            head: self.app.repo.head().unwrap_or_default(),
                            changes: changes.clone(),
                            acted: acted.clone(),
                        });
                    }
                }
                Stage::Completed(_) => {
                    self.submit_completed.get_or_insert(*t);
                }
                _ => {}
            }
        }
    }
}
