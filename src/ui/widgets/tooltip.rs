use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Theme, TypographyScale};

use super::Rect;

/// HOFF tooltip: solid #262626, radius 8, pad 5 12 3, caption-r
/// $text-secondary, shadow 0 1.5px 2px rgba(24,24,24,.15).
const PAD_X: f32 = 12.0;
const PAD_Y: f32 = 4.0;
const GAP: f32 = 6.0;
const MAX_W: f32 = 280.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Hidden,
    /// Hovering, waiting out the delay.
    Pending {
        elapsed: f32,
    },
    Visible,
}

/// Hover tooltip with show delay and viewport-aware placement.
///
/// Drive it with [`set_hover`](Tooltip::set_hover) from the owner's hit
/// test and [`tick`](Tooltip::tick) each frame; render last (overlay
/// layer) so it sits above everything.
#[derive(Clone, Debug)]
pub struct Tooltip {
    pub text: String,
    /// Seconds of continuous hover before showing.
    pub delay: f32,
    state: State,
    anchor: Rect,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            delay: 0.45,
            state: State::Hidden,
            anchor: Rect::default(),
        }
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds;
        self
    }

    pub fn is_visible(&self) -> bool {
        self.state == State::Visible
    }

    /// Update hover state. `anchor` is the rect the tooltip describes.
    /// Returns `true` if visibility changed (request a frame).
    pub fn set_hover(&mut self, hovering: bool, anchor: Rect) -> bool {
        match (hovering, self.state) {
            (true, State::Hidden) => {
                self.anchor = anchor;
                self.state = State::Pending { elapsed: 0.0 };
                false
            }
            (true, _) => {
                self.anchor = anchor;
                false
            }
            (false, State::Hidden) => false,
            (false, prev) => {
                self.state = State::Hidden;
                prev == State::Visible
            }
        }
    }

    /// Advance the show delay. Returns `true` while a frame is needed
    /// (pending countdown or the moment it becomes visible).
    pub fn tick(&mut self, dt: f32) -> bool {
        match self.state {
            State::Pending { elapsed } => {
                let elapsed = elapsed + dt;
                if elapsed >= self.delay {
                    self.state = State::Visible;
                } else {
                    self.state = State::Pending { elapsed };
                }
                true
            }
            _ => false,
        }
    }

    /// Body style: caption-r, the same for measuring and rendering.
    fn text_style() -> TextStyle {
        TypographyScale::hoff().caption_r()
    }

    /// Tooltip rect: centered above the anchor, flipped below when there
    /// is no room, clamped to the viewport.
    pub fn placement(&self, vw: f32, vh: f32) -> Rect {
        let style = Self::text_style();
        let (tw, th) = TextMeasurer::measure_styled(&self.text, &style, Some(MAX_W));
        let w = tw + PAD_X * 2.0;
        let h = th.max(style.line_height) + PAD_Y * 2.0;

        let mut x = self.anchor.x + (self.anchor.w - w) / 2.0;
        x = x.clamp(4.0, (vw - w - 4.0).max(4.0));

        let above = self.anchor.y - GAP - h;
        let y = if above >= 4.0 {
            above
        } else {
            (self.anchor.y + self.anchor.h + GAP).min((vh - h - 4.0).max(4.0))
        };
        Rect::new(x, y, w, h)
    }

    pub fn render(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        if self.state != State::Visible {
            return;
        }
        let rect = self.placement(vw, vh);

        let radius = theme.radius.sm;
        compositor.push_to_layer(
            layer,
            SceneNode::Shadow {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                corner_radius: radius,
                blur_radius: 2.0,
                offset: [0.0, 1.5],
                color: [24.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 0.15],
                inset: false,
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                color: theme.glass.tooltip.0,
                corner_radius: radius,
                border_width: 0.0,
                border_color: [0.0; 4],
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.text, &Self::text_style(), Some(MAX_W)),
                x: rect.x + PAD_X,
                y: rect.y + PAD_Y,
                color: theme.colors.text_mid.0,
            },
        );
    }
}
