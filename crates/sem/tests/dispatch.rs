//! `Dispatcher` with an in-memory git client: the thesis scenario,
//! confirmation, async execution, rejection at the point of truth, illegal
//! transitions, sinks and retention.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use indexmap::IndexMap;
use sem::{
    Actor, Change, Confirm, Constraints, Dispatcher, Execution, Operable, Outcome, ParamSpec,
    Policy, Receipt, ReceiptSink, Rejection, Request, RequestId, Role, SemError, Semantics,
    Snapshot, SnapshotBuilder, Stage, StateRef, Value, ValueKind, Verdict, check,
};

fn at(text: &str) -> StateRef {
    StateRef::parse(text).expect("valid state ref")
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn agent() -> Actor {
    Actor::Agent {
        name: "reviewer".into(),
    }
}

fn text(param: Option<&Value>) -> String {
    match param {
        Some(Value::Text(t)) => t.clone(),
        _ => String::new(),
    }
}

/// a git client in memory. `staged` is what the app shows; `index` is the
/// point of truth (the git index), which a tool outside the app can move
/// without the app knowing.
struct Git {
    staged: BTreeSet<String>,
    index: BTreeSet<String>,
    message: String,
    cursor: i64,
    commits: Vec<(String, Vec<String>)>,
    /// `submit` starts the commit on a worker and returns `Started`.
    background: bool,
    /// requests the worker has not run yet.
    queue: Vec<(RequestId, Request)>,
    /// the worker fails with this error.
    worker_error: Option<String>,
    /// every action `execute` ran, in order.
    executed: Vec<String>,
    /// projects an invalid id, so the projection does not validate.
    broken: bool,
}

impl Git {
    fn new(files: &[&str], message: &str) -> Self {
        Git {
            staged: set(files),
            index: set(files),
            message: message.to_owned(),
            cursor: 0,
            commits: Vec::new(),
            background: false,
            queue: Vec::new(),
            worker_error: None,
            executed: Vec::new(),
            broken: false,
        }
    }

    fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish().unwrap()
    }

    /// what `staged` would be if it showed the index: the state as the
    /// point of truth sees it.
    fn truth(&self) -> Snapshot {
        let view = Git {
            staged: self.index.clone(),
            broken: false,
            ..Git::new(&[], &self.message)
        };
        view.snapshot()
    }

    /// the commit itself, once every check passed.
    fn commit(&mut self) -> Outcome {
        let files: Vec<String> = self.index.iter().cloned().collect();
        let id = format!("c{}", self.commits.len() + 1);
        self.commits.push((id.clone(), files.clone()));
        self.staged.clear();
        self.index.clear();
        Outcome {
            result: IndexMap::from([
                ("commit".to_owned(), Value::Text(id)),
                ("files".to_owned(), Value::List(files)),
            ]),
        }
    }

    /// the worker runs the oldest queued commit: the point of truth checks
    /// the request against the index, then commits. it answers with the id
    /// `execute` received, the one the dispatcher is waiting on.
    fn work(&mut self) -> (RequestId, Execution) {
        let (id, req) = self.queue.remove(0);
        if let Some(error) = self.worker_error.clone() {
            return (id, Execution::Failed(error));
        }
        let done = match check(&req, &self.truth()) {
            Verdict::Rejected(why) => Execution::Rejected(why),
            _ => Execution::Done(self.commit()),
        };
        (id, done)
    }
}

impl Semantics for Git {
    fn project(&self, b: &mut SnapshotBuilder) {
        let files: BTreeSet<String> = self.staged.clone();
        let mut changes = b.node("changes", Role::List, "changes");
        changes.state("staged", files).state("cursor", self.cursor);
        for name in ["stage", "unstage"] {
            changes
                .action(name)
                .reads("changes:staged")
                .param(ParamSpec {
                    kind: ValueKind::Text,
                    constrained: false,
                });
        }
        changes.action("move").param(ParamSpec {
            kind: ValueKind::Int,
            constrained: false,
        });
        changes
            .action("discard")
            .label("discard every staged change")
            .reads("changes:staged")
            .policy(Policy {
                confirm: Confirm::ForAgents,
                destructive: true,
                ..Policy::default()
            });
        b.node("commit.message", Role::TextInput, "commit message")
            .state("text", self.message.as_str())
            .constraints(Constraints {
                editable: true,
                required: true,
                max_len: Some(72),
                ..Constraints::default()
            })
            .action("set")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            });
        b.node("commit.submit", Role::Button, "commit")
            .enabled(!self.staged.is_empty())
            .action("submit")
            .reads("changes:staged")
            .reads("commit.message:text");
        if self.broken {
            b.node("Broken", Role::Group, "");
        }
    }
}

