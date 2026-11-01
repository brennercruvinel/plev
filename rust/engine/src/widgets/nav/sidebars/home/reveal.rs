//! Focus into view in the HomeSidebar's body: where a part is, and the
//! least scroll that shows it (task-104 front H, after front I's rule).

use accord::SemId;

use super::{HomeRow, HomeSidebar};
use crate::control::input::reveal::{Cover, Span, nearest};
use crate::control::input::scroll::ScrollState;
use crate::styling::theme::Theme;
use crate::widgets::widget::Rect;

impl HomeSidebar {
    /// Where the part `id` names is in `bounds`, the body scrolled as it
    /// is: a dropdown's trigger or one of its links (while it is open), a
    /// link, the scenes link, the name's field, a folder. `None` for the
    /// logo (it never scrolls) and for what is not there.
    pub fn part_rect(&self, id: &SemId, bounds: Rect, theme: &Theme) -> Option<Rect> {
        let l = self.layout(bounds, theme);
        for (row, r) in self.rows.iter().zip(&l.rows) {
            match row {
                HomeRow::Menu(m, links) => {
                    if m.sem_id() == Some(id) {
                        return Some(m.trigger_rect(*r, theme));
                    }
                    if let Some(i) = links.position(id) {
                        let at = m.content_rect(*r, links.height(r.w, theme), theme);
                        return m.is_open().then(|| links.link_rects(at, theme)[i]);
                    }
                }
                HomeRow::Link(link) if link.sem_id() == Some(id) => return Some(*r),
                HomeRow::Link(_) => {}
            }
        }
        if self.library && self.scenes.sem_id() == Some(id) {
            return Some(l.scenes);
        }
        if self.name.sem_id() == Some(id) {
            return l.field;
        }
        self.folder_rows()
            .find(|r| self.folder_id(*r).as_ref() == Some(id))
            .map(|r| Self::folder_rect(r, &l))
    }

    /// Focus into view (WCAG 2.4.11) in the body: scrolls it the least
    /// that shows the part `id` names whole, with a spacing step round
    /// its ring (`reveal::nearest`, the page's rule, task-104 front I),
    /// gliding there with the page's spring and the theme's motion
    /// (`reveal::Glide`, the page's own), at once with reduced motion. `true`
    /// when it moves: the body is there already, or on its way
    /// ([`HomeSidebar::wake`] carries it).
    pub fn reveal(&mut self, id: &SemId, bounds: Rect, theme: &Theme) -> bool {
        let Some(r) = self.part_rect(id, bounds, theme) else {
            return false;
        };
        let body = self.body_rect(bounds, theme);
        let by = nearest(
            Span::new(r.y, r.h),
            Span::new(body.y, body.h),
            theme.spacing.sm,
            Cover::default(),
        );
        let from = self.scroll_offset(bounds, theme);
        self.glide_max = self.max_scroll(bounds, theme);
        let to = (from + by).clamp(0.0, self.glide_max);
        if to == from {
            return false;
        }
        let mut scroll = self.glide_scroll(from);
        self.glide.to(&mut scroll, to, &theme.motion);
        self.scroll = scroll.offset();
        true
    }

    /// Whether the body glides toward a focus.
    pub fn revealing(&self) -> bool {
        self.glide.is_gliding()
    }

    /// The glide advanced by `dt`: `true` while it still moves.
    pub(super) fn glide_tick(&mut self, dt: f32) -> bool {
        if !self.glide.is_gliding() {
            return false;
        }
        let mut scroll = self.glide_scroll(self.scroll);
        let moving = self.glide.tick(&mut scroll, dt);
        self.scroll = scroll.offset();
        moving
    }

    /// A scroll state for the glide to drive: the body's reach as it was
    /// when the glide started, at `offset`.
    fn glide_scroll(&self, offset: f32) -> ScrollState {
        let mut scroll = ScrollState::new();
        scroll.set_content(self.glide_max);
        scroll.scroll_to(offset);
        scroll
    }
}
