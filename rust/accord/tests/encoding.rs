//! the JSON form of `Request`, `Receipt` and `Snapshot`, pinned: it is a
//! contract with the agent protocol (task-61) and the recorder (task-63).
//! a change here needs an ADR.

#![cfg(feature = "serde")]

use std::collections::BTreeSet;

use indexmap::IndexMap;
use accord::{
    Acted, ActionDecl, Actor, Actors, Change, Confirm, Constraints, Outcome, Policy, Receipt,
    Rejection, Request, RequestId, Role, Snapshot, SnapshotBuilder, Stage, Value, Verdict, decision,
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

/// the thesis request waited for a person, who declined it.
fn declined_receipt() -> Receipt {
    Receipt {
        id: RequestId::parse("3").unwrap(),
        request: thesis_request(),
        validated_against: Some(git(&["A", "B", "C"], "fix login").revision()),
        stages: vec![
            (Stage::Requested, 1200),
            (Stage::AwaitingConfirmation, 1200),
            (
                Stage::Rejected(Rejection::Declined { by: Actor::Human }),
                1500,
            ),
        ],
    }
}

const DECLINED: &str = r#"{
  "id": "3",
  "request": {
    "actor": {
      "agent": {
        "name": "reviewer"
      }
    },
    "target": "commit.submit",
    "action": "submit",
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
  "validated_against": "3ba694e7d3941ec0",
  "stages": [
    [
      "requested",
      1200
    ],
    [
      "awaiting_confirmation",
      1200
    ],
    [
      {
        "rejected": {
          "declined": {
            "by": "human"
          }
        }
      },
      1500
    ]
  ]
}"#;

#[test]
fn a_declined_receipt_has_a_pinned_json_form_and_roundtrips() {
    let receipt = declined_receipt();
    let text = serde_json::to_string_pretty(&receipt).unwrap();
    assert_eq!(text, DECLINED);
    let back: Receipt = serde_json::from_str(&text).unwrap();
    assert_eq!(back, receipt);

    let assistive = Rejection::Declined {
        by: Actor::Assistive,
    };
    let json = r#"{"declined":{"by":"assistive"}}"#;
    assert_eq!(serde_json::to_string(&assistive).unwrap(), json);
    assert_eq!(serde_json::from_str::<Rejection>(json).unwrap(), assistive);
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

/// the form before task-57b: every default spelled out, `source: null`.
/// it is not written any more, and it still reads.
const OLD_SNAPSHOT: &str = r#"{
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

/// the lean form: `id`, `role` and `label` always; every other field only
/// when it is not at its default; `focused` only when there is one.
const SNAPSHOT: &str = r#"{
  "nodes": [
    {
      "id": "changes",
      "role": "list",
      "label": "changes",
      "state": {
        "staged": {
          "kind": "set",
          "value": [
            "A",
            "B",
            "C"
          ]
        }
      }
    },
    {
      "id": "commit.message",
      "role": "text_input",
      "label": "commit message",
      "state": {
        "text": {
          "kind": "text",
          "value": "fix login"
        }
      },
      "focusable": true
    },
    {
      "id": "commit.submit",
      "role": "button",
      "label": "commit",
      "actions": [
        {
          "name": "submit",
          "reads": [
            "changes:staged",
            "commit.message:text"
          ]
        }
      ],
      "focusable": true
    }
  ]
}"#;

/// the snapshot the agent saw. `source` names a line of this file, which
/// is metadata and not contract, so the pinned text has it removed; the
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
        node.as_object_mut().unwrap().remove("source");
    }
    let unsourced: Snapshot = serde_json::from_value(wire).unwrap();
    assert_eq!(unsourced, seen);
    assert_eq!(serde_json::to_string_pretty(&unsourced).unwrap(), SNAPSHOT);
    for key in [
        "description",
        "parent",
        "constraints",
        "enabled",
        "inert",
        "source",
    ] {
        assert!(
            !SNAPSHOT.contains(key),
            "{key} is at its default everywhere"
        );
    }
}

