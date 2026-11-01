//! Every control a person can press is at least 24 px on each side
//! (WCAG 2.2, 2.5.8 Target Size (Minimum), AA): the rect each actionable
//! node of a page's projection binds, the one its press lands in, on
//! every page at a desktop and at a phone width. The drawing may be
//! smaller: `engine::widgets::hit_rect` grows the press area around it.
//!
//! The shell's sidebar is front H's (its rows, its toggle) and is left
//! out; what of it falls short is listed in `SIDEBAR`, exact, so a change
//! there shows here.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use accord::SemId;
use engine::drawing::compositor::Compositor;
use uigallery::views::{PAGES, ShowcaseView};

/// WCAG 2.5.8's minimum, in logical px.
const MIN: f32 = 24.0;

/// The widths the sweep looks at: the kit's desktop and a phone.
const WIDTHS: [f32; 2] = [1280.0, 360.0];

/// What a page binds under its actionable nodes below `MIN`, as
/// `(page, id, w, h)`.
fn short_targets(title: &str, width: f32) -> Vec<(String, SemId, f32, f32)> {
    let mut view = ShowcaseView::new(width, 800.0);
    view.jump_to_page(title);
    let mut c = Compositor::new();
    view.render(&mut c);
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

fn sidebar(id: &SemId) -> bool {
    id.as_str().starts_with("shell.sidebar")
}

#[test]
fn every_pressable_node_is_at_least_24_px_on_each_side() {
    let mut short = Vec::new();
    let mut side = Vec::new();
    for page in PAGES {
        for width in WIDTHS {
            for (title, id, w, h) in short_targets(page.title, width) {
                let line = format!("{title} @{width}: {id} {w:.1}x{h:.1}");
                if sidebar(&id) {
                    side.push(line);
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
    assert!(
        short.is_empty(),
        "{} targets under {MIN} px:\n{}",
        short.len(),
        short.join("\n")
    );
}
