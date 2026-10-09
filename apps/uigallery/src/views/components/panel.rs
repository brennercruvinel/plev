//! Panel page: the glass surface with rim and key light, on its own.

use engine::widgets::content::Panel;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;

use super::text;

pub struct PanelPage;

impl PanelPage {
    pub fn new() -> Self {
        Self
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            content.w.min(theme.size.readable_max_w),
            theme.size.card_min_w / 2.0,
        )
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.size.card_min_w / 2.0
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
        let rect = Self::rect(content, theme);
        Panel::new().render(c, rect, theme);
        let style = theme.typography.base_2r();
        text(
            c,
            "glass surface, soft edge, top-lit rim",
            style.font_size,
            style.font_weight,
            rect.x + theme.spacing.lg,
            rect.y + theme.spacing.lg,
            theme.colors.text_mid.0,
        );
    }
}
