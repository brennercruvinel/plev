//! the port over a real unix socket in a temporary directory: the test's
//! own thread is the app's (it drains the port and holds the toy), a
//! spawned thread is the agent.

#![cfg(unix)]

mod toy;

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use sem::{ActionName, Actor, Rejection, SemId, Stage, Value};
use toy::ToyHost;
use wire::{
    Act, AgentPort, Client, Config, MAX_LINE, Observe, ProtoError, Reply, WireError, find,
    socket_path,
};

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn action(text: &str) -> ActionName {
    ActionName::parse(text).unwrap()
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

/// what the agent thread asks of the app's thread between two messages.
enum Poke {
    /// a person edits the field.
    Type(String),
}

/// runs `agent` on its own thread while this thread serves the port the
/// way a shell does: wait for the wake, drain; between two drains, a poke
/// from the agent moves the app as a person would.
fn serve<T: Send + 'static>(
    port: &mut AgentPort,
    woke: &Receiver<()>,
    host: &mut ToyHost,
    agent: impl FnOnce(PathBuf, Sender<(Poke, Sender<()>)>) -> T + Send + 'static,
) -> T {
    let path = port.path().to_owned();
    let (poke_tx, pokes) = mpsc::channel::<(Poke, Sender<()>)>();
    let agent = thread::spawn(move || agent(path, poke_tx));
    let deadline = Instant::now() + Duration::from_secs(20);
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the agent did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(5));
        port.drain(host);
        while let Ok((poke, done)) = pokes.try_recv() {
            match poke {
                Poke::Type(text) => host.app.text = text,
            }
            let _ = done.send(());
        }
    }
    agent.join().unwrap()
}

fn poke(pokes: &Sender<(Poke, Sender<()>)>, poke: Poke) {
    let (done, wait) = mpsc::channel();
    pokes.send((poke, done)).unwrap();
    wait.recv().unwrap();
}

fn client(path: &Path, name: &str) -> Client {
    let mut c = Client::connect(path).unwrap();
    c.set_timeout(Some(10_000)).unwrap();
    assert_eq!(c.hello(name).unwrap(), Actor::Agent { name: name.into() });
    c
}

fn revision_of(reply: &Reply) -> sem::Revision {
    match reply {
        Reply::Snapshot(s) => s.revision,
        Reply::Delta(d) => d.revision,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_socket_is_private_and_goes_away_with_the_port() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("plev");
    let (port, _woke) = open(&dir);
    let path = socket_path(&dir, "toy", std::process::id());
    assert_eq!(port.path(), path);
    let meta = std::fs::symlink_metadata(&path).unwrap();
    assert!(meta.file_type().is_socket());
    assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    let dir_mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(dir_mode, 0o700);
    assert_eq!(find(&dir, Some("toy")).unwrap(), path);
    drop(port);
    assert!(!path.exists(), "removed on drop");
    assert!(matches!(find(&dir, Some("toy")), Err(WireError::NoApp(_))));
}

/// the acceptance over the socket: observe, act accepted and completed,
/// and the stale case, with the delta.
#[test]
fn observe_act_and_the_stale_case_over_the_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let (toggled, stale, delta) = serve(&mut port, &woke, &mut host, |path, pokes| {
        let mut c = client(&path, "claude");
        let seen = c
            .observe(Observe {
                scope: Some(sid("todo")),
                ..Observe::default()
            })
            .unwrap();
        let r = revision_of(&seen);
        let toggled = c
            .act(
                Act::from_revision(sid("todo.item[1]"), action("toggle"), r),
                |_| {},
            )
            .unwrap();

        poke(&pokes, Poke::Type("buy bread".into()));
        let r = revision_of(&c.observe(Observe::default()).unwrap());
        poke(&pokes, Poke::Type("buy bread and eggs".into()));
        let stale = c
            .act(
                Act::from_revision(sid("todo.input"), action("submit"), r),
                |_| {},
            )
            .unwrap();
        let delta = c
            .observe(Observe {
                since: Some(r),
                ..Observe::default()
            })
            .unwrap();
        (toggled, stale, delta)
    });

    let names: Vec<&str> = toggled.iter().map(|s| s.stage.name()).collect();
    assert_eq!(names, ["requested", "accepted", "executing", "completed"]);
    assert_eq!(host.app.done(1), Some(true));
    let receipt = host.dispatcher.receipt(toggled[0].request).unwrap();
    assert_eq!(
        receipt.request.actor,
        Actor::Agent {
            name: "claude".into()
        }
    );

    let names: Vec<&str> = stale.iter().map(|s| s.stage.name()).collect();
    assert_eq!(names, ["requested", "rejected"]);
    let Stage::Rejected(Rejection::Stale { changes }) = &stale[1].stage else {
        panic!("{:?}", stale[1].stage);
    };
    assert!(matches!(
        &changes[..],
        [sem::Change::StateChanged { before: Some(Value::Text(b)), after: Some(Value::Text(a)), .. }]
            if b == "buy bread" && a == "buy bread and eggs"
    ));
    assert_eq!(host.app.items.len(), 3, "nothing was added");

    let Reply::Delta(delta) = delta else {
        panic!("a delta: {delta:?}");
    };
    assert_eq!(delta.changes.len(), 1, "only the field moved: {delta:?}");
}

