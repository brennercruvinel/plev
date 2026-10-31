//! Indeterminate spinner. The hoff core kit draws none; it is derived
//! from the kit's progress rail (`theme.message.spinner`): the rail wound
//! into a ring, its track all around and an arc in the rail's head color
//! turning over it at a fixed pace, sized by the theme's icon scale. The owner drives it with
//! `tick(dt)` and only while it is visible — under render-on-demand a
//! spinner that ticks off-screen would busy-loop every frame (the same
//! rule the dock caret follows in the showcase).
//!
//! With the system's "reduce motion" on (`engine::compose::animation::reduced_motion`,
//! the thread's, set by the accessibility bridge) the arc stands still
//! where it is and `tick` says it does not move, so its owner asks for no
//! frame: a still arc still reads as "loading" (task-75 phase 2a).

use std::f32::consts::TAU;

use crate::drawing::compositor::Compositor;
use crate::drawing::path::PathBuilder;
use crate::styling::theme::{IconSize, Theme};

use crate::widgets::recipe::rounded_rect_stroke;
use crate::widgets::widget::Rect;

/// Full turns per second: the pace, behavior apart from the look.
const SPEED: f32 = 0.9;
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

    /// Advance the rotation. Animating while ticked: the caller decides
    /// visibility, so this returns `true` (the view must only call it
    /// while the spinner is on screen), except with reduced motion on,
    /// when the arc stands still and it returns `false`.
    pub fn tick(&mut self, dt: f32) -> bool {
        if crate::compose::animation::reduced_motion() {
            return false;
        }
        self.angle = (self.angle + dt * SPEED * TAU) % TAU;
        true
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.message.spinner;
        let px = self.size.px(theme).min(bounds.w).min(bounds.h);
        let stroke = t.stroke;
        if px < stroke * 2.0 {
            return;
        }
        let (cx, cy) = bounds.center();
        let r = (px - stroke) / 2.0;
        // the track: the whole ring
        compositor.push(rounded_rect_stroke(
            cx - px / 2.0,
            cy - px / 2.0,
            px,
            px,
            px / 2.0,
            t.track.0,
            stroke,
        ));
        // the arc from `angle` over its share; round caps
        let sweep = TAU * t.sweep;
        let mut b = PathBuilder::new();
        for i in 0..=SEGMENTS {
            let a = self.angle + sweep * (i as f32 / SEGMENTS as f32);
            let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
            b = if i == 0 {
                b.move_to(x, y)
            } else {
                b.line_to(x, y)
            };
        }
        compositor.draw_path(b.end_open().stroke_round(t.arc.0, stroke));
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}
