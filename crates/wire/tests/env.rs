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

use wire::AgentPort;

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
