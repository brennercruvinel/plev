//! A person and an agent operate the same git client at the same time,
//! against a real repository and the real git worker: the agent's commit
//! is checked on what it observed at the dispatcher and again at the
//! index right before `git commit`, a valid commit comes back with its sha
//! and files, and every step has a receipt that says who asked.

mod support;

use std::cell::RefCell;
use std::rc::Rc;

use engine::compositor::Compositor;
use sem::{Actor, Change, Receipt, ReceiptSink, Rejection, SemId, Stage, Value};
use support::{Rig, agent, completed, set};

fn tester() -> Actor {
    Actor::Test {
        name: "commit".into(),
    }
}

/// The one change of a stale rejection.
fn stale(receipt: Receipt) -> Change {
    match receipt.stage() {
        Some(Stage::Rejected(Rejection::Stale { changes })) if changes.len() == 1 => {
            changes[0].clone()
        }
        other => panic!("expected one stale change, got {other:?}"),
    }
}

/// Every receipt, as it stood at its last stage, in the order they
/// started.
#[derive(Clone, Default)]
struct Log(Rc<RefCell<Vec<Receipt>>>);

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
    /// `(actor, target, action, stage)` of every receipt.
    fn steps(&self) -> Vec<(String, String, String, &'static str)> {
        self.0
            .borrow()
            .iter()
            .map(|r| {
                (
                    r.request.actor.to_string(),
                    r.request.target.to_string(),
                    r.request.action.to_string(),
                    r.stage().map_or("none", Stage::name),
                )
            })
            .collect()
    }
}

#[test]
fn an_unchanged_observation_commits_and_the_receipt_holds_sha_and_files() {
    let mut rig = Rig::new(&["A", "B", "C"], &["A", "B", "C"]);
    rig.agent_writes("fix login");

    let id = rig.ask(agent(), "commit.submit", "submit", None);
    assert_eq!(
        rig.receipt(id).stage(),
        Some(&Stage::Executing),
        "the commit left for the worker"
    );
    rig.settle(id);

    assert_eq!(
        rig.stages(id),
        ["requested", "accepted", "executing", "completed"]
    );
    let receipt = rig.receipt(id);
    let outcome = completed(&receipt).expect("completed");
    assert_eq!(outcome.result.get("commit"), Some(&Value::Text(rig.head())));
    assert_eq!(
        outcome.result.get("files"),
        Some(&Value::List(rig.head_files()))
    );
    assert_eq!(rig.head_files(), ["A", "B", "C"]);
    assert_eq!(rig.commits(), 2);
    assert_eq!(rig.git(&["log", "-1", "--format=%s"]).trim(), "fix login");

    // The form cleared once the commit existed; the refresh emptied the
    // staged set.
    assert_eq!(rig.value("commit.form:open"), Value::Bool(false));
    assert_eq!(rig.value("commit.message:text"), Value::from(""));
    assert_eq!(rig.value("changes:staged"), set(&[]));
}

#[test]
fn a_person_unstaging_b_makes_the_agent_commit_stale() {
    let mut rig = Rig::new(&["A", "B", "C"], &["A", "B", "C"]);
    rig.agent_writes("fix login");
    let head = rig.head();

    // The agent decides on {A, B, C} and "fix login".
    let commit = rig.observe(agent(), "commit.submit", "submit", None);
    // Meanwhile the person unstages B, through the same dispatcher.
    let unstage = rig.ask(Actor::Human, "changes.file[B]", "unstage", None);
    assert_eq!(rig.receipt(unstage).stage(), Some(&Stage::Executing));
    assert_eq!(rig.value("changes:staged"), set(&["A", "C"]));

    let id = rig.bench.dispatch(commit);
    assert_eq!(rig.stages(id), ["requested", "rejected"]);
    let Change::StateChanged {
        at,
        before,
        after,
        added,
        removed,
    } = stale(rig.receipt(id))
    else {
        panic!("a state change");
    };
    assert_eq!(at.to_string(), "changes:staged");
    assert_eq!(removed, &["B"]);
    assert!(added.is_empty());
    assert_eq!(before.as_ref(), Some(&set(&["A", "B", "C"])));
    assert_eq!(after.as_ref(), Some(&set(&["A", "C"])));

    rig.settle(unstage);
    assert!(completed(&rig.receipt(unstage)).is_some());
    assert_eq!(rig.head(), head, "git log did not move");
    assert_eq!(rig.commits(), 1);
    assert_eq!(rig.value("changes:staged"), set(&["A", "C"]));
    assert_eq!(
        rig.value("changes.file[B]:status"),
        Value::from("untracked")
    );
}

