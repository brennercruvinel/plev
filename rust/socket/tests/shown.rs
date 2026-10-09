//! what an act on `from_revision` rests on (task-89): the state its
//! actor was shown up to that revision, and nothing else. the toy's
//! `todo.clear` reads the counter's `done`, state of another node, as the
//! ide's `commit.submit` reads `changes:staged`.

mod support;

use accord::{Actor, Change, Rejection, RequestId, Revision, Stage, StateRef, Value};
use support::toy::ToyHost;
use socket::{
    ConnId, DeltaReply, Host, MAX_ACTORS, ProtoError, Reply, Server, ServerMsg, SnapshotReply,
    StageReply, decode_client,
};

const A: ConnId = 1;
const B: ConnId = 2;

fn say(server: &mut Server, host: &mut dyn Host, conn: ConnId, line: &str) -> Vec<ServerMsg> {
    let msg = decode_client(line).unwrap_or_else(|r| panic!("{line}: {r:?}"));
    server
        .handle(conn, msg, host)
        .into_iter()
        .map(|o| o.msg)
        .collect()
}

fn one(out: Vec<ServerMsg>) -> Reply {
    match <[ServerMsg; 1]>::try_from(out) {
        Ok([msg]) => msg.reply,
        Err(out) => panic!("one reply expected, got {out:?}"),
    }
}

fn hello(server: &mut Server, host: &mut dyn Host, conn: ConnId, name: &str) {
    let line = format!(r#"{{"v":1,"id":"h","type":"hello","name":"{name}"}}"#);
    assert!(matches!(
        one(say(server, host, conn, &line)),
        Reply::Welcome { .. }
    ));
}

fn observe(server: &mut Server, host: &mut dyn Host, conn: ConnId, fields: &str) -> Reply {
    let line = format!(r#"{{"v":1,"id":"o","type":"observe"{fields}}}"#);
    one(say(server, host, conn, &line))
}

/// the revision of a scoped observation of `scope`.
fn look(server: &mut Server, host: &mut dyn Host, conn: ConnId, scope: &str) -> Revision {
    match observe(server, host, conn, &format!(r#","scope":"{scope}""#)) {
        Reply::Snapshot(s) => s.revision,
        other => panic!("a snapshot expected, got {other:?}"),
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

fn act(
    server: &mut Server,
    host: &mut dyn Host,
    conn: ConnId,
    target: &str,
    action: &str,
    rest: &str,
) -> Vec<ServerMsg> {
    let line =
        format!(r#"{{"v":1,"id":"a","type":"act","target":"{target}","action":"{action}"{rest}}}"#);
    say(server, host, conn, &line)
}

fn from(revision: Revision) -> String {
    format!(r#","from_revision":"{revision}""#)
}

fn stages(out: Vec<ServerMsg>) -> Vec<StageReply> {
    out.into_iter()
        .map(|msg| match msg.reply {
            Reply::Stage(s) => s,
            other => panic!("a stage expected, got {other:?}"),
        })
        .collect()
}

fn names(stages: &[StageReply]) -> Vec<&'static str> {
    stages.iter().map(|s| s.stage.name()).collect()
}

fn landed(stages: &[StageReply]) -> Revision {
    stages
        .last()
        .and_then(|s| s.revision)
        .unwrap_or_else(|| panic!("the last stage carries a revision: {stages:?}"))
}

fn refused(out: Vec<ServerMsg>) -> ProtoError {
    match one(out) {
        Reply::Error(e) => e,
        other => panic!("an error expected, got {other:?}"),
    }
}

fn done() -> StateRef {
    StateRef::parse("todo.counter:done").unwrap()
}

fn not_observed(reads: &[&str]) -> ProtoError {
    ProtoError::NotObserved {
        reads: reads.iter().map(|r| StateRef::parse(r).unwrap()).collect(),
    }
}

fn setup() -> (Server, ToyHost) {
    let mut server = Server::new("toy", 4);
    let mut host = ToyHost::new();
    hello(&mut server, &mut host, A, "claude");
    (server, host)
}

/// nothing reached the dispatcher.
fn nothing_dispatched(host: &ToyHost) {
    assert_eq!(
        host.dispatcher.receipt(RequestId::parse("1").unwrap()),
        None
    );
}

/// the scenario of the task with an observation that shows the read
/// after the person moved it: the act rests on that one and goes through.
#[test]
fn an_observation_that_shows_the_read_after_the_change_carries_the_act() {
    let (mut server, mut host) = setup();
    look(&mut server, &mut host, A, "todo.counter");
    host.click("todo.item[1]", "toggle");
    look(&mut server, &mut host, A, "todo.counter");
    let r = look(&mut server, &mut host, A, "todo.clear");
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(r),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));
    let asked = out[0].asked.as_ref().unwrap();
    let shown: Vec<String> = ["1", "3"].map(str::to_owned).into();
    assert_eq!(
        asked.observed.values,
        [(done(), Some(Value::Set(shown.into_iter().collect())))]
    );
    assert_eq!(host.app.items.len(), 1, "items 1 and 3 cleared, as seen");
}

/// a read out of every scope the agent observed is refused before the
/// dispatcher, named, and an observation of it lets the same act through.
#[test]
fn a_read_never_shown_is_not_observed() {
    let (mut server, mut host) = setup();
    let r = look(&mut server, &mut host, A, "todo.clear");
    let e = refused(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(r),
    ));
    assert_eq!(e, not_observed(&["todo.counter:done"]));
    assert!(e.to_string().contains("observe it"), "{e}");
    nothing_dispatched(&host);
    assert_eq!(server.traffic().errors, 1);

    look(&mut server, &mut host, A, "todo.counter");
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(r),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));
}

/// `keys` that cut the key an act reads show the node without it: not
/// shown, even under a scope that holds the node.
#[test]
fn keys_that_cut_the_read_do_not_show_it() {
    let (mut server, mut host) = setup();
    let r = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo","keys":["completed"]"#,
    ))
    .revision;
    let e = refused(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(r),
    ));
    assert_eq!(e, not_observed(&["todo.counter:done"]));
    // the rows' `completed` was kept: a toggle on the same revision runs.
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.item[2]",
        "toggle",
        &from(r),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));

    let r = snapshot(observe(
        &mut server,
        &mut host,
        A,
        r#","scope":"todo","keys":["done"]"#,
    ))
    .revision;
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(r),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));
}