impl Operable for Git {
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        self.executed.push(req.action.to_string());
        let files = |git: &Git| Outcome {
            result: IndexMap::from([("staged".to_owned(), Value::Set(git.staged.clone()))]),
        };
        match req.action.as_str() {
            "stage" => {
                let file = text(req.param.as_ref());
                self.staged.insert(file.clone());
                self.index.insert(file);
                Execution::Done(files(self))
            }
            "unstage" => {
                let file = text(req.param.as_ref());
                self.staged.remove(&file);
                self.index.remove(&file);
                Execution::Done(files(self))
            }
            "move" => {
                if let Some(Value::Int(n)) = req.param {
                    self.cursor = n;
                }
                Execution::Done(Outcome::default())
            }
            "discard" => {
                let gone: Vec<String> = self.staged.iter().cloned().collect();
                self.staged.clear();
                self.index.clear();
                Execution::Done(Outcome {
                    result: IndexMap::from([("discarded".to_owned(), Value::List(gone))]),
                })
            }
            "set" => {
                self.message = text(req.param.as_ref());
                Execution::Done(Outcome::default())
            }
            "submit" if self.background => {
                self.queue.push((id, req.clone()));
                Execution::Started
            }
            "submit" => match check(req, &self.truth()) {
                Verdict::Rejected(why) => Execution::Rejected(why),
                _ => Execution::Done(self.commit()),
            },
            other => Execution::Failed(format!("no action {other}")),
        }
    }
}

/// ends a started execution the way the app's event loop would, with the
/// id the worker got from `execute`.
fn settle(d: &mut Dispatcher, answer: (RequestId, Execution), now: u64) -> Result<(), SemError> {
    let (id, done) = answer;
    match done {
        Execution::Done(outcome) => d.complete(id, outcome, now),
        Execution::Failed(error) => d.fail(id, error, now),
        Execution::Rejected(why) => d.reject_at_truth(id, why, now),
        Execution::Started => Ok(()),
    }
}

fn stages(receipt: &Receipt) -> Vec<&str> {
    receipt.stages.iter().map(|(s, _)| s.name()).collect()
}

fn receipt(d: &Dispatcher, id: RequestId) -> &Receipt {
    d.receipt(id).expect("the dispatcher keeps the receipt")
}

fn unstage(d: &mut Dispatcher, git: &mut Git, file: &str, now: u64) -> RequestId {
    let seen = git.snapshot();
    let req =
        Request::observe(&seen, Actor::Human, "changes", "unstage", Some(file.into())).unwrap();
    d.dispatch(git, req, now)
}

#[test]
fn the_agent_commit_is_rejected_stale_after_the_human_unstages_b() {
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    let mut d = Dispatcher::new();
    let seen_by_agent = git.snapshot();
    let commit =
        Request::observe(&seen_by_agent, agent(), "commit.submit", "submit", None).unwrap();

    let human = unstage(&mut d, &mut git, "B", 10);
    assert_eq!(
        stages(receipt(&d, human)),
        ["requested", "accepted", "executing", "completed"]
    );
    assert_eq!(receipt(&d, human).request.actor, Actor::Human);

    let id = d.dispatch(&mut git, commit, 20);
    let r = receipt(&d, id);
    assert_eq!(stages(r), ["requested", "rejected"]);
    let Some(Stage::Rejected(Rejection::Stale { changes })) = r.stage() else {
        panic!("expected stale, got {:?}", r.stage());
    };
    assert_eq!(
        changes,
        &[Change::StateChanged {
            at: at("changes:staged"),
            before: Some(Value::Set(set(&["A", "B", "C"]))),
            after: Some(Value::Set(set(&["A", "C"]))),
            added: vec![],
            removed: strings(&["B"]),
        }]
    );
    assert_eq!(r.request.observed.revision, seen_by_agent.revision());
    assert_eq!(r.validated_against, Some(git.snapshot().revision()));
    assert_eq!(
        r.stages,
        [(Stage::Requested, 20), (r.stage().unwrap().clone(), 20)]
    );
    assert!(git.commits.is_empty());
    assert_eq!(git.executed, ["unstage"]);
}