#[test]
fn moving_the_list_cursor_between_observe_and_submit_is_accepted() {
    let mut rig = Rig::new(&["A", "B", "C"], &["A", "B"]);
    rig.agent_writes("fix login");
    let seen = rig.look();
    let commit = rig.observe(agent(), "commit.submit", "submit", None);

    // The person walks the list: the cursor and the diff move.
    assert!(rig.bench.nav_down());
    assert!(rig.bench.nav_down());
    let now = rig.look();
    assert_ne!(now.revision(), seen.revision(), "the projection moved");
    assert_eq!(rig.value("changes:selected"), Value::from("B"));
    assert_eq!(rig.value("diff:target"), Value::from("changes.file[B]"));

    let id = rig.bench.dispatch(commit);
    rig.settle(id);
    assert!(
        completed(&rig.receipt(id)).is_some(),
        "{:?}",
        rig.receipt(id)
    );
    assert_eq!(rig.head_files(), ["A", "B"]);
}

#[test]
fn an_index_moved_outside_is_refused_at_the_point_of_truth() {
    let mut rig = Rig::new(&["A", "B", "C"], &["A", "B"]);
    rig.agent_writes("fix login");
    let head = rig.head();
    let commit = rig.observe(agent(), "commit.submit", "submit", None);

    // A terminal stages C behind the workbench's back; no refresh lands,
    // so the dispatcher still projects {A, B}.
    rig.git(&["add", "C"]);
    assert_eq!(rig.value("changes:staged"), set(&["A", "B"]));

    let id = rig.bench.dispatch(commit);
    assert_eq!(
        rig.receipt(id).stage(),
        Some(&Stage::Executing),
        "the dispatcher saw nothing wrong"
    );
    rig.settle(id);
    assert_eq!(
        rig.stages(id),
        ["requested", "accepted", "executing", "rejected"],
        "rejected at the index"
    );
    let Change::StateChanged {
        at,
        before,
        after,
        added,
        removed,
    } = stale(rig.receipt(id))
    else {
        panic!("a state change");
    };
    assert_eq!(at.to_string(), "changes:staged");
    assert_eq!(added, &["C"]);
    assert!(removed.is_empty());
    assert_eq!(before.as_ref(), Some(&set(&["A", "B"])));
    assert_eq!(after.as_ref(), Some(&set(&["A", "B", "C"])));

    assert_eq!(rig.head(), head, "no commit");
    // The form kept the message for another try; the refresh shows C.
    assert_eq!(rig.value("commit.message:text"), Value::from("fix login"));
    assert_eq!(rig.value("changes:staged"), set(&["A", "B", "C"]));
}

#[test]
fn the_same_observation_submitted_twice_commits_once() {
    let mut rig = Rig::new(&["A"], &["A"]);
    rig.agent_writes("once");
    let commit = rig.observe(agent(), "commit.submit", "submit", None);
    let first = rig.bench.dispatch(commit.clone());
    // The second leaves before the first's answer: the dispatcher still
    // projects {A}, the index no longer stages it.
    let second = rig.bench.dispatch(commit);
    rig.settle(first);
    rig.settle(second);
    assert!(completed(&rig.receipt(first)).is_some());
    let Change::StateChanged { removed, .. } = stale(rig.receipt(second)) else {
        panic!("a state change");
    };
    assert_eq!(removed, &["A"]);
    assert_eq!(rig.commits(), 2);
}

#[test]
fn the_lead_passes_between_agent_and_person_with_every_step_receipted() {
    let mut rig = Rig::new(&["A", "B"], &["B"]);
    let log = Log::default();
    rig.bench.add_sink(Box::new(log.clone()));

    // The agent stages A.
    let stage = rig.ask(agent(), "changes.file[A]", "stage", None);
    rig.settle(stage);
    // The person opens the commit form (the `c` key).
    assert!(rig.bench.toggle_commit_form());
    // The agent writes the message and decides to commit.
    let text = Some("fix login".into());
    let set_value = rig.ask(agent(), "commit.message", "set_value", text);
    assert!(completed(&rig.receipt(set_value)).is_some());
    let commit = rig.observe(agent(), "commit.submit", "submit", None);
    // The person edits the message: typing is editing, the field's own.
    for ch in " flow".chars() {
        rig.bench.workspace_mut().commit_form.append_char(ch);
    }

    let refused = rig.bench.dispatch(commit);
    let Change::StateChanged {
        at, before, after, ..
    } = stale(rig.receipt(refused))
    else {
        panic!("a state change");
    };
    assert_eq!(at.to_string(), "commit.message:text");
    assert_eq!(before.as_ref(), Some(&Value::from("fix login")));
    assert_eq!(after.as_ref(), Some(&Value::from("fix login flow")));

    // The agent observes again and commits what the person wrote.
    let accepted = rig.ask(agent(), "commit.submit", "submit", None);
    rig.settle(accepted);
    assert!(completed(&rig.receipt(accepted)).is_some());
    assert_eq!(rig.head_files(), ["A", "B"]);
    assert_eq!(
        rig.git(&["log", "-1", "--format=%s"]).trim(),
        "fix login flow"
    );

    let who = |actor: &str, target: &str, action: &str, stage: &'static str| {
        (actor.into(), target.into(), action.into(), stage)
    };
    assert_eq!(
        log.steps(),
        [
            who("agent:test-agent", "changes.file[A]", "stage", "completed"),
            who("human", "commit.form", "open", "completed"),
            who(
                "agent:test-agent",
                "commit.message",
                "set_value",
                "completed"
            ),
            who("agent:test-agent", "commit.submit", "submit", "rejected"),
            who("agent:test-agent", "commit.submit", "submit", "completed"),
        ]
    );
}