/// a terminal stage's revision (task-87) is taken, and shows nothing: an
/// act chained on it rests on what the agent observed before it, and is
/// refused when the agent never observed what it reads.
#[test]
fn an_act_chained_on_a_stage_revision_rests_on_what_was_observed_before() {
    let (mut server, mut host) = setup();
    let seen = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let draft = r#","param":{"kind":"text","value":"draft"}"#;
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.input",
        "set_value",
        &format!("{draft}{}", from(seen)),
    ));
    let stage = landed(&out);
    assert_ne!(stage, seen);
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.item[2]",
        "toggle",
        &from(stage),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"), "observed before");

    // another agent that saw only the field: the row was never shown.
    hello(&mut server, &mut host, B, "gemini");
    let field = look(&mut server, &mut host, B, "todo.input");
    let out = stages(act(
        &mut server,
        &mut host,
        B,
        "todo.input",
        "set_value",
        &format!("{draft}{}", from(field)),
    ));
    let stage = landed(&out);
    let e = refused(act(
        &mut server,
        &mut host,
        B,
        "todo.item[1]",
        "toggle",
        &from(stage),
    ));
    assert_eq!(e, not_observed(&["todo.item[1]:completed"]));
    assert_eq!(host.app.done(1), Some(false));
}

/// an empty delta since a stage's revision shows no state: the act still
/// rests on the value the agent was last shown, so a change the stage's
/// revision took in and no observation showed makes it stale. `acted`
/// names what ran since the look the act rests on (task-91): the person's
/// toggle and the agent's own write, though the toggle ran before the
/// revision the act names (task-89 named nobody here).
#[test]
fn an_empty_delta_since_a_stage_revision_shows_nothing() {
    let (mut server, mut host) = setup();
    let seen = snapshot(observe(&mut server, &mut host, A, "")).revision;
    host.click("todo.item[1]", "toggle");
    let draft = r#","param":{"kind":"text","value":"draft"}"#;
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.input",
        "set_value",
        &format!("{draft}{}", from(seen)),
    ));
    let stage = landed(&out);
    let write = out[0].request;
    let d = delta(observe(
        &mut server,
        &mut host,
        A,
        &format!(r#","scope":"todo.counter","since":"{stage}""#),
    ));
    assert!(d.changes.is_empty() && d.revision == stage, "{d:?}");

    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(stage),
    ));
    assert_eq!(names(&out), ["requested", "rejected"]);
    let Stage::Rejected(Rejection::Stale { changes, acted }) = &out[1].stage else {
        panic!("stale: {:?}", out[1].stage);
    };
    assert!(
        matches!(&changes[..], [Change::StateChanged { at, added, .. }] if *at == done() && added == &["1"]),
        "{changes:?}"
    );
    let named: Vec<_> = acted.iter().map(|a| (a.id, a.actor.clone())).collect();
    let claude = Actor::Agent {
        name: "claude".into(),
    };
    let toggle = RequestId::parse("1").unwrap();
    assert_eq!(named, [(toggle, Actor::Human), (write, claude)]);
    assert_eq!(host.app.items.len(), 3, "nothing cleared");

    // a delta that carries the change shows it.
    let d = delta(observe(
        &mut server,
        &mut host,
        A,
        &format!(r#","scope":"todo.counter","since":"{seen}""#),
    ));
    assert_eq!(d.changes.len(), 1, "{d:?}");
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(d.revision),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert_eq!(host.app.items.len(), 1);
}

