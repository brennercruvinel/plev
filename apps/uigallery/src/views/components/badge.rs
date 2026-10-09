//! Badge page: counts and tags, per intent, flowing in a row.

use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::prelude::{Badge, EventResult, Rect, WidgetEvent};

use super::flow;

pub struct BadgePage {
    badges: Vec<Badge>,
}

impl BadgePage {
    pub fn new() -> Self {
        Self {
            badges: vec![
                Badge::count("3"),
                Badge::count("128"),
                Badge::count("1").intent(Intent::Constructive),
                Badge::tag("MODIFIED"),
                Badge::tag("staged").intent(Intent::Constructive),
                Badge::tag("conflict").intent(Intent::Destructive),
            ],
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            self.badges.iter().map(|b| b.preferred_size(theme)),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).1
    }

    pub fn handle_event(
        &mut self,
        _event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (rects, _) = self.layout(content, theme);
        for (badge, rect) in self.badges.iter().zip(rects) {
            badge.render(c, rect, theme);
        }
    }
}
