//! the protocol against the toy app, in process, with no socket: what an
//! agent sees, what it may do, and that its action takes the path a
//! person's click takes.

mod toy;

use proptest::prelude::*;
use sem::{
    Actor, Change, ReceiptSink, Rejection, Request, RequestId, Revision, SemId, Snapshot, Stage,
    StateRef, Value,
};
use toy::ToyHost;
use wire::{
    Bound, ClientMsg, ConnId, DeltaReply, Host, ProtoError, REDACTED, Reply, Server, ServerMsg,
    SnapshotReply, StageReply, decode_client, encode_client,
};

const A: ConnId = 1;
const B: ConnId = 2;

/// one line from `conn`, as the port would take it: refused lines get
/// their error, the rest go to the server.
fn say(server: &mut Server, host: &mut dyn Host, conn: ConnId, line: &str) -> Vec<ServerMsg> {
    match decode_client(line) {
        Ok(msg) => {
            let out = server.handle(conn, msg, host);
            assert!(out.iter().all(|o| o.conn == conn), "{out:?}");
            out.into_iter().map(|o| o.msg).collect()
        }
        Err(r) => vec![r.reply()],
    }
}

fn one(out: Vec<ServerMsg>) -> Reply {
    match <[ServerMsg; 1]>::try_from(out) {
        Ok([msg]) => msg.reply,
        Err(out) => panic!("one reply expected, got {out:?}"),
    }
}

fn hello(server: &mut Server, host: &mut dyn Host, conn: ConnId, name: &str) {
    let line = format!(r#"{{"v":1,"id":"h","type":"hello","name":"{name}"}}"#);
    match one(say(server, host, conn, &line)) {
        Reply::Welcome { app, actor } => {
            assert_eq!(app, "toy");
            assert_eq!(actor, Actor::Agent { name: name.into() });
        }
        other => panic!("{other:?}"),
    }
}

fn snapshot(reply: Reply) -> SnapshotReply {
    match reply {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot expected, got {other:?}"),
    }
}

fn delta(reply: Reply) -> DeltaReply {
    match reply {
        Reply::Delta(d) => d,
        other => panic!("a delta expected, got {other:?}"),
    }
}

fn error(reply: Reply) -> ProtoError {
    match reply {
        Reply::Error(e) => e,
        other => panic!("an error expected, got {other:?}"),
    }
}

fn stages(out: Vec<ServerMsg>, id: &str) -> Vec<StageReply> {
    out.into_iter()
        .map(|msg| {
            assert_eq!(msg.id.as_deref(), Some(id), "{msg:?}");
            match msg.reply {
                Reply::Stage(s) => s,
                other => panic!("a stage expected, got {other:?}"),
            }
        })
        .collect()
}

fn names(stages: &[StageReply]) -> Vec<&'static str> {
    stages.iter().map(|s| s.stage.name()).collect()
}

