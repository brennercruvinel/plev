//! Guard: the architecture docs cannot silently omit a crate or example.
//!
//! The source of truth for what exists is Cargo.toml `members` and the
//! examples/ directory, never a doc. Three architecture views are kept by
//! hand (doc/arc/arc.yaml the canonical machine map, arc.md the human
//! projection, README.md the home page), so they drift on the first rush.
//! This test makes drift a failing build instead of a silent lie: every
//! workspace crate must be named in arc.yaml, arc.md and README, and every
//! example directory must be named in arc.yaml. add a crate or example and
//! forget a doc, and this test tells you which doc, by name.

use std::fs;
use std::path::PathBuf;

/// this guard lives in the engine crate (crates/engine), so the workspace
/// root is two levels up. the workspace Cargo.toml, doc/ and README live
/// there; the examples/ dir is the engine's own.
fn wroot() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn engine_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let p = wroot().join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// crate directory names under crates/ taken from the workspace members,
/// the real source of truth for what crates exist.
fn workspace_crates() -> Vec<String> {
    let toml = read("Cargo.toml");
    let mut out = Vec::new();
    for line in toml.lines() {
        let t = line.trim().trim_matches(',').trim_matches('"');
        if let Some(name) = t.strip_prefix("crates/") {
            out.push(name.to_string());
        }
    }
    assert!(!out.is_empty(), "no crates/ members parsed from Cargo.toml");
    out
}

/// example directory names (examples/<name>/main.rs), the source of truth
/// for what demos exist.
fn examples() -> Vec<String> {
    let mut out = Vec::new();
    for entry in fs::read_dir(engine_dir().join("examples")).expect("read examples/") {
        let entry = entry.expect("dir entry");
        if entry.path().is_dir() && entry.path().join("main.rs").exists() {
            out.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    assert!(!out.is_empty(), "no examples found");
    out
}

#[test]
fn every_crate_is_named_in_arc_yaml_arc_md_and_readme() {
    let crates = workspace_crates();
    for (rel, body) in [
        ("doc/arc/arc.yaml", read("doc/arc/arc.yaml")),
        ("doc/arc/arc.md", read("doc/arc/arc.md")),
        ("README.md", read("README.md")),
    ] {
        let missing: Vec<&String> = crates
            .iter()
            .filter(|c| !body.contains(c.as_str()))
            .collect();
        assert!(
            missing.is_empty(),
            "{rel} does not mention these workspace crates: {missing:?}. \
             update it (the docs track Cargo.toml, not the other way around)."
        );
    }
}

#[test]
fn every_example_is_named_in_arc_yaml() {
    let body = read("doc/arc/arc.yaml");
    let examples = examples();
    let missing: Vec<&String> = examples
        .iter()
        .filter(|e| !body.contains(e.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "doc/arc/arc.yaml does not list these examples: {missing:?}. \
         add them under the examples section."
    );
}