#[test]
fn a_broken_line_answers_an_error_and_the_connection_goes_on() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let replies = serve(&mut port, &woke, &mut host, |path, _| {
        let mut raw = UnixStream::connect(&path).unwrap();
        raw.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        raw.write_all(b"not json\n\n   \r\n").unwrap();
        raw.write_all(b"{\"v\":2,\"id\":\"old\",\"type\":\"hello\",\"name\":\"x\"}\r\n")
            .unwrap();
        raw.write_all(b"\xff\xfe\n").unwrap();
        raw.write_all(b"{\"v\":1,\"id\":\"ok\",\"type\":\"hello\",\"name\":\"x\"}\n")
            .unwrap();
        let mut reader = BufReader::new(raw);
        let mut lines = Vec::new();
        for _ in 0..4 {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            lines.push(wire::decode_server(line.trim_end()).unwrap());
        }
        lines
    });
    let errors: Vec<Option<&str>> = replies.iter().map(|m| m.id.as_deref()).collect();
    assert_eq!(errors, [None, Some("old"), None, Some("ok")]);
    assert!(matches!(
        &replies[1].reply,
        Reply::Error(ProtoError::BadVersion { got }) if got == "2"
    ));
    assert!(matches!(&replies[3].reply, Reply::Welcome { .. }));
}

#[test]
fn a_line_past_the_limit_ends_the_connection() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let (reply, rest) = serve(&mut port, &woke, &mut host, |path, _| {
        let mut raw = UnixStream::connect(&path).unwrap();
        raw.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        let reader = raw.try_clone().unwrap();
        // the port stops reading past the limit; a write may then fail.
        let _ = raw.write_all(&vec![b'x'; MAX_LINE + 10]);
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let mut rest = Vec::new();
        let _ = reader.read_to_end(&mut rest);
        (line, rest)
    });
    let msg = wire::decode_server(reply.trim_end()).unwrap();
    assert!(matches!(
        msg.reply,
        Reply::Error(ProtoError::BadMessage { reason }) if reason.contains("at most")
    ));
    assert!(rest.is_empty(), "then the connection ends");
}

#[test]
fn two_agents_at_once_each_under_its_own_name() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let ids = serve(&mut port, &woke, &mut host, |path, _| {
        let mut a = client(&path, "claude");
        let mut b = client(&path, "gemini");
        let ra = revision_of(&a.observe(Observe::default()).unwrap());
        let rb = revision_of(&b.observe(Observe::default()).unwrap());
        let sa = a
            .act(
                Act::from_revision(sid("todo.item[1]"), action("toggle"), ra),
                |_| {},
            )
            .unwrap();
        let sb = b
            .act(
                Act::from_revision(sid("todo.item[2]"), action("toggle"), rb),
                |_| {},
            )
            .unwrap();
        // confirming is a person's, never an agent's.
        let e = a.call(wire::Ask::Confirm {
            request: sb[0].request,
        });
        assert!(matches!(
            e,
            Err(WireError::Refused(ProtoError::NotEnabled { .. }))
        ));
        (sa[0].request, sb[0].request)
    });
    let actor = |id| host.dispatcher.receipt(id).unwrap().request.actor.clone();
    assert_eq!(
        actor(ids.0),
        Actor::Agent {
            name: "claude".into()
        }
    );
    assert_eq!(
        actor(ids.1),
        Actor::Agent {
            name: "gemini".into()
        }
    );
    assert_eq!(
        (host.app.done(1), host.app.done(2)),
        (Some(true), Some(true))
    );
}

