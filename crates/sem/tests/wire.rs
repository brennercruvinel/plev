//! the JSON form of `Request` and `Receipt`, pinned: it is a contract with
//! the agent protocol (task-61) and the recorder (task-63). a change here
//! needs an ADR.

#![cfg(feature = "serde")]

use std::collections::BTreeSet;

use indexmap::IndexMap;
use sem::{
    Actor, Outcome, Receipt, Request, RequestId, Role, Snapshot, SnapshotBuilder, Stage, Value,
    Verdict, check,
};

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

/// the thesis: the human unstaged B before the agent's commit arrived.
fn thesis_receipt() -> Receipt {
    let request = thesis_request();
    let now = git(&["A", "C"], "fix login");
    let Verdict::Rejected(why) = check(&request, &now) else {
        panic!("the thesis request must be rejected");
    };
    Receipt {
        id: RequestId::parse("1").unwrap(),
        request,
        validated_against: Some(now.revision()),
        stages: vec![(Stage::Requested, 1200), (Stage::Rejected(why), 1200)],
    }
}

const RECEIPT: &str = r#"{
  "id": "1",
  "request": {
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
  },
  "validated_against": "086c720b8c6b2780",
  "stages": [
    [
      "requested",
      1200
    ],
    [
      {
        "rejected": {
          "stale": {
            "changes": [
              {
                "state_changed": {
                  "at": "changes:staged",
                  "before": {
                    "kind": "set",
                    "value": [
                      "A",
                      "B",
                      "C"
                    ]
                  },
                  "after": {
                    "kind": "set",
                    "value": [
                      "A",
                      "C"
                    ]
                  },
                  "added": [],
                  "removed": [
                    "B"
                  ]
                }
              }
            ]
          }
        }
      },
      1200
    ]
  ]
}"#;

#[test]
fn a_receipt_has_a_pinned_json_form_and_roundtrips() {
    let receipt = thesis_receipt();
    let text = serde_json::to_string_pretty(&receipt).unwrap();
    assert_eq!(text, RECEIPT);
    let back: Receipt = serde_json::from_str(&text).unwrap();
    assert_eq!(back, receipt);
}

#[test]
fn a_completed_receipt_roundtrips_with_its_outcome() {
    let receipt = Receipt {
        id: RequestId::parse("2").unwrap(),
        request: thesis_request(),
        validated_against: None,
        stages: vec![
            (Stage::Requested, 1),
            (Stage::AwaitingConfirmation, 1),
            (Stage::Confirmed { by: Actor::Human }, 7),
            (Stage::Executing, 7),
            (
                Stage::Completed(Outcome {
                    result: IndexMap::from([
                        ("commit".to_owned(), Value::Text("c1".into())),
                        ("files".to_owned(), set(&["A", "B", "C"])),
                    ]),
                }),
                9,
            ),
        ],
    };
    let text = serde_json::to_string(&receipt).unwrap();
    assert!(text.contains(r#""id":"2""#), "{text}");
    assert!(text.contains(r#""validated_against":null"#), "{text}");
    assert!(
        text.contains(r#"[{"confirmed":{"by":"human"}},7]"#),
        "{text}"
    );
    assert_eq!(serde_json::from_str::<Receipt>(&text).unwrap(), receipt);
}
