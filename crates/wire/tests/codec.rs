//! the codec of protocol v1: every message roundtrips, the envelope comes
//! first, a wrong version or a broken line is refused with the reason, and
//! no line whatsoever panics.

use std::collections::BTreeSet;

use proptest::prelude::*;
use sem::{
    ActionName, Actor, Change, Observation, Outcome, Rejection, Request, RequestId, Revision, Role,
    SemId, SnapshotBuilder, Stage, StateKey, StateRef, Value,
};
use wire::{
    Act, Ask, Bound, ClientMsg, DeltaReply, Observe, ProtoError, Refusal, Reply, ServerMsg,
    SnapshotReply, StageReply, decode_client, decode_server, encode_client, encode_server,
};

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn action(text: &str) -> ActionName {
    ActionName::parse(text).unwrap()
}

fn revision(hex: &str) -> Revision {
    serde_json::from_value(serde_json::Value::String(hex.to_owned())).unwrap()
}

fn request_id(n: &str) -> RequestId {
    RequestId::parse(n).unwrap()
}

fn snapshot() -> sem::Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("todo", Role::Region, "todo");
    b.node("todo.input", Role::TextInput, "new todo")
        .parent("todo")
        .state("text", "buy milk")
        .action("submit")
        .reads("todo.input:text");
    b.node("todo.item[1]", Role::ListItem, "call mom")
        .parent("todo")
        .state("completed", false)
        .action("toggle")
        .reads("todo.item[1]:completed");
    b.finish().unwrap()
}

fn every_ask() -> Vec<Ask> {
    let seen = snapshot();
    let observed = Request::observe(
        &seen,
        Actor::Agent { name: "a".into() },
        "todo.input",
        "submit",
        None,
    )
    .unwrap()
    .observed;
    vec![
        Ask::Hello {
            name: "claude".into(),
        },
        Ask::Observe(Observe::default()),
        Ask::Observe(Observe {
            scope: Some(sid("todo")),
            keys: Some(vec![StateKey::parse("completed").unwrap()]),
            since: Some(seen.revision()),
            bounds: true,
            source: true,
        }),
        Ask::Act(Act::from_revision(
            sid("todo.item[1]"),
            action("toggle"),
            seen.revision(),
        )),
        Ask::Act(
            Act::observed(sid("todo.input"), action("set_value"), observed)
                .with_param(Value::Text("call mom".into())),
        ),
        Ask::Confirm {
            request: request_id("7"),
        },
        Ask::Decline {
            request: request_id("7"),
        },
    ]
}

fn every_reply() -> Vec<Reply> {
    let seen = snapshot();
    let req = Request::observe(
        &seen,
        Actor::Agent { name: "a".into() },
        "todo.item[1]",
        "toggle",
        None,
    )
    .unwrap();
    let stale = Change::StateChanged {
        at: StateRef::parse("todo.input:text").unwrap(),
        before: Some(Value::Text("buy milk".into())),
        after: Some(Value::Text("buy bread".into())),
        added: Vec::new(),
        removed: Vec::new(),
    };
    let stage = |stage: Stage| StageReply {
        request: request_id("3"),
        stage,
        at: 1200,
        asked: None,
    };
    vec![
        Reply::Welcome {
            app: "showcase".into(),
            actor: Actor::Agent {
                name: "claude".into(),
            },
        },
        Reply::Snapshot(SnapshotReply {
            revision: seen.revision(),
            nodes: seen.nodes().values().cloned().collect(),
            focused: Some(sid("todo.input")),
            bounds: None,
        }),
        Reply::Snapshot(SnapshotReply {
            revision: seen.revision(),
            nodes: Vec::new(),
            focused: None,
            bounds: Some(vec![Bound {
                id: sid("todo"),
                rect: [1.0, 2.5, 300.0, 40.0],
            }]),
        }),
        Reply::Delta(DeltaReply {
            base: revision("00000000000000ff"),
            revision: seen.revision(),
            changes: vec![
                Change::NodeAdded(sid("todo.item[2]")),
                stale.clone(),
                Change::FocusChanged {
                    before: None,
                    after: Some(sid("todo.input")),
                },
            ],
            bounds: None,
        }),
        Reply::Stage(StageReply {
            asked: Some(req),
            ..stage(Stage::Requested)
        }),
        Reply::Stage(stage(Stage::Accepted)),
        Reply::Stage(stage(Stage::AwaitingConfirmation)),
        Reply::Stage(stage(Stage::Executing)),
        Reply::Stage(stage(Stage::Rejected(Rejection::Stale {
            changes: vec![stale],
        }))),
        Reply::Stage(stage(Stage::Completed(Outcome {
            result: [("item".to_owned(), Value::Set(BTreeSet::from(["x".into()])))]
                .into_iter()
                .collect(),
        }))),
        Reply::Stage(stage(Stage::Failed { error: "no".into() })),
        Reply::Error(ProtoError::BadVersion { got: "2".into() }),
        Reply::Error(ProtoError::BadMessage {
            reason: "why".into(),
        }),
        Reply::Error(ProtoError::UnknownRevision {
            revision: seen.revision(),
        }),
        Reply::Error(ProtoError::NotEnabled {
            reason: "why".into(),
        }),
    ]
}

