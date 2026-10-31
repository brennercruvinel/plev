//! Guard: raw `TextNodeKey::new` is confined to the engine.
//!
//! The ADR docs/decisoes/renderizacao/0006-one-text-style-for-measurement-and-drawing.md decides
//! that a text run owns exactly one `TextStyle`, input to both
//! `TextMeasurer::measure_styled` (sizing) and `TextNodeKey::from_style`
//! (drawing). `TextNodeKey::new` hardcodes weight 400, letter spacing 0
//! and the default family, so app code calling it flattens typography and
//! reopens the measure/draw divergence the ADR closed. This test scans
//! every Rust source in the repo and fails, file:line by file:line, when
//! the raw constructor appears outside the allowlist below.

use std::fs;
use std::path::{Path, PathBuf};

/// Directories never scanned (build output, vendored study material,
/// discarded code kept for reference, vcs).
const SKIP_DIRS: [&str; 8] = [
    ".git",
    "target",
    "ref",
    "boiler",
    "codigo-descartado",
    "dist",
    "tmp",
    "node_modules",
];

/// Path prefixes (relative to the repo root, '/'-separated) where the raw
/// constructor is legitimate: the engine that defines it (its widgets
/// included), its inline tests and its benches. rust/motion builds raw keys
/// in codec round-trip tests and is engine-side tooling.
const ALLOWED_PREFIXES: [&str; 4] = [
    "rust/engine/src/",
    "rust/engine/benches/",
    "rust/engine/tests/",
    "rust/motion/src/",
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
    // the repository root: this guard lives in tool/tests
    let root_buf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root_buf.as_path();
    // Assembled at runtime so this file does not match itself.
    let needle: String = ["TextNodeKey", "::new("].concat();

    let mut files = Vec::new();
    collect_rs_files(root, &mut files);
    files.sort();

    let mut violations = Vec::new();
    for path in &files {
        let rel = path
            .strip_prefix(root)
            .expect("walked paths start at the repository root")
            .to_string_lossy()
            .replace('\\', "/");
        if ALLOWED_PREFIXES.iter().any(|p| rel.starts_with(p)) {
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
         build one engine::drawing::text::TextStyle per text run and pass it to\n\
         both TextMeasurer::measure_styled and TextNodeKey::from_style.\n\
         see docs/decisoes/renderizacao/0006-one-text-style-for-measurement-and-drawing.md",
        violations.len(),
        violations.join("\n")
    );
}