/// the observation that showed a read last left the ring: the read counts
/// as never shown, and an older observation the ring still holds is not
/// used in its place (it is not what the agent saw last). a scoped
/// observation whose revision left the ring can no longer tell which
/// nodes it held, so a read it may have shown is refused too.
#[test]
fn a_read_whose_last_showing_left_the_ring_is_not_observed() {
    let (mut server, mut host) = setup();
    hello(&mut server, &mut host, B, "gemini");
    // r0: the agent sees everything (item 3 done).
    let r0 = snapshot(observe(&mut server, &mut host, A, "")).revision;
    // r1: a person marks item 1 done; the agent sees the counter.
    host.click("todo.item[1]", "toggle");
    let r1 = look(&mut server, &mut host, A, "todo.counter");
    // back to r0, served again by another agent, so the ring keeps it.
    host.click("todo.item[1]", "toggle");
    assert_eq!(look(&mut server, &mut host, B, "todo"), r0);
    // three more revisions push r1 out of a ring of four, not r0.
    for n in 0..3 {
        host.app.text = format!("draft {n}");
        look(&mut server, &mut host, B, "todo");
    }
    assert!(server.ring().get(r1).is_none() && server.ring().get(r0).is_some());
    let now = look(&mut server, &mut host, A, "todo.clear");
    let e = refused(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(now),
    ));
    assert_eq!(e, not_observed(&["todo.counter:done"]));
    let e = refused(act(
        &mut server,
        &mut host,
        A,
        "todo.item[2]",
        "toggle",
        &from(now),
    ));
    assert_eq!(e, not_observed(&["todo.item[2]:completed"]));
    assert_eq!(host.app.done(2), Some(false));

    // observing again is the way on.
    look(&mut server, &mut host, A, "todo.counter");
    let now = look(&mut server, &mut host, A, "todo.item[2]");
    for (target, action) in [("todo.clear", "clear"), ("todo.item[2]", "toggle")] {
        let out = stages(act(&mut server, &mut host, A, target, action, &from(now)));
        assert_eq!(names(&out).last(), Some(&"completed"), "{target}");
    }
}

/// a stage that ends later (a worker's) gives its revision to the actor
/// that asked, as one that ends at once does.
#[test]
fn a_late_stage_gives_its_revision_to_the_actor() {
    let (mut server, mut host) = setup();
    let r = snapshot(observe(&mut server, &mut host, A, "")).revision;
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.sync",
        "sync",
        &from(r),
    ));
    assert_eq!(names(&out), ["requested", "accepted", "executing"]);
    host.app.text = "milk".into();
    host.finish_sync();
    let later: Vec<ServerMsg> = server
        .forward(&mut host)
        .into_iter()
        .map(|o| o.msg)
        .collect();
    let end = landed(&stages(later));
    assert_ne!(end, r, "the field moved meanwhile");
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.item[2]",
        "toggle",
        &from(end),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));
}

/// an actor keeps the last `cap` revisions it was given (the ring's cap):
/// past them, what they showed is forgotten, even while the ring holds
/// the revision.
#[test]
fn an_actor_remembers_the_last_revisions_it_was_given_as_the_ring_does() {
    let (mut server, mut host) = setup();
    let r = look(&mut server, &mut host, A, "todo.counter");
    for _ in 0..server.ring().cap() {
        assert_eq!(look(&mut server, &mut host, A, "todo.clear"), r);
    }
    assert!(server.ring().get(r).is_some(), "nothing moved");
    let e = refused(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &from(r),
    ));
    assert_eq!(e, not_observed(&["todo.counter:done"]));
}

