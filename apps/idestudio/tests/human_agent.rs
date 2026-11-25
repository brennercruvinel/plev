//! A person and an agent operate the same git client at the same time,
//! against a real repository and the real git worker: the agent's commit
//! is checked on what it observed at the dispatcher and again at the
//! index right before `git commit`, a valid commit comes back with its sha
//! and files, and every step has a receipt that says who asked.

use crate::support;

use std::cell::RefCell;
use std::rc::Rc;

use accord::{Actor, Change, Receipt, ReceiptSink, Rejection, Revision, SemId, Stage, Value};
use engine::drawing::compositor::Compositor;
use engine::widgets::feedback::{ContextMenu, MenuEntry};
use support::{Rig, agent, completed, set};

fn tester() -> Actor {
    Actor::Test {
        name: "commit".into(),
    }
}

/// The one change of a stale rejection.
fn stale(receipt: Receipt) -> Change {
    match receipt.stage() {
        Some(Stage::Rejected(Rejection::Stale { changes, .. })) if changes.len() == 1 => {
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
    // The person sees why, and who moved it: their own unstage, still
    // executing on the worker, ran on the state the agent saw.
    assert_eq!(
        rig.notices(),
        ["Refused test-agent's submit: B left the staged files (the person unstaged B)"]
    );

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
    assert_eq!(rig.notices(), ["Refused test-agent's submit: C was staged"]);

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

    // The person reads the modal and clicks Discard.
    assert!(rig.bench.workspace().modal_open());
    let _ = frame(&mut rig);
    let (confirm, _) = rig.bench.workspace().modal_buttons().expect("drawn");
    let (x, y) = (confirm[0] + confirm[2] / 2.0, confirm[1] + confirm[3] / 2.0);
    assert!(rig.bench.click(x, y));
    assert!(!rig.bench.workspace().modal_open());
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

#[test]
fn a_person_who_closes_the_modal_declines_and_the_next_request_is_asked() {
    let mut rig = Rig::new(&["README", "new.txt"], &[]);
    let readme = rig.dir.path().join("README");
    let first = rig.ask(agent(), "changes.file[README]", "discard", None);
    let second = rig.ask(agent(), "changes.file[new.txt]", "discard", None);
    for id in [first, second] {
        assert_eq!(rig.stages(id), ["requested", "awaiting_confirmation"]);
    }
    let declined = Some(Stage::Rejected(Rejection::Declined { by: Actor::Human }));

    // The person reads the first modal and clicks Cancel.
    assert!(rig.bench.workspace().modal_open());
    let _ = frame(&mut rig);
    let (_, cancel) = rig.bench.workspace().modal_buttons().expect("drawn");
    let (x, y) = (cancel[0] + cancel[2] / 2.0, cancel[1] + cancel[3] / 2.0);
    assert!(rig.bench.click(x, y));
    assert_eq!(
        rig.stages(first),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert_eq!(rig.receipt(first).stage().cloned(), declined);
    assert_eq!(
        std::fs::read_to_string(&readme).unwrap(),
        "README\n",
        "the file is not discarded"
    );

    // The second is asked next; Escape declines it too.
    assert!(rig.bench.workspace().modal_open(), "the second is asked");
    assert_eq!(rig.stages(second), ["requested", "awaiting_confirmation"]);
    assert!(rig.bench.close_top_overlay());
    assert_eq!(rig.receipt(second).stage().cloned(), declined);
    assert!(!rig.bench.workspace().modal_open());
    assert_eq!(
        rig.notices(),
        [
            "Declined: discard of README by test-agent did not run",
            "Declined: discard of new.txt by test-agent did not run",
        ]
    );

    // Git agrees nothing ran: a fresh status still lists both changes.
    rig.bench.refresh();
    rig.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));
    assert_eq!(
        rig.value("changes:files"),
        Value::List(vec!["README".into(), "new.txt".into()])
    );
    assert_eq!(std::fs::read_to_string(&readme).unwrap(), "README\n");
    assert_eq!(
        std::fs::read_to_string(rig.dir.path().join("new.txt")).unwrap(),
        "new.txt\n"
    );
}

#[test]
fn every_commit_in_the_stacks_exposes_the_files_it_changes() {
    let rig = Rig::new(&["A"], &[]);
    let head = rig.head();
    assert_eq!(
        rig.value(&format!("stacks.commit[{head}]:files")),
        Value::List(rig.head_files()),
        "the log's files, as git lists them"
    );
    assert_eq!(rig.head_files(), ["README"]);
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
    person.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));
    assert!(person.bench.toggle_commit_form());
    for ch in "add A".chars() {
        person.bench.workspace_mut().commit_form.append_char(ch);
    }
    let c = frame(&mut person);
    let (x, y) = center(&c, "commit.submit");
    assert_eq!(c.hit(x, y), Some(&SemId::parse("commit.submit").unwrap()));
    assert!(person.bench.click(x, y));
    person.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));

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

