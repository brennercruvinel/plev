//! ViewContext -- information provided to Views during render.

use crate::layout::ComputedBounds;
use crate::platform::SafeAreaInsets;
use crate::theme::Theme;

pub struct ViewContext {
    pub width: f32,
    pub height: f32,
    pub bounds: ComputedBounds,
    /// Safe area insets in physical pixels (notch, status bar, etc.).
    pub safe_area: SafeAreaInsets,
    /// Display scale factor (e.g. 2.0 for Retina).
    pub scale_factor: f64,
    /// Whether the virtual keyboard is currently visible.
    pub keyboard_visible: bool,
    /// Estimated keyboard height in physical pixels.
    pub keyboard_height: f32,
    /// Theme tokens (RULE-08). None in legacy/test contexts.
    pub theme: Option<Theme>,
}

impl ViewContext {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            bounds: ComputedBounds {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            safe_area: SafeAreaInsets::default(),
            scale_factor: 1.0,
            keyboard_visible: false,
            keyboard_height: 0.0,
            theme: None,
        }
    }

    pub fn with_bounds(width: f32, height: f32, bounds: ComputedBounds) -> Self {
        Self {
            width,
            height,
            bounds,
            safe_area: SafeAreaInsets::default(),
            scale_factor: 1.0,
            keyboard_visible: false,
            keyboard_height: 0.0,
            theme: None,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }
}
