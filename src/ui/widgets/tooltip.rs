use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::Theme;

use super::{Rect, with_alpha};

const FONT: f32 = 12.0;
const PAD_X: f32 = 8.0;
const PAD_Y: f32 = 5.0;
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

    /// Tooltip rect: centered above the anchor, flipped below when there
    /// is no room, clamped to the viewport.
    pub fn placement(&self, vw: f32, vh: f32) -> Rect {
        let style = TextStyle::new(FONT);
        let (tw, th) = TextMeasurer::measure_styled(&self.text, &style, Some(MAX_W));
        let w = tw + PAD_X * 2.0;
        let h = th.max(FONT * 1.3) + PAD_Y * 2.0;

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

        compositor.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                color: with_alpha(theme.colors.bg_panel, 0.98),
                corner_radius: theme.radius.md + 2.0,
                border_width: 1.0,
                border_color: with_alpha(theme.colors.divider, 1.0),
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::new(&self.text, FONT, FONT * 1.3, Some(MAX_W)),
                x: rect.x + PAD_X,
                y: rect.y + PAD_Y,
                color: with_alpha(theme.colors.text, 1.0),
            },
        );
    }
}
