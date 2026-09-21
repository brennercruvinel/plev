//! Panel Header page: the column head every panel starts with, its
//! blurb and its count badge, on a panel.

use comps::content::Panel;
use comps::nav::PanelHeader;
use comps::prelude::{Badge, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

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
