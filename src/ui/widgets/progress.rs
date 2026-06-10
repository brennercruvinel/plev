use crate::compositor::{Compositor, SceneNode};
use crate::theme::{Intent, Theme};

use super::{Rect, intent_fill, with_alpha};

const TRACK_H: f32 = 6.0;

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

        compositor.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: ty,
            w: bounds.w,
            h: TRACK_H,
            color: with_alpha(theme.colors.bg_hover, 1.0),
            corner_radius: TRACK_H / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });

        let fill_w = bounds.w * self.value;
        if fill_w >= 1.0 {
            compositor.push(SceneNode::RoundedRect {
                x: bounds.x,
                y: ty,
                w: fill_w,
                h: TRACK_H,
                color: intent_fill(theme, self.intent),
                corner_radius: TRACK_H / 2.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
        }
    }
}