// -- a refresh read before a mutation the view already shows (task-84) --
//
// The worker runs commands one at a time, in the order they were sent, and
// every mutation's answer sends a `Refresh`. A refresh queued before a
// later mutation reads the status before that mutation runs; its answer
// lands after the view already shows the mutation. The order is fixed by
// the worker's queue, not by timing: each test applies the events one
// kind at a time and looks at the projection in between.

#[test]
fn a_status_read_before_a_stage_never_takes_the_file_back_out() {
    let mut rig = Rig::new(&["A", "B"], &[]);
    let a = rig.ask(agent(), "changes.file[A]", "stage", None);
    // The answer to stage A sends a refresh, which waits at the worker.
    rig.until(|e| matches!(e, gitdrv::GitEvent::Staged { .. }));
    assert!(completed(&rig.receipt(a)).is_some(), "{:?}", rig.receipt(a));

    // Stage B leaves behind that refresh: its status is read before
    // `git add B` runs.
    let b = rig.ask(agent(), "changes.file[B]", "stage", None);
    assert_eq!(rig.value("changes:staged"), set(&["A", "B"]));

    rig.until(|e| matches!(e, gitdrv::GitEvent::Status(_)));
    assert_eq!(
        rig.value("changes:staged"),
        set(&["A", "B"]),
        "a status read before stage B does not take B out"
    );

    rig.settle(b);
    assert!(completed(&rig.receipt(b)).is_some(), "{:?}", rig.receipt(b));
    assert_eq!(rig.value("changes:staged"), set(&["A", "B"]));
    assert_eq!(rig.git(&["diff", "--cached", "--name-only"]), "A\nB\n");
}

#[test]
fn a_status_read_before_an_unstage_never_puts_the_file_back() {
    let mut rig = Rig::new(&["A", "B", "C"], &["A", "B", "C"]);
    rig.agent_writes("only C");
    let a = rig.ask(agent(), "changes.file[A]", "unstage", None);
    rig.until(|e| matches!(e, gitdrv::GitEvent::Unstaged { .. }));

    // The person takes B out while the refresh for A waits at the worker.
    let b = rig.ask(Actor::Human, "changes.file[B]", "unstage", None);
    assert_eq!(rig.value("changes:staged"), set(&["C"]));
    rig.until(|e| matches!(e, gitdrv::GitEvent::Status(_)));
    assert_eq!(
        rig.value("changes:staged"),
        set(&["C"]),
        "a status read before the person's unstage does not put B back"
    );

    // An agent deciding right now decides on what the index will hold
    // when its commit runs, so the commit is not refused for nothing.
    let commit = rig.ask(agent(), "commit.submit", "submit", None);
    rig.settle(commit);
    assert!(
        completed(&rig.receipt(commit)).is_some(),
        "{:?}",
        rig.receipt(commit)
    );
    assert_eq!(rig.head_files(), ["C"]);
    for id in [a, b] {
        assert!(
            completed(&rig.receipt(id)).is_some(),
            "{:?}",
            rig.receipt(id)
        );
    }
}

#[test]
fn a_status_read_before_a_discard_never_brings_the_file_back() {
    let mut rig = Rig::new(&["A", "B"], &[]);
    let a = rig.ask(agent(), "changes.file[A]", "stage", None);
    rig.until(|e| matches!(e, gitdrv::GitEvent::Staged { .. }));

    // The person discards B (a new file: discard deletes it) behind the
    // refresh for A.
    let b = rig.ask(Actor::Human, "changes.file[B]", "discard", None);
    assert_eq!(rig.stages(b), ["requested", "accepted", "executing"]);
    assert_eq!(rig.value("changes:files"), Value::List(vec!["A".into()]));
    rig.until(|e| matches!(e, gitdrv::GitEvent::Status(_)));
    assert_eq!(
        rig.value("changes:files"),
        Value::List(vec!["A".into()]),
        "a status read before the discard does not bring B back"
    );

    rig.settle(b);
    assert!(completed(&rig.receipt(a)).is_some(), "{:?}", rig.receipt(a));
    assert!(completed(&rig.receipt(b)).is_some(), "{:?}", rig.receipt(b));
    assert_eq!(rig.value("changes:files"), Value::List(vec!["A".into()]));
    assert!(!rig.dir.path().join("B").exists());
}

