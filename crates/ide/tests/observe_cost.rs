//! What one observation of the ide weighs on this repository (plev): the
//! workbench connected to a clean clone of it at HEAD (so what the
//! worktree holds uncommitted does not weigh), its first status, log and
//! branches applied, and the line the agent port would write for a whole
//! observation and for one scoped to `stacks`, serialized as the server
//! writes it (the same form the laboratory's screenshot arm weighs). This
//! repository's history moves, so it is a measurement, ignored by default:
//!
//! `cargo test -p ide --test observe_cost -- --ignored --nocapture`

use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

use git::{GitClient, GitEvent};
use ide::workbench::Workbench;
use sem::{SemId, Snapshot};
use wire::{Reply, ServerMsg, SnapshotReply, encode_server};

#[test]
#[ignore = "a measurement on this repository, run by hand"]
fn an_observation_of_this_repository() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let clone = tempfile::tempdir().expect("a temp dir");
    let head = git(here, &["rev-parse", "HEAD"]);
    let top = git(here, &["rev-parse", "--show-toplevel"]);
    let into = clone.path().to_str().expect("a utf-8 temp path");
    git(
        here,
        &["clone", "--quiet", "--no-checkout", top.trim(), into],
    );
    git(
        clone.path(),
        &["checkout", "--quiet", "-B", "main", head.trim()],
    );
    let (tx, events) = channel();
    let client = GitClient::spawn(clone.path(), move |event| {
        let _ = tx.send(event);
    })
    .expect("a git worker on this repository");
    let mut bench = Workbench::new(1280.0, 800.0);
    bench.connect(client);
    loop {
        let event = events
            .recv_timeout(Duration::from_secs(60))
            .expect("the first refresh in time");
        let last = matches!(event, GitEvent::Branches(_));
        bench.apply(event);
        if last {
            break;
        }
    }
    let seen = bench.snapshot().expect("the projection validates");
    let stacks = SemId::parse("stacks").expect("an id");
    let commits = seen.children(&stacks).count();
    let with_files = seen
        .children(&stacks)
        .filter(|node| node.state.keys().any(|key| key.as_str() == "files"))
        .count();
    println!(
        "{} nodes, {commits} commits in the stacks, {with_files} with files",
        seen.len()
    );
    println!("whole observation: {} bytes", bytes(&seen, None));
    println!(
        "observation --scope stacks: {} bytes",
        bytes(&seen, Some(&stacks))
    );
}

/// Runs `git` in `dir`; its stdout.
fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The bytes of the line the server writes for one observation of
/// `snapshot`, whole or under `scope`: a `Reply::Snapshot` through the
/// wire codec, `source` left out as the port leaves it out, and the
/// newline.
fn bytes(snapshot: &Snapshot, scope: Option<&SemId>) -> usize {
    let seen = match scope {
        Some(id) => snapshot.subtree(id),
        None => snapshot.clone(),
    };
    let reply = Reply::Snapshot(SnapshotReply {
        revision: snapshot.revision(),
        nodes: seen
            .nodes()
            .values()
            .map(|node| {
                let mut node = node.clone();
                node.source = None;
                node
            })
            .collect(),
        focused: seen.focused().cloned(),
        bounds: None,
    });
    encode_server(&ServerMsg {
        id: Some("1".to_owned()),
        reply,
    })
    .len()
        + 1
}
