//! the JSON form of `Request`, `Receipt` and `Snapshot`, pinned: it is a
//! contract with the agent protocol (task-61) and the recorder (task-63).
//! a change here needs an ADR.

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

const SNAPSHOT: &str = r#"{
  "nodes": [
    {
      "id": "changes",
      "role": "list",
      "label": "changes",
      "description": null,
      "parent": null,
      "state": {
        "staged": {
          "kind": "set",
          "value": [
            "A",
            "B",
            "C"
          ]
        }
      },
      "actions": [],
      "constraints": {
        "editable": false,
        "required": false,
        "range": null,
        "step": null,
        "max_len": null,
        "options": null,
        "secret": false
      },
      "enabled": true,
      "inert": false,
      "focusable": false,
      "source": null
    },
    {
      "id": "commit.message",
      "role": "text_input",
      "label": "commit message",
      "description": null,
      "parent": null,
      "state": {
        "text": {
          "kind": "text",
          "value": "fix login"
        }
      },
      "actions": [],
      "constraints": {
        "editable": false,
        "required": false,
        "range": null,
        "step": null,
        "max_len": null,
        "options": null,
        "secret": false
      },
      "enabled": true,
      "inert": false,
      "focusable": true,
      "source": null
    },
    {
      "id": "commit.submit",
      "role": "button",
      "label": "commit",
      "description": null,
      "parent": null,
      "state": {},
      "actions": [
        {
          "name": "submit",
          "label": null,
          "enabled": true,
          "reads": [
            "changes:staged",
            "commit.message:text"
          ],
          "param": null,
          "policy": {
            "actors": {
              "human": true,
              "agent": true,
              "assistive": true,
              "test": true
            },
            "confirm": "never",
            "destructive": false
          }
        }
      ],
      "constraints": {
        "editable": false,
        "required": false,
        "range": null,
        "step": null,
        "max_len": null,
        "options": null,
        "secret": false
      },
      "enabled": true,
      "inert": false,
      "focusable": true,
      "source": null
    }
  ],
  "focused": null
}"#;

/// the snapshot the agent saw. `source` names a line of this file, which
/// is metadata and not contract, so the pinned text has it blanked; the
/// fields keep the order serde writes them in.
#[test]
fn a_snapshot_has_a_pinned_json_form_and_roundtrips() {
    let seen = git(&["A", "B", "C"], "fix login");
    let text = serde_json::to_string(&seen).unwrap();
    let back: Snapshot = serde_json::from_str(&text).unwrap();
    assert_eq!(back, seen);
    assert_eq!(back.revision(), seen.revision());

    let mut wire = serde_json::to_value(&seen).unwrap();
    for node in wire["nodes"].as_array_mut().unwrap() {
        assert!(node["source"]["line"].is_u64());
        node["source"] = serde_json::Value::Null;
    }
    let unsourced: Snapshot = serde_json::from_value(wire).unwrap();
    assert_eq!(unsourced, seen);
    assert_eq!(serde_json::to_string_pretty(&unsourced).unwrap(), SNAPSHOT);
}
