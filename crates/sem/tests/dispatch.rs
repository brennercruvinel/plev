//! `Dispatcher` with an in-memory git client: the thesis scenario,
//! confirmation, async execution, rejection at the point of truth, illegal
//! transitions, sinks and retention.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use indexmap::IndexMap;
use sem::{
    Acted, Actor, Change, Confirm, Constraints, Dispatcher, Execution, Mark, Operable, Outcome,
    ParamSpec, Policy, Receipt, ReceiptSink, Rejection, Request, RequestId, Role, SemError,
    Semantics, Snapshot, SnapshotBuilder, Stage, StateRef, Value, ValueKind, Verdict, check,
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

fn illegal(id: RequestId, from: &str, to: &str) -> SemError {
    SemError::IllegalTransition {
        id,
        from: from.into(),
        to: to.into(),
    }
}

fn tester() -> Actor {
    Actor::Test {
        name: "dispatch".into(),
    }
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
    let Some(Stage::Rejected(Rejection::Stale { changes, .. })) = r.stage() else {
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
    for machine in [agent(), tester()] {
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
        format!("request {id} waits for a person; agent:reviewer cannot confirm or decline it")
    );
}

#[test]
fn a_person_declines_at_the_screen_or_through_a_screen_reader() {
    for by in [Actor::Human, Actor::Assistive] {
        let mut git = Git::new(&["A", "B"], "wip");
        let mut d = Dispatcher::new();
        let id = awaiting_discard(&mut git, &mut d);
        let validated = receipt(&d, id).validated_against;

        d.decline(id, by.clone(), 5).unwrap();
        let r = receipt(&d, id);
        assert_eq!(
            stages(r),
            ["requested", "awaiting_confirmation", "rejected"]
        );
        assert_eq!(
            r.stages[2],
            (Stage::Rejected(Rejection::Declined { by: by.clone() }), 5)
        );
        assert!(r.is_finished());
        assert_eq!(
            r.validated_against, validated,
            "a decline does not validate"
        );
        assert!(git.executed.is_empty(), "nothing ran");
        assert_eq!(git.staged, set(&["A", "B"]));

        // the answer is final: no second one.
        assert_eq!(
            d.confirm(&mut git, id, Actor::Human, 6),
            Err(illegal(id, "rejected", "confirmed"))
        );
        assert_eq!(
            d.decline(id, by, 6),
            Err(illegal(id, "rejected", "rejected"))
        );
        assert!(git.executed.is_empty());
    }
}

#[test]
fn an_agent_or_a_test_cannot_decline_and_the_receipt_keeps_waiting() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let id = awaiting_discard(&mut git, &mut d);
    let before = receipt(&d, id).clone();
    for machine in [agent(), tester()] {
        assert_eq!(
            d.decline(id, machine.clone(), 2),
            Err(SemError::ConfirmByMachine { id, by: machine })
        );
        assert_eq!(receipt(&d, id), &before);
    }
    assert!(git.executed.is_empty());

    // still waiting: a person can still confirm it.
    d.confirm(&mut git, id, Actor::Human, 3).unwrap();
    assert!(matches!(receipt(&d, id).stage(), Some(Stage::Completed(_))));
    assert_eq!(git.executed, ["discard"]);
}

#[test]
fn declining_what_does_not_wait_is_an_error_found_before_the_actor() {
    let mut git = Git::new(&["A"], "wip");
    git.background = true;
    let mut d = Dispatcher::new();
    let submit =
        Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    let running = d.dispatch(&mut git, submit, 1);
    let before = receipt(&d, running).clone();
    for by in [Actor::Human, agent()] {
        assert_eq!(
            d.decline(running, by, 2),
            Err(illegal(running, "executing", "rejected"))
        );
    }
    assert_eq!(receipt(&d, running), &before);

    d.complete(running, Outcome::default(), 3).unwrap();
    assert_eq!(
        d.decline(running, Actor::Assistive, 4),
        Err(illegal(running, "completed", "rejected"))
    );

    let ghost = RequestId::parse("999").unwrap();
    for by in [Actor::Human, tester()] {
        assert_eq!(
            d.decline(ghost, by, 5),
            Err(SemError::UnknownRequest(ghost))
        );
    }
    assert!(d.receipt(ghost).is_none());
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
        Some(Stage::Rejected(Rejection::Stale { changes, .. }))
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
        Some(Stage::Rejected(Rejection::Stale { changes, .. }))
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

/// `stage` or `unstage` of `file` by `actor`, on the state now.
fn file_act(
    d: &mut Dispatcher,
    git: &mut Git,
    actor: Actor,
    action: &str,
    file: &str,
) -> RequestId {
    let seen = git.snapshot();
    let req = Request::observe(&seen, actor, "changes", action, Some(file.into())).unwrap();
    d.dispatch(git, req, 5)
}

/// the one entry `acted` holds for request `id`.
fn acted(id: RequestId, actor: Actor, target: &str, action: &str) -> Acted {
    Acted {
        id,
        actor,
        target: target.parse().unwrap(),
        action: action.parse().unwrap(),
    }
}

fn acted_of(r: &Receipt) -> &[Acted] {
    match r.stage() {
        Some(Stage::Rejected(Rejection::Stale { acted, .. })) => acted,
        other => panic!("a stale refusal, got {other:?}"),
    }
}

/// the thesis again, with the room full: the person takes B out of the
/// state the agent saw, then C out of the state that left, and the
/// agent's commit, refused stale, names both unstages. who only observed,
/// who was refused, who waits for a confirmation and the agent itself ran
/// nothing, or are the author, and are not named. (task-83 named only the
/// first unstage, the one checked on exactly the state the agent saw;
/// task-90 names the chain from there.)
#[test]
fn a_stale_refusal_names_who_ran_something_on_the_state_the_agent_saw() {
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    let mut d = Dispatcher::new();
    let seen = git.snapshot();
    let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
    // another agent observes the same state and asks nothing.
    let watcher = Actor::Agent {
        name: "watcher".into(),
    };
    let _looked = Request::observe(&seen, watcher, "commit.submit", "submit", None).unwrap();

    let long = "x".repeat(73);
    let refused =
        Request::observe(&seen, tester(), "commit.message", "set", Some(long.into())).unwrap();
    let refused = d.dispatch(&mut git, refused, 1);
    assert_eq!(stages(receipt(&d, refused)), ["requested", "rejected"]);
    let cleaner = Actor::Agent {
        name: "cleaner".into(),
    };
    let discard = Request::observe(&seen, cleaner.clone(), "changes", "discard", None).unwrap();
    let waiting = d.dispatch(&mut git, discard, 2);
    assert_eq!(
        stages(receipt(&d, waiting)),
        ["requested", "awaiting_confirmation"]
    );
    // the agent's own act runs on that state and leaves it as it was.
    let own = file_act(&mut d, &mut git, agent(), "stage", "A");
    assert!(matches!(
        receipt(&d, own).stage(),
        Some(Stage::Completed(_))
    ));
    assert_eq!(git.snapshot().revision(), seen.revision());

    let person = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
    assert_eq!(receipt(&d, person).validated_against, Some(seen.revision()));
    let chained = file_act(&mut d, &mut git, Actor::Human, "unstage", "C");
    assert_ne!(
        receipt(&d, chained).validated_against,
        Some(seen.revision())
    );

    let id = d.dispatch(&mut git, commit, 6);
    let r = receipt(&d, id);
    assert_eq!(stages(r), ["requested", "rejected"]);
    assert_eq!(
        acted_of(r),
        [
            acted(person, Actor::Human, "changes", "unstage"),
            acted(chained, Actor::Human, "changes", "unstage"),
        ]
    );
    let Some(Stage::Rejected(Rejection::Stale { changes, .. })) = r.stage() else {
        unreachable!();
    };
    assert!(matches!(
        &changes[..],
        [Change::StateChanged { removed, .. }] if *removed == strings(&["B", "C"])
    ));

    // the discard that waited is refused stale at its confirmation, and
    // names the same chain: the agent's own stage is someone else's for
    // the cleaner, and ran on what the cleaner saw.
    d.confirm(&mut git, waiting, Actor::Human, 7).unwrap();
    let r = receipt(&d, waiting);
    assert_eq!(
        stages(r),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert_eq!(
        acted_of(r),
        [
            acted(own, agent(), "changes", "stage"),
            acted(person, Actor::Human, "changes", "unstage"),
            acted(chained, Actor::Human, "changes", "unstage"),
        ]
    );
    assert!(acted_of(r).iter().all(|a| a.actor != cleaner));
}

/// the same at the point of truth: the refusal that comes back through
/// `reject_at_truth` names who ran since the state the request observed,
/// an execution still running and one that failed among them; one the
/// point of truth refused ran nothing.
#[test]
fn a_refusal_at_the_point_of_truth_names_who_acted_too() {
    let mut git = Git::new(&["A", "B"], "fix login");
    git.background = true;
    let mut d = Dispatcher::new();
    let seen = git.snapshot();
    let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
    let late = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
    let theirs = Request::observe(&seen, Actor::Human, "commit.submit", "submit", None).unwrap();
    let other = Actor::Agent {
        name: "other".into(),
    };
    let stale_other = Request::observe(&seen, other, "commit.submit", "submit", None).unwrap();

    let ours = d.dispatch(&mut git, commit, 1);
    let running = d.dispatch(&mut git, theirs, 2);
    assert_eq!(
        stages(receipt(&d, running)),
        ["requested", "accepted", "executing"]
    );
    let person = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");

    // the agent's commit reaches the index, which lost B.
    let answer = git.work();
    assert_eq!(answer.0, ours);
    settle(&mut d, answer, 3).unwrap();
    let r = receipt(&d, ours);
    assert_eq!(
        stages(r),
        ["requested", "accepted", "executing", "rejected"]
    );
    assert_eq!(
        acted_of(r),
        [
            acted(running, Actor::Human, "commit.submit", "submit"),
            acted(person, Actor::Human, "changes", "unstage"),
        ]
    );

    // the person's commit fails on the worker; it still ran.
    git.worker_error = Some("index.lock exists".into());
    let answer = git.work();
    assert_eq!(answer.0, running);
    settle(&mut d, answer, 4).unwrap();
    let id = d.dispatch(&mut git, stale_other, 5);
    assert_eq!(
        acted_of(receipt(&d, id)),
        [
            acted(running, Actor::Human, "commit.submit", "submit"),
            acted(person, Actor::Human, "changes", "unstage"),
        ],
        "the agent's commit, refused at the point of truth, ran nothing"
    );
    // and the author's own refused commit is never named for it.
    let id = d.dispatch(&mut git, late, 6);
    assert!(acted_of(receipt(&d, id)).iter().all(|a| a.id != ours));
}

/// who acted is read from the receipts the dispatcher still holds: past
/// the retention a finished one is gone and is not named. the refusal is
/// read from a sink, since with no retention its own receipt goes too.
#[test]
fn who_acted_is_read_from_the_receipts_the_dispatcher_holds() {
    for (retention, named) in [(0, 0), (1, 1)] {
        let mut git = Git::new(&["A", "B"], "fix login");
        let mut d = Dispatcher::with_retention(retention);
        let log = Log::default();
        d.add_sink(Box::new(log.clone()));
        let seen = git.snapshot();
        let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
        file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
        let id = d.dispatch(&mut git, commit, 6);
        let last = log.0.borrow().last().cloned();
        let Some((at, Stage::Rejected(Rejection::Stale { acted, .. }), ..)) = last else {
            panic!("a stale refusal last, got {last:?}");
        };
        assert_eq!(at, id);
        assert_eq!(acted.len(), named, "retention {retention}");
    }
}

/// `actor` moves the list cursor to `to`, on the state now: state no
/// action but `move` reads.
fn moves(d: &mut Dispatcher, git: &mut Git, actor: Actor, to: i64) -> RequestId {
    let seen = git.snapshot();
    let req = Request::observe(&seen, actor, "changes", "move", Some(Value::Int(to))).unwrap();
    d.dispatch(git, req, 5)
}

/// the last stage a sink saw, which must be a stale refusal of `id`: who
/// it names.
fn last_acted(log: &Log, id: RequestId) -> Vec<Acted> {
    let last = log.0.borrow().last().cloned();
    let Some((at, Stage::Rejected(Rejection::Stale { acted, .. }), ..)) = last else {
        panic!("a stale refusal last, got {last:?}");
    };
    assert_eq!(at, id);
    acted
}

/// the person's chain since the agent looked: a move on the state the
/// agent saw, then an unstage on the state that move left. the first ran
/// on what the agent saw, so it ran after the agent looked, and so did
/// everything after it: the agent's commit, refused stale, names both, in
/// id order. the agent's own move in the middle is not named and does not
/// cut the chain.
#[test]
fn a_stale_refusal_names_the_chain_that_ran_since_the_state_the_agent_saw() {
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    let mut d = Dispatcher::new();
    let seen = git.snapshot();
    let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();

    let first = moves(&mut d, &mut git, Actor::Human, 1);
    assert_eq!(receipt(&d, first).validated_against, Some(seen.revision()));
    let own = moves(&mut d, &mut git, agent(), 2);
    let next = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
    assert_ne!(receipt(&d, next).validated_against, Some(seen.revision()));

    let id = d.dispatch(&mut git, commit, 6);
    let r = receipt(&d, id);
    assert_eq!(stages(r), ["requested", "rejected"]);
    assert_eq!(
        acted_of(r),
        [
            acted(first, Actor::Human, "changes", "move"),
            acted(next, Actor::Human, "changes", "unstage"),
        ]
    );
    assert!(acted_of(r).iter().all(|a| a.id != own));
}

/// the chain can start at the author's own act: the agent moves the
/// cursor on the state it saw, the person unstages B on the state that
/// left, and the agent's commit names the person's unstage and never the
/// agent's own move.
#[test]
fn the_authors_own_act_on_the_state_it_saw_can_start_the_chain() {
    let mut git = Git::new(&["A", "B"], "fix login");
    let mut d = Dispatcher::new();
    let seen = git.snapshot();
    let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();

    let own = moves(&mut d, &mut git, agent(), 1);
    assert_eq!(receipt(&d, own).validated_against, Some(seen.revision()));
    let person = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");

    let id = d.dispatch(&mut git, commit, 6);
    assert_eq!(
        acted_of(receipt(&d, id)),
        [acted(person, Actor::Human, "changes", "unstage")]
    );
}

/// with no act that ran on the state the agent saw there is nowhere to
/// start: the state moved first without a request (a terminal unstaged C
/// and the app shows it), the person unstages B on what that left, and
/// the refusal names nobody. the person's unstage ran after the agent
/// looked, but nothing the dispatcher holds tells it from one that ran
/// before. a refusal of the person's that ran nothing starts nothing
/// either.
#[test]
fn a_change_no_request_made_leaves_the_chain_nowhere_to_start() {
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    let mut d = Dispatcher::new();
    let seen = git.snapshot();
    let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
    let long = "x".repeat(73);
    let refused = Request::observe(
        &seen,
        Actor::Human,
        "commit.message",
        "set",
        Some(long.into()),
    )
    .unwrap();
    let refused = d.dispatch(&mut git, refused, 1);
    assert_eq!(
        receipt(&d, refused).validated_against,
        Some(seen.revision())
    );
    assert_eq!(stages(receipt(&d, refused)), ["requested", "rejected"]);

    git.staged.remove("C");
    git.index.remove("C");
    let person = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
    assert!(matches!(
        receipt(&d, person).stage(),
        Some(Stage::Completed(_))
    ));

    let id = d.dispatch(&mut git, commit, 6);
    let r = receipt(&d, id);
    assert_eq!(stages(r), ["requested", "rejected"]);
    assert!(acted_of(r).is_empty(), "{:?}", r.stage());
}

/// the chain is read from the receipts the dispatcher holds: what
/// retention dropped is not named, and once it dropped the act the chain
/// started at, the acts after it are not named either, since nothing
/// held ran on the state the agent saw.
#[test]
fn a_chain_is_read_from_the_receipts_the_dispatcher_holds() {
    for (retention, named) in [(0, 0), (1, 0), (2, 2)] {
        let mut git = Git::new(&["A", "B"], "fix login");
        let mut d = Dispatcher::with_retention(retention);
        let log = Log::default();
        d.add_sink(Box::new(log.clone()));
        let seen = git.snapshot();
        let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
        moves(&mut d, &mut git, Actor::Human, 1);
        file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
        let id = d.dispatch(&mut git, commit, 6);
        assert_eq!(last_acted(&log, id).len(), named, "retention {retention}");
    }
}

// -- the look (task-91) --

/// knows when `actor` looked: at the mark this sink was added at, as the
/// agent port tells the dispatcher for the acts it dispatches. adding it
/// is the look.
struct Looking {
    actor: Actor,
    at: Option<Mark>,
}

impl ReceiptSink for Looking {
    fn on_stage(&mut self, _receipt: &Receipt, _stage: &Stage) {}

    fn attached(&mut self, mark: Mark) {
        self.at = Some(mark);
    }

    fn looked(&self, receipt: &Receipt) -> Option<Mark> {
        self.at.filter(|_| receipt.request.actor == self.actor)
    }
}

/// `actor` looks now: from here on the dispatcher knows it.
fn looks(d: &mut Dispatcher, actor: Actor) {
    d.add_sink(Box::new(Looking { actor, at: None }));
}

/// the window scene of the ide, in memory: the agent looks, the state
/// moves with no request (a click that only selects, here the cursor set
/// outside the dispatcher), and the person unstages B on what that left.
/// with the look, the person's unstage is named; without it, the rule of
/// task-90 has nowhere to start and names nobody.
#[test]
fn a_look_names_what_ran_after_it_whatever_moved_the_state_first() {
    for look in [true, false] {
        let mut git = Git::new(&["A", "B"], "fix login");
        let mut d = Dispatcher::new();
        let seen = git.snapshot();
        let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
        if look {
            looks(&mut d, agent());
        }
        git.cursor = 3;
        let person = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
        assert_ne!(receipt(&d, person).validated_against, Some(seen.revision()));

        let id = d.dispatch(&mut git, commit, 6);
        let named = acted_of(receipt(&d, id)).to_vec();
        match look {
            true => assert_eq!(named, [acted(person, Actor::Human, "changes", "unstage")]),
            false => assert!(named.is_empty(), "no look, no start: {named:?}"),
        }
    }
}

/// with the look, the author's own acts after it are named too, in id
/// order with the others (the refusal by its own write, task-62), and
/// nothing taken before the look is: the person moved the cursor before
/// the agent looked, the agent then rewrote the message and the person
/// unstaged B, and the agent's commit on its look names those two.
#[test]
fn with_a_look_the_authors_own_acts_after_it_are_named_and_nothing_before_it() {
    let mut git = Git::new(&["A", "B"], "fix login");
    let mut d = Dispatcher::new();
    let before = moves(&mut d, &mut git, Actor::Human, 1);
    let seen = git.snapshot();
    let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
    looks(&mut d, agent());
    let now = git.snapshot();
    let text = Some(Value::from("fix login flow"));
    let write = Request::observe(&now, agent(), "commit.message", "set", text).unwrap();
    let write = d.dispatch(&mut git, write, 3);
    let person = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");

    let id = d.dispatch(&mut git, commit, 6);
    let r = receipt(&d, id);
    assert_eq!(
        acted_of(r),
        [
            acted(write, agent(), "commit.message", "set"),
            acted(person, Actor::Human, "changes", "unstage"),
        ]
    );
    assert!(acted_of(r).iter().all(|a| a.id != before));
}

/// the look only names. the same requests, with and without a look, end
/// in the same stages with the same delta; only `acted` differs: with the
/// look it names the agent's own move after it too, without it the rule
/// of task-90 names the person alone. a look the sink gives for another
/// actor's requests is not this one's.
#[test]
fn a_look_never_decides() {
    let run = |look: Option<Actor>| {
        let mut git = Git::new(&["A", "B"], "fix login");
        let mut d = Dispatcher::new();
        let seen = git.snapshot();
        let stale = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
        if let Some(actor) = look {
            looks(&mut d, actor);
        }
        let person = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
        let fresh = Request::observe(
            &git.snapshot(),
            agent(),
            "changes",
            "move",
            Some(Value::Int(2)),
        )
        .unwrap();
        let ok = d.dispatch(&mut git, fresh, 5);
        let refused = d.dispatch(&mut git, stale, 6);
        let stages = |id| {
            receipt(&d, id)
                .stages
                .iter()
                .map(|(stage, _)| match stage {
                    Stage::Rejected(Rejection::Stale { changes, .. }) => {
                        format!("stale {changes:?}")
                    }
                    other => other.name().to_owned(),
                })
                .collect::<Vec<_>>()
        };
        let named: Vec<RequestId> = acted_of(receipt(&d, refused))
            .iter()
            .map(|a| a.id)
            .collect();
        ((stages(ok), stages(refused)), named, (person, ok))
    };
    let (with, named, (person, own)) = run(Some(agent()));
    let (without, none, _) = run(None);
    let (elsewhere, other, _) = run(Some(Actor::Human));
    assert_eq!(with, without);
    assert_eq!(with, elsewhere);
    assert_eq!(
        named,
        [person, own],
        "the look names the agent's own move too"
    );
    assert_eq!(
        (none, other),
        (vec![person], vec![person]),
        "the rule of task-90"
    );
}

/// the look names at a confirmation's second check and at the point of
/// truth as at dispatch: an agent's discard waits, the person unstages B
/// and then confirms it, and the discard, stale now, names the unstage;
/// an agent's commit starts on the worker, the person unstages A, and the
/// index refuses the commit, naming the unstage and the discard's run.
#[test]
fn a_look_names_at_the_confirmation_and_at_the_point_of_truth_too() {
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    git.background = true;
    let mut d = Dispatcher::new();
    let seen = git.snapshot();
    looks(&mut d, agent());
    let discard = Request::observe(&seen, agent(), "changes", "discard", None).unwrap();
    let waiting = d.dispatch(&mut git, discard, 1);
    let person = file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
    d.confirm(&mut git, waiting, Actor::Human, 2).unwrap();
    assert_eq!(
        acted_of(receipt(&d, waiting)),
        [acted(person, Actor::Human, "changes", "unstage")]
    );

    let commit =
        Request::observe(&git.snapshot(), agent(), "commit.submit", "submit", None).unwrap();
    let running = d.dispatch(&mut git, commit, 3);
    let second = file_act(&mut d, &mut git, Actor::Human, "unstage", "A");
    let answer = git.work();
    assert_eq!(answer.0, running);
    settle(&mut d, answer, 4).unwrap();
    assert_eq!(
        acted_of(receipt(&d, running)),
        [
            acted(person, Actor::Human, "changes", "unstage"),
            acted(second, Actor::Human, "changes", "unstage"),
        ],
        "the refused discard ran nothing; the commit itself is not named"
    );
}

/// with a look, what retention dropped is not named, and the rest still
/// is: no start to lose.
#[test]
fn a_look_reads_the_receipts_the_dispatcher_holds() {
    for (retention, named) in [(0, 0), (1, 1), (2, 2)] {
        let mut git = Git::new(&["A", "B"], "fix login");
        let mut d = Dispatcher::with_retention(retention);
        let log = Log::default();
        d.add_sink(Box::new(log.clone()));
        let seen = git.snapshot();
        let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
        looks(&mut d, agent());
        moves(&mut d, &mut git, Actor::Human, 1);
        file_act(&mut d, &mut git, Actor::Human, "unstage", "B");
        let id = d.dispatch(&mut git, commit, 6);
        assert_eq!(last_acted(&log, id).len(), named, "retention {retention}");
    }
}

/// a mark is how many requests the dispatcher had taken: a sink is told
/// the one it came at, and a request comes after a mark exactly when it
/// was taken later.
#[test]
fn a_sink_is_told_the_mark_it_came_at() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::new();
    assert_eq!(d.mark(), Mark::default());
    let first = moves(&mut d, &mut git, Actor::Human, 1);
    assert_eq!(d.mark(), Mark::after(first));
    let told = Rc::new(RefCell::new(None));
    struct Told(Rc<RefCell<Option<Mark>>>);
    impl ReceiptSink for Told {
        fn on_stage(&mut self, _receipt: &Receipt, _stage: &Stage) {}
        fn attached(&mut self, mark: Mark) {
            *self.0.borrow_mut() = Some(mark);
        }
    }
    d.add_sink(Box::new(Told(Rc::clone(&told))));
    assert_eq!(*told.borrow(), Some(Mark::after(first)));
    let second = moves(&mut d, &mut git, Actor::Human, 2);
    let at = Mark::after(first);
    assert!(!at.precedes(first) && at.precedes(second));
    assert!(Mark::default().precedes(first));
    assert!(Mark::after(first) < Mark::after(second));
}

#[test]
fn illegal_transitions_are_errors_and_leave_the_receipt_as_it_was() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::new();

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
fn a_sink_sees_a_decline_as_the_last_stage() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::with_retention(0);
    let log = Log::default();
    d.add_sink(Box::new(log.clone()));
    let id = awaiting_discard(&mut git, &mut d);
    d.decline(id, Actor::Assistive, 8).unwrap();

    let declined = Stage::Rejected(Rejection::Declined {
        by: Actor::Assistive,
    });
    let seen = log.0.borrow();
    let names: Vec<&str> = seen.iter().map(|(_, s, ..)| s.name()).collect();
    assert_eq!(names, ["requested", "awaiting_confirmation", "rejected"]);
    assert_eq!(seen.last(), Some(&(id, declined, 8, 3)));
    // finished, it left the dispatcher (no retention): only the sink saw it.
    assert!(d.receipt(id).is_none());
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

/// what the agent asks a person before it stages `file` (task-92).
const QUESTION: &str = "the person took C out; the task needs the three: put it back?";

/// `actor` observes the client now and asks to stage `file`, asking a
/// person first.
fn asked_stage(git: &Git, actor: Actor, file: &str) -> Request {
    Request::observe(
        &git.snapshot(),
        actor,
        "changes",
        "stage",
        Some(file.into()),
    )
    .unwrap()
    .asking(QUESTION)
}

#[test]
fn an_agent_that_asks_waits_for_a_person_even_when_the_policy_asks_nothing() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let req = asked_stage(&git, agent(), "C");
    assert!(req.asks_a_person());
    let id = d.dispatch(&mut git, req, 1);
    let r = receipt(&d, id);
    assert_eq!(stages(r), ["requested", "awaiting_confirmation"]);
    assert_eq!(r.request.ask.as_deref(), Some(QUESTION));
    assert!(git.executed.is_empty(), "nothing runs while it waits");

    // nobody answers: it keeps waiting, whatever else runs.
    unstage(&mut d, &mut git, "A", 2);
    assert_eq!(
        stages(receipt(&d, id)),
        ["requested", "awaiting_confirmation"]
    );

    // the same request without the question runs at once: the policy of
    // `stage` asks nothing.
    let plain = Request::observe(
        &git.snapshot(),
        agent(),
        "changes",
        "stage",
        Some("C".into()),
    )
    .unwrap();
    let plain = d.dispatch(&mut git, plain, 3);
    assert_eq!(
        stages(receipt(&d, plain)),
        ["requested", "accepted", "executing", "completed"]
    );
}

#[test]
fn a_question_confirmed_runs_after_the_second_check() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let req = asked_stage(&git, agent(), "C");
    let id = d.dispatch(&mut git, req, 1);
    d.confirm(&mut git, id, Actor::Human, 2).unwrap();
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
    assert_eq!(r.stages[2], (Stage::Confirmed { by: Actor::Human }, 2));
    assert_eq!(git.staged, set(&["A", "B", "C"]));

    // the second check: a person moves what the stage reads while the
    // question waits, then confirms. the request is stale; nothing runs.
    let req = asked_stage(&git, agent(), "D");
    let id = d.dispatch(&mut git, req, 3);
    unstage(&mut d, &mut git, "B", 4);
    d.confirm(&mut git, id, Actor::Human, 5).unwrap();
    let r = receipt(&d, id);
    assert_eq!(
        stages(r),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert!(matches!(
        r.stage(),
        Some(Stage::Rejected(Rejection::Stale { changes, .. }))
            if matches!(&changes[..], [Change::StateChanged { removed, .. }] if *removed == strings(&["B"]))
    ));
    assert_eq!(
        git.executed,
        ["stage", "unstage"],
        "the refused one never ran"
    );
}

#[test]
fn a_question_declined_never_runs() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let req = asked_stage(&git, agent(), "C");
    let id = d.dispatch(&mut git, req, 1);
    assert_eq!(
        d.decline(id, agent(), 2),
        Err(SemError::ConfirmByMachine { id, by: agent() }),
        "the agent that asked never answers itself"
    );
    d.decline(id, Actor::Human, 3).unwrap();
    let r = receipt(&d, id);
    assert_eq!(
        stages(r),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert_eq!(
        r.stage(),
        Some(&Stage::Rejected(Rejection::Declined { by: Actor::Human }))
    );
    assert!(git.executed.is_empty());
    assert_eq!(git.staged, set(&["A", "B"]));
}

#[test]
fn a_question_on_a_request_that_already_waits_changes_nothing() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let discard = Request::observe(&git.snapshot(), agent(), "changes", "discard", None).unwrap();
    let plain = d.dispatch(&mut git, discard.clone(), 1);
    let asked = d.dispatch(&mut git, discard.asking(QUESTION), 2);
    for id in [plain, asked] {
        assert_eq!(
            stages(receipt(&d, id)),
            ["requested", "awaiting_confirmation"]
        );
    }
    d.confirm(&mut git, asked, Actor::Human, 3).unwrap();
    assert_eq!(
        receipt(&d, asked).stage().map(Stage::name),
        Some("completed")
    );
    assert_eq!(git.executed, ["discard"]);
}

