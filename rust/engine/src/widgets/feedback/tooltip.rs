//! The tooltip of the hoff core kit (`components/Tooltip`): a dark bubble
//! with a short line, and the key that runs it when it names one (the
//! toolbar's `Action`), above its anchor, or below it when there is no
//! room, kept inside the container it is given. It shows after a resting
//! pointer's delay, or at once for the keyboard's focus. Every value is
//! `theme.overlay.tooltip`; the placement is the anchored overlays' one
//! ([`place`]), the popover's.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke};

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
    /// The key that runs what it describes, shown beside the text.
    pub key: Option<String>,
    /// Seconds of continuous hover before showing.
    pub delay: f32,
    state: State,
    anchor: Rect,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            key: None,
            delay: DEFAULT_DELAY,
            state: State::Hidden,
            anchor: Rect::default(),
        }
    }

    /// Names `key` beside its text.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds;
        self
    }

    pub fn is_visible(&self) -> bool {
        self.state == State::Visible
    }

    /// Shown at once by `anchor`, with no delay: the keyboard's focus on
    /// it. Returns `true` if visibility changed.
    pub fn show(&mut self, anchor: Rect) -> bool {
        self.anchor = anchor;
        let changed = self.state != State::Visible;
        self.state = State::Visible;
        changed
    }

    /// Hidden now (the focus left, or Escape). Returns `true` if
    /// visibility changed.
    pub fn hide(&mut self) -> bool {
        let changed = self.state == State::Visible;
        self.state = State::Hidden;
        changed
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

    /// The padding around the content: top, end, bottom, start; a
    /// tooltip with a key takes the key's.
    fn pad(&self, theme: &Theme) -> [f32; 4] {
        let t = &theme.overlay.tooltip;
        match self.key {
            Some(_) => t.key.pad,
            None => [t.pad_y, t.pad_x, t.pad_y, t.pad_x],
        }
    }

    /// The bubble's size: the text measured against the widest it gets,
    /// the key beside it, and the padding.
    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.overlay.tooltip;
        let (tw, th) = TextMeasurer::measure_styled(&self.text, &t.text, Some(t.max_w));
        let [top, end, bottom, start] = self.pad(theme);
        let (kw, kh) = match self.key {
            Some(_) => (t.key.gap + t.key.size, t.key.size),
            None => (0.0, 0.0),
        };
        (
            start + tw + kw + end,
            th.max(t.text.line_height).max(kh) + top + bottom,
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
        let rect = self.placement_in(theme, bounds);
        self.render_at(compositor, layer, theme, rect);
    }

    /// The bubble at `rect`, shown or not: its shadow, its fill, its
    /// text, and its key at the end.
    pub fn render_at(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        rect: Rect,
    ) {
        let t = &theme.overlay.tooltip;
        let radius = t.radius.min(rect.h / 2.0);
        for spec in &t.shadow {
            if let Some(shadow) = shadow_node(rect, radius, spec) {
                compositor.push_to_layer(layer, shadow);
            }
        }
        compositor.push_to_layer(
            layer,
            rounded_rect(rect.x, rect.y, rect.w, rect.h, radius, t.fill.0),
        );
        let [top, end, bottom, start] = self.pad(theme);
        let inner_h = rect.h - top - bottom;
        let (_, th) = TextMeasurer::measure_styled(&self.text, &t.text, Some(t.max_w));
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.text, &t.text, Some(t.max_w)),
                x: rect.x + start,
                y: rect.y + top + (inner_h - th.max(t.text.line_height)) / 2.0,
                color: t.text_color.0,
            },
        );
        let Some(key) = &self.key else {
            return;
        };
        let k = &t.key;
        let kx = rect.right() - end - k.size;
        let ky = rect.y + top + (inner_h - k.size) / 2.0;
        compositor.push_to_layer(
            layer,
            rounded_rect(kx, ky, k.size, k.size, k.radius, k.fill.0),
        );
        if k.border.0[3] > 0.0 && k.border_w > 0.0 {
            compositor.push_to_layer(
                layer,
                rounded_rect_stroke(kx, ky, k.size, k.size, k.radius, k.border.0, k.border_w),
            );
        }
        let (kw, _) = TextMeasurer::measure_styled(key, &t.text, None);
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(key, &t.text, None),
                x: kx + (k.size - kw) / 2.0,
                y: ky + (k.size - t.text.line_height) / 2.0,
                color: t.text_color.0,
            },
        );
    }
}
