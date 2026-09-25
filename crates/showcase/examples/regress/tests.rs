//! The harness end to end on real gallery pages, each baseline blessed
//! into a temporary directory: a page against its own bless holds, and a
//! baseline that recorded another label names the control.

use std::fs;
use std::path::Path;

use engine::headless::Viewport;
use sem::{Change, Snapshot};

use crate::baseline::{Baseline, SCENE, SEMANTICS};
use crate::compare::Status;
use crate::pixels::Pixels;
use crate::report::Visual;
use crate::run::{self, Gpu, check_one};

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
    let dir = match run::accept(root, &staged, None) {
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
fn a_page_checked_against_its_own_bless_holds_without_opening_a_gpu() {
    let root = temp();
    let mut gpu = Gpu::default();
    for (page, theme) in [
        ("Todo", "core"),
        ("Todo", "hoff"),
        ("Button", "core"),
        ("Git", "core"),
    ] {
        let b = bless(root.path(), page, theme);
        let o = match check_one(&b, &mut gpu) {
            Ok(o) => o,
            Err(e) => panic!("{page}: {e}"),
        };
        assert!(o.delta.unchanged(), "{page}/{theme}: {:?}", o.delta);
        assert!(matches!(o.pixels, Pixels::NotNeeded));
        assert!(o.rendered.is_none() && o.decoded.is_none() && o.diffed.is_none());
    }
    // nothing moved: not a frame drawn, not even a renderer opened
    assert_eq!(gpu.frames, 0);
    assert!(!gpu.opened());
}

/// A page in a state (task-69 checkpoint b) is blessed and checked as a
/// page is: the Silences with the review open, by the rules, from the
/// Claude's fixture, that fixture adapted (task-70), and both changed by
/// an intent asked through the row (task-70 phase 2), holds against its
/// own bless with no frame drawn; its baseline is named by the state, its
/// projection holds the surface beside the fixed screen, and it settled.
/// Adapted, the detection the fixed screen offers is not the surface's.
#[test]
fn a_page_in_a_state_checked_against_its_own_bless_holds() {
    let root = temp();
    let mut gpu = Gpu::default();
    for (state, label, surface) in [
        (
            "Silences, review by the rules",
            "silences-review-by-the-rules@core@1280x800",
            "proposed",
        ),
        (
            "silences, review by the claude (fixture)",
            "silences-review-by-the-claude-fixture@core@1280x800",
            "review",
        ),
        (
            "Silences, review by the Claude (fixture), adapted",
            "silences-review-by-the-claude-fixture-adapted@core@1280x800",
            "review.cuts_list",
        ),
        (
            "Silences, review by the rules, too much information",
            "silences-review-by-the-rules-too-much-information@core@1280x800",
            "proposed.silences.cuts",
        ),
        (
            "Silences, review by the Claude (fixture), too much, in portuguese",
            "silences-review-by-the-claude-fixture-too-much-in-portuguese@core@1280x800",
            "review.cut_item[1].actions",
        ),
    ] {
        let b = bless(root.path(), state, "core");
        assert_eq!(b.key().label(), label);
        assert!(b.meta.settle.settled, "{state}");
        let o = match check_one(&b, &mut gpu) {
            Ok(o) => o,
            Err(e) => panic!("{state}: {e}"),
        };
        assert!(o.delta.unchanged(), "{state}: {:?}", o.delta);
        let Some(snap) = o.semantics else {
            panic!("{state}: no projection");
        };
        for id in [surface, "silences", "silences_surface"] {
            let at = sem::SemId::parse(id).unwrap_or_else(|e| panic!("{id}: {e}"));
            assert!(snap.get(&at).is_some(), "{state}: {id}");
        }
        let controls = sem::SemId::parse("review.controls").unwrap_or_else(|e| panic!("{e}"));
        let name = state.to_lowercase();
        let (fixture, adapted) = (name.contains("claude"), name.ends_with("adapted"));
        assert_eq!(
            snap.get(&controls).is_some(),
            fixture && !adapted,
            "{state}: the detection card"
        );
    }
    assert_eq!(gpu.frames, 0);
    match run::stage(&key("No such state", "core"), false) {
        Err(e) => assert!(e.contains("no page titled"), "{e}"),
        Ok(_) => panic!("an unknown page stages"),
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
    let mut gpu = Gpu::default();
    let o = match check_one(&b, &mut gpu) {
        Ok(o) => o,
        Err(e) => panic!("{e}"),
    };
    assert!(!o.delta.unchanged());
    // the item's text is its label and its `text` state (task-95): both
    // move, both on that control
    match o.delta.semantics.changes.as_slice() {
        [
            Change::LabelChanged {
                node,
                before,
                after,
            },
            Change::StateChanged {
                at,
                before: was,
                after: now,
                ..
            },
        ] => {
            assert_eq!(node.as_str(), "todo.item[2]");
            assert_eq!(before, "Measure text, then estimate it");
            assert_eq!(after, "Measure text, never estimate it");
            assert_eq!(at.to_string(), "todo.item[2]:text");
            assert_eq!(was.as_ref(), Some(&before.as_str().into()));
            assert_eq!(now.as_ref(), Some(&after.as_str().into()));
        }
        other => panic!("{other:?}"),
    }
    // the drawing is the baseline's: only the meaning moved
    assert!(!o.delta.drawn_changed() && o.delta.bindings.is_empty());
    assert!(o.delta.layers.iter().all(|l| l.status == Status::Unchanged));
    assert!(matches!(o.pixels, Pixels::NotNeeded) && !gpu.opened());
}

/// A baseline whose `todo` layer was drawn otherwise: its hash is not
/// the page's any more, and, when it keeps an image, the block
/// `[x0, x1) x [y0, y1)` of that image is another color.
fn redraw(b: &Baseline, block: Option<[u32; 4]>) {
    let mut meta = b.meta.clone();
    for layer in meta.scene.layers.iter_mut().filter(|l| l.name == "todo") {
        layer.hash ^= 1;
    }
    let scene = match serde_json::to_string_pretty(&meta) {
        Ok(s) => s,
        Err(e) => panic!("{e}"),
    };
    if let Err(e) = fs::write(b.dir.join(SCENE), scene) {
        panic!("{e}");
    }
    let Some([x0, y0, x1, y1]) = block else {
        return;
    };
    let mut image = match b.image() {
        Ok(Some(f)) => f,
        other => panic!("no image: {:?}", other.map(|f| f.map(|f| f.width))),
    };
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * image.width + x) * 4) as usize;
            image.rgba[i] = image.rgba[i].wrapping_add(90);
        }
    }
    if let Err(e) = image.write_png(b.dir.join(crate::baseline::IMAGE)) {
        panic!("{e}");
    }
}