#[test]
fn an_agent_discard_waits_for_a_person_who_confirms_it() {
    let mut rig = Rig::new(&["README", "new.txt"], &[]);
    let readme = rig.dir.path().join("README");
    assert_eq!(std::fs::read_to_string(&readme).unwrap(), "README\n");

    let id = rig.ask(agent(), "changes.file[README]", "discard", None);
    assert_eq!(rig.stages(id), ["requested", "awaiting_confirmation"]);
    assert_eq!(
        std::fs::read_to_string(&readme).unwrap(),
        "README\n",
        "nothing ran while it waits"
    );
    assert!(
        rig.bench.confirm(id, agent()).is_err(),
        "an agent never confirms"
    );

    rig.bench
        .confirm(id, Actor::Human)
        .expect("a person confirms");
    rig.settle(id);
    assert_eq!(
        rig.stages(id),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    assert!(matches!(
        &rig.receipt(id).stages[2].0,
        Stage::Confirmed { by: Actor::Human }
    ));
    assert_eq!(std::fs::read_to_string(&readme).unwrap(), "readme\n");
    assert_eq!(
        rig.value("changes:files"),
        Value::List(vec!["new.txt".into()])
    );
}

/// The frame a person would see: the workbench drawn and bound.
fn frame(rig: &mut Rig) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    rig.bench.render(&mut c);
    c
}

fn center(c: &Compositor, id: &str) -> (f32, f32) {
    let id = SemId::parse(id).expect("id");
    let r = c.binding(&id).expect("bound").rect;
    (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0)
}

#[test]
fn a_click_on_commit_and_a_test_request_commit_the_same() {
    // A person: the cursor to A, `s`, `c`, typing, a click on Commit.
    let mut person = Rig::new(&["A", "B"], &[]);
    assert!(person.bench.nav_down());
    assert!(person.bench.stage_selected());
    person.until(|e| matches!(e, git::GitEvent::Branches(_)));
    assert!(person.bench.toggle_commit_form());
    for ch in "add A".chars() {
        person.bench.workspace_mut().commit_form.append_char(ch);
    }
    let c = frame(&mut person);
    let (x, y) = center(&c, "commit.submit");
    assert_eq!(c.hit(x, y), Some(&SemId::parse("commit.submit").unwrap()));
    assert!(person.bench.click(x, y));
    person.until(|e| matches!(e, git::GitEvent::Branches(_)));

    // A test: the same capabilities, by request.
    let mut test = Rig::new(&["A", "B"], &[]);
    let stage = test.ask(tester(), "changes.file[A]", "stage", None);
    test.settle(stage);
    let open = test.ask(tester(), "commit.form", "open", None);
    let text = Some("add A".into());
    let write = test.ask(tester(), "commit.message", "set_value", text);
    let submit = test.ask(tester(), "commit.submit", "submit", None);
    test.settle(submit);
    for id in [stage, open, write, submit] {
        assert!(
            completed(&test.receipt(id)).is_some(),
            "{:?}",
            test.receipt(id)
        );
    }

    for rig in [&person, &test] {
        assert_eq!(rig.commits(), 2);
        assert_eq!(rig.head_files(), ["A"]);
        assert_eq!(rig.git(&["log", "-1", "--format=%s"]).trim(), "add A");
    }
    for at in [
        "changes:files",
        "changes:staged",
        "changes.file[B]:status",
        "commit.form:open",
        "commit.message:text",
    ] {
        assert_eq!(person.value(at), test.value(at), "{at}");
    }
}
