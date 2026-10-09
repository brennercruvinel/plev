//! Guard: the architecture docs cannot silently omit a crate or a demo.
//!
//! The source of truth for what exists is the workspace Cargo.toml
//! (`members` and `exclude`), never a doc. Two architecture views are kept
//! by hand (docs/ATLAS.toml the map + prose + diagram, README.md the home
//! page), so they drift on the first rush. This test makes drift a failing
//! build instead of a silent lie: every crate must have its table in the
//! atlas and be named in its home's row of the README's crate table, and
//! every demo crate must be listed under the atlas' [demos]. add a crate
//! and forget a doc, and this test tells you which doc, by name.

use std::fs;
use std::path::PathBuf;

/// this guard lives in tool/tests, two levels under the workspace root,
/// where the workspace Cargo.toml, docs/ and README live.
fn wroot() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read(rel: &str) -> String {
    let p = wroot().join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// every crate directory the workspace names, members and excluded ones
/// alike (an excluded crate is still a crate of the repository).
fn workspace_crates() -> Vec<String> {
    let doc: toml::Table = toml::from_str(&read("Cargo.toml")).expect("Cargo.toml parses");
    let ws = doc["workspace"].as_table().expect("[workspace]");
    let mut out: Vec<String> = ["members", "exclude"]
        .iter()
        .filter_map(|k| ws.get(*k).and_then(|v| v.as_array()))
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    out.sort();
    assert!(!out.is_empty(), "no members parsed from Cargo.toml");
    out
}

/// the package name of a crate directory: its last segment.
fn name(dir: &str) -> &str {
    dir.rsplit('/').next().unwrap_or(dir)
}

fn atlas() -> toml::Table {
    toml::from_str(&read("docs/ATLAS.toml")).expect("docs/ATLAS.toml parses")
}

#[test]
fn every_crate_has_its_table_in_the_atlas_and_a_mention_in_the_readme() {
    let crates = workspace_crates();
    let atlas = atlas();
    let tables = atlas["crates"].as_table().expect("[crates] in the atlas");
    let missing: Vec<&String> = crates
        .iter()
        .filter(|c| !tables.contains_key(name(c)))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/ATLAS.toml has no [crates.<name>] table for: {missing:?}. \
         add one (the docs track Cargo.toml, not the other way around)."
    );
    let wrong: Vec<String> = crates
        .iter()
        .filter_map(|c| {
            let path = tables.get(name(c))?.get("path")?.as_str()?;
            (path != c).then(|| format!("{}: {path} (is {c})", name(c)))
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "docs/ATLAS.toml gives these crates another path: {wrong:?}"
    );
    let readme = read("README.md");
    let unnamed: Vec<&String> = crates
        .iter()
        .filter(|c| {
            let home = c.split('/').next().unwrap_or(c);
            !home_row(&readme, home).is_some_and(|row| names(row, name(c)))
        })
        .collect();
    assert!(
        unnamed.is_empty(),
        "README.md's crate table does not name these workspace crates in their \
         home's row: {unnamed:?}"
    );
}

/// the row of the README's crate table for a home (`rust`, `apps`, ...):
/// the line whose `where` cell is that directory.
fn home_row<'a>(readme: &'a str, home: &str) -> Option<&'a str> {
    let cell = format!("| `{home}/` |");
    readme.lines().find(|l| l.contains(&cell))
}

/// `word` stands alone in `text`, not inside a longer name (`record` is
/// not named by `recorder`, nor `inspect` by `plev-inspect`).
fn names(text: &str, word: &str) -> bool {
    let edge = |c: Option<char>| !c.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-');
    text.match_indices(word).any(|(i, _)| {
        edge(text[..i].chars().next_back()) && edge(text[i + word.len()..].chars().next())
    })
}

#[test]
fn a_crate_is_named_by_a_whole_word_only() {
    let row = "| tooling | `tool/` | `bench/measure` (cost tables), `tests` (the recorder) |";
    assert!(names(row, "measure") && names(row, "tests"));
    assert!(!names(row, "record") && !names(row, "bench/m"));
}

#[test]
fn every_demo_is_listed_in_the_atlas() {
    let atlas = atlas();
    let demos = atlas["demos"].as_table().expect("[demos] in the atlas");
    let listed: Vec<&str> = ["windowed", "cli"]
        .iter()
        .filter_map(|k| demos.get(*k).and_then(|v| v.as_array()))
        .flatten()
        .filter_map(|v| v.as_str())
        .collect();
    let missing: Vec<String> = workspace_crates()
        .iter()
        .filter(|c| c.starts_with("demo/"))
        .map(|c| name(c).to_owned())
        .filter(|n| !listed.contains(&n.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/ATLAS.toml does not list these demos: {missing:?}. \
         add them under the [demos] table."
    );
}
