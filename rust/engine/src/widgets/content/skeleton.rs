//! Skeleton: the hoff core kit's loading placeholder
//! (`SceneCardSkeleton`): text lines as short bars with a 2 px corner in
//! one gray, blocks (an image, a disc) in the gray a step lighter, and
//! the kit's scene card that holds an image block and two lines. Each
//! breathes toward the next gray step while ticked. Tick it only while
//! visible: under render-on-demand a skeleton that ticks off-screen would
//! busy-loop every frame. With the system's "reduce motion" on
//! (`engine::compose::animation::reduced_motion`) the pulse stands still
//! and `tick` says so, and the placeholder still reads as one (task-75
//! phase 2a). Every value is `theme.message.skeleton`.

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;

use crate::widgets::recipe::rounded_rect;
use crate::widgets::widget::{Rect, mix};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonShape {
    /// Text lines: the last one shorter, like a paragraph.
    #[default]
    Lines,
    /// One block filling the bounds (an image, a chart).
    Block,
    /// A disc the smaller side wide (an avatar).
    Disc,
    /// The kit's scene card: an image block at its aspect over two lines,
    /// on the paper with a hairline, as wide as its bounds.
    Card,
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

    /// Advance the breathing: `true`, a visible skeleton animates; with
    /// reduced motion on, the pulse stands still and it is `false`.
    pub fn tick(&mut self, dt: f32) -> bool {
        if crate::compose::animation::reduced_motion() {
            return false;
        }
        self.phase += dt;
        true
    }

    /// Breathing progress 0..1 from the kit's color toward the next step
    /// and back, a full cycle every `duration.slow` times
    /// four (slow enough to read as a pulse, not a flicker).
    fn pulse(&self, theme: &Theme) -> f32 {
        let period = theme.duration.slow * 4.0;
        let t = (self.phase % period) / period;
        // 0 at rest: an idle skeleton is the kit's color
        (1.0 - (t * std::f32::consts::TAU).cos()) * 0.5
    }

    /// Line pitch for `Lines`: the kit's bar, its padding and gap.
    pub fn line_pitch(theme: &Theme) -> f32 {
        theme.message.skeleton.pitch()
    }

    /// Natural height: `lines` line boxes for `Lines`, the bounds for
    /// the others (they fill what they are given; a card's height follows
    /// its width, [`Skeleton::height_for`]).
    pub fn preferred_height(&self, theme: &Theme) -> Option<f32> {
        match self.shape {
            SkeletonShape::Lines => Some(theme.message.skeleton.lines_h(self.lines)),
            _ => None,
        }
    }

    /// The height at width `w`: a card's, whose image keeps its aspect;
    /// otherwise [`Skeleton::preferred_height`].
    pub fn height_for(&self, w: f32, theme: &Theme) -> Option<f32> {
        let t = &theme.message.skeleton;
        match self.shape {
            SkeletonShape::Card => {
                let frame = t.card_border_width + t.card_pad;
                let image = (w - frame * 2.0).max(0.0) / t.image_aspect;
                let lines = t.lines_h(t.card_lines.len());
                Some(frame * 2.0 + image + t.image_gap + t.text_pad * 2.0 + lines)
            }
            _ => self.preferred_height(theme),
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let t = &theme.message.skeleton;
        let p = self.pulse(theme);
        let block = mix(t.block.0, t.block_pulse.0, p);
        let line = mix(t.line_fill.0, t.line_pulse.0, p);
        match self.shape {
            SkeletonShape::Lines => {
                let shares = (0..self.lines).map(|i| {
                    // the last line runs short, like the end of a paragraph
                    if i + 1 == self.lines && self.lines > 1 {
                        t.last_line
                    } else {
                        1.0
                    }
                });
                bars(c, layer, bounds, shares, line, theme);
            }
            SkeletonShape::Block => c.push_to_layer(
                layer,
                rounded_rect(
                    bounds.x,
                    bounds.y,
                    bounds.w,
                    bounds.h,
                    t.block_radius.min(bounds.h / 2.0).min(bounds.w / 2.0),
                    block,
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
                        block,
                    ),
                );
            }
            SkeletonShape::Card => {
                c.push_to_layer(
                    layer,
                    crate::drawing::compositor::SceneNode::RoundedRect {
                        x: bounds.x,
                        y: bounds.y,
                        w: bounds.w,
                        h: bounds.h,
                        color: t.card_fill.0,
                        corner_radius: t.card_radius.min(bounds.h / 2.0).min(bounds.w / 2.0),
                        border_width: t.card_border_width,
                        border_color: t.card_border.0,
                    },
                );
                let frame = t.card_border_width + t.card_pad;
                let iw = (bounds.w - frame * 2.0).max(0.0);
                let image = Rect::new(bounds.x + frame, bounds.y + frame, iw, iw / t.image_aspect);
                if iw > 0.0 && image.bottom() <= bounds.bottom() {
                    c.push_to_layer(
                        layer,
                        rounded_rect(
                            image.x,
                            image.y,
                            image.w,
                            image.h,
                            t.block_radius.min(image.h / 2.0),
                            block,
                        ),
                    );
                }
                let pad = t.text_pad;
                let text = Rect::new(
                    image.x + pad,
                    image.bottom() + t.image_gap + pad,
                    (iw - pad * 2.0).max(0.0),
                    (bounds.bottom() - frame - pad - image.bottom() - t.image_gap - pad).max(0.0),
                );
                bars(c, layer, text, t.card_lines, line, theme);
            }
        }
    }
}

/// Bars down `bounds`, one per share of its width less the bars'
/// padding, each in its padded box; a bar that would pass the bottom is
/// not drawn.
fn bars(
    c: &mut Compositor,
    layer: LayerId,
    bounds: Rect,
    shares: impl IntoIterator<Item = f32>,
    color: [f32; 4],
    theme: &Theme,
) {
    let t = &theme.message.skeleton;
    let column = bounds.w - t.line_pad * 2.0;
    if column <= 0.0 {
        return;
    }
    for (i, share) in shares.into_iter().enumerate() {
        let y = bounds.y + i as f32 * t.pitch() + t.line_pad;
        if y + t.line > bounds.bottom() {
            break;
        }
        c.push_to_layer(
            layer,
            rounded_rect(
                bounds.x + t.line_pad,
                y,
                column * share,
                t.line,
                t.line_radius.min(t.line / 2.0),
                color,
            ),
        );
    }
}
