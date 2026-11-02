//! Guard: the tree keeps the shape docs/ATLAS.toml describes.
//!
//! The root holds the workspace's few files and nine directories; every
//! crate sits under a home (rust/, apps/, demo/, tool/) with its manifest
//! and its source; the docs keep their parts by kind; a decision record is
//! `docs/adr/<Theme>/NNNN-<title>/NNNN-<title>.md` with one number per record
//! across every theme, and `docs/Research/<Theme>/` uses the same themes. A
//! file that lands somewhere else fails here, by name.

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

/// the paths .gitmodules declares: a submodule is a project kept from
/// outside (a fork we work on), not a crate of this tree, and a checkout
/// without `--recurse-submodules` leaves its directory empty
fn submodules() -> BTreeSet<String> {
    let Ok(text) = fs::read_to_string(root().join(".gitmodules")) else {
        return BTreeSet::new();
    };
    text.lines()
        .filter_map(|l| l.trim().strip_prefix("path"))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .map(|p| p.trim().to_owned())
        .collect()
}

#[test]
fn the_root_holds_the_workspace_files_and_the_nine_directories() {
    let want = [
        ".cargo",
        ".config",
        ".design",
        ".devops",
        ".editorconfig",
        ".gitattributes",
        ".github",
        ".gitignore",
        ".gitmodules",
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
    let foreign = submodules();
    let mut bad = Vec::new();
    for home in ["rust", "apps", "demo", "tool/bench"] {
        for name in entries(&root.join(home)) {
            let dir = root.join(home).join(&name);
            if !dir.is_dir() || untracked(&name) || foreign.contains(&format!("{home}/{name}")) {
                continue;
            }
            // a static site (apps/zolaviews, a Zola site that reads docs/) is an
            // app without a crate: its zola.toml stands where a Cargo.toml would
            if home == "apps" && dir.join("zola.toml").is_file() {
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
fn the_docs_keep_their_parts_by_kind() {
    let want = [
        "ATLAS.toml",
        "CATALOG.json",
        "CHANGELOG.md",
        "README.md",
        "Research",
        "TERMS.toml",
        "Tasks",
        "adr",
        "img",
    ];
    let got = tracked("docs");
    assert_eq!(got, want, "docs/ holds exactly these");
}

/// the themes adr/ and Research/ share (tool/tasks/docbase.py keeps the same list)
const THEMES: [&str; 9] = [
    "Ecosystem",
    "Importing",
    "Interface",
    "Recording",
    "Rendering",
    "Semantics",
    "Structure",
    "Synthesis",
    "Workflows",
];

/// Research/ has one theme more: the experiments, what was measured
const EXPLORING: &str = "Exploring";

/// a lowercase kebab name: digits, letters and dashes
fn kebab(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// the tracked names under `docs/<dir>` that are folders of themes: every
/// other name is reported unless it is one of `files`
fn themes(dir: &str, files: &[&str], extra: &[&str], bad: &mut Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for name in tracked(&format!("docs/{dir}")) {
        if files.contains(&name.as_str()) {
            continue;
        }
        if THEMES.contains(&name.as_str()) || extra.contains(&name.as_str()) {
            out.push(name);
        } else {
            bad.push(format!(
                "docs/{dir}/{name}: not one of the themes {THEMES:?}"
            ));
        }
    }
    out
}

#[test]
fn a_decision_record_is_numbered_once_and_sits_in_its_theme() {
    let mut numbers: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut bad = Vec::new();
    for theme in themes("adr", &["adr.md", "TEMPLATE.md"], &[], &mut bad) {
        for folder in tracked(&format!("docs/adr/{theme}")) {
            let (num, rest) = folder.split_at(folder.len().min(5));
            let well_named = num.len() == 5
                && num[..4].bytes().all(|b| b.is_ascii_digit())
                && num.ends_with('-')
                && kebab(rest);
            if !well_named {
                bad.push(format!(
                    "docs/adr/{theme}/{folder}: not a folder NNNN-kebab-title"
                ));
                continue;
            }
            let inside = tracked(&format!("docs/adr/{theme}/{folder}"));
            if !inside.contains(&format!("{folder}.md")) {
                bad.push(format!("docs/adr/{theme}/{folder}: holds no {folder}.md"));
            }
            for file in &inside {
                if !file.strip_suffix(".md").is_some_and(kebab) {
                    bad.push(format!(
                        "docs/adr/{theme}/{folder}/{file}: not kebab-title.md"
                    ));
                }
            }
            let n: u32 = num[..4].parse().expect("four digits");
            numbers
                .entry(n)
                .or_default()
                .push(format!("{theme}/{folder}"));
        }
    }
    for (n, folders) in &numbers {
        if folders.len() > 1 {
            bad.push(format!("{n:04} is used by {folders:?}"));
        }
    }
    assert!(
        bad.is_empty(),
        "decision records out of shape:\n  {}",
        bad.join("\n  ")
    );
}

#[test]
fn research_sits_in_the_themes_of_the_decisions() {
    let mut bad = Vec::new();
    for theme in themes("Research", &["Research.md"], &[EXPLORING], &mut bad) {
        for file in tracked(&format!("docs/Research/{theme}")) {
            if !file.strip_suffix(".md").is_some_and(kebab) {
                bad.push(format!("docs/Research/{theme}/{file}: not kebab-name.md"));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "research out of shape:\n  {}",
        bad.join("\n  ")
    );
}

#[test]
fn a_task_is_a_folder_with_its_note_named_like_it() {
    let mut bad = Vec::new();
    for folder in tracked("docs/Tasks") {
        if folder == "Tasks.md" {
            continue;
        }
        let inside = tracked(&format!("docs/Tasks/{folder}"));
        if !folder.starts_with("task-") || !kebab(&folder) {
            bad.push(format!(
                "docs/Tasks/{folder}: not a folder task-NN-kebab-title"
            ));
        } else if !inside.contains(&format!("{folder}.md")) {
            bad.push(format!("docs/Tasks/{folder}: holds no {folder}.md"));
        }
    }
    assert!(
        bad.is_empty(),
        "tasks out of shape:\n  {}",
        bad.join("\n  ")
    );
}
