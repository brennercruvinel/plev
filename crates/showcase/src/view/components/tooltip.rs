//! Tooltip page: hover the button; the tooltip waits, then places
//! itself above the anchor (or below when there is no room). It draws
//! on the topmost layer so it floats over everything.

use comps::prelude::{Button, ButtonVariant, EventResult, Rect, Tooltip, WidgetEvent};
use engine::compositor::{Compositor, LayerId};
use engine::theme::Theme;

pub struct TooltipPage {
    button: Button,
    tooltip: Tooltip,
}

impl TooltipPage {
    pub fn new() -> Self {
        Self {
            button: Button::new("Hover me")
                .variant(ButtonVariant::Outline)
                .icon("eye"),
            tooltip: Tooltip::new(
                "Tooltips wait 450 ms, then place themselves above the anchor, or below when there is no room.",
            ),
        }
    }

    fn rect(&self, content: Rect, theme: &Theme) -> Rect {
        let (w, h) = self.button.preferred_size(theme);
        Rect::new(content.x, content.y, w, h)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.rect(content, theme).h
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rect = self.rect(content, theme);
        let mut r = self.button.handle_event(event, rect);
        if matches!(event, WidgetEvent::MouseMove { .. })
            && self.tooltip.set_hover(self.button.is_hovered(), rect)
        {
            r = r.merge(EventResult::changed());
        }
        r
    }

    /// Advance the tooltip delay. `true` while it needs frames.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.tooltip.tick(dt)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        self.button.render(c, self.rect(content, theme), theme);
    }

    /// The tooltip itself, on `layer`, positioned against the window.
    pub fn render_tooltip(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        self.tooltip.render(c, layer, theme, vw, vh);
    }
}