fn observe(server: &mut Server, host: &mut dyn Host, conn: ConnId, fields: &str) -> Reply {
    let line = format!(r#"{{"v":1,"id":"o","type":"observe"{fields}}}"#);
    one(say(server, host, conn, &line))
}

fn act(
    server: &mut Server,
    host: &mut dyn Host,
    conn: ConnId,
    id: &str,
    target: &str,
    action: &str,
    rest: &str,
) -> Vec<ServerMsg> {
    let line = format!(
        r#"{{"v":1,"id":"{id}","type":"act","target":"{target}","action":"{action}"{rest}}}"#
    );
    say(server, host, conn, &line)
}

fn from(revision: Revision) -> String {
    format!(r#","from_revision":"{revision}""#)
}

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn setup() -> (Server, ToyHost) {
    let mut server = Server::new("toy", 4);
    let mut host = ToyHost::new();
    hello(&mut server, &mut host, A, "claude");
    (server, host)
}

#[test]
fn hello_names_the_actor_once() {
    let (mut server, mut host) = setup();
    assert_eq!(
        server.actor(A),
        Some(Actor::Agent {
            name: "claude".into()
        })
    );
    let again = r#"{"v":1,"id":"h2","type":"hello","name":"other"}"#;
    assert!(matches!(
        error(one(say(&mut server, &mut host, A, again))),
        ProtoError::BadMessage { reason } if reason.contains("agent:claude")
    ));
    for name in ["", "   ", "tab\there", &"x".repeat(65)] {
        let line = encode_client(&ClientMsg {
            id: "n".into(),
            ask: wire::Ask::Hello { name: name.into() },
        });
        let e = error(one(say(&mut server, &mut host, B, &line)));
        assert!(
            matches!(e, ProtoError::BadMessage { .. }),
            "{name:?}: {e:?}"
        );
    }
    assert_eq!(server.actor(B), None);
}

#[test]
fn an_observation_is_the_app_projected_now() {
    let (mut server, mut host) = setup();
    let s = snapshot(observe(&mut server, &mut host, A, ""));
    let now = host.snapshot();
    assert_eq!(s.revision, now.revision());
    assert_eq!(s.nodes.len(), now.len());
    assert_eq!(s.focused, None);
    assert_eq!(s.bounds, None, "no bounds unless asked");
    // the secret pin travels as its length, as the snapshot holds it.
    let pin = s.nodes.iter().find(|n| n.id == sid("todo.pin")).unwrap();
    assert_eq!(pin.state.get("text"), Some(&Value::Int(4)));
    assert!(server.ring().get(s.revision).is_some(), "served, so held");
}

#[test]
fn scope_and_keys_narrow_an_observation() {
    let (mut server, mut host) = setup();
    let s = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo.item[2]""#,
    ));
    let ids: Vec<&str> = s.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, ["todo.item[2]"]);
    assert_eq!(s.revision, host.snapshot().revision(), "the whole revision");

    let s = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo","keys":["completed"],"bounds":true"#,
    ));
    assert_eq!(s.nodes.len(), host.snapshot().len());
    for node in &s.nodes {
        assert!(
            node.state.keys().all(|k| k.as_str() == "completed"),
            "{node:?}"
        );
    }
    let input = s.nodes.iter().find(|n| n.id == sid("todo.input")).unwrap();
    assert!(input.state.is_empty() && !input.actions.is_empty());
    let bounds = s.bounds.unwrap();
    assert_eq!(bounds.len(), 3, "the three bound rows");

    let s = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo.item[3]","bounds":true"#,
    ));
    assert_eq!(
        s.bounds,
        Some(vec![Bound {
            id: sid("todo.item[3]"),
            rect: [0.0, 80.0, 300.0, 40.0],
        }]),
        "only the bounds of the nodes observed"
    );

    let s = snapshot(observe(&mut server, &mut host, A, r#","scope":"gone""#));
    assert!(s.nodes.is_empty() && s.focused.is_none());
}

/// the acceptance: observe with `since` returns only what changed.
#[test]
fn since_a_served_revision_only_the_delta_comes_back() {
    let (mut server, mut host) = setup();
    let first = snapshot(observe(&mut server, &mut host, A, "")).revision;

    let same = delta(observe(&mut server, &mut host, A, &since(first)));
    assert_eq!((same.base, same.revision), (first, first));
    assert!(same.changes.is_empty());

    host.click("todo.item[1]", "toggle");
    let d = delta(observe(&mut server, &mut host, A, &since(first)));
    assert_eq!(d.base, first);
    assert_eq!(d.revision, host.snapshot().revision());
    let completed = StateRef::parse("todo.item[1]:completed").unwrap();
    let done = StateRef::parse("todo.counter:done").unwrap();
    let moved: Vec<&StateRef> = d
        .changes
        .iter()
        .map(|c| match c {
            Change::StateChanged { at, .. } => at,
            other => panic!("only state moved: {other:?}"),
        })
        .collect();
    assert_eq!(moved, [&completed, &done]);

    // scoped and keyed, the delta keeps only what the observation sees.
    let d = delta(observe(
        &mut server,
        &mut host,
        A,
        &format!(r#","scope":"todo.item[1]"{}"#, since(first)),
    ));
    assert_eq!(d.changes.len(), 1);
    let d = delta(observe(
        &mut server,
        &mut host,
        A,
        &format!(r#","keys":["done"]{}"#, since(first)),
    ));
    assert!(matches!(&d.changes[..], [Change::StateChanged { at, .. }] if *at == done));

    // a node that came and a label that moved are changes too.
    host.app.text = "water plants".into();
    host.click("todo.input", "submit");
    let base = d.revision;
    let d = delta(observe(&mut server, &mut host, A, &since(base)));
    assert!(d.changes.contains(&Change::NodeAdded(sid("todo.item[4]"))));

    // a revision the port does not hold gets the whole snapshot.
    let unknown = "0123456789abcdef";
    let s = snapshot(observe(
        &mut server,
        &mut host,
        A,
        &format!(r#","since":"{unknown}""#),
    ));
    assert_eq!(s.nodes.len(), host.snapshot().len());
}

fn since(r: Revision) -> String {
    format!(r#","since":"{r}""#)
}

#[test]
fn an_act_from_a_revision_goes_through_the_dispatcher_and_streams_its_stages() {
    let (mut server, mut host) = setup();
    let r = snapshot(observe(&mut server, &mut host, A, r#","scope":"todo""#)).revision;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "t1",
            "todo.item[2]",
            "toggle",
            &from(r),
        ),
        "t1",
    );
    assert_eq!(
        names(&out),
        ["requested", "accepted", "executing", "completed"]
    );
    assert_eq!(host.app.done(2), Some(true));

    let id = out[0].request;
    assert!(out.iter().all(|s| s.request == id));
    let Some(asked) = &out[0].asked else {
        panic!("requested carries the request");
    };
    assert_eq!(
        asked.actor,
        Actor::Agent {
            name: "claude".into()
        }
    );
    assert_eq!(asked.observed.revision, r);
    assert_eq!(
        asked.observed.values,
        [(
            StateRef::parse("todo.item[2]:completed").unwrap(),
            Some(Value::Bool(false))
        )],
        "rebuilt from the served revision"
    );
    assert!(out[1..].iter().all(|s| s.asked.is_none()));
    // the receipt is the dispatcher's: the same stages, the same times.
    let receipt = host.dispatcher.receipt(id).unwrap();
    let times: Vec<u64> = out.iter().map(|s| s.at).collect();
    let kept: Vec<u64> = receipt.stages.iter().map(|(_, at)| *at).collect();
    assert_eq!(times, kept);
    assert_eq!(
        receipt.request.actor,
        Actor::Agent {
            name: "claude".into()
        }
    );
}

/// the acceptance: the world moved between observe and act.
#[test]
fn a_person_moved_the_state_so_the_act_comes_back_stale_with_the_delta() {
    let (mut server, mut host) = setup();
    host.app.text = "buy bread".into();
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    // the person edits the field before the agent's submit arrives.
    host.app.text = "buy bread and eggs".into();
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "s",
            "todo.input",
            "submit",
            &from(r),
        ),
        "s",
    );
    assert_eq!(names(&out), ["requested", "rejected"]);
    let Stage::Rejected(Rejection::Stale { changes }) = &out[1].stage else {
        panic!("stale: {:?}", out[1].stage);
    };
    assert_eq!(
        changes,
        &[Change::StateChanged {
            at: StateRef::parse("todo.input:text").unwrap(),
            before: Some(Value::Text("buy bread".into())),
            after: Some(Value::Text("buy bread and eggs".into())),
            added: Vec::new(),
            removed: Vec::new(),
        }]
    );
    assert_eq!(host.app.items.len(), 3, "nothing was added");

    // observing again and acting on it goes through.
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "s2",
            "todo.input",
            "submit",
            &from(r),
        ),
        "s2",
    );
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert_eq!(host.app.items.len(), 4);
}

#[test]
fn an_act_with_explicit_observed_values_is_checked_the_same_way() {
    let (mut server, mut host) = setup();
    let seen = host.snapshot();
    let req = Request::observe(&seen, Actor::Human, "todo.item[1]", "toggle", None).unwrap();
    let observed = serde_json::to_string(&req.observed).unwrap();
    let rest = format!(r#","observed":{observed}"#);
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "e",
            "todo.item[1]",
            "toggle",
            &rest,
        ),
        "e",
    );
    assert_eq!(names(&out).last(), Some(&"completed"));
    // the same values again: item 1 is done now, so it is stale.
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "e2",
            "todo.item[1]",
            "toggle",
            &rest,
        ),
        "e2",
    );
    assert_eq!(names(&out), ["requested", "rejected"]);
}

