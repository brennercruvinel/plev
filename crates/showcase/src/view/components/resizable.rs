//! Resizable page: a split pane with a draggable divider.

use comps::prelude::{EventResult, Rect, SplitDirection, SplitPane, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::{panel, text};

pub struct ResizablePage {
    split: SplitPane,
}

impl ResizablePage {
    pub fn new(theme: &Theme) -> Self {
        Self {
            split: SplitPane::new(SplitDirection::Horizontal, 0.35, theme),
        }
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        Rect::new(content.x, content.y, content.w, theme.size.card_min_w)
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.size.card_min_w
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.split.handle_event(event, Self::rect(content, theme))
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        panel(c, rect, theme);
        let first = self.split.first_rect(rect);
        let second = self.split.second_rect(rect);
        let style = theme.typography.base_2r();
        let pad = theme.spacing.lg;
        text(
            c,
            &format!("first pane: {:.0}px (35% desired)", first.w),
            style.font_size,
            style.font_weight,
            first.x + pad,
            first.y + pad,
            theme.colors.text_mid.0,
        );
        text(
            c,
            "second pane",
            style.font_size,
            style.font_weight,
            second.x + pad,
            second.y + pad,
            theme.colors.text_mid.0,
        );
        self.split.render(c, rect, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_drags() {
        let theme = Theme::hoff();
        let mut page = ResizablePage::new(&theme);
        let content = Rect::new(288.0, 80.0, 1272.0, 900.0);
        let d = page
            .split
            .divider_rect(ResizablePage::rect(content, &theme));
        let (x, y) = (d.x + 1.0, d.y + 20.0);
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(page.split.is_dragging());
        page.handle_event(&WidgetEvent::MouseMove { x: x + 100.0, y }, content, &theme);
        assert!(page.split.ratio() > 0.35);
    }
}
