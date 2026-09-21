//! Select page: the pill select and its dropdown (drawn on the overlay
//! layer so it floats over the page). The open dropdown takes events
//! before everything beneath it; Escape closes it, Tab focuses the pill.

use comps::prelude::{EventResult, Rect, Select, WidgetEvent};
use engine::compositor::{Compositor, LayerId};
use engine::theme::{ControlSize, Theme};

use super::EditKey;

pub struct SelectPage {
    select: Select,
}

impl SelectPage {
    pub fn new() -> Self {
        Self {
            select: Select::new(
                ["System default", "Always dark", "Always light", "Scheduled"],
                0,
            ),
        }
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            content.w.min(theme.size.menu_w),
            theme.control.height(ControlSize::Md),
        )
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    pub fn is_open(&self) -> bool {
        self.select.is_open()
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            self.select.set_focused(true);
            return true;
        }
        false
    }

    /// Escape: close the dropdown, else blur. `false` when neither applied.
    pub fn handle_escape(&mut self) -> bool {
        if self.select.is_open() {
            self.select.close();
            true
        } else if self.select.is_focused() {
            self.select.set_focused(false);
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
        self.select
            .handle_event(event, Self::rect(content, theme), theme)
    }

    pub fn render(&self, c: &mut Compositor, overlay: LayerId, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        self.select.render(c, rect, theme);
        self.select.render_dropdown(c, overlay, rect, theme);
    }
}