#[test]
fn the_agent_observes_again_and_its_commit_completes_with_commit_and_files() {
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    let mut d = Dispatcher::new();
    let stale =
        Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    unstage(&mut d, &mut git, "B", 10);
    let refused = d.dispatch(&mut git, stale, 20);
    assert!(matches!(
        receipt(&d, refused).stage(),
        Some(Stage::Rejected(Rejection::Stale { .. }))
    ));

    let seen = git.snapshot();
    let again = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
    let id = d.dispatch(&mut git, again, 30);
    let r = receipt(&d, id);
    assert_eq!(
        stages(r),
        ["requested", "accepted", "executing", "completed"]
    );
    assert_eq!(r.validated_against, Some(seen.revision()));
    let Some(Stage::Completed(outcome)) = r.stage() else {
        panic!("expected completed");
    };
    assert_eq!(
        outcome.result.get("commit"),
        Some(&Value::Text("c1".into()))
    );
    assert_eq!(
        outcome.result.get("files"),
        Some(&Value::List(strings(&["A", "C"])))
    );
    assert_eq!(git.commits, [("c1".to_owned(), strings(&["A", "C"]))]);
}

#[test]
fn a_moved_cursor_does_not_invalidate_the_agent_commit() {
    let mut git = Git::new(&["A", "B"], "fix login");
    let mut d = Dispatcher::new();
    let commit =
        Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    let seen = git.snapshot();
    let moved = Request::observe(&seen, Actor::Human, "changes", "move", Some(Value::Int(1)));
    let moved = d.dispatch(&mut git, moved.unwrap(), 5);
    assert!(receipt(&d, moved).is_finished());
    assert_eq!(git.cursor, 1);
    assert_ne!(git.snapshot().revision(), commit.observed.revision);

    let id = d.dispatch(&mut git, commit, 6);
    assert!(matches!(receipt(&d, id).stage(), Some(Stage::Completed(_))));
    assert_eq!(git.commits.len(), 1);
}

#[test]
fn a_rejected_request_never_reaches_execute() {
    let mut git = Git::new(&["A"], "fix login");
    let mut d = Dispatcher::new();
    let seen = git.snapshot();
    let long = "x".repeat(73);
    let req = Request::observe(
        &seen,
        Actor::Human,
        "commit.message",
        "set",
        Some(long.into()),
    )
    .unwrap();
    let id = d.dispatch(&mut git, req, 1);
    assert!(matches!(
        receipt(&d, id).stage(),
        Some(Stage::Rejected(Rejection::InvalidParam { .. }))
    ));
    assert!(git.executed.is_empty());
    assert_eq!(git.message, "fix login");
}

/// an agent's discard, waiting for a person.
fn awaiting_discard(git: &mut Git, d: &mut Dispatcher) -> RequestId {
    let req = Request::observe(&git.snapshot(), agent(), "changes", "discard", None).unwrap();
    let id = d.dispatch(git, req, 1);
    assert_eq!(
        stages(receipt(d, id)),
        ["requested", "awaiting_confirmation"]
    );
    id
}

#[test]
fn an_agent_discard_waits_for_a_person_to_confirm() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let id = awaiting_discard(&mut git, &mut d);
    assert!(git.executed.is_empty());

    d.confirm(&mut git, id, Actor::Human, 3).unwrap();
    let r = receipt(&d, id);
    assert_eq!(
        stages(r),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    assert_eq!(r.stages[2], (Stage::Confirmed { by: Actor::Human }, 3));
    assert_eq!(git.executed, ["discard"]);
    assert!(git.staged.is_empty());
}

#[test]
fn a_person_confirming_through_a_screen_reader_confirms() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let id = awaiting_discard(&mut git, &mut d);
    d.confirm(&mut git, id, Actor::Assistive, 4).unwrap();
    let r = receipt(&d, id);
    assert_eq!(
        stages(r),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    assert_eq!(
        r.stages[2],
        (
            Stage::Confirmed {
                by: Actor::Assistive
            },
            4
        )
    );
    assert_eq!(git.executed, ["discard"]);
}

