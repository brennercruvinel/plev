//! who acted since the agent looked (task-91): a stale refusal of an act
//! the port dispatched names the requests that ran after the look the act
//! rests on, the oldest observation that showed what it reads, whatever
//! revision the act names. the toy's `todo.clear` reads the counter's
//! `done`, state of another node, as the ide's `commit.submit` reads
//! `changes:staged`.

mod toy;

use sem::{Acted, Actor, Rejection, RequestId, Revision, SemId, Stage};
use toy::ToyHost;
use wire::{ConnId, Host, Reply, Server, ServerMsg, StageReply, decode_client};

const A: ConnId = 1;

fn say(server: &mut Server, host: &mut dyn Host, conn: ConnId, line: &str) -> Vec<ServerMsg> {
    let msg = decode_client(line).unwrap_or_else(|r| panic!("{line}: {r:?}"));
    server
        .handle(conn, msg, host)
        .into_iter()
        .map(|o| o.msg)
        .collect()
}

fn hello(server: &mut Server, host: &mut dyn Host, conn: ConnId, name: &str) {
    let line = format!(r#"{{"v":1,"id":"h","type":"hello","name":"{name}"}}"#);
    assert!(matches!(
        say(server, host, conn, &line).as_slice(),
        [ServerMsg {
            reply: Reply::Welcome { .. },
            ..
        }]
    ));
}

/// the revision of an observation of `scope`.
fn look(server: &mut Server, host: &mut dyn Host, conn: ConnId, scope: &str) -> Revision {
    let line = format!(r#"{{"v":1,"id":"o","type":"observe","scope":"{scope}"}}"#);
    match say(server, host, conn, &line).as_slice() {
        [
            ServerMsg {
                reply: Reply::Snapshot(s),
                ..
            },
        ] => s.revision,
        other => panic!("a snapshot expected, got {other:?}"),
    }
}

/// an act on `from_revision`: its stages.
fn act(
    server: &mut Server,
    host: &mut dyn Host,
    conn: ConnId,
    target: &str,
    action: &str,
    from: Revision,
) -> Vec<StageReply> {
    let line = format!(
        r#"{{"v":1,"id":"a","type":"act","target":"{target}","action":"{action}","from_revision":"{from}"}}"#
    );
    say(server, host, conn, &line)
        .into_iter()
        .map(|msg| match msg.reply {
            Reply::Stage(s) => s,
            other => panic!("a stage expected, got {other:?}"),
        })
        .collect()
}

fn names(stages: &[StageReply]) -> Vec<&'static str> {
    stages.iter().map(|s| s.stage.name()).collect()
}

/// who the stale refusal that ends `stages` names.
fn acted(stages: &[StageReply]) -> Vec<Acted> {
    match stages.last().map(|s| &s.stage) {
        Some(Stage::Rejected(Rejection::Stale { acted, .. })) => acted.clone(),
        other => panic!("a stale refusal, got {other:?}"),
    }
}

fn entry(id: u64, actor: Actor, target: &str, action: &str) -> Acted {
    Acted {
        id: RequestId::parse(&id.to_string()).unwrap(),
        actor,
        target: SemId::parse(target).unwrap(),
        action: action.parse().unwrap(),
    }
}

fn setup() -> (Server, ToyHost) {
    let mut server = Server::new("toy", 4);
    let mut host = ToyHost::new();
    hello(&mut server, &mut host, A, "claude");
    (server, host)
}

/// the reproduction of task-89, as the orchestrator ran it in the
/// laboratory: the agent sees what `clear` would remove (the counter's
/// `done`), a person marks item 1 done, the agent observes only the
/// button and clears on the revision that observation served. the act
/// rests on the counter it was shown first, so it is refused stale, and
/// the person, who acted after that look, is named, though her toggle was
/// checked on no revision the act names.
#[test]

fn the_person_between_the_look_and_the_revision_acted_on_is_named() {
    let (mut server, mut host) = setup();
    look(&mut server, &mut host, A, "todo.counter");
    host.click("todo.item[1]", "toggle");
    let r = look(&mut server, &mut host, A, "todo.clear");
    let out = act(&mut server, &mut host, A, "todo.clear", "clear", r);
    assert_eq!(names(&out), ["requested", "rejected"]);
    assert_eq!(
        acted(&out),
        [entry(1, Actor::Human, "todo.item[1]", "toggle")]
    );
    assert_eq!(host.app.items.len(), 3, "nothing was cleared");
    assert_eq!(host.app.done(1), Some(true));
}

/// the agent's own acts that ran after its look are named too (the
/// refusal by the agent's own write, task-62): the agent looks at the
/// counter, starts a sync, which still runs, a person marks item 1 done,
/// and the clear on that look names the sync and the toggle, in the order
/// they were taken. what ran before the look is not named.
#[test]

fn the_agents_own_act_after_its_look_is_named_while_it_runs() {
    let (mut server, mut host) = setup();
    host.click("todo.item[2]", "toggle");
    let r = look(&mut server, &mut host, A, "todo");
    let sync = act(&mut server, &mut host, A, "todo.sync", "sync", r);
    assert_eq!(names(&sync), ["requested", "accepted", "executing"]);
    host.click("todo.item[1]", "toggle");
    let out = act(&mut server, &mut host, A, "todo.clear", "clear", r);
    assert_eq!(names(&out), ["requested", "rejected"]);
    let claude = Actor::Agent {
        name: "claude".into(),
    };
    assert_eq!(
        acted(&out),
        [
            entry(2, claude, "todo.sync", "sync"),
            entry(3, Actor::Human, "todo.item[1]", "toggle"),
        ]
    );
    host.finish_sync();
}
