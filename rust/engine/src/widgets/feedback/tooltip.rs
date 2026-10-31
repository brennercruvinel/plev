//! The tooltip of the hoff core kit (`components/Tooltip`): a dark bubble
//! with a short line, above its anchor, or below it when there is no room,
//! kept inside the container it is given. Every value is
//! `theme.overlay.tooltip`; the placement is the anchored overlays' one
//! ([`place`]), the popover's.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;

use crate::widgets::overlay::popover::{Align, Side, place};
use crate::widgets::recipe::shadow_node;
use crate::widgets::widget::Rect;

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

    /// The bubble's size: the text measured against the widest it gets,
    /// and the padding.
    fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.overlay.tooltip;
        let (tw, th) = TextMeasurer::measure_styled(&self.text, &t.text, Some(t.max_w));
        (
            tw + t.pad_x * 2.0,
            th.max(t.text.line_height) + t.pad_y * 2.0,
        )
    }

    /// The bubble inside `bounds`, the container that receives it:
    /// centered above the anchor, flipped below when there is no room
    /// above, shifted and capped to stay inside.
    pub fn placement_in(&self, theme: &Theme, bounds: Rect) -> Rect {
        let clear = theme.overlay.tooltip.clearance;
        let size = self.size(theme);
        place(self.anchor, size, bounds, Side::Top, Align::Center, clear).rect
    }

    /// [`Tooltip::placement_in`] a `vw x vh` window, for an owner whose
    /// container is the window.
    pub fn placement(&self, theme: &Theme, vw: f32, vh: f32) -> Rect {
        self.placement_in(theme, Rect::new(0.0, 0.0, vw, vh))
    }

    /// [`Tooltip::render_in`] a `vw x vh` window.
    pub fn render(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        self.render_in(compositor, layer, theme, Rect::new(0.0, 0.0, vw, vh));
    }

    /// The bubble, while visible, on `layer`, inside `bounds`.
    pub fn render_in(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        bounds: Rect,
    ) {
        if self.state != State::Visible {
            return;
        }
        let t = &theme.overlay.tooltip;
        let rect = self.placement_in(theme, bounds);
        let radius = t.radius.min(rect.h / 2.0);
        for spec in &t.shadow {
            if let Some(shadow) = shadow_node(rect, radius, spec) {
                compositor.push_to_layer(layer, shadow);
            }
        }
        compositor.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                color: t.fill.0,
                corner_radius: radius,
                border_width: 0.0,
                border_color: [0.0; 4],
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.text, &t.text, Some(t.max_w)),
                x: rect.x + t.pad_x,
                y: rect.y + t.pad_y,
                color: t.text_color.0,
            },
        );
    }
}
