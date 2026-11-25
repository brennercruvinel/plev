//! The workbench's projection as the plev IR fixtures read it (task-68).
//!
//! A real repository in a temp dir and the real git worker: three changed
//! files, one staged, and the commit form open with a message, written by
//! a person through the dispatcher. The projection is written in the lean
//! `sem` form without `source` to `rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json`,
//! and the IR fixtures of the ide build their capability registry from it
//! (`plevir::Capabilities::from_snapshot`): the registry is the ide's own
//! projection, never a second declaration by hand.
//!
//! The file is a golden. This test fails when the projection moves away
//! from it; `UIR_BLESS=1 cargo test -p idestudio --test suite plevir_fixture::` writes it
//! again, and the uir tests then say whether the IR fixtures still verify.

// shared with the other ide tests: this binary uses the rig and a helper.
#[allow(dead_code)]
use crate::support;

use std::path::PathBuf;
use std::process::Command;

use accord::{Actor, Snapshot};
use gitdrv::GitEvent;
use support::{Rig, completed};

/// The date of the rig's first commit, so its sha, the key of its node in
/// the projection, is the same on every run.
const DATE: &str = "2026-01-01T00:00:00+0000";

const FIXTURE: &str = "../../rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json";

#[test]
fn the_workbench_projection_is_the_uir_fixture() {
    let mut rig = Rig::new(
        &["notes.md", "src/lexer.rs", "src/parser.rs"],
        &["src/lexer.rs"],
    );
    let out = Command::new("git")
        .arg("-C")
        .arg(rig.dir.path())
        .env("GIT_AUTHOR_DATE", DATE)
        .env("GIT_COMMITTER_DATE", DATE)
        .args(["commit", "--amend", "--only", "--no-edit", "--reset-author"])
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    rig.bench.refresh();
    rig.until(|e| matches!(e, GitEvent::Branches(_)));
    let open = rig.ask(Actor::Human, "commit.form", "open", None);
    let text = Some("fix parser".into());
    let write = rig.ask(Actor::Human, "commit.message", "set_value", text);
    for id in [open, write] {
        assert!(
            completed(&rig.receipt(id)).is_some(),
            "{:?}",
            rig.receipt(id)
        );
    }

    let snap = rig.look();
    print!("{}", capabilities(&snap));
    golden(&lean(&snap));
}

/// The snapshot in the lean serde form of `sem`, each node without its
/// `source`: the place in the code that projected a node moves with every
/// edit of that code and means nothing to a capability. The keys of each
/// object are written in order (`sorted`).
fn lean(snap: &Snapshot) -> String {
    let mut json = serde_json::to_value(snap).expect("a snapshot serializes");
    if let Some(nodes) = json.get_mut("nodes").and_then(|n| n.as_array_mut()) {
        for node in nodes {
            if let Some(node) = node.as_object_mut() {
                node.remove("source");
            }
        }
    }
    let mut text = serde_json::to_string_pretty(&sorted(json)).expect("json prints");
    text.push('\n');
    text
}

/// `value` with the keys of every object in order, whatever map serde_json
/// was built with (`preserve_order` reaches every member through `unions`,
/// and then a map keeps the order of insertion): the fixture never depends
/// on a cargo feature.
fn sorted(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let mut pairs: Vec<(String, Value)> = map.into_iter().collect();
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            Value::Object(pairs.into_iter().map(|(k, v)| (k, sorted(v))).collect())
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sorted).collect()),
        other => other,
    }
}

/// What the probe of task-68 reads: every state with its kind and every
/// action with its parameter and policy, one per line.
fn capabilities(snap: &Snapshot) -> String {
    let mut out = String::new();
    for node in snap.nodes().values() {
        for (key, value) in &node.state {
            let kind = value.kind().as_str();
            out.push_str(&format!("data {}:{key} {kind}\n", node.id));
        }
        for action in &node.actions {
            let param = action.param.map_or("none".to_owned(), |p| format!("{p:?}"));
            let policy = format!("{:?}", action.policy);
            let name = &action.name;
            out.push_str(&format!(
                "action {} {name} param={param} policy={policy}\n",
                node.id
            ));
        }
    }
    out
}

/// Compares `text` with the fixture, or writes it with `UIR_BLESS=1`.
fn golden(text: &str) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    if std::env::var_os("UIR_BLESS").is_some() {
        std::fs::write(&path, text).expect("the fixture writes");
        return;
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        kept == text,
        "the workbench projection moved away from {FIXTURE}; regenerate it with \
         UIR_BLESS=1 cargo test -p idestudio --test suite plevir_fixture::, then run cargo test -p plevir"
    );
}