#[test]
fn a_change_from_outside_lands_with_the_refresh_after_the_last_mutation() {
    let mut rig = Rig::new(&["A", "B", "C"], &[]);

    // Nothing pending: a terminal stages C and the watcher's refresh
    // shows it.
    rig.git(&["add", "C"]);
    rig.bench.refresh();
    rig.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));
    assert_eq!(rig.value("changes:staged"), set(&["C"]));

    // A terminal stages B and the watcher asks for a refresh; stage A
    // leaves behind it.
    rig.git(&["add", "B"]);
    rig.bench.refresh();
    let a = rig.ask(agent(), "changes.file[A]", "stage", None);
    rig.until(|e| matches!(e, gitdrv::GitEvent::Status(_)));
    assert_eq!(
        rig.value("changes:staged"),
        set(&["A", "C"]),
        "the watcher's status, read before stage A, does not take A out"
    );

    rig.settle(a);
    assert_eq!(
        rig.value("changes:staged"),
        set(&["A", "B", "C"]),
        "the terminal's B arrives with the refresh stage A sent"
    );
}

#[test]
fn a_stage_git_refuses_is_rolled_back_by_its_refresh() {
    let mut rig = Rig::new(&["A", "B"], &[]);
    // A leaves the disk behind the view's back: the view still lists it,
    // and `git add A` will fail.
    std::fs::remove_file(rig.dir.path().join("A")).expect("remove A");
    let a = rig.ask(agent(), "changes.file[A]", "stage", None);
    assert_eq!(rig.value("changes:staged"), set(&["A"]), "the view guessed");

    rig.settle(a);
    assert!(
        matches!(rig.receipt(a).stage(), Some(Stage::Failed { .. })),
        "{:?}",
        rig.receipt(a)
    );
    assert_eq!(rig.value("changes:staged"), set(&[]));
    assert_eq!(rig.value("changes:files"), Value::List(vec!["B".into()]));
}

// -- the commit a receipt announces is in the next observation (task-85) --
//
// The worker's `Committed` ends the submit's receipt; the refresh it sends
// comes after. Each test applies the answer to the commit alone, looks at
// the projection an agent would observe next, then applies what follows.

/// The list value `[a, b]`.
fn list(paths: &[&str]) -> Value {
    Value::List(paths.iter().map(|p| p.to_string()).collect())
}

/// The sha and the files a completed commit receipt holds.
fn committed(receipt: &Receipt) -> (String, Vec<String>) {
    let outcome = completed(receipt).unwrap_or_else(|| panic!("completed: {receipt:?}"));
    match (outcome.result.get("commit"), outcome.result.get("files")) {
        (Some(Value::Text(sha)), Some(Value::List(files))) => (sha.clone(), files.clone()),
        other => panic!("a commit and its files: {other:?}"),
    }
}

/// The projection does not contradict the receipt: the commit tops the
/// stack with the receipt's files, and none of them is staged.
fn shows_commit(rig: &Rig, sha: &str, files: &[String]) {
    let node = format!("stacks.commit[{sha}]");
    assert_eq!(
        rig.value(&format!("{node}:files")),
        Value::List(files.to_vec())
    );
    let seen = rig.look();
    let stacks = SemId::parse("stacks").expect("id");
    let top = seen
        .children(&stacks)
        .next()
        .expect("a commit in the stacks");
    assert_eq!(top.id.as_str(), node, "the commit tops the stack");
    let Value::Set(staged) = rig.value("changes:staged") else {
        panic!("changes:staged is a set");
    };
    for file in files {
        assert!(!staged.contains(file), "{file} is committed, not staged");
    }
}

