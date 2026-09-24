//! The Surface page as its one dispatcher runs a request over it
//! (task-69 phases 3a and 3b-ii): `Page`, an `Operable` that projects the
//! todo, the surface and the proposing row, runs the row's capabilities
//! and passes everything else to the todo's own `execute`.
//!
//! The rules propose inside `execute` and the request ends at once. The
//! Claude is asked off the app's thread (`asking`): `propose` answers
//! `Execution::Started` and keeps the request open; `settle`, called from
//! the page's wake, reads what the worker sent, and when the answer
//! arrives shows the surface (`SurfaceHost::transform` in place, by
//! `SemId`, or `open`) and ends the request with `Dispatcher::complete`
//! or `fail`, through the same lending of the todo's dispatcher
//! (`TodoPage::with_dispatcher`) at the page clock's now. A cancel ends
//! the proposal it cancelled in the same lending as the cancel itself.
//!
//! A person's click, the Enter key, an agent through the port and a
//! screen reader take the same path, and the receipt is kept where every
//! other receipt of the page is. The author of the IR is whoever asked
//! (`GenerationPolicy::by`), so a surface an agent proposes binds only
//! what agents may ask for.
//!
//! While the app sleeps (`engine::window::Background`, task-76 item 7) a
//! `propose` by the Claude is held (`asking::Pending`): the request
//! starts and stays open, and the child runs only after the app wakes.
//! `settle` reads nothing meanwhile, so an answer that arrives asleep
//! waits on the channel for the first wake after the app wakes.

use engine::window::Background;
use propose::{Ask, Rules, propose_verified};
use sem::{
    Actor, Execution, Operable, Outcome, Request, RequestId, SemId, Semantics, SnapshotBuilder,
    Value,
};
use surface::{Surface, SurfaceHost};
use uir::{Capabilities, GenerationPolicy, VerifiedIr};

use super::super::app::TodoPage;
use super::asking::{Answer, Pending};
use super::proposing::{Op, Proposer, Proposing, Said, int};

/// The outcome of a proposal shown: the surface and how many nodes it
/// has.
fn result(surface: &SemId, nodes: usize) -> Outcome {
    let mut outcome = Outcome::default();
    outcome
        .result
        .insert("surface".to_owned(), Value::Text(surface.to_string()));
    outcome
        .result
        .insert("nodes".to_owned(), Value::Int(int(nodes)));
    outcome
}

/// The page as its one dispatcher sees it for a request: the todo, the
/// surface and the proposing row, one projection; the row's capabilities
/// run here, everything else in the todo's `execute`.
pub(super) struct Page<'a> {
    pub todo: &'a mut TodoPage,
    pub host: &'a mut SurfaceHost,
    /// The surface the page shows, by id.
    pub id: &'a mut Option<SemId>,
    /// Why no surface opened, when none did.
    pub refused: &'a mut Option<String>,
    pub proposing: &'a mut Proposing,
}

