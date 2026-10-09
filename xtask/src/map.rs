//! migration.toml, read once: one entry per file, from -> to by kind.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub from: String,
    pub to: String,
    pub kind: Kind,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Move,
    Merge,
    New,
    Drop,
}

#[derive(Deserialize)]
struct File {
    file: Vec<Entry>,
}

pub struct Map {
    pub entries: Vec<Entry>,
}

impl Map {
    pub fn load(root: &Path) -> Map {
        let text = std::fs::read_to_string(root.join("migration.toml"))
            .expect("migration.toml at the repository root");
        let file: File = toml::from_str(&text).expect("migration.toml parses");
        Map { entries: file.file }
    }

    pub fn of(&self, kind: Kind) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(move |e| e.kind == kind)
    }

    /// Every source path (non-empty `from`).
    pub fn sources(&self) -> Vec<&str> {
        self.entries.iter().filter(|e| !e.from.is_empty()).map(|e| e.from.as_str()).collect()
    }

    /// Every target path (non-empty `to`), once.
    pub fn targets(&self) -> Vec<&str> {
        let mut t: Vec<&str> =
            self.entries.iter().filter(|e| !e.to.is_empty()).map(|e| e.to.as_str()).collect();
        t.sort();
        t.dedup();
        t
    }

    /// from -> to for moves and merges (the paths a reader must translate).
    pub fn renames(&self) -> BTreeMap<&str, &str> {
        self.entries
            .iter()
            .filter(|e| matches!(e.kind, Kind::Move | Kind::Merge))
            .map(|e| (e.from.as_str(), e.to.as_str()))
            .collect()
    }
}