#[test]
fn the_commit_a_receipt_announces_is_projected_before_any_refresh() {
    let mut rig = Rig::new(&["A", "B", "C"], &["A", "B"]);
    // B changes again in the worktree: a staged row and an unstaged one.
    std::fs::write(rig.dir.path().join("B"), "B again\n").expect("write B");
    rig.bench.refresh();
    rig.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));
    assert_eq!(rig.value("changes:staged"), set(&["A", "B"]));
    assert_eq!(rig.value("changes.file[B]:unstaged"), Value::Bool(true));
    rig.agent_writes("fix login");

    let id = rig.ask(agent(), "commit.submit", "submit", None);
    // The answer to the commit, and not one event more.
    rig.until(|e| matches!(e, gitdrv::GitEvent::Committed(_)));
    let (sha, files) = committed(&rig.receipt(id));
    assert_eq!(files, ["A", "B"]);

    shows_commit(&rig, &sha, &files);
    let node = format!("stacks.commit[{sha}]");
    assert_eq!(
        rig.value(&format!("{node}:summary")),
        Value::from("fix login")
    );
    // the author git gave the commit: the repository's own, or the one the
    // environment exports (GIT_AUTHOR_NAME, an agent's session), which git
    // puts first.
    let author = rig.git(&["log", "-1", "--format=%an", &sha]);
    assert_eq!(
        rig.value(&format!("{node}:author")),
        Value::from(author.trim())
    );
    // A had only its staged row and leaves; B keeps its worktree change.
    assert_eq!(rig.value("changes:files"), list(&["B", "C"]));
    assert_eq!(rig.value("changes:staged"), set(&[]));
    assert_eq!(rig.value("changes.file[B]:unstaged"), Value::Bool(true));

    // The refresh the answer sent: the same, now read from git.
    rig.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));
    shows_commit(&rig, &sha, &files);
    assert_eq!(rig.head(), sha);
    assert_eq!(rig.head_files(), files);
    assert_eq!(
        rig.value(&format!("{node}:summary")),
        Value::from("fix login")
    );
    assert_eq!(
        rig.value(&format!("{node}:author")),
        Value::from(author.trim())
    );
    assert_eq!(rig.value("changes:files"), list(&["B", "C"]));
    assert_eq!(rig.value("changes.file[B]:status"), Value::from("modified"));
}

#[test]
fn no_answer_after_the_commit_takes_it_back() {
    let mut rig = Rig::new(&["A", "B"], &["A"]);
    rig.agent_writes("add A");
    // The watcher asks for a refresh before the commit leaves (read before
    // it) and another after (read after it).
    rig.bench.refresh();
    let id = rig.ask(agent(), "commit.submit", "submit", None);
    rig.bench.refresh();
    rig.until(|e| matches!(e, gitdrv::GitEvent::Committed(_)));
    let (sha, files) = committed(&rig.receipt(id));
    shows_commit(&rig, &sha, &files);

    // The watcher's second refresh, then the one the commit's answer sent:
    // after every answer, the commit is still there.
    for _ in 0..6 {
        rig.until(|_| true);
        shows_commit(&rig, &sha, &files);
    }
    assert_eq!(rig.value("changes:files"), list(&["B"]));
    assert_eq!(rig.head_files(), files);
}

#[test]
fn a_path_unstaged_after_the_commit_left_is_left_for_the_refresh() {
    let mut rig = Rig::new(&["A", "B"], &["A", "B"]);
    rig.agent_writes("both");
    let id = rig.ask(agent(), "commit.submit", "submit", None);
    // The person takes B out once the commit left: the view shows B
    // unstaged, and the worker commits B before it unstages it.
    let unstage = rig.ask(Actor::Human, "changes.file[B]", "unstage", None);
    assert_eq!(rig.value("changes:staged"), set(&["A"]));

    rig.until(|e| matches!(e, gitdrv::GitEvent::Committed(_)));
    let (sha, files) = committed(&rig.receipt(id));
    assert_eq!(files, ["A", "B"]);
    shows_commit(&rig, &sha, &files);
    // The view does not show B staged, so the guess cannot tell what the
    // commit left of it: B stays as it was.
    assert_eq!(rig.value("changes:files"), list(&["B"]));

    // Git: B is in the commit, and unstaging it from there changes nothing.
    rig.settle(unstage);
    assert!(completed(&rig.receipt(unstage)).is_some());
    shows_commit(&rig, &sha, &files);
    assert_eq!(rig.value("changes:files"), list(&[]));
}

