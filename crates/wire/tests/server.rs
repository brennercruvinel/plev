//! the protocol against the toy app, in process, with no socket: what an
//! agent sees, what it may do, and that its action takes the path a
//! person's click takes.

mod toy;

use proptest::prelude::*;
use sem::redact::REDACTED;
use sem::{
    Actor, Change, Node, ReceiptSink, Rejection, Request, RequestId, Revision, SemId, Snapshot,
    Stage, StateRef, Value,
};
use toy::ToyHost;
use wire::{
    Bound, ClientMsg, ConnId, DeltaReply, Host, ProtoError, Reply, Server, ServerMsg,
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

/// where a node was projected is not part of its meaning: an observation
/// carries no `source` unless asked, and then every node's.
#[test]
fn source_travels_only_when_asked() {
    let (mut server, mut host) = setup();
    let live = host.snapshot();
    assert!(
        live.nodes().values().all(|n| n.source.is_some()),
        "the toy projects with a source"
    );
    let plain = snapshot(observe(&mut server, &mut host, A, ""));
    assert_eq!(plain.nodes.len(), live.len());
    assert!(plain.nodes.iter().all(|n| n.source.is_none()), "{plain:?}");
    let scoped = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo.item[1]","keys":["completed"]"#,
    ));
    assert!(scoped.nodes.iter().all(|n| n.source.is_none()));

    let sourced = snapshot(observe(&mut server, &mut host, A, r#","source":true"#));
    assert_eq!(sourced.nodes.len(), live.len());
    for node in &sourced.nodes {
        let projected = live.get(&node.id).and_then(|n| n.source.clone());
        assert!(projected.is_some() && node.source == projected, "{node:?}");
    }
    // the same snapshot either way: source is not in the revision.
    assert_eq!(plain.revision, sourced.revision);
    let line = serde_json::to_string(&plain.nodes).unwrap();
    assert!(!line.contains("source"), "{line}");
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
    assert!(
        d.changes
            .iter()
            .any(|c| matches!(c, Change::NodeAdded(n) if n.id == sid("todo.item[4]"))),
        "{d:?}"
    );

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

/// the node a delta's `node_added` carries.
fn added<'a>(d: &'a DeltaReply, id: &str) -> &'a Node {
    d.changes
        .iter()
        .find_map(|c| match c {
            Change::NodeAdded(node) if node.id == sid(id) => Some(&**node),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{id} added in {d:?}"))
}

/// a node that came travels whole in the delta and is served as a
/// snapshot serves it: no `source` unless asked, the state cut to `keys`.
#[test]
fn a_node_added_in_a_delta_is_served_as_a_snapshot_serves_it() {
    let (mut server, mut host) = setup();
    let base = snapshot(observe(&mut server, &mut host, A, "")).revision;
    host.app.text = "water plants".into();
    host.click("todo.input", "submit");
    let live = host.snapshot();
    let projected = live.get(&sid("todo.item[4]")).unwrap();
    assert!(projected.source.is_some(), "the toy projects with a source");

    let d = delta(observe(&mut server, &mut host, A, &since(base)));
    let node = added(&d, "todo.item[4]");
    assert_eq!(node, projected, "the node as the app projects it");
    assert_eq!(node.label, "water plants");
    assert_eq!(node.state.get("completed"), Some(&Value::Bool(false)));
    assert_eq!(node.source, None, "no source unless asked");
    let line = serde_json::to_string(&d.changes).unwrap();
    assert!(!line.contains("source"), "{line}");
    // the same bytes a snapshot of that node serves.
    let row = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo.item[4]""#,
    ));
    assert_eq!(
        serde_json::to_string(&row.nodes[0]).unwrap(),
        serde_json::to_string(node).unwrap()
    );

    let keyed = |keys: &str, source: bool| {
        let source = if source { r#","source":true"# } else { "" };
        format!(r#","keys":[{keys}]{source}{}"#, since(base))
    };
    let d = delta(observe(
        &mut server,
        &mut host,
        A,
        &keyed(r#""done""#, false),
    ));
    let node = added(&d, "todo.item[4]");
    assert!(node.state.is_empty(), "completed is not asked: {node:?}");
    assert_eq!(node.actions, projected.actions, "the other fields stay");
    assert_eq!(node.source, None);

    let d = delta(observe(
        &mut server,
        &mut host,
        A,
        &keyed(r#""completed""#, true),
    ));
    let node = added(&d, "todo.item[4]");
    assert_eq!(node.state.get("completed"), Some(&Value::Bool(false)));
    assert!(node.source.is_some() && node.source == projected.source);
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
    let Stage::Rejected(Rejection::Stale { changes, .. }) = &out[1].stage else {
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

/// who acted crosses the port as the dispatcher names it: the person's
/// toggle, checked against the revision the agent acts from; the agent's
/// own earlier act on that row is not named.
#[test]
fn a_stale_refusal_names_the_person_who_acted_on_what_the_agent_saw() {
    let (mut server, mut host) = setup();
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    host.click("todo.item[1]", "toggle");
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "t",
            "todo.item[1]",
            "toggle",
            &from(r),
        ),
        "t",
    );
    assert_eq!(names(&out), ["requested", "rejected"]);
    let Stage::Rejected(Rejection::Stale { changes, acted }) = &out[1].stage else {
        panic!("stale: {:?}", out[1].stage);
    };
    assert_eq!(changes.len(), 1);
    let [who] = &acted[..] else {
        panic!("one actor named: {acted:?}");
    };
    assert_eq!(who.actor, Actor::Human);
    assert_eq!(who.target, sid("todo.item[1]"));
    assert_eq!(who.action.as_str(), "toggle");
    assert_eq!(
        host.dispatcher
            .receipt(who.id)
            .map(|r| r.request.actor.clone()),
        Some(Actor::Human)
    );
    let line = serde_json::to_string(&out[1].stage).unwrap();
    assert!(line.contains(r#""acted":[{"id":"#), "{line}");
}

/// the counter's `done`, what `todo.clear` reads.
fn done_set(items: &[&str]) -> Value {
    Value::Set(items.iter().map(|i| (*i).to_owned()).collect())
}

/// task-89, the hole the session inspector found in the session of A2:
/// the agent sees what `clear` would remove (the counter's `done`, item
/// 3), a person marks item 1 done, the agent observes only the button and
/// clears on the revision that observation served. the act rests on the
/// `done` the agent was shown, not on the one the revision holds whole:
/// it is refused as stale, with the delta, and nothing is removed.
#[test]
fn an_act_rests_on_what_was_shown_not_on_the_whole_revision() {
    let (mut server, mut host) = setup();
    let seen = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo.counter""#,
    ));
    let counter = seen.nodes.iter().find(|n| n.id == sid("todo.counter"));
    assert_eq!(
        counter.and_then(|n| n.state.get("done")),
        Some(&done_set(&["3"]))
    );
    host.click("todo.item[1]", "toggle");
    let r = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo.clear""#,
    ))
    .revision;
    assert_ne!(r, seen.revision, "the person moved the app");

    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "c",
            "todo.clear",
            "clear",
            &from(r),
        ),
        "c",
    );
    let done = StateRef::parse("todo.counter:done").unwrap();
    let asked = out[0].asked.as_ref().unwrap();
    assert_eq!(asked.observed.revision, r, "the revision still traces");
    assert_eq!(
        asked.observed.values,
        [(done.clone(), Some(done_set(&["3"])))],
        "what the agent was shown, not what the revision holds"
    );
    assert_eq!(names(&out), ["requested", "rejected"]);
    let Stage::Rejected(Rejection::Stale { changes, .. }) = &out[1].stage else {
        panic!("stale: {:?}", out[1].stage);
    };
    assert_eq!(
        changes,
        &[Change::StateChanged {
            at: done,
            before: Some(done_set(&["3"])),
            after: Some(done_set(&["1", "3"])),
            added: vec!["1".to_owned()],
            removed: Vec::new(),
        }]
    );
    assert_eq!(host.app.items.len(), 3, "nothing was cleared");
}

