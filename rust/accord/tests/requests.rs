//! `Request::observe`: the preconditions an actor carries, `Actor`.

use std::collections::BTreeSet;

use accord::{
    ActionName, Actor, Constraints, Request, Role, SemError, SemId, Snapshot, SnapshotBuilder,
    StateRef, Value,
};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("valid id")
}

fn at(text: &str) -> StateRef {
    StateRef::parse(text).expect("valid state ref")
}

fn set(items: &[&str]) -> Value {
    Value::Set(items.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>())
}

fn agent() -> Actor {
    Actor::Agent {
        name: "reviewer".into(),
    }
}

/// the git client of the thesis: staged files, a list cursor, a message
/// and the commit action that reads the files and the message.
fn git(staged: &[&str], message: &str) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("changes", Role::List, "changes")
        .state("staged", set(staged))
        .state("cursor", 0i64);
    b.node("commit.message", Role::TextInput, "commit message")
        .state("text", message);
    b.node("commit.submit", Role::Button, "commit")
        .action("submit")
        .reads("changes:staged")
        .reads("commit.message:text");
    b.finish().unwrap()
}

#[test]
fn observe_keeps_the_value_of_every_read_and_the_revision() {
    let seen = git(&["A", "B", "C"], "fix login");
    let req = Request::observe(&seen, agent(), "commit.submit", "submit", None).unwrap();
    assert_eq!(req.actor, agent());
    assert_eq!(req.target, id("commit.submit"));
    assert_eq!(req.action.as_str(), "submit");
    assert_eq!(req.param, None);
    assert_eq!(req.observed.revision, seen.revision());
    assert_eq!(
        req.observed.values,
        [
            (at("changes:staged"), Some(set(&["A", "B", "C"]))),
            (
                at("commit.message:text"),
                Some(Value::Text("fix login".into()))
            ),
        ]
    );
}

#[test]
fn observe_leaves_out_state_the_action_does_not_read() {
    let seen = git(&["A"], "m");
    let req = Request::observe(&seen, Actor::Human, "commit.submit", "submit", None).unwrap();
    assert!(
        req.observed
            .values
            .iter()
            .all(|(r, _)| *r != at("changes:cursor"))
    );
}

#[test]
fn observe_takes_target_and_action_typed_or_as_text() {
    let seen = git(&["A"], "m");
    let by_text = Request::observe(
        &seen,
        Actor::Human,
        "commit.submit",
        "submit",
        Some(Value::Int(1)),
    )
    .unwrap();
    let typed = Request::observe(
        &seen,
        Actor::Human,
        id("commit.submit"),
        ActionName::parse("submit").unwrap(),
        Some(Value::Int(1)),
    )
    .unwrap();
    let borrowed = Request::observe(
        &seen,
        Actor::Human,
        &id("commit.submit"),
        &ActionName::parse("submit").unwrap(),
        Some(Value::Int(1)),
    )
    .unwrap();
    assert_eq!(by_text, typed);
    assert_eq!(by_text, borrowed);
}

#[test]
fn observe_rejects_a_node_or_an_action_the_snapshot_does_not_offer() {
    let seen = git(&["A"], "m");
    assert_eq!(
        Request::observe(&seen, agent(), "ghost", "submit", None),
        Err(SemError::UnknownAction {
            node: id("ghost"),
            action: ActionName::parse("submit").unwrap(),
        })
    );
    assert_eq!(
        Request::observe(&seen, agent(), "changes", "submit", None),
        Err(SemError::UnknownAction {
            node: id("changes"),
            action: ActionName::parse("submit").unwrap(),
        })
    );
}

#[test]
fn observe_rejects_text_that_does_not_parse() {
    let seen = git(&["A"], "m");
    assert!(matches!(
        Request::observe(&seen, agent(), "Commit", "submit", None),
        Err(SemError::InvalidId { text, .. }) if text == "Commit"
    ));
    assert!(matches!(
        Request::observe(&seen, agent(), "commit.submit", "Submit", None),
        Err(SemError::InvalidKey { text, .. }) if text == "Submit"
    ));
}

#[test]
fn a_secret_read_is_observed_as_its_length() {
    let mut b = SnapshotBuilder::new();
    b.node("login.password", Role::TextInput, "password")
        .constraints(Constraints {
            editable: true,
            secret: true,
            ..Constraints::default()
        })
        .state("text", "hunter2");
    b.node("login.submit", Role::Button, "sign in")
        .action("submit")
        .reads("login.password:text");
    let seen = b.finish().unwrap();
    let req = Request::observe(&seen, Actor::Human, "login.submit", "submit", None).unwrap();
    assert_eq!(
        req.observed.values,
        [(at("login.password:text"), Some(Value::Int(7)))]
    );
}

#[cfg(feature = "serde")]
#[test]
fn observe_on_a_snapshot_whose_read_has_no_value_is_dangling_read() {
    let mut wire = serde_json::to_value(git(&["A"], "m")).unwrap();
    let state = &mut wire["nodes"][1]["state"];
    assert!(state.as_object_mut().unwrap().remove("text").is_some());
    let seen: Snapshot = serde_json::from_value(wire).unwrap();
    assert_eq!(
        Request::observe(&seen, agent(), "commit.submit", "submit", None),
        Err(SemError::DanglingRead {
            node: id("commit.submit"),
            action: ActionName::parse("submit").unwrap(),
            read: at("commit.message:text"),
        })
    );
}

#[test]
fn an_actor_prints_as_its_kind_and_name() {
    assert_eq!(Actor::Human.to_string(), "human");
    assert_eq!(agent().to_string(), "agent:reviewer");
    assert_eq!(Actor::Assistive.to_string(), "assistive");
    assert_eq!(
        Actor::Test {
            name: "thesis".into()
        }
        .to_string(),
        "test:thesis"
    );
}
