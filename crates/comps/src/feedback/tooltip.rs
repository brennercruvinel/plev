//! HOFF tooltip: the solid tooltip body at the tooltip radius with the
//! hint shadow, caption-r text-secondary, placed above its anchor (or
//! below when there is no room) and kept inside the viewport by the
//! `xs` step.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;

use crate::core::Rect;
use crate::recipe::shadow_node;

/// The per-theme geometry placement and render share.
struct Metrics {
    pad_x: f32,
    pad_y: f32,
    /// Gap between the anchor and the tooltip.
    gap: f32,
    /// Smallest distance from the viewport edge.
    margin: f32,
    max_w: f32,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        Self {
            pad_x: theme.spacing.md,
            pad_y: theme.spacing.xs,
            gap: theme.spacing.xs + theme.spacing.xs / 2.0,
            margin: theme.spacing.xs,
            max_w: theme.size.tooltip_max_w,
        }
    }
}

/// Seconds of continuous hover before a tooltip shows (the platform
/// convention on macOS and windows sits between 0.4 and 0.5).
const DEFAULT_DELAY: f32 = 0.45;

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
            delay: DEFAULT_DELAY,
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
    fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_r()
    }

    /// Tooltip rect: centered above the anchor, flipped below when there
    /// is no room, clamped to the viewport.
    pub fn placement(&self, theme: &Theme, vw: f32, vh: f32) -> Rect {
        let m = Metrics::of(theme);
        let style = Self::text_style(theme);
        let (tw, th) = TextMeasurer::measure_styled(&self.text, &style, Some(m.max_w));
        let w = tw + m.pad_x * 2.0;
        let h = th.max(style.line_height) + m.pad_y * 2.0;

        let mut x = self.anchor.x + (self.anchor.w - w) / 2.0;
        x = x.clamp(m.margin, (vw - w - m.margin).max(m.margin));

        let above = self.anchor.y - m.gap - h;
        let y = if above >= m.margin {
            above
        } else {
            (self.anchor.y + self.anchor.h + m.gap).min((vh - h - m.margin).max(m.margin))
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
        let m = Metrics::of(theme);
        let rect = self.placement(theme, vw, vh);

        let radius = theme.shape.tooltip;
        if let Some(shadow) = shadow_node(rect, radius, &theme.shadows.hint) {
            compositor.push_to_layer(layer, shadow);
        }
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
                key: TextNodeKey::from_style(&self.text, &Self::text_style(theme), Some(m.max_w)),
                x: rect.x + m.pad_x,
                y: rect.y + m.pad_y,
                color: theme.colors.text_mid.0,
            },
        );
    }
}
