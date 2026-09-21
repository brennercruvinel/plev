//! Icon Button page: solid, outline, ghost and toolbar variants, two
//! sizes, one disabled.

use comps::prelude::{
    EventResult, IconButton, IconButtonSize, IconButtonVariant, Rect, WidgetEvent,
};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::flow;

pub struct IconButtonPage {
    buttons: Vec<IconButton>,
}

impl IconButtonPage {
    pub fn new() -> Self {
        Self {
            buttons: vec![
                IconButton::new("copy"),
                IconButton::new("search").variant(IconButtonVariant::Outline),
                IconButton::new("settings").variant(IconButtonVariant::Ghost),
                IconButton::new("trash")
                    .variant(IconButtonVariant::Toolbar)
                    .active(true),
                IconButton::new("save").size(IconButtonSize::Sm),
                IconButton::new("play").size(IconButtonSize::Md),
                IconButton::new("undo").disabled(true),
            ],
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            self.buttons.iter().map(|b| b.preferred_size(theme)),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).1
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let (rects, _) = self.layout(content, theme);
        let mut result = EventResult::IGNORED;
        for (button, rect) in self.buttons.iter_mut().zip(rects) {
            result = result.merge(button.handle_event(event, rect));
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (rects, _) = self.layout(content, theme);
        for (button, rect) in self.buttons.iter().zip(rects) {
            button.render(c, rect, theme);
        }
    }
}
