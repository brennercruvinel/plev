//! Guard: which crate may depend on which.
//!
//! The crates live in four homes by what they are, and the arrows between
//! the homes go one way: a library (rust/) builds on libraries only; an app
//! (apps/) on libraries and other apps; a demo (demo/) on libraries; the
//! tooling (tool/) on libraries and apps. Nothing depends on a demo or on
//! the tooling. Dev-dependencies count too: a library's test that needs an
//! app belongs to the app.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn wroot() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read_toml(rel: &str) -> toml::Table {
    let p = wroot().join(rel);
    let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    toml::from_str(&text).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

/// the home of a crate directory: rust, apps, demo or tool
fn home(dir: &str) -> &str {
    dir.split('/').next().unwrap_or(dir)
}

/// package name -> crate directory, for every crate of the repository
fn crates() -> BTreeMap<String, String> {
    let root = read_toml("Cargo.toml");
    let ws = root["workspace"].as_table().expect("[workspace]");
    let mut out = BTreeMap::new();
    for key in ["members", "exclude"] {
        for dir in ws.get(key).and_then(|v| v.as_array()).into_iter().flatten() {
            let dir = dir.as_str().expect("a path").to_owned();
            let manifest = read_toml(&format!("{dir}/Cargo.toml"));
            let name = manifest["package"]["name"].as_str().expect("package.name").to_owned();
            out.insert(name, dir);
        }
    }
    out
}

/// every dependency key of a manifest, in every table that holds one
fn dependency_names(manifest: &toml::Table) -> Vec<String> {
    let mut out = Vec::new();
    let mut tables: Vec<&toml::Table> = Vec::new();
    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(t) = manifest.get(key).and_then(|v| v.as_table()) {
            tables.push(t);
        }
    }
    if let Some(targets) = manifest.get("target").and_then(|v| v.as_table()) {
        for target in targets.values().filter_map(|v| v.as_table()) {
            for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(t) = target.get(key).and_then(|v| v.as_table()) {
                    tables.push(t);
                }
            }
        }
    }
    for t in tables {
        for (key, spec) in t {
            // a renamed dependency names its package
            let package = spec
                .as_table()
                .and_then(|s| s.get("package"))
                .and_then(|p| p.as_str())
                .unwrap_or(key);
            out.push(package.to_owned());
        }
    }
    out.sort();
    out.dedup();
    out
}

fn allowed(from: &str, to: &str) -> bool {
    match (home(from), home(to)) {
        ("rust", "rust") => true,
        ("apps", "rust" | "apps") => true,
        ("demo", "rust") => true,
        ("tool", "rust" | "apps") => true,
        _ => false,
    }
}

#[test]
fn every_dependency_points_the_way_the_homes_allow() {
    let crates = crates();
    let mut wrong = Vec::new();
    for (name, dir) in &crates {
        let manifest = read_toml(&format!("{dir}/Cargo.toml"));
        for dep in dependency_names(&manifest) {
            let Some(to) = crates.get(&dep) else {
                continue; // an external crate
            };
            if !allowed(dir, to) {
                wrong.push(format!("{name} ({dir}) -> {dep} ({to})"));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "these dependencies go against the homes (rust <- apps, demo, tool; \
         apps <- tool; nothing depends on a demo or on the tooling):\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn the_homes_are_the_only_places_a_crate_lives() {
    let stray: Vec<String> = crates()
        .into_values()
        .filter(|dir| !matches!(home(dir), "rust" | "apps" | "demo" | "tool"))
        .collect();
    assert!(stray.is_empty(), "crates outside rust/, apps/, demo/, tool/: {stray:?}");
}
