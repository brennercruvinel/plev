//! Dialog page: two buttons open a plain and a destructive modal. The
//! view owns the overlay stack; the page only reports which to open.

use comps::prelude::{Button, ButtonVariant, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::flow;

pub struct DialogPage {
    open: Button,
    delete: Button,
}

impl DialogPage {
    pub fn new() -> Self {
        Self {
            open: Button::new("Open dialog").variant(ButtonVariant::Outline),
            delete: Button::new("Delete repository…")
                .variant(ButtonVariant::Warning)
                .icon("trash"),
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            [&self.open, &self.delete]
                .into_iter()
                .map(|b| b.preferred_size(theme)),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).1
    }

    /// Route an event; the second value is `Some(destructive)` when a
    /// button asked for its dialog.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<bool>) {
        let (rects, _) = self.layout(content, theme);
        let mut action = None;
        let r = self.open.handle_event(event, rects[0]);
        if r.clicked {
            action = Some(false);
        }
        let d = self.delete.handle_event(event, rects[1]);
        if d.clicked {
            action = Some(true);
        }
        (r.merge(d), action)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (rects, _) = self.layout(content, theme);
        self.open.render(c, rects[0], theme);
        self.delete.render(c, rects[1], theme);
    }
}
