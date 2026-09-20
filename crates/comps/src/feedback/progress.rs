//! HOFF download-manager progress: a pill track with the strong edge
//! rim, a thinner fill bar inside it with the 90 degree white gradient
//! (or the intent tint). Track and fill heights are `control` tokens.

use engine::compositor::{Compositor, SceneNode};
use engine::theme::{Intent, Theme};

use crate::core::{Rect, intent_fill};

/// Determinate progress bar, colored by intent.
#[derive(Clone, Debug)]
pub struct ProgressBar {
    /// Progress in 0.0..=1.0.
    value: f32,
    pub intent: Intent,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            intent: Intent::Neutral,
        }
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let track_h = theme.control.progress_track;
        let fill_h = theme.control.progress_fill.min(track_h);
        let ty = bounds.y + (bounds.h - track_h) / 2.0;
        let glass = &theme.glass;

        // Track: transparent with the strong edge rim.
        compositor.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: ty,
            w: bounds.w,
            h: track_h,
            color: [0.0; 4],
            corner_radius: track_h / 2.0,
            border_width: theme.control.edge_width_strong,
            border_color: glass.surface_active.0,
        });

        // Fill: gradient from transparent to the faint text alpha. Neutral
        // runs white (the HOFF monochrome); other intents tint the
        // gradient from the wash alpha to the knob highlight alpha.
        let inset = (track_h - fill_h) / 2.0;
        let fill_w = (bounds.w - inset * 2.0) * self.value;
        if fill_w >= 1.0 {
            let (c, a0, a1) = match self.intent {
                Intent::Neutral => (theme.colors.text.0, 0.0, glass.text_faint.0[3]),
                other => (
                    intent_fill(theme, other),
                    glass.wash_alpha,
                    glass.knob_gradient[0].0[3],
                ),
            };
            compositor.push(SceneNode::GradientRect {
                x: bounds.x + inset,
                y: ty + inset,
                w: fill_w,
                h: fill_h,
                color: [c[0], c[1], c[2], a0],
                color2: [c[0], c[1], c[2], a1],
                angle_deg: 90.0,
                corner_radius: fill_h / 2.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
        }
    }
}