/// a request that ends later reaches the agent when the shell drains
/// after the work ends.
#[test]
fn a_stage_that_ends_later_reaches_the_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let path = port.path().to_owned();
    let (seen_tx, seen) = mpsc::channel();
    let agent = thread::spawn(move || {
        let mut c = client(&path, "claude");
        let r = revision_of(&c.observe(Observe::default()).unwrap());
        c.act(
            Act::from_revision(sid("todo.sync"), action("sync"), r),
            |s| {
                let _ = seen_tx.send(s.stage.name());
            },
        )
        .unwrap()
    });
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut heard = Vec::new();
    while !agent.is_finished() {
        assert!(Instant::now() < deadline);
        let _ = woke.recv_timeout(Duration::from_millis(5));
        port.drain(&mut host);
        heard.extend(seen.try_iter());
        if heard.last() == Some(&"executing") && !host.app.syncing.is_empty() {
            // a person clicks meanwhile, on the same dispatcher: the
            // agent never hears that receipt.
            host.click("todo.item[2]", "toggle");
            host.finish_sync();
        }
    }
    let stages = agent.join().unwrap();
    let names: Vec<&str> = stages.iter().map(|s| s.stage.name()).collect();
    assert_eq!(names, ["requested", "accepted", "executing", "completed"]);
    assert_eq!(host.app.done(2), Some(true));
}

#[test]
fn a_bad_place_keeps_the_port_closed() {
    let tmp = tempfile::tempdir().unwrap();
    let wake = || {};
    let config = |app: &str, dir: &Path| Config {
        app: app.into(),
        dir: dir.to_owned(),
        ring: 4,
    };
    for app in ["", "a/b", "../x", "toy app", &"x".repeat(33)] {
        let e = AgentPort::open_with(config(app, tmp.path()), wake).err();
        assert!(matches!(e, Some(WireError::BadApp(_))), "{app:?}: {e:?}");
    }
    // a symlink where the directory goes.
    let real = tmp.path().join("real");
    std::fs::create_dir(&real).unwrap();
    let link = tmp.path().join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let e = AgentPort::open_with(config("toy", &link), wake).err();
    assert!(matches!(e, Some(WireError::NotPrivate(_))), "{e:?}");
    // a loose directory is tightened to 0700.
    let loose = tmp.path().join("loose");
    std::fs::create_dir(&loose).unwrap();
    std::fs::set_permissions(&loose, std::fs::Permissions::from_mode(0o777)).unwrap();
    drop(AgentPort::open_with(config("toy", &loose), wake).unwrap());
    let mode = std::fs::metadata(&loose).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    // a file where the socket goes is left alone.
    let path = socket_path(&loose, "toy", std::process::id());
    std::fs::write(&path, b"mine").unwrap();
    let e = AgentPort::open_with(config("toy", &loose), wake).err();
    assert!(matches!(e, Some(WireError::NotASocket(_))), "{e:?}");
    assert_eq!(std::fs::read(&path).unwrap(), b"mine");
    std::fs::remove_file(&path).unwrap();
    // a socket a dead process of our pid left behind is replaced.
    drop(UnixListener::bind(&path).unwrap());
    assert!(path.exists());
    let port = AgentPort::open_with(config("toy", &loose), wake).unwrap();
    assert!(UnixStream::connect(port.path()).is_ok());
}

#[test]
fn find_names_the_one_live_socket() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    assert!(matches!(find(dir, None), Err(WireError::NoApp(None))));
    let _a = UnixListener::bind(dir.join("showcase-11.sock")).unwrap();
    let _b = UnixListener::bind(dir.join("ide-12.sock")).unwrap();
    // a socket whose process ended: nobody answers.
    drop(UnixListener::bind(dir.join("showcase-10.sock")).unwrap());
    std::fs::write(dir.join("notes.txt"), b"x").unwrap();
    assert_eq!(
        find(dir, Some("showcase")).unwrap(),
        dir.join("showcase-11.sock")
    );
    assert_eq!(find(dir, Some("ide")).unwrap(), dir.join("ide-12.sock"));
    assert!(matches!(
        find(dir, Some("urna")),
        Err(WireError::NoApp(Some(_)))
    ));
    match find(dir, None) {
        Err(WireError::Ambiguous { found, .. }) => assert_eq!(found.len(), 2),
        other => panic!("{other:?}"),
    }
    let _c = UnixListener::bind(dir.join("showcase-13.sock")).unwrap();
    assert!(matches!(
        find(dir, Some("showcase")),
        Err(WireError::Ambiguous { .. })
    ));
}

