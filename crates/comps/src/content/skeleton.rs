//! Skeleton: the loading placeholder. Lines in the surface-active wash
//! (the HOFF "fake text" bar) at the progress-fill thickness, or a block
//! at the card radius, breathing between the surface and surface-hover
//! alphas. Tick it only while visible: under render-on-demand a
//! skeleton that ticks off-screen would busy-loop every frame.

use engine::compositor::{Compositor, LayerId};
use engine::theme::Theme;

use crate::core::{Rect, mix};
use crate::recipe::rounded_rect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonShape {
    /// Text lines: the last one shorter, like a paragraph.
    #[default]
    Lines,
    /// One block filling the bounds (an image, a chart).
    Block,
    /// A disc the smaller side wide (an avatar).
    Disc,
}

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub shape: SkeletonShape,
    /// Line count for `Lines`.
    pub lines: usize,
    /// Breathing phase in seconds.
    phase: f32,
}

impl Skeleton {
    pub fn new(shape: SkeletonShape) -> Self {
        Self {
            shape,
            lines: 3,
            phase: 0.0,
        }
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = lines.max(1);
        self
    }

    /// Advance the breathing. Always `true`: a visible skeleton animates.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.phase += dt;
        true
    }

    /// Breathing progress 0..1, a full cycle every `duration.slow` times
    /// four (slow enough to read as a pulse, not a flicker).
    fn pulse(&self, theme: &Theme) -> f32 {
        let period = theme.duration.slow * 4.0;
        let t = (self.phase % period) / period;
        (t * std::f32::consts::TAU).sin() * 0.5 + 0.5
    }

    /// Line pitch for `Lines`: the body line box.
    pub fn line_pitch(theme: &Theme) -> f32 {
        theme.typography.body().line_height
    }

    /// Natural height: `lines` line boxes for `Lines`, the bounds for
    /// the others (they fill what they are given).
    pub fn preferred_height(&self, theme: &Theme) -> Option<f32> {
        match self.shape {
            SkeletonShape::Lines => Some(self.lines as f32 * Self::line_pitch(theme)),
            _ => None,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let color = mix(
            glass.surface_hover.0,
            glass.surface_active.0,
            self.pulse(theme),
        );
        match self.shape {
            SkeletonShape::Lines => {
                let pitch = Self::line_pitch(theme);
                let bar = theme.control.progress_fill * 2.0;
                for i in 0..self.lines {
                    let y = bounds.y + i as f32 * pitch + (pitch - bar) / 2.0;
                    if y + bar > bounds.y + bounds.h {
                        break;
                    }
                    // The last line runs short, like the end of a paragraph.
                    let w = if i + 1 == self.lines && self.lines > 1 {
                        bounds.w * 0.6
                    } else {
                        bounds.w
                    };
                    c.push_to_layer(layer, rounded_rect(bounds.x, y, w, bar, bar / 2.0, color));
                }
            }
            SkeletonShape::Block => c.push_to_layer(
                layer,
                rounded_rect(
                    bounds.x,
                    bounds.y,
                    bounds.w,
                    bounds.h,
                    theme.shape.card.min(bounds.h / 2.0),
                    color,
                ),
            ),
            SkeletonShape::Disc => {
                let d = bounds.w.min(bounds.h);
                c.push_to_layer(
                    layer,
                    rounded_rect(
                        bounds.x + (bounds.w - d) / 2.0,
                        bounds.y + (bounds.h - d) / 2.0,
                        d,
                        d,
                        d / 2.0,
                        color,
                    ),
                );
            }
        }
    }
}
