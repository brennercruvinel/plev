//! `cargo xtask <command>`: the tree migration driven by migration.toml.
//!
//! ```text
//! cargo xtask migrate [--dry-run]   git mv every `move` (commit 1: moves only)
//! cargo xtask audit pre|post        the map checked in both directions
//! ```

mod map;

use map::{Kind, Map};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

/// Paths of the migration tooling itself, never part of either tree.
const TRANSIENT: &[&str] = &["migration.toml", "xtask/", "boiler/"];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = repo_root();
    let map = Map::load(&root);
    match args.first().map(String::as_str) {
        Some("migrate") => migrate(&root, &map, args.iter().any(|a| a == "--dry-run")),
        Some("audit") => audit(&root, &map, args.get(1).map(String::as_str).unwrap_or("pre")),
        _ => {
            eprintln!("usage: cargo xtask migrate [--dry-run] | audit pre|post");
            exit(2);
        }
    }
}

fn repo_root() -> PathBuf {
    let out = git(&PathBuf::from("."), &["rev-parse", "--show-toplevel"]);
    PathBuf::from(out.trim())
}

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(root).output().expect("git runs");
    if !out.status.success() {
        eprintln!("git {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr));
        exit(3);
    }
    String::from_utf8(out.stdout).expect("utf-8")
}

fn tracked(root: &Path) -> BTreeSet<String> {
    git(root, &["ls-files", "-z"])
        .split('\0')
        .filter(|p| !p.is_empty())
        .filter(|p| !TRANSIENT.iter().any(|t| p == t || (t.ends_with('/') && p.starts_with(t))))
        .map(str::to_owned)
        .collect()
}

/// Directories of the sources whose name differs only in case from the
/// directory a target puts in the same place (`docs/ADR` -> `docs/adr`). On
/// a case-insensitive file system the target directory would resolve to the
/// old one, so these are renamed first, through a temporary name.
fn case_folds(map: &Map) -> BTreeMap<String, String> {
    let mut target_dirs = BTreeSet::new();
    for t in map.targets() {
        let mut p = Path::new(t).parent();
        while let Some(d) = p.filter(|d| !d.as_os_str().is_empty()) {
            target_dirs.insert(d.to_string_lossy().into_owned());
            p = d.parent();
        }
    }
    let mut folds = BTreeMap::new();
    for s in map.sources() {
        for d in Path::new(s).ancestors().skip(1) {
            if d.as_os_str().is_empty() {
                continue;
            }
            let d = d.to_string_lossy().into_owned();
            if let Some(t) = target_dirs.iter().find(|t| t.to_lowercase() == d.to_lowercase() && **t != d) {
                folds.insert(d.clone(), t.clone());
            }
        }
    }
    folds
}

fn migrate(root: &Path, map: &Map, dry: bool) {
    let folds = case_folds(map);
    for (from, to) in &folds {
        let tmp = format!("{to}.case-fold-tmp");
        println!("fold {from} -> {to} (via {tmp})");
        if !dry {
            git(root, &["mv", from, &tmp]);
            git(root, &["mv", &tmp, to]);
        }
    }
    let refold = |p: &str| -> String {
        for (from, to) in &folds {
            if let Some(rest) = p.strip_prefix(&format!("{from}/")) {
                return format!("{to}/{rest}");
            }
        }
        p.to_owned()
    };
    let mut n = 0;
    for e in map.of(Kind::Move) {
        let from = refold(&e.from);
        if from == e.to {
            continue;
        }
        n += 1;
        if dry {
            println!("mv {from} -> {}", e.to);
            continue;
        }
        let dst = root.join(&e.to);
        if dst.exists() {
            eprintln!("refusing to overwrite {}", e.to);
            exit(3);
        }
        std::fs::create_dir_all(dst.parent().expect("a parent")).expect("mkdir");
        git(root, &["mv", &from, &e.to]);
    }
    println!("{n} moves{}", if dry { " (dry run)" } else { "" });
}

fn audit(root: &Path, map: &Map, phase: &str) {
    let files = tracked(root);
    let mut problems = Vec::new();
    // the map itself: one move per target, merges point at a moved target
    let mut moved_to: BTreeMap<&str, usize> = BTreeMap::new();
    for e in map.of(Kind::Move) {
        *moved_to.entry(e.to.as_str()).or_default() += 1;
    }
    for (t, n) in &moved_to {
        if *n > 1 {
            problems.push(format!("{n} moves into {t}"));
        }
    }
    for e in map.of(Kind::Merge) {
        if !moved_to.contains_key(e.to.as_str()) {
            problems.push(format!("merge {} -> {}: no move into the target", e.from, e.to));
        }
    }
    let targets: BTreeSet<&str> = map.targets().into_iter().collect();
    match phase {
        "pre" => {
            // every file of today has exactly one entry, every entry a file
            let sources: BTreeSet<&str> = map.sources().into_iter().collect();
            for f in &files {
                if !sources.contains(f.as_str()) {
                    problems.push(format!("tracked but not in the map: {f}"));
                }
            }
            for s in &sources {
                if !files.contains(*s) {
                    problems.push(format!("in the map but not tracked: {s}"));
                }
            }
            // every file of the target layout has a source or is new
            let skeleton = skeleton(root);
            for t in &skeleton {
                if !targets.contains(t.as_str()) {
                    problems.push(format!("target layout file with no entry: {t}"));
                }
            }
            for t in &targets {
                if !skeleton.contains(*t) {
                    problems.push(format!("entry outside the target layout: {t}"));
                }
            }
        }
        "post" => {
            // the tree is exactly the targets: nothing missing, nothing extra
            for t in &targets {
                if !files.contains(*t) {
                    problems.push(format!("target missing: {t}"));
                }
            }
            for f in &files {
                if !targets.contains(f.as_str()) {
                    problems.push(format!("not a target: {f}"));
                }
            }
        }
        _ => problems.push(format!("unknown phase {phase}")),
    }
    let count = |k| map.of(k).count();
    println!(
        "audit {phase}: {} tracked, {} entries (move {}, merge {}, new {}, drop {})",
        files.len(),
        map.entries.len(),
        count(Kind::Move),
        count(Kind::Merge),
        count(Kind::New),
        count(Kind::Drop)
    );
    for p in &problems {
        println!("  {p}");
    }
    if !problems.is_empty() {
        println!("{} problems", problems.len());
        exit(1);
    }
    println!("ok");
}

/// The target layout: the empty skeleton under boiler/ while it exists.
fn skeleton(root: &Path) -> BTreeSet<String> {
    let base = root.join("boiler");
    let mut out = BTreeSet::new();
    let mut stack = vec![base.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).expect("boiler/ exists until the end").flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(&base).unwrap().to_string_lossy().into_owned();
                if rel != "boiler-tree.md" {
                    out.insert(rel);
                }
            }
        }
    }
    out
}
