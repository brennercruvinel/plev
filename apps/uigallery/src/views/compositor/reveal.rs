//! Focus into view on the page (task-104 front I, WCAG 2.4.11): when the
//! focus the page projects moves to a control outside the page's window,
//! by Tab, a screen reader's `Focus` or an agent, the page scrolls the
//! least that shows it whole, with a breathing margin of the theme's
//! spacing, gliding with the theme's motion (at once with reduced
//! motion). A focus the pointer moved stays put: the click is where the
//! person looks.
//!
//! It runs after the page drew, for any actor: the rect is the focused
//! node's binding in this frame (where it draws and is hit), or the union
//! of its bound descendants'. A widget that scrolls inside the page (a
//! list, a table, a message thread) brings its row into its own window
//! when it takes the focus, before the draw; the page then shows that
//! row in its window, so the inner container scrolls first and the outer
//! one after, as `engine::control::input::reveal::reveal_nested` orders.
//! The page's window already starts under the header band, so nothing
//! fixed covers it (`Cover::default()`).

use accord::{SemId, Snapshot};
use engine::control::input::reveal::{Cover, Span};
use engine::drawing::compositor::Compositor;

use super::super::ShowcaseView;

/// The smallest rect holding `a` and `b`, `(x, y, w, h)`.
fn union(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let (x, y) = (a[0].min(b[0]), a[1].min(b[1]));
    let right = (a[0] + a[2]).max(b[0] + b[2]);
    let bottom = (a[1] + a[3]).max(b[1] + b[3]);
    [x, y, right - x, bottom - y]
}

/// The rect `id` draws in this frame: its own binding, else the union of
/// its bound descendants' (a group the page projects but does not bind).
fn drawn(snapshot: &Snapshot, c: &Compositor, id: &SemId) -> Option<[f32; 4]> {
    if let Some(b) = c.binding(id) {
        return Some(b.rect);
    }
    snapshot
        .children(id)
        .filter_map(|n| drawn(snapshot, c, &n.id))
        .reduce(union)
}

impl ShowcaseView {
    /// After the page drew into `c`: a focus that moved to a control the
    /// page's window does not show whole scrolls the page toward it.
    /// `true` when the page jumped there (reduced motion) and the frame
    /// must be drawn again at the new offset; a glide moves on `wake`.
    pub(in crate::views) fn reveal_focus(&mut self, c: &Compositor) -> bool {
        if self.is_app() {
            return false;
        }
        let snapshot = self.semantics();
        let focused = snapshot
            .as_ref()
            .and_then(Snapshot::focused)
            .filter(|id| !self.is_overlay(id));
        let Some(id) = self.reveal.moved(focused) else {
            return false;
        };
        let Some(rect) = snapshot.as_ref().and_then(|s| drawn(s, c, &id)) else {
            return false;
        };
        let viewport = self.content_rect();
        let view = Span::new(viewport.y, viewport.h);
        let target = Span::new(rect[1], rect[3]);
        let margin = self.theme.spacing.lg;
        let scroll = &mut self.page_scroll[self.page];
        let to = scroll.revealed(target, view, margin, Cover::default());
        if to == scroll.offset() {
            return false;
        }
        self.glide.to(scroll, to, &self.theme.motion);
        !self.glide.is_gliding()
    }

    /// The glide toward a focus, advanced by `dt`: `true` while it moves.
    pub(in crate::views) fn reveal_tick(&mut self, dt: f32) -> bool {
        if !self.glide.is_gliding() {
            return false;
        }
        self.sync_page_scroll();
        let scroll = &mut self.page_scroll[self.page];
        self.glide.tick(scroll, dt)
    }

    /// Whether the page glides toward a focus: the frame that started it
    /// asks the loop for the next one (`shell/window/pace`).
    pub fn revealing(&self) -> bool {
        self.glide.is_gliding()
    }
}
