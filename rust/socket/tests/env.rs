//! `PLEV_AGENT` read for real, in a child process of this test binary, so
//! no test changes the environment of another.
//!
//! these tests spawn processes, so they live in their own binary, apart
//! from `socket.rs`. on macOS std sets `FD_CLOEXEC` on a socket after
//! creating it (there is no `SOCK_CLOEXEC`), so a child spawned from
//! another test thread in between inherits the listener and keeps it
//! answering. `socket.rs` drops a listener to stand for an app that ended,
//! which only holds in a process that spawns nothing.

#![cfg(unix)]

use std::process::Command;

use socket::AgentPort;

/// the acceptance: with `PLEV_AGENT` unset (or not `1`) no socket is
/// created. `1` opens the port only in a build with debug assertions.
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

/// the scenes are shared only when the person who launches the app says
/// so, `PLEV_MIRROR=1` beside `PLEV_AGENT=1` (task-79): without it, or
/// with anything else, a `scene` request is refused as
/// `scene_not_shared`. a build without debug assertions opens no port
/// from the environment at all, so there is nothing to ask there.
#[test]
fn the_scene_is_shared_only_with_plev_mirror() {
    if std::env::var_os("WIRE_T79_CHILD").is_some() || !cfg!(debug_assertions) {
        return;
    }
    let exe = std::env::current_exe().unwrap();
    for (value, expect) in [
        (None, "scene_not_shared"),
        (Some("0"), "scene_not_shared"),
        (Some("true"), "scene_not_shared"),
        (Some("1"), "watching"),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let mut cmd = Command::new(&exe);
        cmd.args([
            "--exact",
            "scene_from_env_in_a_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("WIRE_T79_CHILD", "1")
        .env("PLEV_AGENT", "1")
        .env("TMPDIR", tmp.path());
        match value {
            None => cmd.env_remove("PLEV_MIRROR"),
            Some(v) => cmd.env("PLEV_MIRROR", v),
        };
        let out = cmd.output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{value:?}: {stdout}");
        assert!(
            stdout.contains(&format!("scene: {expect}")),
            "{value:?}: {stdout}"
        );
    }
}

/// an app that projects nothing and takes nothing.
struct Nothing;

impl socket::Host for Nothing {
    fn project(&self) -> Option<accord::Snapshot> {
        None
    }

    fn dispatch(&mut self, _req: accord::Request) -> Option<accord::RequestId> {
        None
    }

    fn add_sink(&mut self, _sink: Box<dyn accord::ReceiptSink>) {}
}

#[test]
fn scene_from_env_in_a_child() {
    use socket::{Client, ProtoError, Reply, WireError};
    use std::time::{Duration, Instant};

    if std::env::var_os("WIRE_T79_CHILD").is_none() {
        return;
    }
    let mut port = AgentPort::from_env("toy", || {})
        .unwrap()
        .expect("PLEV_AGENT=1 opens the port");
    let path = port.path().to_owned();
    let asker = std::thread::spawn(move || {
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(10_000)).unwrap();
        c.hello("claude").unwrap();
        let id = c.watch_scene("main").unwrap();
        c.set_timeout(Some(300)).unwrap();
        let said = match c.recv_for(&id) {
            Ok(Reply::Error(ProtoError::SceneNotShared { .. })) => "scene_not_shared".to_owned(),
            Err(WireError::Timeout(_)) => "no answer".to_owned(),
            other => format!("{other:?}"),
        };
        // the connection stays open while the app is asked who watches
        (said, c)
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    while !asker.is_finished() {
        assert!(Instant::now() < deadline, "the asker did not finish");
        std::thread::sleep(Duration::from_millis(5));
        port.drain(&mut Nothing);
    }
    let (said, _c) = asker.join().unwrap();
    port.drain(&mut Nothing);
    let said = if said == "no answer" && port.watching("main") {
        "watching".to_owned()
    } else {
        said
    };
    println!("scene: {said}");
}
