use crate::compositor::{Compositor, SceneNode};
use crate::theme::{Intent, Theme};

use super::{Rect, intent_fill};

/// HOFF download-manager progress: 12px track (radius 6) with a 1.5px
/// 10%-white border, 4px fill bar with the 90° white gradient.
const TRACK_H: f32 = 12.0;
const FILL_H: f32 = 4.0;

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
        let ty = bounds.y + (bounds.h - TRACK_H) / 2.0;
        let glass = &theme.glass;

        // Track: transparent with the 1.5px edge border (rgba(248,248,248,.1)).
        compositor.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: ty,
            w: bounds.w,
            h: TRACK_H,
            color: [0.0; 4],
            corner_radius: TRACK_H / 2.0,
            border_width: 1.5,
            border_color: glass.surface_active.0,
        });

        // Fill: 4px bar, gradient 90deg 0 -> 40% alpha. Neutral runs white
        // (the HOFF monochrome); other intents tint the gradient.
        let inset = (TRACK_H - FILL_H) / 2.0;
        let fill_w = (bounds.w - inset * 2.0) * self.value;
        if fill_w >= 1.0 {
            let (c, a0, a1) = match self.intent {
                Intent::Neutral => (theme.colors.text.0, 0.0, 0.40),
                other => (intent_fill(theme, other), 0.15, 0.90),
            };
            compositor.push(SceneNode::GradientRect {
                x: bounds.x + inset,
                y: ty + inset,
                w: fill_w,
                h: FILL_H,
                color: [c[0], c[1], c[2], a0],
                color2: [c[0], c[1], c[2], a1],
                angle_deg: 90.0,
                corner_radius: FILL_H / 2.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
        }
    }
}
