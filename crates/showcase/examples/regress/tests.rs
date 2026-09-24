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
    let lines = crate::report::pixel_lines(&o, &b.meta.scene);
    assert_eq!(
        lines,
        ["visual changed, no baseline image to diff (bless --png keeps one)"]
    );
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
    let lines = crate::report::pixel_lines(&o, &b.meta.scene);
    let named: Vec<&str> = lines
        .iter()
        .filter_map(|l| l.split(':').next())
        .filter(|id| id.starts_with("todo"))
        .collect();
    assert_eq!(named, ["todo", "todo.item[2]"], "{lines:#?}");
    assert_eq!(gpu.frames, 2, "the bless and the check each drew once");
}