impl Page<'_> {
    /// Runs `req` through the todo's dispatcher over the whole page. A
    /// cancel ends the proposal it cancelled in the same lending.
    pub fn dispatch(self, req: Request) -> RequestId {
        let Page {
            todo,
            host,
            id,
            refused,
            proposing,
        } = self;
        todo.with_dispatcher(|dispatcher, todo, now| {
            let mut page = Page {
                todo,
                host,
                id,
                refused,
                proposing,
            };
            let asked = dispatcher.dispatch(&mut page, req, now);
            if let Some((pending, why)) = page.cancelled() {
                if let Err(e) = dispatcher.fail(pending, why.clone(), now) {
                    log::warn!("surface page: the cancelled proposal did not end: {e}");
                }
                page.refuse(why);
            }
            asked
        })
    }

    /// The proposal someone cancelled, when one was: its request and why
    /// it ends.
    fn cancelled(&self) -> Option<(RequestId, String)> {
        let pending = self.proposing.pending.as_ref()?;
        let by = pending.cancelled.as_ref()?;
        Some((pending.id, format!("cancelled by {by}")))
    }

    /// No proposal is asked any more; the status says `why`.
    fn refuse(&mut self, why: String) {
        self.proposing.said = Some(Said::Refused(why));
        self.proposing.pending = None;
    }

    /// What happened to the proposal the Claude is answering since the
    /// page last looked: a cancel ends it now; the attempt or the seconds
    /// moved; or the answer arrived, the surface opens or takes the form
    /// in place and the request ends `completed` (or `failed`, with the
    /// reason). True when what the page shows moved. Nothing while the
    /// app sleeps: the channel keeps the answer for the wake.
    pub fn settle(&mut self) -> bool {
        if Background::asleep() {
            return false;
        }
        if let Some((pending, why)) = self.cancelled() {
            let ended = self
                .todo
                .with_dispatcher(|d, _, now| d.fail(pending, why.clone(), now));
            if let Err(e) = ended {
                log::warn!("surface page: the cancelled proposal did not end: {e}");
            }
            self.refuse(why);
            return true;
        }
        let Some(pending) = self.proposing.pending.as_mut() else {
            return false;
        };
        let (moved, answer) = pending.poll();
        let Some(answer) = answer else {
            return moved;
        };
        if let Some(pending) = self.proposing.pending.take() {
            self.arrived(pending, answer);
        }
        true
    }

    /// The Claude answered `pending`: the surface shows the proposal and
    /// the request completes with it, the attempts, the seconds and the
    /// cost; or the request fails with why there is none.
    fn arrived(&mut self, pending: Pending, answer: Answer) {
        let Answer {
            proposed,
            attempts,
            took,
            cost_usd,
        } = answer;
        let id = pending.id;
        let shown = proposed
            .map_err(|e| e.to_string())
            .and_then(|ir| self.show(ir));
        let ended = match shown {
            Ok((surface, nodes)) => {
                let mut outcome = result(&surface, nodes);
                outcome
                    .result
                    .insert("attempts".to_owned(), Value::Int(int(attempts)));
                outcome
                    .result
                    .insert("seconds".to_owned(), Value::Num(took.as_secs_f64()));
                if let Some(cost) = cost_usd {
                    outcome
                        .result
                        .insert("cost_usd".to_owned(), Value::Num(cost));
                }
                self.proposing.said = Some(Said::Answered {
                    intent: pending.intent,
                    attempts,
                    took,
                    cost_usd,
                });
                self.proposing.shown = true;
                self.todo
                    .with_dispatcher(|d, _, now| d.complete(id, outcome, now))
            }
            Err(why) => {
                self.proposing.said = Some(Said::Refused(why.clone()));
                self.todo.with_dispatcher(|d, _, now| d.fail(id, why, now))
            }
        };
        if let Err(e) = ended {
            log::warn!("surface page: the proposal's request did not end: {e}");
        }
    }

    /// Shows `ir`: the surface opens, or takes the form in place when its
    /// id is open (`SurfaceHost::transform`, reconciled by `SemId`). Its
    /// id and how many nodes it has.
    fn show(&mut self, ir: VerifiedIr) -> Result<(SemId, usize), String> {
        let app = self.todo.snapshot().map_err(|e| e.to_string())?;
        let open = self.host.get(&ir.ir().surface.id).is_some();
        let surface = if open {
            self.host.transform(ir, &app)
        } else {
            self.host.open(ir, &app)
        };
        let (id, nodes) = surface
            .map(|s| (s.id().clone(), s.nodes().count()))
            .map_err(|e| e.to_string())?;
        *self.id = Some(id.clone());
        *self.refused = None;
        Ok((id, nodes))
    }

    fn refused(&mut self, why: String) -> Execution {
        self.proposing.said = Some(Said::Refused(why.clone()));
        Execution::Failed(why)
    }

    /// A proposal for the intent, as `author` wrote it, over the todo as
    /// it is and the surface shown. By the rules it is verified and shown
    /// now, and the request is done; by the Claude the worker starts,
    /// with everything it needs owned (or is held until the app wakes),
    /// and the request `id` stays open until `settle` ends it.
    fn propose(&mut self, id: RequestId, author: &Actor) -> Execution {
        let intent = self.proposing.intent.text().trim().to_owned();
        let app = match self.todo.snapshot() {
            Ok(app) => app,
            Err(e) => return Execution::Failed(e.to_string()),
        };
        let caps = Capabilities::from_snapshot(&app);
        let policy = GenerationPolicy::by(author.clone());
        let current: Option<VerifiedIr> = self
            .id
            .as_ref()
            .and_then(|id| self.host.get(id))
            .map(Surface::verified)
            .cloned();
        match self.proposing.chosen() {
            Proposer::Rules => {
                let ask = Ask {
                    intent: &intent,
                    caps: &caps,
                    app: &app,
                    current: current.as_ref(),
                    policy: &policy,
                };
                let proposed = propose_verified(&mut Rules, &ask).map_err(|e| e.to_string());
                match proposed.and_then(|ir| self.show(ir)) {
                    Ok((surface, nodes)) => {
                        self.proposing.said = Some(Said::Proposed(intent));
                        self.proposing.shown = true;
                        Execution::Done(result(&surface, nodes))
                    }
                    Err(why) => self.refused(why),
                }
            }
            Proposer::Claude => {
                if self.proposing.waiting() {
                    let why = "a proposal is already being asked of the Claude: wait for it, \
                               or cancel it";
                    return self.refused(why.to_owned());
                }
                let started = self
                    .proposing
                    .claude
                    .start(id, intent, app, caps, current, policy);
                match started {
                    Ok(pending) => {
                        self.proposing.pending = Some(pending);
                        Execution::Started
                    }
                    Err(why) => self.refused(why),
                }
            }
        }
    }
}

