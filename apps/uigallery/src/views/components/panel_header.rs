//! Panel Header page: the column head every panel starts with, its
//! blurb and its count badge, on a panel.

use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::Panel;
use engine::widgets::nav::PanelHeader;
use engine::widgets::prelude::{Badge, EventResult, Rect, WidgetEvent};

pub struct PanelHeaderPage {
    header: PanelHeader,
}

impl PanelHeaderPage {
    pub fn new() -> Self {
        Self {
            header: PanelHeader::new("Changes")
                .blurb("the column head every panel starts with")
                .badge(Badge::tag("12")),
        }
    }

    fn rect(&self, content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            content.w.min(theme.size.readable_max_w),
            self.header.height(theme),
        )
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        self.header.height(theme)
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
        let rect = self.rect(content, theme);
        Panel::new().render(c, rect, theme);
        self.header.render(c, rect, theme);
    }
}
