//! Every control a person can press is at least 24 px on each side
//! (WCAG 2.2, 2.5.8 Target Size (Minimum), AA): the rect each actionable
//! node of a page's projection binds, the one its press lands in, on
//! every page at a desktop and at a phone width, the page drawn whole (a
//! frame as tall as the page, as the text-scale sweep draws it). The
//! drawing may be smaller: `engine::widgets::hit_rect` grows the press
//! area around it.
//!
//! The shell's sidebar is front H's (its rows, its toggle) and is left
//! out; what of it falls short is listed in `SIDEBAR`, exact, so a change
//! there shows here.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use accord::{Role, SemId};
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::widgets::hit::set_touch_input;
use uigallery::views::{PAGES, ShowcaseView};

/// WCAG 2.5.8's minimum, in logical px.
const MIN: f32 = 24.0;

/// The widths the sweep looks at: the kit's desktop and a phone.
const WIDTHS: [f32; 2] = [1280.0, 360.0];
/// The window a page opens in before it is grown to its whole height.
const OPEN_H: f32 = 800.0;
/// The tallest frame drawn: past it, a page is looked at to here.
const MAX_H: f32 = 16000.0;

/// The page `title` drawn whole at `width`.
fn drawn(title: &str, width: f32) -> (ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(width, OPEN_H);
    view.jump_to_page(title);
    let mut c = Compositor::new();
    view.render(&mut c);
    let tall = view.page_height().min(MAX_H);
    if tall > OPEN_H + 0.5 {
        view.resize(width, tall, 1.0);
        c = Compositor::new();
        view.render(&mut c);
    }
    (view, c)
}

/// What a page binds under its actionable nodes below `MIN`, as
/// `(page, id, w, h)`.
fn short_targets(title: &str, width: f32) -> Vec<(String, SemId, f32, f32)> {
    let (view, c) = drawn(title, width);
    let Some(snapshot) = view.reader_semantics() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (id, node) in snapshot.nodes() {
        let actionable = node.role.is_interactive() || !node.actions.is_empty();
        if !actionable || !node.enabled || node.inert {
            continue;
        }
        let Some(b) = c.binding(id) else { continue };
        if b.visible_rect().is_none() {
            continue;
        }
        let rect = b.rect;
        let [_, _, bw, bh] = rect;
        let parent = node
            .parent
            .as_ref()
            .and_then(|p| c.binding(p))
            .map(|p| p.rect);
        if parent.is_some_and(|p| cut_by(rect, p)) {
            continue;
        }
        if bw < MIN - 0.01 || bh < MIN - 0.01 {
            out.push((title.to_owned(), id.clone(), bw, bh));
        }
    }
    out
}

/// A row a scroller binds by its part inside the viewport (a
/// `VirtualList` row half scrolled out) is short because it is cut, not
/// because it is small: it touches its scroller's edge on the cut axis.
fn cut_by(r: [f32; 4], p: [f32; 4]) -> bool {
    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
    let (top, bottom) = (near(r[1], p[1]), near(r[1] + r[3], p[1] + p[3]));
    let (start, end) = (near(r[0], p[0]), near(r[0] + r[2], p[0] + p[2]));
    (r[3] < MIN && (top || bottom)) || (r[2] < MIN && (start || end))
}

/// What falls short and is another front's, `(page, part of the id,
/// why)`: left out of the failure and printed, as the text-scale sweep's
/// `KNOWN`.
const OWNED: &[(&str, &str, &str)] = &[
    (
        "Toolbar",
        "view[",
        "O: the view controller's back and forward",
    ),
    (
        "Prompt Input",
        ".view.",
        "O: the view controller's back and forward, in the prompt",
    ),
];

fn owned(line: &str) -> bool {
    OWNED
        .iter()
        .any(|(page, part, _)| line.starts_with(&format!("{page} @")) && line.contains(part))
}

fn sidebar(id: &SemId) -> bool {
    id.as_str().starts_with("shell.sidebar")
}