impl Semantics for Page<'_> {
    fn project(&self, b: &mut SnapshotBuilder) {
        self.todo.project(b);
        self.host.project(b);
        self.proposing.project(b);
    }
}

impl Operable for Page<'_> {
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        let Some(op) = self.proposing.op(req) else {
            return self.todo.execute(id, req);
        };
        let done = || Execution::Done(Outcome::default());
        match op {
            Op::Write(text) => {
                self.proposing.intent.set_text(&text);
                done()
            }
            Op::Propose => self.propose(id, &req.actor),
            Op::Choose(name) => {
                if self.proposing.proposer.select_label(&name) {
                    done()
                } else {
                    Execution::Failed(format!("no proposer is named {name:?}"))
                }
            }
            Op::ChooseAt(at) => {
                if self.proposing.proposer.select_index(at) {
                    done()
                } else {
                    Execution::Failed(format!("no proposer at {at}"))
                }
            }
            Op::Open => {
                self.proposing.proposer.open();
                done()
            }
            Op::Close => {
                self.proposing.proposer.close();
                done()
            }
            Op::Cancel => match self.proposing.pending.as_mut() {
                Some(pending) => {
                    pending.cancel(req.actor.clone());
                    let mut outcome = Outcome::default();
                    outcome
                        .result
                        .insert("cancelled".to_owned(), Value::Text(pending.id.to_string()));
                    Execution::Done(outcome)
                }
                None => Execution::Failed("nothing is being asked".to_owned()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    //! The Claude asked while the app sleeps (task-76 item 7), against
    //! the fake claude of `claude-model` (`tests/fake-claude`, a shell
    //! script): the request starts and stays open with no child running,
    //! the wakes meanwhile start and settle nothing, and the first wake
    //! after the app wakes runs the child and the answer ends the
    //! request; an answer that arrives while the app sleeps waits on the
    //! channel for that wake. The desktop's: there is no child elsewhere.

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    mod asleep {

        use std::path::PathBuf;
        use std::thread;
        use std::time::{Duration, Instant};

        use claude_model::ClaudeCli;
        use engine::window::Background;
        use sem::{Stage, Value};
        use serde_json::json;
        use tempfile::TempDir;

        use crate::view::apps::app::TodoPage;
        use crate::view::apps::surface_page::SurfacePage;
        use crate::view::apps::surface_page::asking::Claude;

        /// One turn of a 60 Hz display.
        const TURN: f32 = 1.0 / 60.0;
        const INTENT: &str = "the items as a list with the add field on top";

        fn manifest(rel: &str) -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
        }

        /// The fake answering the two-columns form of the todo's surface at
        /// once, and a `ClaudeCli` that runs it in a directory of its own.
        fn fake() -> (TempDir, ClaudeCli) {
            let dir = tempfile::tempdir().unwrap();
            let columns = manifest("../surface/tests/fixtures/showcase-todo-columns.ir.json");
            let columns = std::fs::read_to_string(&columns)
                .unwrap_or_else(|e| panic!("{}: {e}", columns.display()));
            let good = json!({
                "type": "result", "subtype": "success", "is_error": false,
                "num_turns": 1, "total_cost_usd": 0.05, "result": columns
            })
            .to_string();
            std::fs::write(dir.path().join("mode"), "ok").unwrap();
            std::fs::write(dir.path().join("good"), good).unwrap();
            let mut cli = ClaudeCli::new();
            cli.program = manifest("../claude-model/tests/fake-claude");
            cli.dir = dir.path().to_owned();
            (dir, cli)
        }

        /// The page over the fake, its intent typed, the Claude chosen.
        fn setup() -> (TodoPage, SurfacePage, TempDir) {
            let todo = TodoPage::new();
            let mut page = SurfacePage::new(&todo);
            let (dir, cli) = fake();
            page.proposing.set_claude(Claude::with(cli));
            page.proposing.intent.set_text(INTENT);
            (todo, page, dir)
        }

        fn stage(todo: &TodoPage, id: sem::RequestId) -> Stage {
            todo.receipt(id)
                .expect("kept")
                .stage()
                .expect("a stage")
                .clone()
        }

        /// Wakes the page until the request `id` ends, within `max`.
        fn pump(page: &mut SurfacePage, todo: &mut TodoPage, id: sem::RequestId, max: Duration) {
            let started = Instant::now();
            while matches!(stage(todo, id), Stage::Executing) {
                assert!(started.elapsed() < max, "the fake answers within {max:?}");
                thread::sleep(Duration::from_millis(25));
                page.wake(TURN, todo);
            }
        }

        #[test]
        fn a_propose_asked_while_the_app_sleeps_is_held_and_runs_on_the_wake() {
            let (mut todo, mut page, dir) = setup();
            Background::sleeping(true);
            let asked = page.propose(&mut todo).expect("dispatched");
            assert_eq!(stage(&todo, asked), Stage::Executing, "started, open");
            assert!(page.proposing.waiting());
            assert!(
                page.proposing.pending.as_ref().is_some_and(|p| p.held()),
                "held: no child"
            );
            // the loop wakes for other reasons: nothing starts, nothing moves
            for _ in 0..5 {
                let wake = page.wake(TURN, &mut todo);
                assert!(!wake.now, "asleep: nothing moved");
                thread::sleep(Duration::from_millis(20));
            }
            assert!(
                !dir.path().join("prompt.1").exists(),
                "the child never ran asleep"
            );
            assert_eq!(stage(&todo, asked), Stage::Executing);

            // the app wakes: the first wake starts the child, the answer ends
            // the request and shows the surface
            Background::sleeping(false);
            page.wake(TURN, &mut todo);
            assert!(
                page.proposing.pending.as_ref().is_some_and(|p| !p.held()),
                "started on the wake"
            );
            pump(&mut page, &mut todo, asked, Duration::from_secs(10));
            let Stage::Completed(outcome) = stage(&todo, asked) else {
                panic!("the proposal completes: {:?}", stage(&todo, asked));
            };
            assert_eq!(outcome.result.get("attempts"), Some(&Value::Int(1)));
            assert!(
                dir.path().join("prompt.1").exists(),
                "one call, after the wake"
            );
            assert!(page.proposing.shown);
            assert!(!page.proposing.waiting());
        }

        #[test]
        fn an_answer_that_arrives_while_the_app_sleeps_waits_on_the_channel_for_the_wake() {
            let (mut todo, mut page, dir) = setup();
            let asked = page.propose(&mut todo).expect("dispatched");
            assert!(
                page.proposing.pending.as_ref().is_some_and(|p| !p.held()),
                "awake: the child runs"
            );
            // the app sleeps before the answer is read; the fake answers
            // meanwhile (its prompt is written as it runs)
            Background::sleeping(true);
            let started = Instant::now();
            while !dir.path().join("prompt.1").exists() {
                assert!(started.elapsed() < Duration::from_secs(10), "the child ran");
                thread::sleep(Duration::from_millis(20));
            }
            thread::sleep(Duration::from_millis(200));
            for _ in 0..3 {
                let wake = page.wake(TURN, &mut todo);
                assert!(!wake.now, "asleep: the answer waits on the channel");
                assert_eq!(stage(&todo, asked), Stage::Executing);
            }
            assert!(!page.proposing.shown, "the surface did not change asleep");

            Background::sleeping(false);
            pump(&mut page, &mut todo, asked, Duration::from_secs(10));
            assert!(matches!(stage(&todo, asked), Stage::Completed(_)));
            assert!(page.proposing.shown, "shown on the wake");
        }
    }
}
