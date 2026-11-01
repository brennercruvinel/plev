//! A low window (a laptop split in two, a phone held sideways): every
//! page 400 px high, each trigger that opens something (a node whose
//! `expanded` is false) pressed in turn; what it opens must be reachable
//! in that window: every new control lies inside the window, or inside a
//! region that scrolls (its binding clipped), and no text of the frame is
//! cut across.

use accord::{SemId, Value};
use engine::drawing::compositor::Compositor;
use engine::widgets::prelude::WidgetEvent;
use uigallery::views::{PAGES, ShowcaseView};

use super::{EPS, fmt, intersect, short, sidebar, targets, texts};

/// The low window's height.
pub const LOW_H: f32 = 400.0;
/// The triggers pressed per page, at most (the first in reading order).
const MAX_TRIGGERS: usize = 6;
/// Frames a press is given to open (reduced motion ends a transition on
/// its first tick).
const SETTLE: usize = 4;

fn open(title: &str, width: f32) -> (ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(width, LOW_H);
    view.jump_to_page(title);
    let mut c = Compositor::new();
    view.render(&mut c);
    (view, c)
}

/// The page's closed triggers, in reading order.
fn triggers(view: &ShowcaseView, c: &Compositor) -> Vec<SemId> {
    let Some(snapshot) = view.reader_semantics() else {
        return Vec::new();
    };
    let mut out: Vec<(f32, f32, SemId)> = Vec::new();
    for (id, node) in snapshot.nodes() {
        let closed = node
            .state
            .iter()
            .any(|(k, v)| k.as_str() == "expanded" && *v == Value::Bool(false));
        if !closed || !node.enabled || node.inert || sidebar(id) {
            continue;
        }
        let Some(b) = c.binding(id) else { continue };
        if b.visible_rect().is_none() {
            continue;
        }
        out.push((b.rect[1], b.rect[0], id.clone()));
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    out.into_iter().take(MAX_TRIGGERS).map(|t| t.2).collect()
}

/// What breaks `title` at `width` in the low window, a line each.
pub fn breaks(title: &str, width: f32) -> Vec<String> {
    let (view, c) = open(title, width);
    let mut out = Vec::new();
    for trigger in triggers(&view, &c) {
        let (mut view, mut c) = open(title, width);
        let before: Vec<SemId> = targets(&view, &c).into_iter().map(|t| t.id).collect();
        let Some(r) = c.binding(&trigger).and_then(|b| b.visible_rect()) else {
            continue;
        };
        let (x, y) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
        view.handle_event(&WidgetEvent::MouseDown { x, y });
        view.handle_event(&WidgetEvent::MouseUp { x, y });
        for _ in 0..SETTLE {
            view.tick(1.0 / 60.0);
            view.render(&mut c);
        }
        let window = [0.0, 0.0, view.width, view.height];
        let at = format!("{title} @{width}x{LOW_H} after {trigger}");
        for t in targets(&view, &c) {
            if before.contains(&t.id) {
                continue;
            }
            let clipped = c.binding(&t.id).is_some_and(|b| b.clip.is_some());
            let inside = t.rect[1] >= -EPS && t.rect[1] + t.rect[3] <= LOW_H + EPS;
            if !inside && !clipped {
                out.push(format!("{at}: {} {} out of the window", t.id, fmt(t.rect)));
            }
        }
        for s in texts(&c, window) {
            if intersect(s.rect, s.clip).is_none() {
                continue;
            }
            if s.rect[0] < s.clip[0] - EPS || s.rect[0] + s.rect[2] > s.clip[0] + s.clip[2] + EPS {
                out.push(format!(
                    "{at}: '{}' {} cut across by {}",
                    short(&s.text),
                    fmt(s.rect),
                    fmt(s.clip)
                ));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Every page at both widths in the low window; fails with the list.
pub fn report() {
    engine::compose::animation::set_reduced_motion(true);
    let mut all = Vec::new();
    for page in PAGES {
        for width in super::WIDTHS {
            all.extend(breaks(page.title, width));
        }
    }
    engine::compose::animation::set_reduced_motion(false);
    let all: Vec<String> = all
        .into_iter()
        .filter(|l| super::known(l).is_none())
        .collect();
    assert!(
        all.is_empty(),
        "{} breaks in a {LOW_H} px window:\n{}",
        all.len(),
        all.join("\n")
    );
}