/// the revision the terminal stage of `stages` carries; the stages
/// before it carry none.
fn landed(stages: &[StageReply]) -> Revision {
    let Some((last, before)) = stages.split_last() else {
        panic!("no stage");
    };
    assert!(last.is_terminal(), "{last:?}");
    assert!(
        before.iter().all(|s| s.revision.is_none()),
        "only the terminal stage carries a revision: {stages:?}"
    );
    last.revision
        .unwrap_or_else(|| panic!("the terminal stage carries a revision: {last:?}"))
}

/// the acceptance: the `completed` of an act carries the revision the app
/// is at, `since` it is a delta, and the next act rests on it with no
/// observation in between. each act of the chain reads a row none of the
/// acts before it moved, so what it reads is what the agent was shown
/// (task-89; a row its own act moved is the next test).
#[test]
fn a_completed_act_carries_the_revision_the_next_one_rests_on() {
    let (mut server, mut host) = setup();
    let seen = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "t1",
            "todo.item[1]",
            "toggle",
            &from(seen),
        ),
        "t1",
    );
    assert_eq!(
        names(&out),
        ["requested", "accepted", "executing", "completed"]
    );
    let first = landed(&out);
    assert_eq!(
        first,
        host.snapshot().revision(),
        "the app as the act left it"
    );
    assert_ne!(first, seen);
    assert!(server.ring().get(first).is_some(), "held, as a served one");

    // `since` it: a delta, empty while nothing moves.
    let d = delta(observe(&mut server, &mut host, A, &since(first)));
    assert_eq!((d.base, d.revision), (first, first));
    assert!(d.changes.is_empty(), "{d:?}");

    // `from_revision` it: the next act is taken and runs, and so is the
    // one after it, each on the revision the one before left, each on
    // what the first observation showed of its row.
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "t2",
            "todo.item[2]",
            "toggle",
            &from(first),
        ),
        "t2",
    );
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert_eq!(
        out[0].asked.as_ref().map(|a| a.observed.revision),
        Some(first)
    );
    let second = landed(&out);
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "t3",
            "todo.item[3]",
            "toggle",
            &from(second),
        ),
        "t3",
    );
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert_eq!(
        (host.app.done(1), host.app.done(2), host.app.done(3)),
        (Some(true), Some(true), Some(false))
    );

    // what the two acts did, since the first one's end.
    let d = delta(observe(&mut server, &mut host, A, &since(first)));
    assert_eq!(d.base, first);
    assert_eq!(d.revision, landed(&out));
    let moved: Vec<String> = d
        .changes
        .iter()
        .map(|c| match c {
            Change::StateChanged { at, .. } => at.to_string(),
            other => panic!("only state moved: {other:?}"),
        })
        .collect();
    assert_eq!(
        moved,
        [
            "todo.item[2]:completed",
            "todo.item[3]:completed",
            "todo.counter:done"
        ]
    );
}

