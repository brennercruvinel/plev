//! Guard: no source file grows past the file-hygiene limit unnoticed.
//!
//! The repository's file-hygiene rule caps production code at 639 lines
//! per source file, with test code out of the count. The rule lived only in prose, and three
//! deliveries in one cycle went past it before anyone counted
//! (docs/learnings/general.md, process). This test counts, so the
//! gate says it first.
//!
//! What counts as test code, and is left out: files under a `tests` or
//! `benches` directory, a file named `tests.rs` or `tests_*.rs`, and the
//! `#[cfg(test)] mod … {` block at column 0 with everything after it (the
//! repository keeps in-file tests at the bottom of the file).
//!
//! The files that were already past the limit when the guard came are
//! listed in `FROZEN` with their size then: they may shrink, never grow.
//! Splitting one takes it off the list.

use std::fs;
use std::path::{Path, PathBuf};

/// The hard limit of the convention, in lines of production code.
const LIMIT: usize = 639;

/// Past the limit when this guard came (2026-09-23), with the production
/// lines each had then: debt, frozen. A split that brings one under the
/// limit removes it from here.
const FROZEN: &[(&str, usize)] = &[
    ("rust/engine/src/widgets/content/card.rs", 1343),
    ("apps/urnadbapp/src/view/explorer/mod.rs", 1019),
    ("apps/urnadbapp/src/backend/sources/reader.rs", 794),
    ("apps/urnadbapp/src/view/search.rs", 752),
];

/// The repository root: this guard lives in `tool/tests`.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Test code by where it lives: a `tests` or `benches` directory, or a
/// file named `tests.rs` / `tests_*.rs`.
fn is_test_file(rel: &Path) -> bool {
    let in_test_dir = rel
        .components()
        .any(|c| matches!(c.as_os_str().to_str(), Some("tests" | "benches")));
    let stem = rel.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    in_test_dir || stem == "tests" || stem.starts_with("tests_")
}

/// The lines before the file's `#[cfg(test)]` module with an inline body
/// (at column 0), or all of them.
fn production_lines(text: &str) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_end() != "#[cfg(test)]" {
            continue;
        }
        let Some(next) = lines.get(i + 1) else {
            continue;
        };
        let item = next
            .trim_start_matches("pub(crate) ")
            .trim_start_matches("pub ");
        if item.starts_with("mod ") && next.trim_end().ends_with('{') {
            return i;
        }
    }
    lines.len()
}

/// Every `.rs` file under `crates/`, relative to the root, build output
/// left out.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            if name != "target" && name != "fixtures" {
                sources(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_source_file_grows_past_the_limit() {
    let root = root();
    let mut files = Vec::new();
    for home in ["rust", "apps", "demo", "tool"] {
        sources(&root.join(home), &mut files);
    }
    assert!(
        !files.is_empty(),
        "no source files found under rust/, apps/, demo/, tool/"
    );

    let mut failures = Vec::new();
    for path in files {
        let rel = path.strip_prefix(&root).unwrap_or(&path).to_path_buf();
        if is_test_file(&rel) {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_default();
        let lines = production_lines(&text);
        let key = rel.to_string_lossy().replace('\\', "/");
        let ceiling = FROZEN
            .iter()
            .find(|(name, _)| *name == key)
            .map_or(LIMIT, |(_, then)| (*then).max(LIMIT));
        if lines > ceiling {
            failures.push(format!("{key}: {lines} production lines (limit {ceiling})"));
        }
    }
    assert!(
        failures.is_empty(),
        "past the file-hygiene limit ({LIMIT} lines of production code, tests \
         out of the count). \
         split by responsibility:\n  {}",
        failures.join("\n  ")
    );
}

/// A file on the frozen list that is now under the limit should leave it,
/// so the list only ever names real debt.
#[test]
fn the_frozen_list_names_only_files_still_past_the_limit() {
    let root = root();
    let mut stale = Vec::new();
    for (name, _) in FROZEN {
        let text = fs::read_to_string(root.join(name)).unwrap_or_default();
        if production_lines(&text) <= LIMIT {
            stale.push(*name);
        }
    }
    assert!(
        stale.is_empty(),
        "these files are under the limit now: take them off FROZEN in {}: {stale:?}",
        file!()
    );
}

#[test]
fn test_code_is_left_out_of_the_count() {
    let text = "fn a() {}\nfn b() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n";
    assert_eq!(production_lines(text), 2);
    let declared = "fn a() {}\n#[cfg(test)]\nmod tests;\n";
    assert_eq!(
        production_lines(declared),
        3,
        "a declared module is one line"
    );
    assert!(is_test_file(Path::new("rust/x/tests/a.rs")));
    assert!(is_test_file(Path::new("rust/x/src/y/tests.rs")));
    assert!(is_test_file(Path::new("rust/x/src/tests_clock.rs")));
    assert!(!is_test_file(Path::new("rust/x/src/testing.rs")));
}
