//! Slider page: continuous, stepped and disabled, with the live value
//! beside each. Tab moves the focus ring over the enabled ones.

use comps::prelude::{EventResult, Rect, Slider, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{ControlSize, Theme};

use super::{EditKey, Focus, text};

pub struct SliderPage {
    sliders: [Slider; 3],
    focus: Focus,
}

impl SliderPage {
    pub fn new() -> Self {
        Self {
            sliders: [
                Slider::new(0.0, 100.0, 65.0),
                Slider::new(0.0, 10.0, 4.0).step(1.0),
                Slider::new(0.0, 100.0, 30.0).disabled(true),
            ],
            focus: Focus::default(),
        }
    }

    /// The value caption's gutter to the right of every slider.
    fn gutter(theme: &Theme) -> f32 {
        theme.spacing.xxl * 2.0
    }

    fn rects(content: Rect, theme: &Theme) -> [Rect; 3] {
        let h = theme.control.height(ControlSize::Xs);
        let row = h + theme.spacing.md;
        let w = (content.w.min(theme.size.readable_max_w / 2.0) - Self::gutter(theme)).max(0.0);
        std::array::from_fn(|i| Rect::new(content.x, content.y + i as f32 * row, w, h))
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        (theme.control.height(ControlSize::Xs) + theme.spacing.md) * 3.0
    }

    fn set_focus(&mut self, slot: Option<usize>) {
        self.focus.slot = slot;
        self.sliders[0].set_focused(slot == Some(0));
        self.sliders[1].set_focused(slot == Some(1));
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
        for (s, rect) in self.sliders.iter_mut().zip(Self::rects(content, theme)) {
            r = r.merge(s.handle_event(event, rect, theme));
        }
        r
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let style = theme.typography.caption_r();
        let captions = [
            format!("{:.0}", self.sliders[0].value()),
            format!("step 1: {:.0}", self.sliders[1].value()),
            "disabled".to_string(),
        ];
        for ((s, rect), caption) in self
            .sliders
            .iter()
            .zip(Self::rects(content, theme))
            .zip(captions)
        {
            s.render(c, rect, theme);
            text(
                c,
                &caption,
                style.font_size,
                style.font_weight,
                rect.right() + theme.spacing.md,
                rect.y + (rect.h - style.line_height) / 2.0,
                theme.colors.text_mid.0,
            );
        }
    }
}
