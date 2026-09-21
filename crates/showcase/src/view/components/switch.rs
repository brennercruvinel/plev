//! Switch page: two live spring toggles and a disabled one, each with a
//! caption. Tab moves the focus ring over the enabled ones.

use comps::prelude::{EventResult, Rect, Switch, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{ControlSize, Theme};

use super::{EditKey, Focus, text};

const LABELS: [&str; 3] = ["Focus mode", "Wrap long lines", "Disabled switch"];

pub struct SwitchPage {
    switches: [Switch; 3],
    focus: Focus,
}

impl SwitchPage {
    pub fn new(theme: &Theme) -> Self {
        Self {
            switches: [
                Switch::new(true).with_motion(&theme.motion),
                Switch::new(false).with_motion(&theme.motion),
                Switch::new(false).with_motion(&theme.motion).disabled(true),
            ],
            focus: Focus::default(),
        }
    }

    fn rects(content: Rect, theme: &Theme) -> [Rect; 3] {
        let row = theme.control.height(ControlSize::Xs) + theme.spacing.xs;
        let [w, h] = theme.control.switch_size;
        std::array::from_fn(|i| Rect::new(content.x, content.y + i as f32 * row, w, h))
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        (theme.control.height(ControlSize::Xs) + theme.spacing.xs) * 3.0
    }

    fn set_focus(&mut self, slot: Option<usize>) {
        self.focus.slot = slot;
        self.switches[0].set_focused(slot == Some(0));
        self.switches[1].set_focused(slot == Some(1));
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
        for (s, rect) in self.switches.iter_mut().zip(Self::rects(content, theme)) {
            r = r.merge(s.handle_event(event, rect));
        }
        r
    }

    /// Knob springs. `true` while any is moving.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = false;
        for s in &mut self.switches {
            animating |= s.tick(dt);
        }
        animating
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let style = theme.typography.base_2r();
        for ((s, rect), label) in self
            .switches
            .iter()
            .zip(Self::rects(content, theme))
            .zip(LABELS)
        {
            s.render(c, rect, theme);
            text(
                c,
                label,
                style.font_size,
                style.font_weight,
                rect.right() + theme.spacing.md,
                rect.y + (rect.h - style.line_height) / 2.0,
                if s.disabled {
                    theme.colors.text_dim.0
                } else {
                    theme.colors.text.0
                },
            );
        }
    }
}
