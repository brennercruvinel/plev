//! A person and an agent operate the same git client at the same time,
//! against a real repository and the real git worker: the agent's commit
//! is checked on what it observed at the dispatcher and again at the
//! index right before `git commit`, a valid commit comes back with its sha
//! and files, and every step has a receipt that says who asked.

mod support;

use sem::{Stage, Value};
use support::{Rig, agent, completed};

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
    assert_eq!(rig.value("changes:staged"), support::set(&[]));
}
