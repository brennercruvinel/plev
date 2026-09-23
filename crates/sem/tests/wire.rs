//! the JSON form of `Request`, pinned: it is a contract with the agent
//! protocol (task-61) and the recorder (task-63). a change here needs an
//! ADR.

#![cfg(feature = "serde")]

use std::collections::BTreeSet;

use sem::{Actor, Request, Role, Snapshot, SnapshotBuilder, Value};

fn set(items: &[&str]) -> Value {
    Value::Set(items.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>())
}

fn git(staged: &[&str], message: &str) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("changes", Role::List, "changes")
        .state("staged", set(staged));
    b.node("commit.message", Role::TextInput, "commit message")
        .state("text", message);
    b.node("commit.submit", Role::Button, "commit")
        .action("submit")
        .reads("changes:staged")
        .reads("commit.message:text");
    b.finish().unwrap()
}

fn thesis_request() -> Request {
    let seen = git(&["A", "B", "C"], "fix login");
    let agent = Actor::Agent {
        name: "reviewer".into(),
    };
    Request::observe(&seen, agent, "commit.submit", "submit", None).unwrap()
}

const REQUEST: &str = r#"{
  "actor": {
    "agent": {
      "name": "reviewer"
    }
  },
  "target": "commit.submit",
  "action": "submit",
  "param": null,
  "observed": {
    "revision": "3ba694e7d3941ec0",
    "values": [
      [
        "changes:staged",
        {
          "kind": "set",
          "value": [
            "A",
            "B",
            "C"
          ]
        }
      ],
      [
        "commit.message:text",
        {
          "kind": "text",
          "value": "fix login"
        }
      ]
    ]
  }
}"#;

#[test]
fn a_request_has_a_pinned_json_form_and_roundtrips() {
    let req = thesis_request();
    let text = serde_json::to_string_pretty(&req).unwrap();
    assert_eq!(text, REQUEST);
    let back: Request = serde_json::from_str(&text).unwrap();
    assert_eq!(back, req);
}

#[test]
fn every_actor_roundtrips_with_a_snake_case_tag() {
    for (actor, json) in [
        (Actor::Human, r#""human""#),
        (Actor::Assistive, r#""assistive""#),
        (
            Actor::Agent { name: "a".into() },
            r#"{"agent":{"name":"a"}}"#,
        ),
        (Actor::Test { name: "t".into() }, r#"{"test":{"name":"t"}}"#),
    ] {
        assert_eq!(serde_json::to_string(&actor).unwrap(), json);
        assert_eq!(serde_json::from_str::<Actor>(json).unwrap(), actor);
    }
}
