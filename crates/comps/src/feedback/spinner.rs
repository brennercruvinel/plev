//! Indeterminate spinner: a 270° arc rotating at a fixed angular speed,
//! sized by the theme's small icon scale. The owner drives it with
//! `tick(dt)` and only while it is visible — under render-on-demand a
//! spinner that ticks off-screen would busy-loop every frame (the same
//! rule the dock caret follows in the showcase).

use std::f32::consts::TAU;

use engine::compositor::Compositor;
use engine::path::PathBuilder;
use engine::theme::{IconSize, Theme};

use crate::core::Rect;

/// Full turns per second.
const SPEED: f32 = 0.9;
/// Arc sweep (270°).
const SWEEP: f32 = TAU * 0.75;
/// Arc segments (smooth enough at the largest size).
const SEGMENTS: usize = 24;

/// Spinner sizes, on the theme's icon scale.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SpinnerSize {
    /// Inline / inside buttons: the small icon.
    Sm,
    /// Default: the large icon.
    #[default]
    Md,
    /// Page-level loading: the large icon plus the `sm` step.
    Lg,
}

impl SpinnerSize {
    pub fn px(self, theme: &Theme) -> f32 {
        match self {
            SpinnerSize::Sm => theme.control.icon(IconSize::Sm),
            SpinnerSize::Md => theme.control.icon(IconSize::Lg),
            SpinnerSize::Lg => theme.control.icon(IconSize::Lg) + theme.spacing.sm,
        }
    }
}

/// Rotating indeterminate indicator. Construct once per visible spinner
/// and `tick` it from the view's animation pass; `render` draws the arc
/// at the current angle into `bounds` (centered, square).
#[derive(Clone, Debug)]
pub struct Spinner {
    pub size: SpinnerSize,
    angle: f32,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: SpinnerSize::default(),
            angle: 0.0,
        }
    }

    pub fn size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }

    /// Current rotation (radians) — exposed for tests and for owners that
    /// sync multiple spinners.
    pub fn angle(&self) -> f32 {
        self.angle
    }

    /// Advance the rotation. Always animating while ticked: the caller
    /// decides visibility, so this returns `true` unconditionally (the
    /// view must only call it while the spinner is on screen).
    pub fn tick(&mut self, dt: f32) -> bool {
        self.angle = (self.angle + dt * SPEED * TAU) % TAU;
        true
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let px = self.size.px(theme).min(bounds.w).min(bounds.h);
        let stroke = theme.control.spinner_stroke;
        if px < stroke * 2.0 {
            return;
        }
        let (cx, cy) = bounds.center();
        let r = (px - stroke) / 2.0;

        // Arc polyline from `angle` over SWEEP; round caps read as a
        // rotating comma. Color: the quiet text token (spinners are
        // chrome, not data).
        let color = theme.colors.text_mid.0;
        let mut b = PathBuilder::new();
        for i in 0..=SEGMENTS {
            let a = self.angle + SWEEP * (i as f32 / SEGMENTS as f32);
            let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
            b = if i == 0 {
                b.move_to(x, y)
            } else {
                b.line_to(x, y)
            };
        }
        compositor.draw_path(b.end_open().stroke_round(color, stroke));
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}
