//! Chip page: selected, per intent, and one interactive chip that
//! toggles on click.

use comps::prelude::{Chip, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{Intent, Theme};

use super::flow;

pub struct ChipPage {
    chips: Vec<Chip>,
}

impl ChipPage {
    pub fn new() -> Self {
        Self {
            chips: vec![
                Chip::new("exact"),
                Chip::new("ann").selected(true),
                Chip::new("bm25").intent(Intent::Constructive),
                Chip::new("missing").intent(Intent::Destructive),
                Chip::new("beta").intent(Intent::Informational),
                Chip::new("click me").interactive(true),
            ],
        }
    }

    /// The interactive chip's state, for the tests.
    #[cfg(test)]
    pub fn toggled(&self) -> bool {
        self.chips[5].selected
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            self.chips.iter().map(|c| c.preferred_size(theme)),
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
        for (chip, rect) in self.chips.iter_mut().zip(rects) {
            let r = chip.handle_event(event, rect);
            if r.clicked {
                chip.selected = !chip.selected;
                return EventResult::clicked();
            }
            result = result.merge(r);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (rects, _) = self.layout(content, theme);
        for (chip, rect) in self.chips.iter().zip(rects) {
            chip.render(c, rect, theme);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_chip_toggles_on_click() {
        let theme = Theme::hoff();
        let mut page = ChipPage::new();
        let content = Rect::new(288.0, 80.0, 1272.0, 900.0);
        let (rects, _) = page.layout(content, &theme);
        let chip = rects[5];
        let (x, y) = (chip.x + 4.0, chip.y + 4.0);
        let r = page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(r.changed);
        let r = page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert!(r.clicked);
        assert!(page.toggled());
    }
}
