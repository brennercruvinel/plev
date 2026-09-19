//! Indeterminate spinner: a 270° arc rotating at a fixed angular speed,
//! sized by the theme's small icon scale. The owner drives it with
//! `tick(dt)` and only while it is visible — under render-on-demand a
//! spinner that ticks off-screen would busy-loop every frame (the same
//! rule the dock caret follows in the showcase).

use std::f32::consts::TAU;

use crate::compositor::Compositor;
use crate::path::PathBuilder;
use crate::theme::Theme;

use super::Rect;

/// Full turns per second.
const SPEED: f32 = 0.9;
/// Arc sweep (270°).
const SWEEP: f32 = TAU * 0.75;
/// Arc segments (smooth enough at the largest size).
const SEGMENTS: usize = 24;

/// Spinner sizes, on the HOFF icon scale (16 / 24 / 32).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SpinnerSize {
    /// Inline / inside buttons: 16px.
    Sm,
    /// Default: 24px.
    #[default]
    Md,
    /// Page-level loading: 32px.
    Lg,
}

impl SpinnerSize {
    pub fn px(self) -> f32 {
        match self {
            SpinnerSize::Sm => 16.0,
            SpinnerSize::Md => 24.0,
            SpinnerSize::Lg => 32.0,
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
        let px = self.size.px().min(bounds.w).min(bounds.h);
        if px < 4.0 {
            return;
        }
        let (cx, cy) = bounds.center();
        let stroke = (px / 8.0).max(1.5);
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
