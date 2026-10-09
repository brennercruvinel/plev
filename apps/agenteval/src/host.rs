//! the workbench as the port serves it, with the laboratory listening:
//! every observation served is recorded, the scripted person acts right
//! after one or right before an agent's submit reaches the dispatcher (by
//! the scenario), a request of an agent that waits for a person is handed
//! to her to answer (`crate::answer`), every projection and dispatch is
//! timed, and a sink keeps every receipt as it stands.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::rc::Rc;
use std::time::{Duration, Instant};

use idestudio::capability::Workbench;
use record::{InputKind, Recorder};
use accord::{
    Actor, Millis, Receipt, ReceiptSink, Request, RequestId, Revision, SemId, Snapshot, Stage,
    StateRef, Value,
};
use socket::Host;

use crate::answer::Answers;
use crate::args::Human;
use crate::repo::{Repo, TASK_FILES};

/// one observation the port served.
#[derive(Clone, Debug)]
pub struct Served {
    pub t: Millis,
    pub actor: Actor,
    pub scope: Option<SemId>,
    pub revision: Revision,
}

/// an agent's act as the port dispatched it: which request, how many
/// observations had been served by then (the order of the story), and,
/// for a `submit`, the `changes:staged` the app held right before the
/// dispatcher took it, with the revision of that projection.
#[derive(Clone, Debug)]
pub struct ActSeen {
    pub request: RequestId,
    pub served_before: usize,
    pub live: Option<(Revision, BTreeSet<String>)>,
}

/// what the person acted right after (`unstage-after-observe`) or right
/// before (`unstage-before-submit`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// an observation just served to the agent.
    Observation,
    /// an agent's submit, about to reach the dispatcher.
    Submit,
}

impl Trigger {
    pub fn name(self) -> &'static str {
        match self {
            Trigger::Observation => "observation served",
            Trigger::Submit => "submit arriving",
        }
    }
}

/// the scripted person's one act, once it happened.
#[derive(Clone, Debug)]
pub struct Intervention {
    /// when, on the workbench's clock.
    pub t: Millis,
    pub trigger: Trigger,
    /// the index in the served observations of the last one served: the
    /// one that triggered it, after an observation. as information: the
    /// stale measurement is by value.
    pub served_index: usize,
    /// the path taken out of the index.
    pub path: String,
    /// the revision the agent had just been served, after an observation;
    /// the revision the arriving submit named, before a submit.
    pub revision_served: Revision,
    /// `git rev-parse HEAD` at that instant.
    pub head: String,
    /// the person's request through the dispatcher.
    pub request: RequestId,
    /// true when the live projection the person acted on had
    /// `revision_served`. after an observation, false would mean something
    /// moved in between, which one thread cannot do; before a submit, it
    /// means the submit named a revision the app had since left.
    pub revision_matched: bool,
    /// `changes:staged` right before and right after the person's act.
    pub staged_before: BTreeSet<String>,
    pub staged_after: BTreeSet<String>,
}

/// the scripted person: waits for the condition, acts once; and answers
/// what agents ask her (`answers`).
pub struct Person {
    pub scenario: Human,
    pub fired: Option<Intervention>,
    pub answers: Answers,
}

impl Person {
    pub fn new(scenario: Human, answers: Answers) -> Self {
        Person {
            scenario,
            fired: None,
            answers,
        }
    }

    /// true while this person may still act on `trigger`: the scenario
    /// acts on it, and has not acted yet. asked before the workbench is
    /// projected, so a run with no person pays nothing for having one.
    fn waiting(&self, trigger: Trigger) -> bool {
        let on = match self.scenario {
            Human::None => None,
            Human::UnstageAfterObserve(_) => Some(Trigger::Observation),
            Human::UnstageBeforeSubmit(_) => Some(Trigger::Submit),
        };
        on == Some(trigger) && self.fired.is_none()
    }

    /// the path to take out when the live state shows the task ready: the
    /// path is staged, every file of the task is staged, and the message
    /// is written (the submit's precondition would pass).
    fn ready(&self, seen: &Snapshot) -> Option<&str> {
        let (Human::UnstageAfterObserve(path) | Human::UnstageBeforeSubmit(path)) = &self.scenario
        else {
            return None;
        };
        if self.fired.is_some() {
            return None;
        }
        let staged = match seen.value(&StateRef::parse("changes:staged").ok()?) {
            Some(Value::Set(staged)) => staged,
            _ => return None,
        };
        let all = TASK_FILES.iter().all(|f| staged.contains(*f));
        if !staged.contains(path.as_str()) || !all {
            return None;
        }
        match seen.value(&StateRef::parse("commit.message:text").ok()?) {
            Some(Value::Text(text)) if !text.trim().is_empty() => Some(path),
            _ => None,
        }
    }
}

/// what the laboratory timed on the app's thread while serving the port.
/// cells, because `Host::project` takes `&self`.
#[derive(Clone, Debug, Default)]
pub struct Costs {
    /// time inside `bench.snapshot()` when the port asked (`project`, and
    /// the person's look at the live state).
    pub project: Cell<Duration>,
    pub project_calls: Cell<u64>,
    /// time inside `bench.dispatch()` when the port asked.
    pub dispatch: Cell<Duration>,
    pub dispatch_calls: Cell<u64>,
    /// time inside the laboratory's own look at the live state when a
    /// `submit` arrives, for the stale measurement: not the port's work,
    /// so the loop leaves it out of the time inside `drain`.
    pub look: Cell<Duration>,
    pub look_calls: Cell<u64>,
}

impl Costs {
    fn timed<T>(cell: &Cell<Duration>, calls: &Cell<u64>, f: impl FnOnce() -> T) -> T {
        let at = Instant::now();
        let out = f();
        cell.set(cell.get() + at.elapsed());
        calls.set(calls.get() + 1);
        out
    }