/// task-89: an act that reads a row the agent's own act moved, with no
/// observation in between, rests on what the agent was shown before its
/// own act, so it is stale. the refusal names the agent's own first
/// toggle in `acted`, the one act that ran since the look (task-91; the
/// rule of task-90 never named the author, and the refusal named nobody).
/// observing the row and acting on that observation goes through.
#[test]
fn a_row_the_agents_own_act_moved_is_stale_until_it_observes_it() {
    let (mut server, mut host) = setup();
    let seen = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "t1",
            "todo.item[1]",
            "toggle",
            &from(seen),
        ),
        "t1",
    );
    let first = landed(&out);
    let own = out[0].request;
    assert_eq!(host.app.done(1), Some(true));

    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "t2",
            "todo.item[1]",
            "toggle",
            &from(first),
        ),
        "t2",
    );
    assert_eq!(names(&out), ["requested", "rejected"]);
    let completed = StateRef::parse("todo.item[1]:completed").unwrap();
    assert_eq!(
        out[0].asked.as_ref().map(|a| a.observed.values.clone()),
        Some(vec![(completed.clone(), Some(Value::Bool(false)))]),
        "what the first observation showed, not what the landed revision holds"
    );
    let Stage::Rejected(Rejection::Stale { changes, acted }) = &out[1].stage else {
        panic!("stale: {:?}", out[1].stage);
    };
    assert!(
        matches!(&changes[..], [Change::StateChanged { at, before: Some(Value::Bool(false)), after: Some(Value::Bool(true)), .. }] if *at == completed),
        "{changes:?}"
    );
    let [who] = &acted[..] else {
        panic!("the agent's own toggle: {acted:?}");
    };
    assert_eq!(
        (who.id, &who.actor, who.action.as_str()),
        (
            own,
            &Actor::Agent {
                name: "claude".into()
            },
            "toggle"
        ),
        "the agent itself moved it"
    );
    assert_eq!(host.app.done(1), Some(true), "nothing ran");

    let row = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo.item[1]""#,
    ))
    .revision;
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "t3",
            "todo.item[1]",
            "toggle",
            &from(row),
        ),
        "t3",
    );
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert_eq!(host.app.done(1), Some(false));
}

