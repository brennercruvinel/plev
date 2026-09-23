//! the `plev` binary against the toy app's port: what it prints and how it
//! exits (0 observed or completed, 1 rejected, 2 failed, 3 error).

#![cfg(unix)]

mod toy;

use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use sem::Revision;
use toy::ToyHost;
use wire::{AgentPort, Config, ProtoError, Reply, ServerMsg, decode_server};

struct Run {
    code: i32,
    out: Vec<ServerMsg>,
    err: String,
}

/// runs `plev <args>` with `TMPDIR` at `tmp` while this thread serves the
/// port as the app's shell would.
fn plev(
    port: &mut AgentPort,
    woke: &Receiver<()>,
    host: &mut ToyHost,
    tmp: &Path,
    args: &[&str],
) -> Run {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_plev"));
    cmd.args(args)
        .env("TMPDIR", tmp)
        .env("PLEV_AGENT_NAME", "cli");
    let child = thread::spawn(move || cmd.output().unwrap());
    let deadline = Instant::now() + Duration::from_secs(30);
    while !child.is_finished() {
        assert!(Instant::now() < deadline, "plev did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(5));
        port.drain(host);
        // a sync ends a turn after it started, as a worker's result would,
        // and the shell drains again to send the late stage.
        if !host.app.syncing.is_empty() {
            host.finish_sync();
            port.drain(host);
        }
    }
    let output = child.join().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    Run {
        code: output.status.code().unwrap_or(-1),
        out: stdout
            .lines()
            .filter(|l| l.starts_with('{'))
            .map(|l| decode_server(l).unwrap_or_else(|e| panic!("{l}: {e:?}")))
            .collect(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn open(tmp: &Path) -> (AgentPort, Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    let config = Config {
        app: "toy".into(),
        dir: tmp.join("plev"),
        ring: 8,
    };
    let port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .unwrap();
    (port, rx)
}

fn revision(run: &Run) -> Revision {
    match run.out.first().map(|m| &m.reply) {
        Some(Reply::Snapshot(s)) => s.revision,
        other => panic!("{other:?}"),
    }
}

fn stages(run: &Run) -> Vec<&'static str> {
    run.out
        .iter()
        .map(|m| match &m.reply {
            Reply::Stage(s) => s.stage.name(),
            other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn observe_and_act_by_app_name_with_their_exit_codes() {
    let tmp = tempfile::tempdir().unwrap();
    let (mut port, woke) = open(tmp.path());
    let mut host = ToyHost::new();
    let t = tmp.path();

    let seen = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["observe", "--scope", "todo", "--app", "toy"],
    );
    assert_eq!(seen.code, 0, "{}", seen.err);
    assert_eq!(seen.out.len(), 1);
    let r = revision(&seen);
    let r = r.to_string();

    let done = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.item[2]", "toggle", "--from-revision", &r],
    );
    assert_eq!(done.code, 0, "{}", done.err);
    assert_eq!(
        stages(&done),
        ["requested", "accepted", "executing", "completed"]
    );
    assert_eq!(host.app.done(2), Some(true));
    assert!(
        done.out
            .iter()
            .all(|m| m.id == done.out[0].id && m.id.is_some()),
        "every stage under the act's id"
    );
    let Reply::Stage(first) = &done.out[0].reply else {
        panic!("a stage");
    };
    let asked = first.asked.as_ref().unwrap();
    assert_eq!(asked.actor, sem::Actor::Agent { name: "cli".into() });

    // a person toggles item 1 after the observation: the act on the
    // revision the agent saw is stale.
    host.click("todo.item[1]", "toggle");
    let stale = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &[
            "act",
            "todo.item[1]",
            "toggle",
            "--from-revision",
            &r,
            "--app",
            "toy",
        ],
    );
    assert_eq!(stale.code, 1, "{}", stale.err);
    assert_eq!(stages(&stale), ["requested", "rejected"]);
    assert_eq!(host.app.done(1), Some(true), "only the person's toggle ran");

    // an execution that ends later: plev waits for its last stage.
    let later = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.sync", "sync"],
    );
    assert_eq!(later.code, 0, "{}", later.err);
    assert_eq!(
        stages(&later),
        ["requested", "accepted", "executing", "completed"]
    );

    // without a revision, plev acts on the state as it is now.
    let now = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.item[2]", "toggle"],
    );
    assert_eq!(now.code, 0, "{}", now.err);
    assert_eq!(host.app.done(2), Some(false));

    // the app ran it and it failed; the secret stays in.
    let failed = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["act", "todo.pin", "set_value", "--param", r#""123456""#],
    );
    assert_eq!(failed.code, 2, "{}", failed.err);
    let printed = format!("{:?}", failed.out);
    assert!(!printed.contains("123456"), "{printed}");

    // a delta since the first observation.
    let delta = plev(&mut port, &woke, &mut host, t, &["observe", "--since", &r]);
    assert_eq!(delta.code, 0, "{}", delta.err);
    assert!(matches!(delta.out[0].reply, Reply::Delta(_)));
}

#[test]
fn a_refusal_or_no_app_is_the_error_exit() {
    let tmp = tempfile::tempdir().unwrap();
    let t = tmp.path();
    let (mut port, woke) = open(t);
    let mut host = ToyHost::new();

    let unknown = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &[
            "act",
            "todo.item[1]",
            "toggle",
            "--from-revision",
            "0123456789abcdef",
        ],
    );
    assert_eq!(unknown.code, 3);
    assert!(matches!(
        unknown.out[0].reply,
        Reply::Error(ProtoError::UnknownRevision { .. })
    ));
    assert!(unknown.err.contains("observe again"), "{}", unknown.err);

    let socket = port.path().display().to_string();
    let bad = plev(
        &mut port,
        &woke,
        &mut host,
        t,
        &["observe", "--scope", "Not An Id", "--app", &socket],
    );
    assert_eq!(bad.code, 3);
    assert!(bad.out.is_empty(), "refused before connecting");

    let none = plev(&mut port, &woke, &mut host, t, &["observe", "--app", "ide"]);
    assert_eq!(none.code, 3);
    assert!(none.err.contains("no plev app"), "{}", none.err);

    let help = plev(&mut port, &woke, &mut host, t, &["--help"]);
    assert_eq!(help.code, 0);
    assert!(help.out.is_empty());
}