    pub fn project<T>(&self, f: impl FnOnce() -> T) -> T {
        Costs::timed(&self.project, &self.project_calls, f)
    }

    pub fn dispatch<T>(&self, f: impl FnOnce() -> T) -> T {
        Costs::timed(&self.dispatch, &self.dispatch_calls, f)
    }

    pub fn look<T>(&self, f: impl FnOnce() -> T) -> T {
        Costs::timed(&self.look, &self.look_calls, f)
    }
}

/// every receipt the dispatcher opened, as it stood at its last stage, in
/// the order they started; shared with the loop through `Rc`.
#[derive(Clone, Default)]
pub struct Log(pub Rc<RefCell<Vec<Receipt>>>);

impl ReceiptSink for Log {
    fn on_stage(&mut self, receipt: &Receipt, _stage: &Stage) {
        let mut all = self.0.borrow_mut();
        match all.iter_mut().find(|r| r.id == receipt.id) {
            Some(r) => *r = receipt.clone(),
            None => all.push(receipt.clone()),
        }
    }
}

impl Log {
    pub fn receipts(&self) -> Vec<Receipt> {
        self.0.borrow().clone()
    }
}

/// the workbench served through the port for one `drain`.
pub struct LabHost<'a> {
    pub bench: &'a mut Workbench,
    pub rec: &'a Recorder,
    pub repo: &'a Repo,
    pub person: &'a mut Person,
    pub served: &'a mut Vec<Served>,
    pub acts: &'a mut Vec<ActSeen>,
    pub costs: &'a Costs,
    /// a projection that did not validate, once, for the report.
    pub projection_error: &'a RefCell<Option<String>>,
}

impl LabHost<'_> {
    fn snapshot(&self) -> Option<Snapshot> {
        match self.costs.project(|| self.bench.snapshot()) {
            Ok(snap) => Some(snap),
            Err(errors) => {
                let mut first = self.projection_error.borrow_mut();
                if first.is_none() {
                    *first = Some(errors.to_string());
                }
                None
            }
        }
    }

    /// the scripted person acts, right after an observation was served or
    /// right before a submit reaches the dispatcher: `unstage` on the
    /// path, as `Actor::Human`, through the same dispatcher, recorded as
    /// an input on the file's control.
    fn intervene(&mut self, trigger: Trigger, revision_served: Revision) {
        if !self.person.waiting(trigger) {
            return;
        }
        let Some(live) = self.snapshot() else {
            return;
        };
        let Some(path) = self.person.ready(&live).map(str::to_owned) else {
            return;
        };
        let Ok(target) = SemId::parse(&format!("changes.file[{path}]")) else {
            return;
        };
        let req = match Request::observe(&live, Actor::Human, &target, "unstage", None) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("ide-lab: the person cannot unstage {path}: {e}");
                return;
            }
        };
        let head = self.repo.head().unwrap_or_default();
        let now = self.bench.now();
        self.rec
            .input(now, InputKind::PointerDown, None, Some(target));
        let request = self.bench.dispatch(req);
        let staged_after = self.snapshot().map(|s| staged_of(&s)).unwrap_or_default();
        self.person.fired = Some(Intervention {
            t: now,
            trigger,
            served_index: self.served.len().saturating_sub(1),
            path,
            revision_served,
            head,
            request,
            revision_matched: live.revision() == revision_served,
            staged_before: staged_of(&live),
            staged_after,
        });
    }
}

impl Host for LabHost<'_> {
    fn project(&self) -> Option<Snapshot> {
        self.snapshot()
    }

    fn dispatch(&mut self, req: Request) -> Option<RequestId> {
        let submit = is_submit(&req);
        if submit {
            self.intervene(Trigger::Submit, req.observed.revision);
        }
        // the value the submit will be checked against: the dispatcher
        // projects the app again to validate it, and nothing runs on this
        // thread in between, so the revisions agree (the story checks).
        let live = if submit {
            match self.costs.look(|| self.bench.snapshot()) {
                Ok(snap) => Some((snap.revision(), staged_of(&snap))),
                Err(_) => None,
            }
        } else {
            None
        };
        let bench = &mut *self.bench;
        let request = self.costs.dispatch(|| bench.dispatch(req));
        // an agent's question, or an action that asks a person: the
        // dispatcher holds it for her, and the modal asks it.
        let waits = bench.receipt(request).is_some_and(|r| {
            r.stage() == Some(&Stage::AwaitingConfirmation)
                && matches!(r.request.actor, Actor::Agent { .. })
        });
        if waits {
            self.person.answers.waits(request, bench.now());
        }
        self.acts.push(ActSeen {
            request,
            served_before: self.served.len(),
            live,
        });
        Some(request)
    }

    fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        self.bench.add_sink(sink);
    }

    fn observed(&mut self, actor: &Actor, scope: Option<&SemId>, revision: Revision) {
        let now = self.bench.now();
        self.rec
            .observed(now, actor.clone(), scope.cloned(), revision);
        self.served.push(Served {
            t: now,
            actor: actor.clone(),
            scope: scope.cloned(),
            revision,
        });
        self.intervene(Trigger::Observation, revision);
    }
}

/// an agent's `submit` of the commit (every request the port dispatches
/// is an agent's).
fn is_submit(req: &Request) -> bool {
    req.target.as_str() == "commit.submit" && req.action.as_str() == "submit"
}

/// the staged set a snapshot projects, for the report.
pub fn staged_of(seen: &Snapshot) -> BTreeSet<String> {
    match StateRef::parse("changes:staged")
        .ok()
        .and_then(|at| seen.value(&at).cloned())
    {
        Some(Value::Set(staged)) => staged,
        _ => BTreeSet::new(),
    }
}