/// a snapshot written before task-57b reads as the same snapshot, with
/// the same revision as the one projected now.
#[test]
fn the_form_before_the_lean_one_still_reads() {
    let seen = git(&["A", "B", "C"], "fix login");
    let old: Snapshot = serde_json::from_str(OLD_SNAPSHOT).unwrap();
    assert_eq!(old, seen);
    assert_eq!(old.revision(), seen.revision());
    let lean: Snapshot = serde_json::from_str(SNAPSHOT).unwrap();
    assert_eq!(lean, old);
    assert_eq!(lean.revision(), seen.revision());
    assert_eq!(serde_json::to_string_pretty(&old).unwrap(), SNAPSHOT);

    // a request with `"param": null`, as it was written, reads too.
    let with_null = REQUEST.replacen("\"observed\"", "\"param\": null,\n  \"observed\"", 1);
    assert_ne!(with_null, REQUEST);
    let back: Request = serde_json::from_str(&with_null).unwrap();
    assert_eq!(back, thesis_request());
}

/// what is left out reads as its default, field by field.
#[test]
fn a_field_left_out_reads_at_its_default() {
    let bare: Snapshot =
        serde_json::from_str(r#"{"nodes":[{"id":"x","role":"button","label":"x"}]}"#).unwrap();
    assert_eq!(bare.focused(), None);
    let node = bare.get(&"x".parse().unwrap()).unwrap();
    assert_eq!(node.description, None);
    assert_eq!(node.parent, None);
    assert!(node.state.is_empty() && node.actions.is_empty());
    assert_eq!(node.constraints, Constraints::default());
    assert!(node.enabled && !node.inert && !node.focusable);
    assert_eq!(node.source, None);

    let partial = r#"{"nodes":[{"id":"x","role":"slider","label":"x",
        "constraints":{"step":0.5},
        "actions":[
            {"name":"go"},
            {"name":"drop","enabled":false,"policy":{"destructive":true}},
            {"name":"ask","policy":{"confirm":"for_agents","actors":{"human":true,"agent":true,"assistive":false,"test":true}}}
        ]}],"focused":"x"}"#;
    let snap: Snapshot = serde_json::from_str(partial).unwrap();
    assert_eq!(snap.focused().map(|f| f.as_str()), Some("x"));
    let node = snap.get(&"x".parse().unwrap()).unwrap();
    assert_eq!(
        node.constraints,
        Constraints {
            step: Some(0.5),
            ..Constraints::default()
        }
    );
    assert_eq!(node.actions[0], ActionDecl::new("go".parse().unwrap()));
    assert_eq!(
        node.actions[1],
        ActionDecl {
            enabled: false,
            policy: Policy {
                destructive: true,
                ..Policy::default()
            },
            ..ActionDecl::new("drop".parse().unwrap())
        }
    );
    assert_eq!(
        node.actions[2].policy,
        Policy {
            actors: Actors {
                assistive: false,
                ..Actors::default()
            },
            confirm: Confirm::ForAgents,
            destructive: false,
        }
    );
    // and written back, only what is not at its default appears.
    let text = serde_json::to_string(&snap).unwrap();
    assert!(text.contains(r#"{"name":"go"}"#), "{text}");
    assert!(
        text.contains(r#"{"name":"drop","enabled":false,"policy":{"destructive":true}}"#),
        "{text}"
    );
    assert!(
        text.contains(r#""actors":{"human":true,"agent":true,"assistive":false,"test":true}"#),
        "a policy that leaves an actor out writes them all: {text}"
    );
    assert!(text.ends_with(r#""focused":"x"}"#), "{text}");
}

/// `source` is not contract, but it is kept when the node has one: the
/// recorder needs it to go from a replay back to the code.
#[test]
fn a_node_with_a_source_keeps_it_through_the_wire() {
    let seen = git(&["A"], "fix login");
    let text = serde_json::to_string(&seen).unwrap();
    assert!(text.contains(r#""source":{"file":"#), "{text}");
    let back: Snapshot = serde_json::from_str(&text).unwrap();
    for (id, node) in seen.nodes() {
        let kept = back.get(id).and_then(|n| n.source.clone());
        assert!(kept.is_some(), "{id}");
        assert_eq!(kept, node.source, "{id}");
    }
    assert_eq!(back.revision(), seen.revision());
}

/// a node that came, in a delta: the node in the lean form of a snapshot,
/// under `node_added` (task-83). `source` is removed from the pinned text
/// for the reason the snapshot's is.
const NODE_ADDED: &str = r#"{
  "node_added": {
    "id": "commit.submit",
    "role": "button",
    "label": "commit",
    "actions": [
      {
        "name": "submit",
        "reads": [
          "changes:staged",
          "commit.message:text"
        ]
      }
    ],
    "focusable": true
  }
}"#;

#[test]
fn a_node_added_has_a_pinned_json_form_and_roundtrips() {
    let mut b = SnapshotBuilder::new();
    b.node("changes", Role::List, "changes")
        .state("staged", set(&["A", "B", "C"]));
    b.node("commit.message", Role::TextInput, "commit message")
        .state("text", "fix login");
    let before = b.finish().unwrap();
    let after = git(&["A", "B", "C"], "fix login");
    let delta = Snapshot::diff(&before, &after);
    let [added @ Change::NodeAdded(node)] = &delta[..] else {
        panic!("one node added: {delta:?}");
    };
    assert!(node.source.is_some(), "the node as after holds it");

    let text = serde_json::to_string(added).unwrap();
    assert!(
        text.starts_with(r#"{"node_added":{"id":"commit.submit","#),
        "{text}"
    );
    assert_eq!(&serde_json::from_str::<Change>(&text).unwrap(), added);

    let mut wire = serde_json::to_value(added).unwrap();
    let node = wire["node_added"].as_object_mut().unwrap();
    assert!(node.remove("source").is_some_and(|s| s["line"].is_u64()));
    let unsourced: Change = serde_json::from_value(wire).unwrap();
    assert_eq!(
        unsourced, *added,
        "source is not part of the node's meaning"
    );
    assert_eq!(
        serde_json::to_string_pretty(&unsourced).unwrap(),
        NODE_ADDED
    );

    // the form before, a bare id, is not a change: without the snapshot
    // that holds it there is no node. a session reader, which has that
    // snapshot, converts it (crates/rec/src/persist.rs).
    assert!(serde_json::from_str::<Change>(r#"{"node_added":"commit.submit"}"#).is_err());
}

/// the thesis refusal naming who acted: the person's unstage, request 3.
/// ids, an actor, a target and an action name; no value.
const STALE_ACTED: &str = r#"{
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
    ],
    "acted": [
      {
        "id": "3",
        "actor": "human",
        "target": "changes",
        "action": "unstage"
      }
    ]
  }
}"#;

/// `acted` is written only when someone is named, and reads back either
/// way; a refusal written before task-83 (no `acted`) reads as one that
/// names nobody.
#[test]
fn a_stale_refusal_roundtrips_with_and_without_who_acted() {
    let Verdict::Rejected(bare) = check(&thesis_request(), &git(&["A", "C"], "fix login")) else {
        panic!("the thesis request must be rejected");
    };
    let Rejection::Stale { changes, acted } = &bare else {
        panic!("stale: {bare:?}");
    };
    assert!(acted.is_empty(), "check names nobody");
    assert_eq!(bare, Rejection::stale(changes.clone()));
    let text = serde_json::to_string(&bare).unwrap();
    assert!(!text.contains("acted"), "{text}");
    assert_eq!(serde_json::from_str::<Rejection>(&text).unwrap(), bare);

    let named = Rejection::Stale {
        changes: changes.clone(),
        acted: vec![Acted {
            id: RequestId::parse("3").unwrap(),
            actor: Actor::Human,
            target: "changes".parse().unwrap(),
            action: "unstage".parse().unwrap(),
        }],
    };
    let text = serde_json::to_string_pretty(&named).unwrap();
    assert_eq!(text, STALE_ACTED);
    assert_eq!(serde_json::from_str::<Rejection>(&text).unwrap(), named);
}

/// an agent's question to a person (task-92) is written after the
/// observation, and only when there is one: the request without it keeps
/// the form pinned above, and a request written before the field existed
/// reads with no question.
#[test]
fn a_question_travels_only_when_there_is_one() {
    let req = thesis_request().asking("B left the index; commit A and C?");
    let text = serde_json::to_string(&req).unwrap();
    assert!(
        text.ends_with(r#"]]},"ask":"B left the index; commit A and C?"}"#),
        "{text}"
    );
    let back: Request = serde_json::from_str(&text).unwrap();
    assert_eq!(back, req);

    let old: Request = serde_json::from_str(REQUEST).unwrap();
    assert_eq!(old.ask, None);
    assert_eq!(old, thesis_request());
    assert!(!serde_json::to_string(&old).unwrap().contains("ask"));
}
