//! Behaviour snapshots of the plev workspace, recorded before the first
//! move and asserted unchanged after the migration.
//!
//! Each probe runs a real program of the workspace (the regression
//! gallery, the session inspector, the importers, the agent CLI) and
//! snapshots what it prints and what it writes. A probe names its command
//! twice, in the layout of today (`crates/`) and in the target layout
//! (`rust/`, `apps/`, `tool/`); its path arguments are written once, in
//! today's layout, and translated through migration.toml after the move.
//! The "before" side of every output is normalised the same way, so the
//! comparison is between names of the target layout on both sides.
//!
//! ```text
//! INSTA_UPDATE=always cargo test --manifest-path xtask/Cargo.toml   # record (before)
//! INSTA_UPDATE=no     cargo test --manifest-path xtask/Cargo.toml   # assert (after)
//! ```

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Probe {
    name: &'static str,
    before: &'static [&'static str],
    after: &'static [&'static str],
    /// program arguments, paths in today's layout; `<tmp>` is the probe's
    /// scratch directory
    args: &'static [&'static str],
    /// files the program writes under `<tmp>` are hashed into the snapshot
    writes: bool,
    /// the output carries wall-clock durations (redacted); data times stay
    wallclock: bool,
}

const PROBES: &[Probe] = &[
    Probe {
        name: "regress_check",
        before: &["-p", "showcase", "--example", "regress"],
        after: &["-p", "regress"],
        args: &["check"],
        writes: false,
        wallclock: true,
    },
    Probe {
        name: "regress_bless_core",
        before: &["-p", "showcase", "--example", "regress"],
        after: &["-p", "regress"],
        args: &["bless", "--all", "--theme", "core", "--no-png", "--dir", "<tmp>"],
        writes: true,
        wallclock: true,
    },
    Probe {
        name: "regress_bless_hoff",
        before: &["-p", "showcase", "--example", "regress"],
        after: &["-p", "regress"],
        args: &["bless", "--all", "--theme", "hoff", "--no-png", "--dir", "<tmp>"],
        writes: true,
        wallclock: true,
    },
    Probe {
        name: "inspect_a2ppa",
        before: &["-p", "inspect", "--bin", "plev-inspect"],
        after: &["-p", "replay", "--bin", "plev-inspect"],
        args: &["crates/ide/tests/fixtures/sessions/a2ppa.monster", "--timeline"],
        writes: false,
        wallclock: false,
    },
    Probe {
        name: "inspect_bug84",
        before: &["-p", "inspect", "--bin", "plev-inspect"],
        after: &["-p", "replay", "--bin", "plev-inspect"],
        args: &["crates/ide/tests/fixtures/sessions/bug84.monster", "--timeline"],
        writes: false,
        wallclock: false,
    },
    Probe {
        name: "inspect_silences_demo",
        before: &["-p", "inspect", "--bin", "plev-inspect"],
        after: &["-p", "replay", "--bin", "plev-inspect"],
        args: &["crates/showcase/fixtures/demo/silences-demo.monster"],
        writes: false,
        wallclock: false,
    },
    Probe {
        name: "inspect_silences_demo_claude",
        before: &["-p", "inspect", "--bin", "plev-inspect"],
        after: &["-p", "replay", "--bin", "plev-inspect"],
        args: &["crates/showcase/fixtures/demo/silences-demo-claude.monster", "--json"],
        writes: false,
        wallclock: false,
    },
    Probe {
        name: "transpile_react",
        before: &["-p", "parser", "--example", "transpile"],
        after: &["-p", "import", "--bin", "transpile"],
        args: &[
            "crates/parser/fixtures/react/index.tsx",
            "crates/parser/fixtures/react/NoteCard.module.sass",
            "crates/parser/fixtures/react/note-card-variables.sass",
        ],
        writes: false,
        wallclock: false,
    },
    Probe {
        name: "convert_lottie",
        before: &["--release", "-p", "engine", "--example", "lot2monsters"],
        after: &["--release", "-p", "import", "--bin", "convert"],
        args: &["<copy:refs/lottie/girl/json.json:in/girl.json>", "<tmp>/girl.monster"],
        writes: true,
        wallclock: false,
    },
    Probe {
        name: "convert_svg",
        before: &["--release", "-p", "engine", "--example", "svg2monster"],
        after: &["--release", "-p", "import", "--bin", "convert"],
        args: &["<copy:refs/lottie/cards/cards.svg:in/cards.svg>", "<tmp>/cards.monster"],
        writes: true,
        wallclock: false,
    },
    Probe {
        name: "plev_help",
        before: &["-p", "wire", "--bin", "plev"],
        after: &["-p", "socket", "--bin", "plev"],
        args: &["--help"],
        writes: false,
        wallclock: false,
    },
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

/// today's path -> target path, longest first, from migration.toml
fn renames(root: &Path) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(root.join("migration.toml")).unwrap();
    let doc: toml::Table = toml::from_str(&text).unwrap();
    let mut out: Vec<(String, String)> = doc["file"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| {
            let from = e["from"].as_str()?;
            let to = e["to"].as_str()?;
            (!from.is_empty() && !to.is_empty() && from != to)
                .then(|| (from.to_owned(), to.to_owned()))
        })
        .collect();
    out.sort_by_key(|(f, _)| std::cmp::Reverse(f.len()));
    out
}