/// a person does not ask a person: the question of a person's request, or
/// a test's, asks nothing, as `Confirm::ForAgents` asks nothing of them.
#[test]
fn a_person_or_a_test_asks_nobody() {
    let mut git = Git::new(&["A"], "wip");
    let mut d = Dispatcher::new();
    for (actor, file) in [
        (Actor::Human, "B"),
        (Actor::Assistive, "C"),
        (tester(), "D"),
    ] {
        let req = asked_stage(&git, actor.clone(), file);
        assert!(!req.asks_a_person(), "{actor}");
        let id = d.dispatch(&mut git, req, 1);
        assert_eq!(
            stages(receipt(&d, id)),
            ["requested", "accepted", "executing", "completed"],
            "{actor}"
        );
    }
    assert_eq!(git.staged, set(&["A", "B", "C", "D"]));
}

/// a question is asked last: a request the state refuses is refused as it
/// would be without it, and no person is asked.
#[test]
fn a_question_never_passes_a_refusal() {
    let mut git = Git::new(&["A", "B"], "wip");
    let mut d = Dispatcher::new();
    let late = asked_stage(&git, agent(), "C");
    unstage(&mut d, &mut git, "B", 1);
    let id = d.dispatch(&mut git, late, 2);
    assert!(matches!(
        receipt(&d, id).stage(),
        Some(Stage::Rejected(Rejection::Stale { .. }))
    ));

    let mut empty = Git::new(&[], "wip");
    let submit = Request::observe(&empty.snapshot(), agent(), "commit.submit", "submit", None)
        .unwrap()
        .asking(QUESTION);
    let id = d.dispatch(&mut empty, submit, 3);
    assert!(matches!(
        receipt(&d, id).stage(),
        Some(Stage::Rejected(Rejection::Disabled { .. }))
    ));
}
