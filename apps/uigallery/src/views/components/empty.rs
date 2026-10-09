//! Empty page: the empty state with a call to action and the plain one,
//! side by side when the page is wide.

use engine::widgets::prelude::{Button, EmptyState, EventResult, Rect, WidgetEvent};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;

use super::panel;

pub struct EmptyPage {
    with_cta: EmptyState,
    plain: EmptyState,
}

impl EmptyPage {
    pub fn new() -> Self {
        Self {
            with_cta: EmptyState::new(
                "No results yet",
                "Run a search and the hits land here, \
                 with scores, citations and the explain panel.",
            )
            .icon("search")
            .cta(Button::new("Run a search").icon("search")),
            plain: EmptyState::new("Nothing archived", "Items you archive show up here."),
        }
    }

    fn rects(content: Rect, theme: &Theme) -> ([Rect; 2], f32) {
        let gap = theme.spacing.md;
        let h = theme.size.card_min_w;
        if content.w >= theme.size.card_min_w * 2.0 + gap {
            let w = (content.w - gap) / 2.0;
            (
                [
                    Rect::new(content.x, content.y, w, h),
                    Rect::new(content.x + w + gap, content.y, w, h),
                ],
                h,
            )
        } else {
            (
                [
                    Rect::new(content.x, content.y, content.w, h),
                    Rect::new(content.x, content.y + h + gap, content.w, h),
                ],
                h * 2.0 + gap,
            )
        }
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        Self::rects(content, theme).1
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let ([a, b], _) = Self::rects(content, theme);
        self.with_cta
            .handle_event(event, a, theme)
            .merge(self.plain.handle_event(event, b, theme))
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let ([a, b], _) = Self::rects(content, theme);
        panel(c, a, theme);
        self.with_cta.render(c, a, theme);
        panel(c, b, theme);
        self.plain.render(c, b, theme);
    }
}