/// the acceptance: with `PLEV_AGENT` unset (or not `1`) no socket is
/// created. the variable is read by `from_env` in a child process of this
/// test binary, so no test changes the environment of another. `1` opens
/// the port only in a build with debug assertions.
#[test]
fn without_the_env_var_no_socket_is_created() {
    if std::env::var_os("WIRE_T61_CHILD").is_some() {
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let one = if cfg!(debug_assertions) {
        "open"
    } else {
        "none"
    };
    for (value, expect) in [(None, "none"), (Some("0"), "none"), (Some("1"), one)] {
        let tmp = tempfile::tempdir().unwrap();
        let mut cmd = Command::new(&exe);
        cmd.args([
            "--exact",
            "from_env_in_a_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("WIRE_T61_CHILD", "1")
        .env("TMPDIR", tmp.path());
        match value {
            None => cmd.env_remove("PLEV_AGENT"),
            Some(v) => cmd.env("PLEV_AGENT", v),
        };
        let out = cmd.output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{value:?}: {stdout}");
        assert!(
            stdout.contains(&format!("port: {expect}")),
            "{value:?}: {stdout}"
        );
        let dir = tmp.path().join("plev");
        if expect == "none" {
            assert!(!dir.exists(), "{value:?}: nothing was created");
        }
    }
}

#[test]
fn from_env_in_a_child() {
    if std::env::var_os("WIRE_T61_CHILD").is_none() {
        return;
    }
    let port = AgentPort::from_env("toy", || {}).unwrap();
    let sockets = std::fs::read_dir(std::env::temp_dir().join("plev"))
        .map(|d| d.filter_map(Result::ok).count())
        .unwrap_or(0);
    assert_eq!(port.is_some(), sockets == 1);
    println!("port: {}", if port.is_some() { "open" } else { "none" });
}

/// a few hundred random lines through a real socket: each gets its error
/// or its answer, the port lives on, and the connection still observes.
#[test]
fn random_lines_over_the_socket_never_break_the_port() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let r = host.snapshot().revision();
    let pieces = [
        r#"{"v":1,"id":"f","type":"#.to_owned(),
        r#""act","target":"todo.item[1]","action":"toggle""#.to_owned(),
        r#""observe","scope":"todo""#.to_owned(),
        r#""hello","name":"fuzz""#.to_owned(),
        format!(r#","from_revision":"{r}""#),
        r#","param":{"kind":"text","value":"1"}"#.to_owned(),
        r#""confirm","request":"1""#.to_owned(),
        "}".to_owned(),
        "{".to_owned(),
        ",".to_owned(),
        "\u{0}\u{7f}\u{e9}\u{1F600}".to_owned(),
    ];
    let (answered, last) = serve(&mut port, &woke, &mut host, move |path, _| {
        let mut raw = UnixStream::connect(&path).unwrap();
        raw.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        let reader = raw.try_clone().unwrap();
        let writer = thread::spawn(move || {
            let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
            let mut next = move || {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed
            };
            for _ in 0..400 {
                let mut line = Vec::new();
                for _ in 0..(next() % 10) {
                    if next() % 3 == 0 {
                        line.push((next() % 256) as u8);
                    } else {
                        let piece = &pieces[(next() as usize) % pieces.len()];
                        line.extend_from_slice(piece.as_bytes());
                    }
                }
                line.retain(|b| *b != b'\n');
                line.push(b'\n');
                raw.write_all(&line).unwrap();
            }
            raw.write_all(b"{\"v\":1,\"id\":\"last\",\"type\":\"observe\"}\n")
                .unwrap();
        });
        let mut reader = BufReader::new(reader);
        let mut answered = 0;
        let last = loop {
            let mut line = String::new();
            assert!(reader.read_line(&mut line).unwrap() > 0, "the port hung up");
            let msg = wire::decode_server(line.trim_end()).unwrap();
            answered += 1;
            if msg.id.as_deref() == Some("last") {
                break msg.reply;
            }
        };
        writer.join().unwrap();
        (answered, last)
    });
    assert!(answered > 100, "{answered}");
    assert!(matches!(last, Reply::Snapshot(_)), "{last:?}");
}