#[test]
fn an_agent_or_a_test_cannot_confirm_and_the_receipt_keeps_waiting() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let id = awaiting_discard(&mut git, &mut d);
    let tester = Actor::Test {
        name: "dispatch".into(),
    };
    for machine in [agent(), tester] {
        assert_eq!(
            d.confirm(&mut git, id, machine.clone(), 2),
            Err(SemError::ConfirmByMachine { id, by: machine })
        );
    }
    assert_eq!(
        stages(receipt(&d, id)),
        ["requested", "awaiting_confirmation"]
    );
    assert!(git.executed.is_empty());
    assert_eq!(
        SemError::ConfirmByMachine { id, by: agent() }.to_string(),
        format!("request {id} waits for a person; agent:reviewer cannot confirm it")
    );
}

#[test]
fn a_human_discard_needs_no_confirmation() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::new();
    let req = Request::observe(&git.snapshot(), Actor::Human, "changes", "discard", None).unwrap();
    let id = d.dispatch(&mut git, req, 1);
    assert_eq!(
        stages(receipt(&d, id)),
        ["requested", "accepted", "executing", "completed"]
    );
}

#[test]
fn a_confirmation_checks_the_live_state_again() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let req = Request::observe(&git.snapshot(), agent(), "changes", "discard", None).unwrap();
    let id = d.dispatch(&mut git, req, 1);

    let seen = git.snapshot();
    let stage_d =
        Request::observe(&seen, Actor::Human, "changes", "stage", Some("D".into())).unwrap();
    d.dispatch(&mut git, stage_d, 2);

    d.confirm(&mut git, id, Actor::Human, 3).unwrap();
    let r = receipt(&d, id);
    assert_eq!(
        stages(r),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert!(matches!(
        r.stage(),
        Some(Stage::Rejected(Rejection::Stale { changes }))
            if matches!(&changes[..], [Change::StateChanged { added, .. }] if *added == strings(&["D"]))
    ));
    assert_eq!(r.validated_against, Some(git.snapshot().revision()));
    assert_eq!(git.executed, ["stage"]);
    assert_eq!(git.staged, set(&["A", "B", "D"]));
}

#[test]
fn an_async_commit_is_executing_until_the_app_completes_it() {
    let mut git = Git::new(&["A"], "fix login");
    git.background = true;
    let mut d = Dispatcher::new();
    let req = Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    let id = d.dispatch(&mut git, req, 1);
    let r = receipt(&d, id);
    assert_eq!(stages(r), ["requested", "accepted", "executing"]);
    assert!(!r.is_finished());
    assert!(git.commits.is_empty());

    let answer = git.work();
    assert_eq!(answer.0, id, "execute received the id dispatch returned");
    settle(&mut d, answer, 50).unwrap();
    let r = receipt(&d, id);
    assert_eq!(
        stages(r),
        ["requested", "accepted", "executing", "completed"]
    );
    assert_eq!(r.stages.last().map(|(_, t)| *t), Some(50));
    assert_eq!(git.commits.len(), 1);
}

#[test]
fn an_async_commit_can_fail() {
    let mut git = Git::new(&["A"], "fix login");
    git.background = true;
    git.worker_error = Some("index.lock exists".into());
    let mut d = Dispatcher::new();
    let req = Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    let id = d.dispatch(&mut git, req, 1);
    let answer = git.work();
    assert_eq!(answer.0, id);
    settle(&mut d, answer, 9).unwrap();
    assert_eq!(
        receipt(&d, id).stage(),
        Some(&Stage::Failed {
            error: "index.lock exists".into()
        })
    );
    assert!(git.commits.is_empty());
}

#[test]
fn the_point_of_truth_rejects_a_commit_whose_index_moved() {
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    let mut d = Dispatcher::new();
    let req = Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    // a terminal outside the app unstages C: the index moved, the app did
    // not see it, so the dispatcher accepts and the point of truth refuses.
    git.index.remove("C");
    let id = d.dispatch(&mut git, req, 7);
    let r = receipt(&d, id);
    assert_eq!(
        stages(r),
        ["requested", "accepted", "executing", "rejected"]
    );
    assert!(matches!(
        r.stage(),
        Some(Stage::Rejected(Rejection::Stale { changes }))
            if matches!(&changes[..], [Change::StateChanged { removed, .. }] if *removed == strings(&["C"]))
    ));
    assert!(git.commits.is_empty());
}

