//! Progress page: three bars, one per intent.

use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::prelude::{EventResult, ProgressBar, Rect, WidgetEvent};

pub struct ProgressPage {
    bars: [ProgressBar; 3],
}

impl ProgressPage {
    pub fn new() -> Self {
        Self {
            bars: [
                ProgressBar::new(0.65),
                ProgressBar::new(1.0).intent(Intent::Constructive),
                ProgressBar::new(0.35).intent(Intent::Destructive),
            ],
        }
    }

    fn rects(content: Rect, theme: &Theme) -> [Rect; 3] {
        let h = theme.control.progress_track;
        let row = h + theme.spacing.lg;
        let w = content.w.min(theme.size.readable_max_w / 2.0);
        std::array::from_fn(|i| Rect::new(content.x, content.y + i as f32 * row, w, h))
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        (theme.control.progress_track + theme.spacing.lg) * 3.0
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
        for (bar, rect) in self.bars.iter().zip(Self::rects(content, theme)) {
            bar.render(c, rect, theme);
        }
    }
}