/// a stale refusal ends too: it carries the app as it is, the change that
/// made it stale in, and an act on that revision goes through.
#[test]
fn a_stale_refusal_carries_the_revision_the_app_moved_to() {
    let (mut server, mut host) = setup();
    host.app.text = "buy bread".into();
    let seen = snapshot(observe(&mut server, &mut host, A, "")).revision;
    host.app.text = "buy bread and eggs".into();
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "s",
            "todo.input",
            "submit",
            &from(seen),
        ),
        "s",
    );
    assert_eq!(names(&out), ["requested", "rejected"]);
    let now = landed(&out);
    assert_eq!(now, host.snapshot().revision(), "the person's edit in");
    let d = delta(observe(&mut server, &mut host, A, &since(seen)));
    assert_eq!(d.revision, now);
    assert!(
        matches!(&d.changes[..], [Change::StateChanged { at, .. }] if at.to_string() == "todo.input:text"),
        "{d:?}"
    );
    // the revision decides nothing: the act on it is checked on its reads,
    // which it holds as they are now, so it runs.
    let out = stages(
        act(
            &mut server,
            &mut host,
            A,
            "s2",
            "todo.input",
            "submit",
            &from(now),
        ),
        "s2",
    );
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert_eq!(
        host.app.items.last().map(|(_, text, _)| text.as_str()),
        Some("buy bread and eggs")
    );
}

/// the toy as a shell holds it, counting its projections, and dark when
/// it projects nothing.
struct Counted {
    host: ToyHost,
    projections: std::cell::Cell<usize>,
    dark: bool,
}

impl Counted {
    fn new() -> Self {
        Counted {
            host: ToyHost::new(),
            projections: std::cell::Cell::new(0),
            dark: false,
        }
    }
}

impl Host for Counted {
    fn project(&self) -> Option<Snapshot> {
        self.projections.set(self.projections.get() + 1);
        if self.dark { None } else { self.host.project() }
    }
    fn dispatch(&mut self, req: Request) -> Option<RequestId> {
        self.host.dispatch(req)
    }
    fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        self.host.add_sink(sink);
    }
}

/// the app is projected once for every terminal stage a forward sends,
/// and not at all when none goes out.
#[test]
fn one_projection_serves_every_terminal_stage_of_a_forward() {
    let mut server = Server::new("toy", 4);
    let mut host = Counted::new();
    hello(&mut server, &mut host, A, "claude");
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    for id in ["s1", "s2"] {
        let before = host.projections.get();
        let out = stages(
            act(&mut server, &mut host, A, id, "todo.sync", "sync", &from(r)),
            id,
        );
        assert_eq!(names(&out), ["requested", "accepted", "executing"]);
        assert_eq!(
            host.projections.get() - before,
            1,
            "the act reads which nodes are secret; no stage ended, so no more"
        );
    }
    host.host.click("todo.item[2]", "toggle");
    host.host.finish_sync();
    host.host.finish_sync();
    let before = host.projections.get();
    let later = server.forward(&mut host);
    assert_eq!(host.projections.get() - before, 1, "one for both");
    let ends: Vec<(String, Option<Revision>)> = later
        .into_iter()
        .map(|o| match o.msg.reply {
            Reply::Stage(s) => (o.msg.id.unwrap_or_default(), s.revision),
            other => panic!("{other:?}"),
        })
        .collect();
    let now = Some(host.host.snapshot().revision());
    assert_eq!(ends, [("s1".to_owned(), now), ("s2".to_owned(), now)]);
    let before = host.projections.get();
    assert!(server.forward(&mut host).is_empty());
    assert_eq!(host.projections.get(), before, "nothing went out");
}

