//! Checkbox page: checked, unchecked and disabled. Tab moves the focus
//! ring over the enabled ones, Escape blurs.

use comps::prelude::{Checkbox, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{ControlSize, Theme};

use super::{EditKey, Focus};

pub struct CheckboxPage {
    boxes: [Checkbox; 3],
    focus: Focus,
}

impl CheckboxPage {
    pub fn new() -> Self {
        Self {
            boxes: [
                Checkbox::new(true).label("Autosave on focus loss"),
                Checkbox::new(false).label("Share anonymous usage data"),
                Checkbox::new(true)
                    .label("Managed by your organization")
                    .disabled(true),
            ],
            focus: Focus::default(),
        }
    }

    fn rects(content: Rect, theme: &Theme) -> [Rect; 3] {
        let h = theme.control.height(ControlSize::Xs);
        let w = content.w.min(theme.size.readable_max_w / 2.0);
        std::array::from_fn(|i| Rect::new(content.x, content.y + i as f32 * h, w, h))
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Xs) * 3.0
    }

    /// Focus slots: the two enabled boxes.
    fn set_focus(&mut self, slot: Option<usize>) {
        self.focus.slot = slot;
        self.boxes[0].set_focused(slot == Some(0));
        self.boxes[1].set_focused(slot == Some(1));
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            self.set_focus(Some(self.focus.next(2)));
            return true;
        }
        false
    }

    pub fn handle_escape(&mut self) -> bool {
        if self.focus.clear() {
            self.set_focus(None);
            true
        } else {
            false
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut r = EventResult::IGNORED;
        for (b, rect) in self.boxes.iter_mut().zip(Self::rects(content, theme)) {
            r = r.merge(b.handle_event(event, rect));
        }
        r
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        for (b, rect) in self.boxes.iter().zip(Self::rects(content, theme)) {
            b.render(c, rect, theme);
        }
    }
}