#[test]
fn a_malformed_act_is_refused_before_the_dispatcher() {
    let (mut server, mut host) = setup();
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let observed = r#","observed":{"revision":"0000000000000001","values":[]}"#;
    for (rest, target, action, want) in [
        (String::new(), "todo.item[1]", "toggle", "not on both"),
        (
            format!("{}{observed}", from(r)),
            "todo.item[1]",
            "toggle",
            "not on both",
        ),
        (from(r), "todo.item[9]", "toggle", "todo.item[9]"),
        (from(r), "todo.item[1]", "explode", "explode"),
    ] {
        let e = error(one(act(
            &mut server,
            &mut host,
            A,
            "x",
            target,
            action,
            &rest,
        )));
        match e {
            ProtoError::BadMessage { reason } => {
                let needle = if want == "not on both" && rest.is_empty() {
                    "from_revision"
                } else {
                    want
                };
                assert!(reason.contains(needle), "{rest}: {reason}");
            }
            other => panic!("{rest}: {other:?}"),
        }
    }
    assert_eq!(
        host.dispatcher.receipt(RequestId::parse("1").unwrap()),
        None
    );
}

#[test]
fn an_act_needs_a_hello_first() {
    let mut server = Server::new("toy", 4);
    let mut host = ToyHost::new();
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let e = error(one(act(
        &mut server,
        &mut host,
        A,
        "x",
        "todo.item[1]",
        "toggle",
        &from(r),
    )));
    assert!(matches!(e, ProtoError::NotEnabled { reason } if reason.contains("hello")));
    assert_eq!(host.app.done(1), Some(false));
}