/// `old::a::` -> `new::x::` for every module the move renamed, longest
/// first (xtask/tests/modpaths.txt, generated from migration.toml): Rust
/// paths an output carries (generated code, a report) cross the move too
fn modpaths(root: &Path) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(root.join("xtask/tests/modpaths.txt")).unwrap();
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once(' '))
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .collect()
}

/// one pass, left to right: at a path's start (not inside a longer path)
/// the longest old module that begins there is written as its new one
fn translate_mods(text: &str, mods: &[(String, String)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let at_start = i == 0
            || !text[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ':');
        if at_start {
            if let Some((from, to)) = mods.iter().find(|(f, _)| text[i..].starts_with(f.as_str())) {
                out.push_str(to);
                i += from.len();
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn before_layout(root: &Path) -> bool {
    root.join("crates").is_dir()
}

fn translate(text: &str, renames: &[(String, String)]) -> String {
    let mut s = translate_mods(text, &modpaths(&root()));
    for (from, to) in renames {
        if s.contains(from.as_str()) {
            s = s.replace(from.as_str(), to);
        }
    }
    s
}

fn run(p: &Probe) -> String {
    let root = root();
    let before = before_layout(&root);
    let renames = renames(&root);
    let tmp = std::env::temp_dir().join(format!("plev-behaviour-{}", p.name));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let tmp_s = tmp.to_string_lossy().into_owned();
    // inputs live beside the outputs' directory, never hashed with them
    let scratch = std::env::temp_dir().join(format!("plev-behaviour-{}-in", p.name));
    let _ = std::fs::remove_dir_all(&scratch);
    let args: Vec<String> = p
        .args
        .iter()
        .map(|a| {
            let a = a.replace("<tmp>", &tmp_s);
            let a = if before { a } else { translate(&a, &renames) };
            // <copy:src:name>: the input under a fixed name, so the output
            // does not carry the input's file name across the rename
            match a.strip_prefix("<copy:").and_then(|r| r.strip_suffix('>')) {
                Some(spec) => {
                    let (src, name) = spec.rsplit_once(':').unwrap();
                    let dst = scratch.join(name);
                    std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
                    std::fs::copy(root.join(src), &dst).unwrap();
                    dst.to_string_lossy().into_owned()
                }
                None => a,
            }
        })
        .collect();
    let head = if before { p.before } else { p.after };
    let out = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .args(head)
        .arg("--")
        .args(&args)
        .current_dir(&root)
        .env("CARGO_BUILD_JOBS", "4")
        .output()
        .expect("cargo runs");
    let mut text = format!(
        "exit: {}\n--- stdout\n{}\n--- stderr\n{}\n",
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if p.writes {
        text.push_str("--- files\n");
        for (rel, digest) in hashes(&tmp, before, &renames) {
            text.push_str(&format!("{digest}  {rel}\n"));
        }
    }
    let text = text
        .replace(&*scratch.to_string_lossy(), "<in>")
        .replace(&tmp_s, "<tmp>")
        .replace(&*root.to_string_lossy(), "<root>");
    let text = if p.wallclock { redact_timings(&text) } else { text };
    if before { translate(&text, &renames) } else { text }
}

/// wall-clock durations vary run to run: `270.1 ms` -> `<t> ms`
fn redact_timings(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_digit() && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'.')) {
            let mut j = i;
            while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'.') {
                j += 1;
            }
            let rest = &text[j..];
            let unit = ["ms", "µs", "us", "ns", "s"]
                .into_iter()
                .find(|u| {
                    let r = rest.strip_prefix(' ').unwrap_or(rest);
                    r.starts_with(u) && !r[u.len()..].starts_with(|c: char| c.is_alphanumeric())
                });
            if let Some(u) = unit {
                let sp = if rest.starts_with(' ') { " " } else { "" };
                out.push_str(&format!("<t>{sp}{u}"));
                i = j + sp.len() + u.len();
                continue;
            }
            out.push_str(&text[i..j]);
            i = j;
            continue;
        }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// sha256 of every file under `dir`, contents normalised like the output
fn hashes(dir: &Path, before: bool, renames: &[(String, String)]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let bytes = std::fs::read(&p).unwrap();
            let bytes = match (before, String::from_utf8(bytes.clone())) {
                (true, Ok(s)) => translate(&s, renames).into_bytes(),
                _ => bytes,
            };
            let rel = p.strip_prefix(dir).unwrap().to_string_lossy().into_owned();
            out.insert(rel, format!("{:x}", Sha256::digest(&bytes)));
        }
    }
    out
}

#[test]
fn behaviour_is_unchanged() {
    let mut failed = Vec::new();
    for p in PROBES {
        let text = run(p);
        let r = std::panic::catch_unwind(|| insta::assert_snapshot!(p.name, text));
        if r.is_err() {
            failed.push(p.name);
        }
    }
    assert!(failed.is_empty(), "behaviour moved: {failed:?}");
}
