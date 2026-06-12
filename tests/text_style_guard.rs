//! Guard: raw `TextNodeKey::new` is confined to the engine.
//!
//! The ADR kdb/adr/one-text-style-for-measurement-and-drawing.md decides
//! that a text run owns exactly one `TextStyle`, input to both
//! `TextMeasurer::measure_styled` (sizing) and `TextNodeKey::from_style`
//! (drawing). `TextNodeKey::new` hardcodes weight 400, letter spacing 0
//! and the default family, so app code calling it flattens typography and
//! reopens the measure/draw divergence the ADR closed. This test scans
//! every Rust source in the repo and fails, file:line by file:line, when
//! the raw constructor appears outside the allowlist below.

use std::fs;
use std::path::{Path, PathBuf};

/// Directories never scanned (build output, vendored study material, vcs).
const SKIP_DIRS: [&str; 6] = [".git", "target", "ref", "dist", "tmp", "node_modules"];

/// Path prefixes (relative to the repo root, '/'-separated) where the raw
/// constructor is legitimate: the engine that defines it, its inline
/// tests, its benches, and this guard itself. crates/monster builds raw keys
/// in codec round-trip tests and is engine-side tooling.
const ALLOWED_PREFIXES: [&str; 4] = ["src/", "benches/", "tests/", "crates/monster/src/"];

/// Known pre-ADR call sites pending their own migration, listed file by
/// file so no new file (and no new crate) can add a raw constructor call.
/// Shrink this list, never grow it.
const PENDING_MIGRATION: [&str; 13] = [
    "crates/showcase/src/view/icons_gallery.rs",
    "crates/showcase/src/view/lists.rs",
    "crates/showcase/src/view/mod.rs",
    "crates/showcase/src/view/theme_gallery.rs",
    "crates/ide/src/components/checkbox.rs",
    "crates/ide/src/components/context_menu.rs",
    "crates/ide/src/components/modal.rs",
    "crates/ide/src/components/panel_header.rs",
    "crates/ide/src/views/commit_form.rs",
    "crates/ide/src/views/diff_view.rs",
    "crates/ide/src/views/multi_stack_view.rs",
    "crates/ide/src/views/sidebar.rs",
    "crates/ide/src/views/unassigned_view.rs",
];

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if SKIP_DIRS.contains(&name.as_ref()) {
                continue;
            }
            collect_rs_files(&path, out);
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}

#[test]
fn raw_text_node_key_stays_inside_the_engine() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // Assembled at runtime so this file does not match itself.
    let needle: String = ["TextNodeKey", "::new("].concat();

    let mut files = Vec::new();
    collect_rs_files(root, &mut files);
    files.sort();

    let mut violations = Vec::new();
    for path in &files {
        let rel = path
            .strip_prefix(root)
            .expect("walked paths start at the manifest dir")
            .to_string_lossy()
            .replace('\\', "/");
        if ALLOWED_PREFIXES.iter().any(|p| rel.starts_with(p))
            || PENDING_MIGRATION.contains(&rel.as_str())
        {
            continue;
        }
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        for (i, line) in source.lines().enumerate() {
            if line.contains(&needle) {
                violations.push(format!("  {}:{}", rel, i + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "raw TextNodeKey::new found outside the engine ({} occurrence(s)):\n{}\n\
         build one plev::text::TextStyle per text run and pass it to both\n\
         TextMeasurer::measure_styled and TextNodeKey::from_style.\n\
         see kdb/adr/one-text-style-for-measurement-and-drawing.md",
        violations.len(),
        violations.join("\n")
    );
}