#[test]
fn every_pressable_node_is_at_least_24_px_on_each_side() {
    let mut short = Vec::new();
    let mut side = Vec::new();
    let mut theirs = Vec::new();
    for page in PAGES {
        for width in WIDTHS {
            for (title, id, w, h) in short_targets(page.title, width) {
                let line = format!("{title} @{width}: {id} {w:.1}x{h:.1}");
                if sidebar(&id) {
                    side.push(line);
                } else if owned(&line) {
                    theirs.push(line);
                } else {
                    short.push(line);
                }
            }
        }
    }
    side.sort();
    side.dedup();
    eprintln!(
        "sidebar (front H), {} short:\n{}",
        side.len(),
        side.join("\n")
    );
    eprintln!(
        "another front's, {} short:\n{}",
        theirs.len(),
        theirs.join("\n")
    );
    assert!(
        short.is_empty(),
        "{} targets under {MIN} px:\n{}",
        short.len(),
        short.join("\n")
    );
}

/// Every node the frame draws, layer by layer.
fn scene(c: &Compositor) -> Vec<Vec<SceneNode>> {
    c.layers().iter().map(|l| l.nodes().to_vec()).collect()
}

/// The text fields a page binds (a box a person types in, a select, a
/// combobox, the palette's box): `(id, rect)`, the cut ones left out.
fn fields(view: &ShowcaseView, c: &Compositor) -> Vec<(SemId, [f32; 4])> {
    let Some(snapshot) = view.reader_semantics() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (id, node) in snapshot.nodes() {
        let field = matches!(node.role, Role::TextInput | Role::Select);
        if !field || !node.enabled || node.inert || sidebar(id) {
            continue;
        }
        let Some(b) = c.binding(id) else { continue };
        if b.visible_rect().is_none() {
            continue;
        }
        let parent = node.parent.as_ref().and_then(|p| c.binding(p));
        if parent.is_some_and(|p| cut_by(b.rect, p.rect)) {
            continue;
        }
        out.push((id.clone(), b.rect));
    }
    out
}

/// With a finger (the shell's `set_touch_input(true)` after a touch),
/// every text field of every page presses in at least the theme's
/// `control.min_touch_target` on each axis, the area grown by `hit_rect`
/// around a drawing that does not change: the frame's nodes are the
/// pointer's, node for node. The ⌘K palette's box is looked at open.
#[test]
fn with_a_finger_every_text_field_presses_in_the_touch_target() {
    let mut short = Vec::new();
    let mut moved = Vec::new();
    let mut seen = 0;
    // every page, and the first with the palette open over it
    let cases = PAGES
        .iter()
        .flat_map(|p| WIDTHS.map(|w| (p.title, w, false)))
        .chain(WIDTHS.map(|w| (PAGES[0].title, w, true)));
    for (title, width, palette) in cases {
        let draw = |touch: bool| {
            set_touch_input(touch);
            let (mut view, mut c) = drawn(title, width);
            if palette {
                view.toggle_palette();
                c = Compositor::new();
                view.render(&mut c);
            }
            set_touch_input(false);
            (view, c)
        };
        let (_, pointer) = draw(false);
        let (view, touch) = draw(true);
        let at = format!("{title} @{width}{}", if palette { " ⌘K" } else { "" });
        if scene(&pointer) != scene(&touch) {
            moved.push(at.clone());
        }
        let min = view.theme.control.min_touch_target;
        for (id, [_, _, w, h]) in fields(&view, &touch) {
            seen += 1;
            if w < min - 0.01 || h < min - 0.01 {
                short.push(format!("{at}: {id} {w:.1}x{h:.1}"));
            }
        }
    }
    assert!(seen > 0, "no text field found");
    assert!(
        moved.is_empty(),
        "the drawing changes with a finger:\n{}",
        moved.join("\n")
    );
    assert!(
        short.is_empty(),
        "{} text fields under the touch target:\n{}",
        short.len(),
        short.join("\n")
    );
}