#[test]
fn an_async_point_of_truth_rejects_through_reject_at_truth() {
    let mut git = Git::new(&["A", "B"], "fix login");
    git.background = true;
    let mut d = Dispatcher::new();
    let req = Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    let id = d.dispatch(&mut git, req, 1);
    git.index.insert("Z".into());
    let answer = git.work();
    assert_eq!(answer.0, id);
    assert!(matches!(
        answer.1,
        Execution::Rejected(Rejection::Stale { .. })
    ));
    settle(&mut d, answer, 2).unwrap();
    assert_eq!(
        stages(receipt(&d, id)),
        ["requested", "accepted", "executing", "rejected"]
    );
    assert!(git.commits.is_empty());
}

#[test]
fn illegal_transitions_are_errors_and_leave_the_receipt_as_it_was() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::new();
    let illegal = |id: RequestId, from: &str, to: &str| SemError::IllegalTransition {
        id,
        from: from.into(),
        to: to.into(),
    };

    let discard = Request::observe(&git.snapshot(), agent(), "changes", "discard", None).unwrap();
    let waiting = d.dispatch(&mut git, discard, 1);
    let before = receipt(&d, waiting).clone();
    assert_eq!(
        d.complete(waiting, Outcome::default(), 2),
        Err(illegal(waiting, "awaiting_confirmation", "completed"))
    );
    assert_eq!(
        d.fail(waiting, "x".into(), 2),
        Err(illegal(waiting, "awaiting_confirmation", "failed"))
    );
    let why = Rejection::InvalidParam { reason: "x".into() };
    assert_eq!(
        d.reject_at_truth(waiting, why.clone(), 2),
        Err(illegal(waiting, "awaiting_confirmation", "rejected"))
    );
    assert_eq!(receipt(&d, waiting), &before);

    git.background = true;
    let submit =
        Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    let running = d.dispatch(&mut git, submit, 3);
    assert_eq!(
        d.confirm(&mut git, running, Actor::Human, 4),
        Err(illegal(running, "executing", "confirmed"))
    );
    d.complete(running, Outcome::default(), 5).unwrap();
    assert_eq!(
        d.complete(running, Outcome::default(), 6),
        Err(illegal(running, "completed", "completed"))
    );
    assert_eq!(
        d.reject_at_truth(running, why, 6),
        Err(illegal(running, "completed", "rejected"))
    );
    assert_eq!(
        d.confirm(&mut git, running, Actor::Human, 6),
        Err(illegal(running, "completed", "confirmed"))
    );

    let ghost = RequestId::parse("999").unwrap();
    assert_eq!(
        d.complete(ghost, Outcome::default(), 7),
        Err(SemError::UnknownRequest(ghost))
    );
    assert_eq!(
        d.confirm(&mut git, ghost, Actor::Human, 7),
        Err(SemError::UnknownRequest(ghost))
    );
    assert!(d.receipt(ghost).is_none());
}

/// records every stage a sink sees, with the length of the receipt at
/// that moment.
#[derive(Clone, Default)]
struct Log(Rc<RefCell<Vec<Seen>>>);

/// the request, the stage, its time, and how many stages the receipt held.
type Seen = (RequestId, Stage, u64, usize);

impl ReceiptSink for Log {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        let at = receipt.stages.last().map_or(0, |(_, t)| *t);
        self.0
            .borrow_mut()
            .push((receipt.id, stage.clone(), at, receipt.stages.len()));
    }
}

#[test]
fn a_sink_sees_every_stage_in_order() {
    let mut git = Git::new(&["A", "B"], "wip");
    git.background = true;
    let mut d = Dispatcher::new();
    let log = Log::default();
    d.add_sink(Box::new(log.clone()));

    let discard = Request::observe(&git.snapshot(), agent(), "changes", "discard", None).unwrap();
    let submit =
        Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    // the commit starts in the background; the discard waits, is
    // confirmed and runs; then the worker finds the index emptied and the
    // commit is rejected at the point of truth.
    let a = d.dispatch(&mut git, submit, 1);
    let b = d.dispatch(&mut git, discard, 2);
    d.confirm(&mut git, b, Actor::Human, 3).unwrap();
    let answer = git.work();
    assert_eq!(answer.0, a);
    settle(&mut d, answer, 4).unwrap();

    let seen = log.0.borrow();
    for id in [a, b] {
        let r = receipt(&d, id);
        let mine: Vec<(Stage, u64)> = seen
            .iter()
            .filter(|(i, ..)| *i == id)
            .map(|(_, s, t, _)| (s.clone(), *t))
            .collect();
        assert_eq!(mine, r.stages, "request {id}");
        let lengths: Vec<usize> = seen
            .iter()
            .filter(|(i, ..)| *i == id)
            .map(|(.., n)| *n)
            .collect();
        assert_eq!(lengths, (1..=r.stages.len()).collect::<Vec<_>>());
    }
    let order: Vec<(RequestId, &str)> = seen.iter().map(|(i, s, ..)| (*i, s.name())).collect();
    assert_eq!(
        order,
        [
            (a, "requested"),
            (a, "accepted"),
            (a, "executing"),
            (b, "requested"),
            (b, "awaiting_confirmation"),
            (b, "confirmed"),
            (b, "executing"),
            (b, "completed"),
            (a, "rejected"),
        ]
    );
}