#[test]
fn a_commit_with_a_rename_holds_both_paths_in_the_receipt_and_the_node() {
    let mut rig = Rig::new(&[], &[]);
    // README moves to docs/README, unchanged: the index will pair the two
    // into one rename, at the new path.
    std::fs::create_dir_all(rig.dir.path().join("docs")).expect("docs");
    std::fs::rename(
        rig.dir.path().join("README"),
        rig.dir.path().join("docs/README"),
    )
    .expect("move README");
    rig.bench.refresh();
    rig.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));
    for path in ["README", "docs/README"] {
        let id = rig.ask(agent(), &format!("changes.file[{path}]"), "stage", None);
        rig.settle(id);
        assert!(
            completed(&rig.receipt(id)).is_some(),
            "{:?}",
            rig.receipt(id)
        );
    }
    assert_eq!(rig.value("changes:staged"), set(&["docs/README"]));
    assert_eq!(
        rig.value("changes.file[docs/README]:status"),
        Value::from("renamed")
    );
    rig.agent_writes("move the readme");

    let id = rig.ask(agent(), "commit.submit", "submit", None);
    rig.until(|e| matches!(e, gitdrv::GitEvent::Committed(_)));
    let (sha, files) = committed(&rig.receipt(id));
    assert_eq!(
        files,
        ["README", "docs/README"],
        "both paths the commit changes"
    );
    shows_commit(&rig, &sha, &files);
    assert_eq!(rig.value("changes:files"), list(&[]));

    // After the refresh the node comes from the log: the same paths.
    rig.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));
    shows_commit(&rig, &sha, &files);
    assert_eq!(rig.head(), sha);
    assert_eq!(
        rig.head_files(),
        files,
        "what git diff-tree --no-renames lists"
    );
}

#[test]
fn the_last_commit_the_ide_made_keeps_its_files_past_the_newest() {
    let mut rig = Rig::new(&["A"], &["A"]);
    rig.agent_writes("add A");
    let id = rig.ask(agent(), "commit.submit", "submit", None);
    rig.settle(id);
    let (sha, files) = committed(&rig.receipt(id));

    // Enough commits land from a terminal to push it past the newest.
    let newest = idestudio::capability::FILES_PROJECTED;
    for n in 0..newest {
        rig.git(&[
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            &format!("outside {n}"),
        ]);
    }
    rig.bench.refresh();
    rig.until(|e| matches!(e, gitdrv::GitEvent::Branches(_)));

    let seen = rig.look();
    let stacks = SemId::parse("stacks").expect("id");
    let order: Vec<String> = seen.children(&stacks).map(|n| n.id.to_string()).collect();
    let node = format!("stacks.commit[{sha}]");
    assert_eq!(
        order.get(newest),
        Some(&node),
        "the ide's commit is past the newest"
    );
    assert_eq!(rig.value(&format!("{node}:files")), Value::List(files));
    // The initial commit, older still, carries none.
    let initial = order.last().expect("the initial commit");
    let key = format!("{initial}:files").parse().expect("a state ref");
    assert_eq!(seen.value(&key), None, "{initial} carries no files");
}

// -- who acted since the agent looked (task-90) --
//
// Both scenes run through the pointer, as the window does: a left click on
// a row, a right click that opens the file menu, a click on its first item.

/// The receipt the person's last click or menu item asked for.
fn last(log: &Log) -> Receipt {
    log.0.borrow().last().cloned().expect("a receipt")
}

/// The person right-clicks the row of `path` and clicks the menu's first
/// item ("Unstage file" on a staged row). The menu is laid out by the
/// design-system widget the view draws, at the point the view opens it
/// (half an `xs` right of the pointer); its first item's row is the same
/// with or without the view's icons, which only widen it.
fn unstage_from_the_menu(rig: &mut Rig, path: &str) {
    let c = frame(rig);
    let (x, y) = center(&c, &format!("changes.file[{path}]"));
    assert!(rig.bench.right_click(x, y));
    let _ = frame(rig);
    let theme = rig.bench.workspace().theme().clone();
    let menu = ContextMenu::new(vec![
        MenuEntry::item(0, "Unstage file"),
        MenuEntry::item(1, "Discard changes"),
        MenuEntry::item(2, "Ignore file"),
    ]);
    let first = menu.entry_rects(x + theme.spacing.xs / 2.0, y, &theme)[0];
    assert!(
        rig.bench
            .click(first.x + first.w / 2.0, first.y + first.h / 2.0)
    );
}

/// `(actor, target, action)` of a receipt.
fn asked(receipt: &Receipt) -> (String, String, String) {
    let req = &receipt.request;
    (
        req.actor.to_string(),
        req.target.to_string(),
        req.action.to_string(),
    )
}

fn human(target: &str, action: &str) -> (String, String, String) {
    ("human".into(), target.into(), action.into())
}