#[test]
fn every_client_message_roundtrips_with_the_envelope_first() {
    for (n, ask) in every_ask().into_iter().enumerate() {
        let msg = ClientMsg {
            id: format!("c{n}"),
            ask,
        };
        let line = encode_client(&msg);
        assert!(!line.contains('\n'), "{line}");
        assert!(
            line.starts_with(&format!(r#"{{"v":1,"id":"c{n}","type":""#)),
            "{line}"
        );
        assert_eq!(decode_client(&line), Ok(msg), "{line}");
    }
}

#[test]
fn every_server_message_roundtrips_with_the_envelope_first() {
    for (n, reply) in every_reply().into_iter().enumerate() {
        let msg = ServerMsg {
            id: Some(format!("s{n}")),
            reply,
        };
        let line = encode_server(&msg);
        assert!(!line.contains('\n'), "{line}");
        assert!(
            line.starts_with(&format!(r#"{{"v":1,"id":"s{n}","type":""#)),
            "{line}"
        );
        assert_eq!(decode_server(&line), Ok(msg), "{line}");
    }
    let orphan = ServerMsg {
        id: None,
        reply: Reply::Error(ProtoError::BadMessage {
            reason: "not JSON".into(),
        }),
    };
    let line = encode_server(&orphan);
    assert!(
        line.starts_with(r#"{"v":1,"id":null,"type":"error""#),
        "{line}"
    );
    assert_eq!(decode_server(&line), Ok(orphan));
}

/// the forms an agent writes by hand, pinned.
#[test]
fn the_forms_an_agent_writes_are_pinned() {
    let act = r#"{"v":1,"id":"3","type":"act","target":"todo.item[1]","action":"toggle","from_revision":"3ba694e7d3941ec0"}"#;
    let msg = ClientMsg {
        id: "3".into(),
        ask: Ask::Act(Act::from_revision(
            sid("todo.item[1]"),
            action("toggle"),
            revision("3ba694e7d3941ec0"),
        )),
    };
    assert_eq!(decode_client(act), Ok(msg.clone()));
    assert_eq!(encode_client(&msg), act);

    let observe = r#"{"v":1,"id":"o","type":"observe","scope":"todo","keys":["text","completed"]}"#;
    let Ok(ClientMsg {
        ask: Ask::Observe(o),
        ..
    }) = decode_client(observe)
    else {
        panic!("an observe");
    };
    assert_eq!(o.scope, Some(sid("todo")));
    assert_eq!(o.keys.map(|k| k.len()), Some(2));
    assert!(!o.bounds && !o.source && o.since.is_none());

    let param = r#"{"v":1,"id":"p","type":"act","target":"todo.input","action":"set_value","param":{"kind":"text","value":"x"},"observed":{"revision":"0000000000000001","values":[["todo.input:text",{"kind":"text","value":""}]]}}"#;
    let Ok(ClientMsg {
        ask: Ask::Act(a), ..
    }) = decode_client(param)
    else {
        panic!("an act");
    };
    assert_eq!(a.param, Some(Value::Text("x".into())));
    let Some(Observation { values, .. }) = a.observed else {
        panic!("observed");
    };
    assert_eq!(values[0].1, Some(Value::Text(String::new())));

    let stage = r#"{"v":1,"id":"3","type":"stage","request":"4","stage":"accepted","at":1200}"#;
    assert_eq!(
        decode_server(stage).map(|m| m.reply),
        Ok(Reply::Stage(StageReply {
            request: request_id("4"),
            stage: Stage::Accepted,
            at: 1200,
            asked: None,
        }))
    );
    let error = r#"{"v":1,"id":"9","type":"error","error":"unknown_revision","revision":"3ba694e7d3941ec0"}"#;
    let back = ServerMsg {
        id: Some("9".into()),
        reply: Reply::Error(ProtoError::UnknownRevision {
            revision: revision("3ba694e7d3941ec0"),
        }),
    };
    assert_eq!(encode_server(&back), error);
}

fn refused(line: &str) -> Refusal {
    match decode_client(line) {
        Err(r) => r,
        Ok(m) => panic!("{line} was taken as {m:?}"),
    }
}

#[test]
fn a_wrong_version_is_refused_before_the_shape() {
    for (line, got) in [
        (r#"{"v":2,"id":"a","type":"hello","name":"x"}"#, "2"),
        (r#"{"v":2,"id":"a","type":"teleport"}"#, "2"),
        (r#"{"v":"1","id":"a","type":"hello","name":"x"}"#, "\"1\""),
        (r#"{"v":1.5,"id":"a","type":"hello","name":"x"}"#, "1.5"),
        (r#"{"id":"a","type":"hello","name":"x"}"#, "missing"),
    ] {
        let r = refused(line);
        assert_eq!(r.id.as_deref(), Some("a"), "{line}");
        assert_eq!(
            r.error,
            ProtoError::BadVersion { got: got.into() },
            "{line}"
        );
    }
}

#[test]
fn a_broken_line_is_a_bad_message_with_the_reason() {
    for (line, id, needle) in [
        ("", None, "not JSON"),
        ("{", None, "not JSON"),
        ("hello", None, "not JSON"),
        ("[1,2]", None, "object"),
        ("42", None, "object"),
        (r#"{"v":1,"type":"hello","name":"x"}"#, None, "id"),
        (r#"{"v":1,"id":7,"type":"hello","name":"x"}"#, None, "id"),
        (r#"{"v":1,"id":"a"}"#, Some("a"), "type"),
        (
            r#"{"v":1,"id":"a","type":"teleport"}"#,
            Some("a"),
            "teleport",
        ),
        (r#"{"v":1,"id":"a","type":"hello"}"#, Some("a"), "name"),
        (
            r#"{"v":1,"id":"a","type":"act","target":"Todo!","action":"toggle"}"#,
            Some("a"),
            "Todo!",
        ),
        (
            r#"{"v":1,"id":"a","type":"observe","since":"xyz"}"#,
            Some("a"),
            "revision",
        ),
        (
            r#"{"v":1,"id":"a","type":"confirm","request":"007"}"#,
            Some("a"),
            "request id",
        ),
        (
            r#"{"v":1,"id":"a","type":"act","target":"t","action":"a","param":{"kind":"bool","value":"yes"}}"#,
            Some("a"),
            "bool",
        ),
    ] {
        let r = refused(line);
        assert_eq!(r.id.as_deref(), id, "{line}");
        match &r.error {
            ProtoError::BadMessage { reason } => {
                assert!(
                    reason.contains(needle),
                    "{line}: {reason:?} lacks {needle:?}"
                )
            }
            other => panic!("{line}: {other:?}"),
        }
        let reply = encode_server(&r.reply());
        assert!(
            reply.contains(r#""type":"error","error":"bad_message""#),
            "{reply}"
        );
    }
}

/// strings made of the protocol's own pieces, so the fuzz reaches past the
/// JSON parser into the shapes.
fn piece() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r#"{"v":1,"id":"x","type":"#.to_owned()),
        Just(r#""act","target":"todo.item[1]","action":"toggle""#.to_owned()),
        Just(r#""observe","scope":"todo","since":"#.to_owned()),
        Just(r#""0000000000000001""#.to_owned()),
        Just(r#""from_revision":"#.to_owned()),
        Just(r#""param":{"kind":"text","value":"#.to_owned()),
        Just(r#""observed":{"revision":"#.to_owned()),
        Just(r#""values":[["todo.input:text",null]]"#.to_owned()),
        Just(r#""hello","name":"#.to_owned()),
        Just(r#""confirm","request":"1""#.to_owned()),
        Just("}".to_owned()),
        Just("{".to_owned()),
        Just(",".to_owned()),
        Just(":".to_owned()),
        Just("[".to_owned()),
        Just("]".to_owned()),
        Just("null".to_owned()),
        Just("\"\\u0000\\ud800\"".to_owned()),
        Just("1e999".to_owned()),
        Just("-0".to_owned()),
        ".{0,12}",
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn random_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let line = String::from_utf8_lossy(&bytes);
        let _ = decode_client(&line);
        let _ = decode_server(&line);
    }

    #[test]
    fn random_protocol_shaped_lines_never_panic(parts in proptest::collection::vec(piece(), 0..24)) {
        let line = parts.concat();
        if let Err(r) = decode_client(&line) {
            let back = encode_server(&r.reply());
            prop_assert!(decode_server(&back).is_ok(), "{}", back);
        }
        let _ = decode_server(&line);
    }
}

#[test]
fn deep_nesting_is_refused_not_a_stack_overflow() {
    let deep = format!("{}{}", "[".repeat(100_000), "]".repeat(100_000));
    assert!(decode_client(&deep).is_err());
    let object = format!(
        r#"{{"v":1,"id":"a","type":"hello","name":{}1{}}}"#,
        "[".repeat(10_000),
        "]".repeat(10_000)
    );
    assert!(decode_client(&object).is_err());
}
