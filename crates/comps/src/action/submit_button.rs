//! The submit button of the kit: a square primary surface that turns
//! into the dark secondary one on hover, and stays dark while `active`
//! (a request in flight). The arrow icon follows: ink on paper, paper on
//! ink.

use engine::compositor::{Compositor, LayerId};
use engine::theme::{ButtonRecipe, ButtonSurface, IconButtonMetrics, Theme};

use super::IconButtonSize;
use super::press::{Press, paint_surface};
use crate::core::{EventResult, Rect, WidgetEvent};
use crate::icons;

#[derive(Clone, Debug)]
pub struct SubmitButton {
    pub icon: &'static str,
    pub size: IconButtonSize,
    pub disabled: bool,
    pub active: bool,
    press: Press,
}

impl SubmitButton {
    pub fn new() -> Self {
        Self {
            icon: "arrow-up",
            size: IconButtonSize::Lg,
            disabled: false,
            active: false,
            press: Press::default(),
        }
    }

    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.press.hovered
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.press.focused = focused && !self.disabled;
    }

    fn metrics(&self, theme: &Theme) -> IconButtonMetrics {
        self.size.metrics(theme)
    }

    /// Primary at rest; secondary hovered, pressed or active (hovered
    /// while active eases back to the secondary hover state).
    fn surface<'a>(&self, theme: &'a Theme) -> &'a ButtonSurface {
        let primary: &ButtonRecipe = &theme.button.primary;
        let secondary: &ButtonRecipe = &theme.button.secondary;
        if self.active {
            if self.press.hovered {
                &secondary.hover
            } else {
                &secondary.rest
            }
        } else if self.press.pressed {
            &secondary.active
        } else if self.press.hovered {
            &secondary.rest
        } else {
            &primary.rest
        }
    }

    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let m = self.metrics(theme);
        (m.size, m.size)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.press.handle(event, bounds, self.disabled)
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(compositor, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        theme: &Theme,
    ) {
        let m = self.metrics(theme);
        let alpha = if self.disabled {
            theme.button.primary.disabled_alpha
        } else {
            1.0
        };
        let radius = m.radius.min(bounds.h / 2.0);
        let fg = paint_surface(
            compositor,
            layer,
            bounds,
            radius,
            self.surface(theme),
            alpha,
            self.press.focused,
            theme,
        );
        // The arrow is one step under the size's icon (16 in a 40).
        let icon = m.icon * 0.8;
        if let Some(node) = icons::icon_at(
            self.icon,
            icon,
            fg,
            bounds.x + (bounds.w - icon) / 2.0,
            bounds.y + (bounds.h - icon) / 2.0,
        ) {
            compositor.push_to_layer(layer, node);
        }
    }
}

impl Default for SubmitButton {
    fn default() -> Self {
        Self::new()
    }
}