/// A chain that starts on the state the agent saw. The person had clicked
/// B before the agent decided (selected, its diff shown). After the
/// decision the person unstages B from the menu of that row: opening the
/// menu moves nothing projected, so the unstage is checked on the state
/// the agent saw. Then the person right-clicks C (the row is selected and
/// its diff asked for) and unstages C from its menu. The agent's commit,
/// refused stale, names the three in id order, and the toast, which is
/// the `notices` node the agent reads, says it in one clause.
#[test]
fn a_chain_the_person_starts_on_what_the_agent_saw_is_named_whole() {
    let mut rig = Rig::new(&["A", "B", "C"], &["A", "B", "C"]);
    let log = Log::default();
    rig.bench.add_sink(Box::new(log.clone()));
    rig.agent_writes("fix login");
    let c = frame(&mut rig);
    let (x, y) = center(&c, "changes.file[B]");
    assert!(rig.bench.click(x, y));
    rig.until(|e| matches!(e, gitdrv::GitEvent::DiffWorkdir { .. }));

    let seen = rig.look();
    assert_eq!(rig.value("changes:selected"), Value::from("B"));
    let commit = rig.observe(agent(), "commit.submit", "submit", None);

    unstage_from_the_menu(&mut rig, "B");
    let unstage_b = last(&log);
    assert_eq!(asked(&unstage_b), human("changes.file[B]", "unstage"));
    assert_eq!(unstage_b.validated_against, Some(seen.revision()));
    unstage_from_the_menu(&mut rig, "C");
    let (show_c, unstage_c) = match log.0.borrow().as_slice() {
        [.., show, unstage] => (show.clone(), unstage.clone()),
        all => panic!("receipts: {all:?}"),
    };
    assert_eq!(asked(&show_c), human("changes.file[C]", "show_diff"));
    assert_eq!(asked(&unstage_c), human("changes.file[C]", "unstage"));

    let id = rig.bench.dispatch(commit);
    assert_eq!(rig.stages(id), ["requested", "rejected"]);
    let receipt = rig.receipt(id);
    let Some(Stage::Rejected(Rejection::Stale { changes, acted })) = receipt.stage() else {
        panic!("stale: {:?}", receipt.stage());
    };
    assert!(matches!(
        changes.as_slice(),
        [Change::StateChanged { removed, .. }] if removed == &["B", "C"]
    ));
    let named: Vec<_> = acted
        .iter()
        .map(|a| {
            let who = (
                a.actor.to_string(),
                a.target.to_string(),
                a.action.to_string(),
            );
            (a.id, who)
        })
        .collect();
    assert_eq!(
        named,
        [
            (unstage_b.id, asked(&unstage_b)),
            (show_c.id, asked(&show_c)),
            (unstage_c.id, asked(&unstage_c)),
        ]
    );
    assert_eq!(
        rig.notices(),
        [
            "Refused test-agent's submit: B left the staged files, C left the staged files \
             (the person unstaged B, opened the diff of C and unstaged C)"
        ]
    );

    rig.settle(unstage_b.id);
    rig.settle(unstage_c.id);
    assert_eq!(rig.git(&["diff", "--cached", "--name-only"]), "A\n");
    assert_eq!(rig.commits(), 1);
}

/// The window scene of task-86 through the pointer, after the agent
/// looked at `seen` with no row selected. The person clicks B: the view
/// moves the selection itself, outside the dispatcher, and the projected
/// `changes:selected` with it, so the revision leaves the one the agent
/// saw before any request; only then is the diff asked for. The person
/// unstages B from its menu. The receipts of the diff and of the unstage.
fn click_a_new_row_then_unstage(rig: &mut Rig, log: &Log, seen: Revision) -> (Receipt, Receipt) {
    assert_eq!(rig.value("changes:selected"), Value::from(""));
    let c = frame(rig);
    let (x, y) = center(&c, "changes.file[B]");
    assert!(rig.bench.click(x, y));
    let show = last(log);
    assert_eq!(asked(&show), human("changes.file[B]", "show_diff"));
    assert!(show.validated_against.is_some());
    assert_ne!(
        show.validated_against,
        Some(seen),
        "the selection moved the revision before the diff was asked for"
    );
    rig.until(|e| matches!(e, gitdrv::GitEvent::DiffWorkdir { .. }));
    unstage_from_the_menu(rig, "B");
    let unstage = last(log);
    assert_eq!(asked(&unstage), human("changes.file[B]", "unstage"));
    assert_ne!(unstage.validated_against, Some(seen));
    (show, unstage)
}

