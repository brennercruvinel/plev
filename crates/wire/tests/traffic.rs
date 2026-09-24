//! what a port tells the app about the traffic it carries: `Host::observed`
//! after every observation served to a named actor, and `Traffic`, the
//! counters the server keeps (observations, acts, stages, stale
//! rejections, errors) and the transport adds (connections, lines and
//! bytes both ways). the server side runs in process against the toy; the
//! transport side over a real socket in a temporary directory.

#![cfg(unix)]

mod toy;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use sem::{Actor, ReceiptSink, Request, RequestId, Revision, SemId, Snapshot};
use toy::ToyHost;
use wire::{
    AgentPort, Bound, Config, ConnId, Host, Reply, Server, ServerMsg, Traffic, decode_client,
    decode_server,
};

const A: ConnId = 1;
const B: ConnId = 2;

/// the toy's host with an ear on `observed`.
struct Ear {
    host: ToyHost,
    heard: Vec<(Actor, Option<SemId>, Revision)>,
}

impl Ear {
    fn new() -> Self {
        Ear {
            host: ToyHost::new(),
            heard: Vec::new(),
        }
    }
}

impl Host for Ear {
    fn project(&self) -> Option<Snapshot> {
        self.host.project()
    }

    fn bounds(&self) -> Vec<Bound> {
        self.host.bounds()
    }

    fn dispatch(&mut self, req: Request) -> Option<RequestId> {
        self.host.dispatch(req)
    }

    fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        self.host.add_sink(sink);
    }

    fn observed(&mut self, actor: &Actor, scope: Option<&SemId>, revision: Revision) {
        self.heard.push((actor.clone(), scope.cloned(), revision));
    }
}

/// one line from `conn`, as the port would take it.
fn say(server: &mut Server, host: &mut dyn Host, conn: ConnId, line: &str) -> Vec<ServerMsg> {
    match decode_client(line) {
        Ok(msg) => server
            .handle(conn, msg, host)
            .into_iter()
            .map(|o| o.msg)
            .collect(),
        Err(r) => vec![r.reply()],
    }
}

fn one(out: Vec<ServerMsg>) -> Reply {
    match <[ServerMsg; 1]>::try_from(out) {
        Ok([msg]) => msg.reply,
        Err(out) => panic!("one reply expected, got {out:?}"),
    }
}

fn revision_of(reply: &Reply) -> Revision {
    match reply {
        Reply::Snapshot(s) => s.revision,
        Reply::Delta(d) => d.revision,
        other => panic!("{other:?}"),
    }
}

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn claude() -> Actor {
    Actor::Agent {
        name: "claude".into(),
    }
}

