//! The harness end to end on real gallery pages, each baseline blessed
//! into a temporary directory: a page against its own bless holds, and a
//! baseline that recorded another label names the control.

use std::fs;
use std::path::Path;

use engine::headless::Viewport;
use sem::{Change, Snapshot};

use crate::baseline::{Baseline, SCENE, SEMANTICS};
use crate::compare::Status;
use crate::run::{self, check_one};

const DESKTOP: Viewport = Viewport {
    width: 1280.0,
    height: 800.0,
    scale: 1.0,
};

fn key(page: &str, theme: &str) -> crate::baseline::Key {
    crate::baseline::Key {
        page: page.to_string(),
        theme: theme.to_string(),
        viewport: DESKTOP,
    }
}

/// Bless `page` under `root` and read the baseline back.
fn bless(root: &Path, page: &str, theme: &str) -> Baseline {
    let staged = match run::stage(&key(page, theme), true) {
        Ok(s) => s,
        Err(e) => panic!("{page}: {e}"),
    };
    let dir = match run::accept(root, &staged) {
        Ok(d) => d,
        Err(e) => panic!("{page}: {e}"),
    };
    match Baseline::read(&dir) {
        Ok(b) => b,
        Err(e) => panic!("{page}: {e}"),
    }
}

fn temp() -> tempfile::TempDir {
    match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn a_page_checked_against_its_own_bless_holds() {
    let root = temp();
    for (page, theme) in [
        ("Todo", "core"),
        ("Todo", "hoff"),
        ("Button", "core"),
        ("Git", "core"),
    ] {
        let b = bless(root.path(), page, theme);
        let o = match check_one(&b) {
            Ok(o) => o,
            Err(e) => panic!("{page}: {e}"),
        };
        assert!(o.delta.unchanged(), "{page}/{theme}: {:?}", o.delta);
    }
}

/// A baseline blessed when the second item read otherwise: rewrite its
/// label in `semantics.json` and the revision `scene.json` keeps.
fn relabel(b: &Baseline, from: &str, to: &str) {
    let path = b.dir.join(SEMANTICS);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => panic!("{e}"),
    };
    assert!(text.contains(from), "{from} is not in {}", path.display());
    let text = text.replace(from, to);
    let snapshot: Snapshot = match serde_json::from_str(&text) {
        Ok(s) => s,
        Err(e) => panic!("{e}"),
    };
    let mut meta = b.meta.clone();
    if let Some(p) = meta.semantics.as_mut() {
        p.revision = snapshot.revision().to_string();
    }
    let scene = match serde_json::to_string_pretty(&meta) {
        Ok(s) => s,
        Err(e) => panic!("{e}"),
    };
    for (file, body) in [(SEMANTICS, text), (SCENE, scene)] {
        if let Err(e) = fs::write(b.dir.join(file), body) {
            panic!("{e}");
        }
    }
}

#[test]
fn another_label_in_the_baseline_is_a_semantic_change_of_that_control_only() {
    let root = temp();
    let b = bless(root.path(), "Todo", "core");
    relabel(
        &b,
        "Measure text, never estimate it",
        "Measure text, then estimate it",
    );
    let b = match Baseline::read(&b.dir) {
        Ok(b) => b,
        Err(e) => panic!("{e}"),
    };
    let o = match check_one(&b) {
        Ok(o) => o,
        Err(e) => panic!("{e}"),
    };
    assert!(!o.delta.unchanged());
    match o.delta.semantics.changes.as_slice() {
        [
            Change::LabelChanged {
                node,
                before,
                after,
            },
        ] => {
            assert_eq!(node.as_str(), "todo.item[2]");
            assert_eq!(before, "Measure text, then estimate it");
            assert_eq!(after, "Measure text, never estimate it");
        }
        other => panic!("{other:?}"),
    }
    // the drawing is the baseline's: only the meaning moved
    assert!(!o.delta.drawn_changed() && o.delta.bindings.is_empty());
    assert!(o.delta.layers.iter().all(|l| l.status == Status::Unchanged));
}