/// The window scene with the agent looking in the process
/// (`Request::observe`), not through the port: no look is known, and the
/// rule of task-90 is the fallback. No request ran on the state the agent
/// saw, so the chain has nowhere to start: the commit is refused stale
/// with the right delta and names nobody. Through the port the look
/// anchors instead, and the same scene names the person
/// (`through_the_port`; task-90 pinned this as the limit, task-91 moved
/// it there).
#[test]
fn without_a_look_a_click_on_a_new_row_leaves_the_chain_nowhere_to_start() {
    let mut rig = Rig::new(&["A", "B"], &["A", "B"]);
    let log = Log::default();
    rig.bench.add_sink(Box::new(log.clone()));
    rig.agent_writes("login and session");
    let seen = rig.look().revision();
    let commit = rig.observe(agent(), "commit.submit", "submit", None);
    let (_, unstage) = click_a_new_row_then_unstage(&mut rig, &log, seen);

    let id = rig.bench.dispatch(commit);
    assert_eq!(rig.stages(id), ["requested", "rejected"]);
    let receipt = rig.receipt(id);
    let Some(Stage::Rejected(Rejection::Stale { changes, acted })) = receipt.stage() else {
        panic!("stale: {:?}", receipt.stage());
    };
    assert!(matches!(
        changes.as_slice(),
        [Change::StateChanged { removed, .. }] if removed == &["B"]
    ));
    assert!(acted.is_empty(), "no look, no start: {acted:?}");
    assert_eq!(
        rig.notices(),
        ["Refused test-agent's submit: B left the staged files"]
    );
    rig.settle(unstage.id);
    assert_eq!(rig.git(&["diff", "--cached", "--name-only"]), "A\n");
}

// -- who acted since the agent looked, through the port (task-91) --

/// The agent through the ide's agent port, in process: the protocol's
/// server over the ide's own host (`idestudio::capability::agent::Served`, what the window
/// shell serves), with no socket. What it observes goes through the port
/// as the window's agent's does.
#[cfg(unix)]
mod through_the_port {
    use accord::{ActionName, Rejection, Revision, SemId, Stage, Value};
    use socket::{Act, Ask, ClientMsg, Observe, Reply, Server, StageReply};

    use super::support::{Rig, agent};
    use super::{Log, asked, click_a_new_row_then_unstage, frame, human};

    /// The one connection of the agent.
    const CONN: socket::ConnId = 1;

    struct Port {
        server: Server,
    }

    impl Port {
        /// A port on the workbench of `rig`, and the agent's hello.
        fn hello(rig: &mut Rig) -> Port {
            let mut port = Port {
                server: Server::new(idestudio::capability::agent::APP, 16),
            };
            let name = match agent() {
                accord::Actor::Agent { name } => name,
                other => panic!("an agent: {other:?}"),
            };
            match port.ask(rig, Ask::Hello { name }).as_slice() {
                [Reply::Welcome { .. }] => port,
                other => panic!("welcome: {other:?}"),
            }
        }

        /// One message of the agent, served through the ide's host with
        /// the frame on screen; the replies.
        fn ask(&mut self, rig: &mut Rig, ask: Ask) -> Vec<Reply> {
            let compositor = frame(rig);
            let mut host = idestudio::capability::agent::Served {
                bench: &mut rig.bench,
                compositor: &compositor,
                recorder: None,
            };
            let msg = ClientMsg {
                id: "1".into(),
                ask,
            };
            self.server
                .handle(CONN, msg, &mut host)
                .into_iter()
                .map(|o| o.msg.reply)
                .collect()
        }

        /// An observation of `scope` (the whole workbench without one):
        /// its revision.
        fn observe(&mut self, rig: &mut Rig, scope: Option<&str>) -> Revision {
            let o = Observe {
                scope: scope.map(|s| SemId::parse(s).expect("an id")),
                ..Observe::default()
            };
            match self.ask(rig, Ask::Observe(o)).as_slice() {
                [Reply::Snapshot(s)] => s.revision,
                other => panic!("a snapshot: {other:?}"),
            }
        }

        /// An act on `from`: its stages as the port sent them.
        fn act(
            &mut self,
            rig: &mut Rig,
            target: &str,
            action: &str,
            param: Option<Value>,
            from: Revision,
        ) -> Vec<StageReply> {
            let name = ActionName::parse(action).expect("an action");
            let mut act = Act::from_revision(SemId::parse(target).expect("an id"), name, from);
            if let Some(param) = param {
                act = act.with_param(param);
            }
            self.ask(rig, Ask::Act(act))
                .into_iter()
                .map(|reply| match reply {
                    Reply::Stage(s) => s,
                    other => panic!("a stage: {other:?}"),
                })
                .collect()
        }
    }