#[test]
fn a_layer_that_moved_without_a_baseline_image_says_so_and_draws_nothing() {
    let root = temp();
    let b = bless(root.path(), "Todo", "core");
    redraw(&b, None);
    let b = match Baseline::read(&b.dir) {
        Ok(b) => b,
        Err(e) => panic!("{e}"),
    };
    let mut gpu = Gpu::default();
    let o = match check_one(&b, &mut gpu) {
        Ok(o) => o,
        Err(e) => panic!("{e}"),
    };
    assert!(o.delta.semantics.changes.is_empty() && o.delta.drawn_changed());
    assert!(matches!(o.pixels, Pixels::NoBaselineImage));
    assert!(!gpu.opened());
    let r = crate::report::build(&o, &b.meta.scene);
    assert_eq!(r.pixels.status, "no_baseline_image");
    // every control bound in the layer that moved: changed as far as can
    // be told, the meaning unchanged
    for c in &r.controls {
        assert!(c.semantics.is_empty(), "{}", c.id);
        match c.layer.as_deref() {
            Some("todo") => assert_eq!(c.visual, Visual::LayerChanged, "{}", c.id),
            _ => assert_eq!(c.visual, Visual::Unbound, "{}", c.id),
        }
    }
    let text = crate::report::text(&r);
    assert!(text.contains("no baseline image to diff"), "{text}");
}

/// The whole path on a GPU: the baseline keeps its image, a block over
/// the second item differs, and the check draws the frame, compares only
/// inside the `todo` layer's region and names `todo.item[2]` (and the
/// `todo` card around it), with the block as the region, and nothing
/// else. Skips without an adapter, as the headless tests do.
#[test]
fn changed_pixels_are_named_by_the_control_they_fall_in() {
    let root = temp();
    let staged = match run::stage(&key("Todo", "core"), true) {
        Ok(s) => s,
        Err(e) => panic!("{e}"),
    };
    let mut page = staged;
    let mut gpu = Gpu::default();
    let frame = match gpu.frame(&mut page.page, DESKTOP) {
        Ok(Some((f, _))) => f,
        Ok(None) => {
            eprintln!("no GPU adapter; skipping");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    let dir = match run::accept(root.path(), &page, Some(&frame)) {
        Ok(d) => d,
        Err(e) => panic!("{e}"),
    };
    let b = match Baseline::read(&dir) {
        Ok(b) => b,
        Err(e) => panic!("{e}"),
    };
    let Some(item) = b
        .meta
        .scene
        .bindings
        .iter()
        .find(|x| x.id == "todo.item[2]")
    else {
        panic!("todo.item[2] is not bound");
    };
    // a block inside the row, clear of its edges
    let [x, y, _, _] = item.rect;
    let (x0, y0) = (x.ceil() as u32 + 100, y.ceil() as u32 + 10);
    redraw(&b, Some([x0, y0, x0 + 30, y0 + 12]));
    let b = match Baseline::read(&dir) {
        Ok(b) => b,
        Err(e) => panic!("{e}"),
    };
    let o = match check_one(&b, &mut gpu) {
        Ok(o) => o,
        Err(e) => panic!("{e}"),
    };
    let Pixels::Diffed(changed) = &o.pixels else {
        panic!("the pixels were not compared");
    };
    assert_eq!(changed.total().pixels, 30 * 12);
    let block = Some([x0 as f32, y0 as f32, 30.0, 12.0]);
    assert_eq!(changed.total().region, block);
    // the frame is 1280x800; only the todo layer's region was compared
    assert!(changed.compared < 1280 * 800, "{}", changed.compared);
    let r = crate::report::build(&o, &b.meta.scene);
    let text = crate::report::text(&r);
    let named: Vec<&str> = r
        .controls
        .iter()
        .filter(|c| c.visual == Visual::Changed)
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(named, ["todo", "todo.item[2]"], "{text}");
    for c in r.controls.iter().filter(|c| c.visual == Visual::Changed) {
        assert_eq!((c.pixels, c.region), (Some(360), block), "{text}");
        assert_eq!(c.layer.as_deref(), Some("todo"));
    }
    assert!(r.controls.iter().all(|c| c.semantics.is_empty()));
    let todo = r.layers.iter().find(|l| l.name == "todo");
    assert_eq!(todo.and_then(|l| l.unbound_pixels), Some(0));
    assert_eq!(gpu.frames, 2, "the bless and the check each drew once");
}