#[test]
fn an_observation_served_to_a_named_actor_reaches_the_host() {
    let mut server = Server::new("toy", 4);
    let mut ear = Ear::new();

    // before hello there is no actor: served, not reported.
    let early = one(say(
        &mut server,
        &mut ear,
        A,
        r#"{"v":1,"id":"0","type":"observe"}"#,
    ));
    assert!(matches!(early, Reply::Snapshot(_)));
    assert!(ear.heard.is_empty(), "{:?}", ear.heard);

    say(
        &mut server,
        &mut ear,
        A,
        r#"{"v":1,"id":"h","type":"hello","name":"claude"}"#,
    );
    let whole = one(say(
        &mut server,
        &mut ear,
        A,
        r#"{"v":1,"id":"1","type":"observe"}"#,
    ));
    let r1 = revision_of(&whole);
    assert_eq!(ear.heard, [(claude(), None, r1)]);

    // a scoped observation reports the scope and the whole revision.
    let scoped = one(say(
        &mut server,
        &mut ear,
        A,
        r#"{"v":1,"id":"2","type":"observe","scope":"todo.item[1]"}"#,
    ));
    assert_eq!(revision_of(&scoped), r1, "nothing moved");
    assert_eq!(ear.heard[1], (claude(), Some(sid("todo.item[1]")), r1));

    // a person moves the app; a delta reports the new revision.
    ear.host.click("todo.item[2]", "toggle");
    let delta = one(say(
        &mut server,
        &mut ear,
        A,
        &format!(r#"{{"v":1,"id":"3","type":"observe","since":"{r1}"}}"#),
    ));
    assert!(matches!(delta, Reply::Delta(_)), "{delta:?}");
    let r2 = revision_of(&delta);
    assert_ne!(r2, r1);
    assert_eq!(ear.heard[2], (claude(), None, r2));
    assert_eq!(ear.heard.len(), 3);

    // a `since` the ring does not hold answers the whole snapshot, which
    // is served and reported like any other.
    let unknown = say(
        &mut server,
        &mut ear,
        A,
        r#"{"v":1,"id":"4","type":"observe","since":"0000000000000000"}"#,
    );
    assert!(matches!(one(unknown), Reply::Snapshot(_)));
    assert_eq!(ear.heard.len(), 4);
    assert_eq!(ear.heard[3], (claude(), None, r2));

    // the revision a terminal stage carries is not an observation: the
    // agent is given its name, not what it holds, and nothing is reported.
    let stages = say(
        &mut server,
        &mut ear,
        A,
        &format!(
            r#"{{"v":1,"id":"5","type":"act","target":"todo.item[1]","action":"toggle","from_revision":"{r2}"}}"#
        ),
    );
    let Some(Reply::Stage(end)) = stages.last().map(|m| &m.reply) else {
        panic!("{stages:?}");
    };
    assert!(end.is_terminal() && end.revision.is_some(), "{end:?}");
    assert_eq!(ear.heard.len(), 4, "{:?}", ear.heard);
}

#[test]
fn the_server_counts_what_it_answered() {
    let mut server = Server::new("toy", 4);
    let mut host = ToyHost::new();
    assert_eq!(server.traffic(), Traffic::default());

    say(
        &mut server,
        &mut host,
        A,
        r#"{"v":1,"id":"h","type":"hello","name":"claude"}"#,
    );
    let r = revision_of(&one(say(
        &mut server,
        &mut host,
        A,
        r#"{"v":1,"id":"1","type":"observe"}"#,
    )));
    say(
        &mut server,
        &mut host,
        A,
        &format!(r#"{{"v":1,"id":"2","type":"observe","since":"{r}"}}"#),
    );
    let t = server.traffic();
    assert_eq!((t.observes_snapshot, t.observes_delta), (1, 1));
    assert_eq!(
        (t.acts, t.stages_out, t.rejected_stale, t.errors),
        (0, 0, 0, 0)
    );

    // an act that completes: four stages out.
    let toggled = say(
        &mut server,
        &mut host,
        A,
        &format!(
            r#"{{"v":1,"id":"3","type":"act","target":"todo.item[1]","action":"toggle","from_revision":"{r}"}}"#
        ),
    );
    assert_eq!(toggled.len(), 4, "{toggled:?}");
    assert_eq!(host.app.done(1), Some(true));
    let t = server.traffic();
    assert_eq!((t.acts, t.stages_out, t.rejected_stale), (1, 4, 0));
    // its `completed` carries a revision, which is not an observation.
    assert!(matches!(
        toggled.last().map(|m| &m.reply),
        Some(Reply::Stage(s)) if s.revision.is_some()
    ));
    assert_eq!((t.observes_snapshot, t.observes_delta), (1, 1));

    // a stage that ends later is counted when it goes out, not before.
    let started = say(
        &mut server,
        &mut host,
        A,
        &format!(
            r#"{{"v":1,"id":"s","type":"act","target":"todo.sync","action":"sync","from_revision":"{r}"}}"#
        ),
    );
    assert_eq!(
        started.len(),
        3,
        "requested, accepted, executing: {started:?}"
    );
    assert_eq!(server.traffic().stages_out, 7);
    host.finish_sync();
    assert_eq!(server.forward(&mut host).len(), 1, "the completed stage");
    assert_eq!(server.traffic().stages_out, 8);

    // an act on a field a person changed since: requested, rejected stale.
    host.app.text = "buy bread".into();
    let r = revision_of(&one(say(
        &mut server,
        &mut host,
        A,
        r#"{"v":1,"id":"4","type":"observe"}"#,
    )));
    host.app.text = "buy bread and eggs".into();
    let stale = say(
        &mut server,
        &mut host,
        A,
        &format!(
            r#"{{"v":1,"id":"5","type":"act","target":"todo.input","action":"submit","from_revision":"{r}"}}"#
        ),
    );
    assert_eq!(stale.len(), 2, "{stale:?}");
    let t = server.traffic();
    assert_eq!((t.acts, t.stages_out, t.rejected_stale), (3, 10, 1));

    // an act before hello is an error, counted as an act asked and an
    // error answered.
    let refused = one(say(
        &mut server,
        &mut host,
        B,
        &format!(
            r#"{{"v":1,"id":"6","type":"act","target":"todo.item[1]","action":"toggle","from_revision":"{r}"}}"#
        ),
    ));
    assert!(matches!(refused, Reply::Error(_)));
    let t = server.traffic();
    assert_eq!((t.acts, t.errors), (4, 1));
    assert_eq!(
        (t.conns, t.msgs_in, t.msgs_out, t.bytes_in, t.bytes_out),
        (0, 0, 0, 0, 0),
        "the transport counts these, not the server"
    );
}

/// a port for the toy in `dir`, and the channel its wake sends on.
fn open(dir: &Path) -> (AgentPort, Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    let config = Config {
        app: "toy".into(),
        dir: dir.to_owned(),
        ring: 8,
    };
    let port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .unwrap();
    (port, rx)
}

/// what the raw client sent and got back: bytes written, lines read.
struct Exchange {
    sent: u64,
    lines: Vec<String>,
}

/// a raw client: known lines in, every line out read back, so the bytes
/// can be checked to the byte. hello, observe, an act on the revision
/// observed, a line that is not JSON, then hangs up.
fn exchange(path: &Path) -> Exchange {
    let mut stream = UnixStream::connect(path).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut sent = 0u64;
    let mut lines = Vec::new();
    let mut send = |stream: &mut UnixStream, line: &str| {
        stream.write_all(line.as_bytes()).unwrap();
        sent += line.len() as u64;
    };
    let read = |reader: &mut BufReader<UnixStream>| -> String {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        line
    };
    send(
        &mut stream,
        "{\"v\":1,\"id\":\"1\",\"type\":\"hello\",\"name\":\"raw\"}\n",
    );
    lines.push(read(&mut reader));
    send(&mut stream, "{\"v\":1,\"id\":\"2\",\"type\":\"observe\"}\n");
    let snapshot = read(&mut reader);
    let r = revision_of(&decode_server(snapshot.trim_end()).unwrap().reply);
    lines.push(snapshot);
    send(
        &mut stream,
        &format!(
            "{{\"v\":1,\"id\":\"3\",\"type\":\"act\",\"target\":\"todo.item[1]\",\"action\":\"toggle\",\"from_revision\":\"{r}\"}}\n"
        ),
    );
    for _ in 0..4 {
        lines.push(read(&mut reader));
    }
    send(&mut stream, "this is not json\n");
    lines.push(read(&mut reader));
    Exchange { sent, lines }
}

#[test]
fn the_port_counts_lines_and_bytes_both_ways() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let path = port.path().to_owned();
    let agent = thread::spawn(move || exchange(&path));
    let deadline = Instant::now() + Duration::from_secs(20);
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the client did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(5));
        port.drain(&mut host);
    }
    let got = agent.join().unwrap();
    assert_eq!(got.lines.len(), 7, "{:?}", got.lines);
    let bytes_out: u64 = got.lines.iter().map(|l| l.len() as u64).sum();

    // the writer counts after the write lands; the client may have read
    // the last line a moment before that.
    let t = loop {
        let t = port.traffic();
        if t.msgs_out == 7 || Instant::now() > deadline {
            break t;
        }
        thread::sleep(Duration::from_millis(1));
    };
    assert_eq!(t.conns, 1);
    assert_eq!(t.msgs_in, 4, "hello, observe, act, the broken line");
    assert_eq!(t.bytes_in, got.sent);
    assert_eq!(t.msgs_out, 7, "welcome, snapshot, four stages, one error");
    assert_eq!(t.bytes_out, bytes_out);
    assert_eq!(t.observes_snapshot, 1);
    assert_eq!(t.observes_delta, 0);
    assert_eq!(t.acts, 1);
    assert_eq!(t.stages_out, 4);
    assert_eq!(t.rejected_stale, 0);
    assert_eq!(t.errors, 1, "the line that was not json");
    assert!(t.bytes_out > t.bytes_in, "replies outweigh asks here");
}