/// an app that projects nothing when the stage goes out leaves the field
/// out; the act itself is not refused for it.
#[test]
fn no_projection_no_revision() {
    let mut server = Server::new("toy", 4);
    let mut host = Counted::new();
    hello(&mut server, &mut host, A, "claude");
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    host.dark = true;
    let out = act(
        &mut server,
        &mut host,
        A,
        "d",
        "todo.item[1]",
        "toggle",
        &from(r),
    );
    let line = wire::encode_server(&out[out.len() - 1]);
    let out = stages(out, "d");
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert!(out.iter().all(|s| s.revision.is_none()), "{out:?}");
    assert!(!line.contains("revision"), "{line}");
    assert_eq!(host.host.app.done(1), Some(true));
}

/// the revisions stages carry take places in the bounded ring as served
/// ones: enough acts push out a revision an agent observed.
#[test]
fn the_ring_stays_bounded_with_the_revisions_stages_carry() {
    let (mut server, mut host) = setup();
    let seen = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let mut at = seen;
    for n in 0..4 {
        let id = format!("v{n}");
        let rest = format!(
            r#","param":{{"kind":"text","value":"draft {n}"}}{}"#,
            from(at)
        );
        let out = stages(
            act(
                &mut server,
                &mut host,
                A,
                &id,
                "todo.input",
                "set_value",
                &rest,
            ),
            &id,
        );
        at = landed(&out);
        assert!(server.ring().len() <= server.ring().cap());
    }
    assert!(server.ring().get(at).is_some());
    assert!(server.ring().get(seen).is_none(), "four ends pushed it out");
    let e = error(one(act(
        &mut server,
        &mut host,
        A,
        "u",
        "todo.item[1]",
        "toggle",
        &from(seen),
    )));
    assert_eq!(e, ProtoError::UnknownRevision { revision: seen });
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
    assert!(
        out.iter().all(|s| s.revision.is_none()),
        "waiting is not an end: {out:?}"
    );
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
    let later = server.forward(&mut host);
    assert!(later.iter().all(|o| o.conn == A));
    let later = stages(later.into_iter().map(|o| o.msg).collect(), "rm");
    assert_eq!(names(&later), ["confirmed", "executing", "completed"]);
    assert_eq!(host.app.items.len(), 2);
    assert_eq!(landed(&later), host.snapshot().revision(), "the item gone");
    assert!(
        server.forward(&mut host).is_empty(),
        "forgotten after the end"
    );
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
    assert!(out.iter().all(|s| s.revision.is_none()), "{out:?}");
    assert!(server.forward(&mut host).is_empty());
    // a person moves the app while the sync runs; the stage that ends it
    // carries the app as it is when it goes out, the person's toggle in.
    host.click("todo.item[1]", "toggle");
    host.finish_sync();
    let later: Vec<ServerMsg> = server
        .forward(&mut host)
        .into_iter()
        .map(|o| o.msg)
        .collect();
    let later = stages(later, "sy");
    assert_eq!(names(&later), ["completed"]);
    assert_eq!(later[0].at, host.clock);
    assert_eq!(landed(&later), host.snapshot().revision());
    assert_ne!(landed(&later), r, "item 1 moved since");
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
    // an act rests on a revision given to its own actor (task-89).
    let mine = snapshot(observe(&mut server, &mut host, A, "")).revision;
    assert_eq!(mine, r, "nothing moved between the two");
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
    assert!(server.forward(&mut host).is_empty());
    host.finish_sync();
    let later = server.forward(&mut host);
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
    assert!(server.forward(&mut host).is_empty());
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
    // the revision is a hash of the projection, where the pin is only its
    // length: the same whatever four digits it holds.
    let pinned = landed(&out);
    assert_eq!(pinned, host.snapshot().revision());
    host.app.pin = "0000".into();
    assert_eq!(host.snapshot().revision(), pinned);
    host.app.pin = "9876".into();

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
    assert_eq!(
        landed(&out),
        pinned,
        "a failure ends with the app as it was"
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
        let _ = server.forward(&mut host);
    }
}