#[test]
fn a_revision_out_of_the_ring_is_unknown() {
    let (mut server, mut host) = setup();
    let first = snapshot(observe(&mut server, &mut host, A, "")).revision;
    // four more distinct revisions push the first out of a ring of four.
    for n in 0..4 {
        host.app.text = format!("draft {n}");
        observe(&mut server, &mut host, A, "");
    }
    assert!(server.ring().get(first).is_none());
    let e = error(one(act(
        &mut server,
        &mut host,
        A,
        "u",
        "todo.item[1]",
        "toggle",
        &from(first),
    )));
    assert_eq!(e, ProtoError::UnknownRevision { revision: first });
    assert_eq!(host.app.done(1), Some(false));
}

/// an agent never answers a confirmation; a person does, and the agent
/// hears how its request ended.
#[test]
fn confirm_and_decline_are_refused_and_a_person_answers() {
    let (mut server, mut host) = setup();
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "rm",
            "todo.item[3]",
            "remove",
            &from(r),
        ),
        "rm",
    );
    assert_eq!(names(&out), ["requested", "awaiting_confirmation"]);
    let id = out[0].request;
    for verb in ["confirm", "decline"] {
        let line = format!(r#"{{"v":1,"id":"c","type":"{verb}","request":"{id}"}}"#);
        let e = error(one(say(&mut server, &mut host, A, &line)));
        assert!(
            matches!(&e, ProtoError::NotEnabled { reason } if reason.contains("person") && reason.contains("agent:claude")),
            "{e:?}"
        );
    }
    assert_eq!(host.app.items.len(), 3, "still waiting");

    host.clock += 10;
    host.dispatcher
        .confirm(&mut host.app, id, Actor::Human, host.clock)
        .unwrap();
    let later = server.forward();
    assert!(later.iter().all(|o| o.conn == A));
    let later = stages(later.into_iter().map(|o| o.msg).collect(), "rm");
    assert_eq!(names(&later), ["confirmed", "executing", "completed"]);
    assert_eq!(host.app.items.len(), 2);
    assert!(server.forward().is_empty(), "forgotten after the end");
}

#[test]
fn an_execution_that_ends_later_reaches_the_agent_on_the_next_forward() {
    let (mut server, mut host) = setup();
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "sy",
            "todo.sync",
            "sync",
            &from(r),
        ),
        "sy",
    );
    assert_eq!(names(&out), ["requested", "accepted", "executing"]);
    assert!(server.forward().is_empty());
    host.finish_sync();
    let later: Vec<ServerMsg> = server.forward().into_iter().map(|o| o.msg).collect();
    let later = stages(later, "sy");
    assert_eq!(names(&later), ["completed"]);
    assert_eq!(later[0].at, host.clock);
    let Stage::Completed(outcome) = &later[0].stage else {
        panic!("completed");
    };
    assert_eq!(outcome.result.get("synced"), Some(&Value::Int(3)));
}

#[test]
fn stages_reach_only_the_connection_that_asked() {
    let (mut server, mut host) = setup();
    hello(&mut server, &mut host, B, "gemini");
    let r = snapshot(observe(&mut server, &mut host, B, "")).revision;
    let out = server.handle(
        A,
        ClientMsg {
            id: "mine".into(),
            ask: wire::Ask::Act(wire::Act::from_revision(
                sid("todo.sync"),
                sem::ActionName::parse("sync").unwrap(),
                r,
            )),
        },
        &mut host,
    );
    assert!(out.iter().all(|o| o.conn == A));
    // a person's click is never forwarded to an agent.
    host.click("todo.item[1]", "toggle");
    assert!(server.forward().is_empty());
    host.finish_sync();
    let later = server.forward();
    assert_eq!(later.len(), 1);
    assert_eq!(later[0].conn, A);
    // a connection that ended is forgotten with what it waited on.
    let out = act(
        &mut server,
        &mut host,
        B,
        "b",
        "todo.sync",
        "sync",
        &from(r),
    );
    assert_eq!(stages(out, "b").len(), 3);
    server.close(B);
    host.finish_sync();
    assert!(server.forward().is_empty());
    assert_eq!(server.actor(B), None);
}