/// a revision given to another actor, or served before hello, is not one
/// this actor can rest on.
#[test]
fn a_revision_given_to_someone_else_is_unknown() {
    let mut server = Server::new("toy", 4);
    let mut host = ToyHost::new();
    let early = snapshot(observe(&mut server, &mut host, A, "")).revision;
    hello(&mut server, &mut host, A, "claude");
    let e = refused(act(
        &mut server,
        &mut host,
        A,
        "todo.item[1]",
        "toggle",
        &from(early),
    ));
    assert_eq!(e, ProtoError::UnknownRevision { revision: early });

    hello(&mut server, &mut host, B, "gemini");
    host.app.text = "milk".into();
    let theirs = snapshot(observe(&mut server, &mut host, B, "")).revision;
    let e = refused(act(
        &mut server,
        &mut host,
        A,
        "todo.item[1]",
        "toggle",
        &from(theirs),
    ));
    assert_eq!(e, ProtoError::UnknownRevision { revision: theirs });
    nothing_dispatched(&host);
}

/// two connections under one name are one actor: what one was shown the
/// other rests on, after the first ended (the `plev` command line opens
/// one connection per command).
#[test]
fn one_name_is_one_actor_across_connections() {
    let (mut server, mut host) = setup();
    look(&mut server, &mut host, A, "todo.counter");
    let r = look(&mut server, &mut host, A, "todo.clear");
    server.close(A);
    hello(&mut server, &mut host, B, "claude");
    let out = stages(act(
        &mut server,
        &mut host,
        B,
        "todo.clear",
        "clear",
        &from(r),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));
    assert_eq!(
        host.dispatcher
            .receipt(out[0].request)
            .map(|r| r.request.actor.clone()),
        Some(Actor::Agent {
            name: "claude".into()
        })
    );
}

/// at most `MAX_ACTORS` actors are remembered: the one served longest ago
/// is forgotten, and its act is refused, never admitted.
#[test]
fn past_the_actors_kept_the_one_served_longest_ago_is_forgotten() {
    let mut server = Server::new("toy", 4);
    let mut host = ToyHost::new();
    hello(&mut server, &mut host, 100, "first");
    let r = look(&mut server, &mut host, 100, "todo");
    for n in 0..MAX_ACTORS {
        let conn = 200 + n as ConnId;
        hello(&mut server, &mut host, conn, &format!("agent{n}"));
        assert_eq!(look(&mut server, &mut host, conn, "todo"), r);
    }
    let e = refused(act(
        &mut server,
        &mut host,
        100,
        "todo.item[1]",
        "toggle",
        &from(r),
    ));
    assert_eq!(e, ProtoError::UnknownRevision { revision: r });
    // the most recent ones are still remembered.
    let last = 200 + (MAX_ACTORS - 1) as ConnId;
    let out = stages(act(
        &mut server,
        &mut host,
        last,
        "todo.clear",
        "clear",
        &from(r),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));
    // observing again makes the first one known again.
    let r = look(&mut server, &mut host, 100, "todo.item[1]");
    let out = stages(act(
        &mut server,
        &mut host,
        100,
        "todo.item[1]",
        "toggle",
        &from(r),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"));
}

/// `observed` given explicitly is the agent's claim, as before: no
/// observation is needed, and it is checked on its reads like any act.
#[test]
fn explicit_observed_values_are_taken_as_the_agent_claims_them() {
    let (mut server, mut host) = setup();
    let claim = |set: &str| {
        format!(
            r#","observed":{{"revision":"0000000000000001","values":[["todo.counter:done",{{"kind":"set","value":[{set}]}}]]}}"#
        )
    };
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &claim(r#""1""#),
    ));
    assert_eq!(names(&out), ["requested", "rejected"], "stale on its reads");
    let out = stages(act(
        &mut server,
        &mut host,
        A,
        "todo.clear",
        "clear",
        &claim(r#""3""#),
    ));
    assert_eq!(names(&out).last(), Some(&"completed"), "never observed");
    assert_eq!(host.app.items.len(), 2);
}