/// how many times the server called the wake it was given.
fn counting(server: &mut Server) -> std::rc::Rc<std::cell::Cell<u32>> {
    let woke = std::rc::Rc::new(std::cell::Cell::new(0));
    let count = std::rc::Rc::clone(&woke);
    server.wake_with(move || count.set(count.get() + 1));
    woke
}

/// the P0 of task-92: a stage of a request an agent asked for here, heard
/// after the act that dispatched it, wakes the port, so it goes out
/// without the agent sending anything: a person's decline, a person's
/// confirmation with what runs after it, an execution that ends later.
/// nothing else wakes it: the stages of the act's own dispatch, a
/// person's click, an agent's request that did not come through the
/// port, a request whose connection ended.
#[test]
fn a_stage_heard_after_its_act_wakes_the_port() {
    let (mut server, mut host) = setup();
    let woke = counting(&mut server);
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = act(
        &mut server,
        &mut host,
        A,
        "rm",
        "todo.item[3]",
        "remove",
        &from(r),
    );
    let out = stages(out, "rm");
    assert_eq!(names(&out), ["requested", "awaiting_confirmation"]);
    assert_eq!(woke.get(), 0, "the act's own drain sends its stages");
    let id = out[0].request;

    host.click("todo.item[1]", "toggle");
    assert_eq!(woke.get(), 0, "a person's click is not the agent's");

    host.clock += 1;
    host.dispatcher
        .decline(id, Actor::Human, host.clock)
        .unwrap();
    assert_eq!(woke.get(), 1, "the decline woke the port");
    let later = stages(
        server
            .forward(&mut host)
            .into_iter()
            .map(|o| o.msg)
            .collect(),
        "rm",
    );
    assert_eq!(
        later[0].stage,
        Stage::Rejected(Rejection::Declined { by: Actor::Human })
    );

    // a confirmation wakes it once per stage it hears: confirmed,
    // executing, completed; the wakes coalesce in the port.
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = act(
        &mut server,
        &mut host,
        A,
        "rm2",
        "todo.item[2]",
        "remove",
        &from(r),
    );
    let id = stages(out, "rm2")[0].request;
    host.clock += 1;
    host.dispatcher
        .confirm(&mut host.app, id, Actor::Human, host.clock)
        .unwrap();
    assert_eq!(woke.get(), 1 + 3);
    let later = stages(
        server
            .forward(&mut host)
            .into_iter()
            .map(|o| o.msg)
            .collect(),
        "rm2",
    );
    assert_eq!(names(&later), ["confirmed", "executing", "completed"]);

    // an execution that ends later, on a worker.
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = act(
        &mut server,
        &mut host,
        A,
        "sy",
        "todo.sync",
        "sync",
        &from(r),
    );
    assert_eq!(
        names(&stages(out, "sy")),
        ["requested", "accepted", "executing"]
    );
    assert_eq!(woke.get(), 4);
    host.finish_sync();
    assert_eq!(woke.get(), 5);
    assert_eq!(server.forward(&mut host).len(), 1);

    // an agent's request that did not come through the port.
    let seen = host.snapshot();
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let req = Request::observe(&seen, agent, "todo.sync", "sync", None).unwrap();
    host.dispatch(req);
    host.finish_sync();
    assert_eq!(woke.get(), 5, "not a request of this port");
    assert!(server.forward(&mut host).is_empty());

    // a request whose connection ended is forgotten, and wakes nothing.
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = act(
        &mut server,
        &mut host,
        A,
        "sy2",
        "todo.sync",
        "sync",
        &from(r),
    );
    assert_eq!(stages(out, "sy2").len(), 3);
    server.close(A);
    host.finish_sync();
    assert_eq!(woke.get(), 5, "nobody waits on it");
}