    fn names(stages: &[StageReply]) -> Vec<&'static str> {
        stages.iter().map(|s| s.stage.name()).collect()
    }

    /// `(id, (actor, target, action))` of each entry of the stale refusal
    /// the last stage of `stages` holds.
    fn named(stages: &[StageReply]) -> Vec<(accord::RequestId, (String, String, String))> {
        let Some(Stage::Rejected(Rejection::Stale { acted, .. })) = stages.last().map(|s| &s.stage)
        else {
            panic!("stale: {stages:?}");
        };
        acted
            .iter()
            .map(|a| {
                let who = (
                    a.actor.to_string(),
                    a.target.to_string(),
                    a.action.to_string(),
                );
                (a.id, who)
            })
            .collect()
    }

    /// The window scene of task-86 and task-90, with the agent's look and
    /// act through the port. The agent looks with no row selected; the
    /// person clicks B (the selection moves outside the dispatcher, then
    /// the diff is asked for) and unstages it from its menu; the agent's
    /// submit on what it looked at is refused stale and names both, since
    /// both ran after the look.
    #[test]
    fn a_click_on_a_new_row_and_an_unstage_are_named_since_the_agents_look() {
        let mut rig = Rig::new(&["A", "B"], &["A", "B"]);
        let log = Log::default();
        rig.bench.add_sink(Box::new(log.clone()));
        rig.agent_writes("login and session");
        let mut port = Port::hello(&mut rig);
        let seen = port.observe(&mut rig, None);
        let (show, unstage) = click_a_new_row_then_unstage(&mut rig, &log, seen);

        let stages = port.act(&mut rig, "commit.submit", "submit", None, seen);
        assert_eq!(names(&stages), ["requested", "rejected"]);
        assert_eq!(
            named(&stages),
            [(show.id, asked(&show)), (unstage.id, asked(&unstage))]
        );
        assert_eq!(
            rig.notices(),
            ["Refused test-agent's submit: B left the staged files \
                 (the person opened the diff of B and unstaged B)"]
        );
        rig.settle(unstage.id);
        assert_eq!(rig.git(&["diff", "--cached", "--name-only"]), "A\n");
    }

    /// The reproduction of task-89 with a stage's revision. The agent
    /// observes the form (the message); the person opens the diff of A;
    /// the agent observes `changes` (A and B staged); the person unstages
    /// B through the dispatcher; the agent writes the message again, and
    /// submits on the revision that act's last stage carried, which shows
    /// no state: the submit rests on the `staged` of the earlier
    /// observation and is refused stale. Named are the acts that ran since
    /// the oldest look the submit rests on (the form's, where its message
    /// comes from): the diff, the unstage and the agent's own write, which
    /// the toast says in the third person.
    #[test]
    fn a_submit_on_a_stage_revision_names_who_acted_since_the_look_it_rests_on() {
        let mut rig = Rig::new(&["A", "B"], &["A", "B"]);
        rig.agent_writes("login and session");
        let mut port = Port::hello(&mut rig);
        port.observe(&mut rig, Some("commit.form"));
        let show = rig.ask(accord::Actor::Human, "changes.file[A]", "show_diff", None);
        rig.until(|e| matches!(e, gitdrv::GitEvent::DiffWorkdir { .. }));
        let changes = port.observe(&mut rig, Some("changes"));
        let unstage = rig.ask(accord::Actor::Human, "changes.file[B]", "unstage", None);

        let text = Some(Value::from("login and session"));
        let set = port.act(&mut rig, "commit.message", "set_value", text, changes);
        assert_eq!(names(&set).last(), Some(&"completed"));
        let landed = set.last().and_then(|s| s.revision).expect("a revision");
        let submit = port.act(&mut rig, "commit.submit", "submit", None, landed);
        assert_eq!(names(&submit), ["requested", "rejected"]);
        let own = set[0].request;
        assert_eq!(
            named(&submit),
            [
                (show, human("changes.file[A]", "show_diff")),
                (unstage, human("changes.file[B]", "unstage")),
                (
                    own,
                    (
                        "agent:test-agent".into(),
                        "commit.message".into(),
                        "set_value".into()
                    )
                ),
            ]
        );
        assert_eq!(
            rig.notices(),
            ["Refused test-agent's submit: B left the staged files \
                 (the person opened the diff of A and unstaged B; \
                 test-agent itself edited the commit message)"]
        );
        rig.settle(unstage);
    }
}