#[test]
fn finished_receipts_are_dropped_past_the_retention_and_open_ones_stay() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::with_retention(2);
    let log = Log::default();
    d.add_sink(Box::new(log.clone()));
    let discard = Request::observe(&git.snapshot(), agent(), "changes", "discard", None).unwrap();
    let open = d.dispatch(&mut git, discard, 0);
    let ids: Vec<RequestId> = (1..=3)
        .map(|n| {
            let seen = git.snapshot();
            let req = Request::observe(&seen, Actor::Human, "changes", "move", Some(Value::Int(n)));
            d.dispatch(&mut git, req.unwrap(), n as u64)
        })
        .collect();
    assert!(d.receipt(ids[0]).is_none());
    assert!(d.receipt(ids[1]).is_some());
    assert!(d.receipt(ids[2]).is_some());
    assert!(d.receipt(open).is_some());
    assert_eq!(
        d.complete(ids[0], Outcome::default(), 9),
        Err(SemError::UnknownRequest(ids[0]))
    );
    let dropped_seen = log.0.borrow().iter().filter(|(i, ..)| *i == ids[0]).count();
    assert_eq!(dropped_seen, 4);
}

#[test]
fn with_no_retention_a_finished_receipt_only_reaches_the_sinks() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::with_retention(0);
    let log = Log::default();
    d.add_sink(Box::new(log.clone()));
    let req = Request::observe(
        &git.snapshot(),
        Actor::Human,
        "changes",
        "move",
        Some(Value::Int(1)),
    );
    let id = d.dispatch(&mut git, req.unwrap(), 1);
    assert!(d.receipt(id).is_none());
    assert_eq!(log.0.borrow().len(), 4);
}

#[test]
fn requests_are_numbered_from_one_and_times_are_the_callers() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::new();
    let ids: Vec<u64> = [100, 40, 7]
        .into_iter()
        .map(|t| {
            let req = Request::observe(
                &git.snapshot(),
                Actor::Human,
                "changes",
                "move",
                Some(Value::Int(0)),
            );
            let id = d.dispatch(&mut git, req.unwrap(), t);
            assert!(receipt(&d, id).stages.iter().all(|(_, at)| *at == t));
            id.as_u64()
        })
        .collect();
    assert_eq!(ids, [1, 2, 3]);
}

#[test]
fn a_projection_that_does_not_validate_rejects_without_executing() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::new();
    let req = Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    git.broken = true;
    let id = d.dispatch(&mut git, req, 1);
    let r = receipt(&d, id);
    assert!(matches!(
        r.stage(),
        Some(Stage::Rejected(Rejection::InvalidProjection { errors })) if !errors.0.is_empty()
    ));
    assert_eq!(r.validated_against, None);
    assert!(git.executed.is_empty());
}

#[test]
fn every_actor_goes_through_the_same_dispatch() {
    let mut d = Dispatcher::new();
    for actor in [
        Actor::Human,
        agent(),
        Actor::Assistive,
        Actor::Test {
            name: "dispatch".into(),
        },
    ] {
        let mut git = Git::new(&["A"], "fix login");
        let req = Request::observe(
            &git.snapshot(),
            actor.clone(),
            "commit.submit",
            "submit",
            None,
        );
        let id = d.dispatch(&mut git, req.unwrap(), 1);
        let r = receipt(&d, id);
        assert_eq!(r.request.actor, actor);
        assert!(matches!(r.stage(), Some(Stage::Completed(_))), "{actor}");
    }
}
