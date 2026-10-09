//! Guard: the tree keeps the shape docs/ATLAS.toml describes.
//!
//! The root holds the workspace's few files and nine directories; every
//! crate sits under a home (rust/, apps/, demo/, tool/) with its manifest
//! and its source; the docs keep their five parts; a decision record is
//! `docs/adr/<theme>/NNNN-<title>.md` with one number per record across
//! every theme. A file that lands somewhere else fails here, by name.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// the tracked names of a directory (hidden ones included), sorted
fn entries(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    out.sort();
    out
}

/// the tracked names directly under `dir` (relative to the root), from git:
/// what a checkout holds besides (build output, a study clone, an editor's
/// files) is not the tree
fn tracked(dir: &str) -> Vec<String> {
    let mut git = Command::new("git");
    git.args(["ls-files", "-z"]).current_dir(root());
    if !dir.is_empty() {
        git.args(["--", dir]);
    }
    let out = git.output().expect("git runs");
    assert!(
        out.status.success(),
        "git ls-files: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let prefix = if dir.is_empty() {
        String::new()
    } else {
        format!("{dir}/")
    };
    let names: BTreeSet<String> = String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter_map(|p| p.strip_prefix(&prefix))
        .filter_map(|rest| rest.split('/').next())
        .filter(|n| !n.is_empty())
        .map(str::to_owned)
        .collect();
    names.into_iter().collect()
}

/// what a checkout may hold beside the tree: build output, editor and OS
/// files, local scratch the .gitignore keeps out
fn untracked(name: &str) -> bool {
    matches!(
        name,
        "target"
            | ".git"
            | ".DS_Store"
            | ".claude"
            | ".idea"
            | ".vscode"
            | "tmp"
            | "tmps"
            | "TMP"
            | ".temp"
            | "refs"
            | "out"
            | "dist"
            | "memory"
            | "MEMORY.md"
            | ".env"
            | ".kimi-code"
            | "node_modules"
            | ".venv"
            | "venv"
    ) || name.ends_with(".log")
}

#[test]
fn the_root_holds_the_workspace_files_and_the_nine_directories() {
    let want = [
        ".cargo",
        ".design",
        ".devops",
        ".editorconfig",
        ".gitattributes",
        ".github",
        ".gitignore",
        ".secops",
        "Cargo.lock",
        "Cargo.toml",
        "LICENSE",
        "README.md",
        "apps",
        "data",
        "demo",
        "docs",
        "fuzz",
        "pkgs",
        "rust",
        "rust-toolchain.toml",
        "rustfmt.toml",
        "tool",
    ];
    let extra: Vec<String> = tracked("")
        .into_iter()
        .filter(|n| !want.contains(&n.as_str()))
        .collect();
    assert!(extra.is_empty(), "not part of the root layout: {extra:?}");
    let missing: Vec<&str> = want
        .iter()
        .copied()
        .filter(|n| !root().join(n).exists())
        .collect();
    assert!(missing.is_empty(), "missing at the root: {missing:?}");
}

#[test]
fn every_crate_has_its_manifest_and_its_source() {
    let root = root();
    let mut bad = Vec::new();
    for home in ["rust", "apps", "demo", "tool/bench"] {
        for name in entries(&root.join(home)) {
            let dir = root.join(home).join(&name);
            if !dir.is_dir() || untracked(&name) {
                continue;
            }
            if !dir.join("Cargo.toml").is_file() {
                bad.push(format!("{home}/{name}: no Cargo.toml"));
            }
            if !dir.join("src").is_dir() {
                bad.push(format!("{home}/{name}: no src/"));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "crates out of shape:\n  {}",
        bad.join("\n  ")
    );
}

#[test]
fn the_docs_keep_their_five_parts() {
    let want = [
        "ATLAS.toml",
        "README.md",
        "TERMS.toml",
        "adr",
        "img",
        "log",
        "pub",
        "ref",
    ];
    let got = tracked("docs");
    assert_eq!(got, want, "docs/ holds exactly these");
}

#[test]
fn a_decision_record_is_numbered_once_and_sits_in_its_theme() {
    let adr = root().join("docs/adr");
    let mut numbers: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut bad = Vec::new();
    for theme in entries(&adr) {
        if untracked(&theme) {
            continue;
        }
        let dir = adr.join(&theme);
        if !dir.is_dir() {
            if !matches!(theme.as_str(), "README.md" | "TEMPLATE.md") {
                bad.push(format!("docs/adr/{theme}: a file outside a theme"));
            }
            continue;
        }
        for file in entries(&dir) {
            if untracked(&file) {
                continue;
            }
            let (num, rest) = file.split_at(file.len().min(5));
            let well_named = num.len() == 5
                && num[..4].bytes().all(|b| b.is_ascii_digit())
                && num.ends_with('-')
                && rest.ends_with(".md")
                && rest[..rest.len() - 3]
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
            if !well_named {
                bad.push(format!("docs/adr/{theme}/{file}: not NNNN-kebab-title.md"));
                continue;
            }
            let n: u32 = num[..4].parse().expect("four digits");
            numbers
                .entry(n)
                .or_default()
                .push(format!("{theme}/{file}"));
        }
    }
    for (n, files) in &numbers {
        if files.len() > 1 {
            bad.push(format!("{n:04} is used by {files:?}"));
        }
    }
    assert!(
        bad.is_empty(),
        "decision records out of shape:\n  {}",
        bad.join("\n  ")
    );
}