#[test]
fn a_secret_never_leaves_through_a_stage() {
    let (mut server, mut host) = setup();
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let param = r#","param":{"kind":"text","value":"9876"}"#;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "p",
            "todo.pin",
            "set_value",
            &format!("{param}{}", from(r)),
        ),
        "p",
    );
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert_eq!(host.app.pin, "9876", "the app got the real value");
    let asked = out[0].asked.as_ref().unwrap();
    assert_eq!(asked.param, Some(Value::Int(4)));
    assert_eq!(asked.observed.values[0].1, Some(Value::Int(4)));
    let Stage::Completed(outcome) = &out[3].stage else {
        panic!("completed");
    };
    assert_eq!(outcome.result.get("pin"), Some(&Value::Int(4)));

    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let param = r#","param":{"kind":"text","value":"secret-12345"}"#;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "p2",
            "todo.pin",
            "set_value",
            &format!("{param}{}", from(r)),
        ),
        "p2",
    );
    assert_eq!(
        out.last().map(|s| &s.stage),
        Some(&Stage::Failed {
            error: REDACTED.to_owned()
        })
    );
    let line = serde_json::to_string(&out).unwrap();
    assert!(!line.contains("12345") && !line.contains("9876"), "{line}");
}

/// a host that projects nothing, or forgets to wire the port's sink.
struct Blank;

impl Host for Blank {
    fn project(&self) -> Option<Snapshot> {
        None
    }
    fn dispatch(&mut self, _req: Request) -> Option<RequestId> {
        None
    }
    fn add_sink(&mut self, _sink: Box<dyn ReceiptSink>) {}
}

#[test]
fn nothing_projected_or_no_stage_reported_is_said_plainly() {
    let mut server = Server::new("toy", 4);
    let mut blank = Blank;
    hello(&mut server, &mut blank, A, "claude");
    let e = error(observe(&mut server, &mut blank, A, ""));
    assert!(matches!(e, ProtoError::NotEnabled { reason } if reason.contains("projects nothing")));

    let mut server = Server::new("toy", 4);
    let mut host = ToyHost::new();
    host.wires_sinks = false;
    hello(&mut server, &mut host, A, "claude");
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let e = error(one(act(
        &mut server,
        &mut host,
        A,
        "n",
        "todo.item[1]",
        "toggle",
        &from(r),
    )));
    assert!(
        matches!(&e, ProtoError::NotEnabled { reason } if reason.contains("request 1 was dispatched")),
        "{e:?}"
    );
}

fn piece() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r#"{"v":1,"id":"x","type":"#.to_owned()),
        Just(r#""act","target":"todo.item[1]","action":"toggle""#.to_owned()),
        Just(r#""act","target":"todo.pin","action":"set_value""#.to_owned()),
        Just(r#""observe","scope":"todo","since":"#.to_owned()),
        Just(r#""hello","name":"fuzz""#.to_owned()),
        Just(r#""0000000000000001""#.to_owned()),
        Just(r#","from_revision":"#.to_owned()),
        Just(r#","param":{"kind":"text","value":"1"}"#.to_owned()),
        Just(r#","observed":{"revision":"0000000000000001","values":[["todo.item[1]:completed",null]]}"#.to_owned()),
        Just(r#","keys":["completed"],"bounds":true"#.to_owned()),
        Just(r#""confirm","request":"1""#.to_owned()),
        Just("}".to_owned()),
        Just(",".to_owned()),
        ".{0,8}",
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// whatever an agent sends, the server answers without panicking and
    /// every reply goes back to the connection that sent it.
    #[test]
    fn random_lines_through_the_server_never_panic(
        lines in proptest::collection::vec(proptest::collection::vec(piece(), 0..12), 1..8)
    ) {
        let mut server = Server::new("toy", 2);
        let mut host = ToyHost::new();
        let r = host.snapshot().revision();
        for parts in lines {
            let mut line = parts.concat();
            line = line.replace("0000000000000001", &r.to_string());
            let _ = say(&mut server, &mut host, A, &line);
        }
        let _ = server.forward();
    }
}
