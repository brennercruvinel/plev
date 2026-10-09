//! Stat page: readouts with unit and delta in three sizes.

use comps::content::{Stat, StatSize};
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{Intent, Theme};

use super::flow;

pub struct StatPage {
    stats: [Stat; 3],
}

impl StatPage {
    pub fn new() -> Self {
        Self {
            stats: [
                Stat::new("38 412", "chunks").size(StatSize::Sm),
                Stat::new("1.3", "on disk")
                    .unit("GB")
                    .delta("+12%", Intent::Constructive),
                Stat::new("97", "recall@10")
                    .unit("%")
                    .size(StatSize::Lg)
                    .delta("-2", Intent::Destructive),
            ],
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.xxl,
            self.stats.iter().map(|s| s.preferred_size(theme)),
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
        for (stat, rect) in self.stats.iter().zip(rects) {
            stat.render(c, rect, theme);
        }
    }
}
