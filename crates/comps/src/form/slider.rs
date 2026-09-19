//! HOFF slider: 4px glass track with the white 90deg progress gradient
//! and a top-lit 14px knob that grows on hover/drag. The full bounds are
//! the hit area (no 4px precision games); keyboard focus
//! ([`Slider::set_focused`]) rings those bounds with the accent ring.

use crate::compositor::{Compositor, SceneNode};
use crate::theme::Theme;

use super::{EventResult, Rect, WidgetEvent, focus_ring, with_alpha};

const TRACK_H: f32 = 4.0;
const KNOB: f32 = 14.0;

/// Horizontal slider. Dragging anywhere on the bounds moves the knob —
/// the full height is the hit area so 4px tracks aren't a precision game.
#[derive(Clone, Debug)]
pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub disabled: bool,
    /// Snap increment; `None` for continuous.
    pub step: Option<f32>,
    value: f32,
    hovered: bool,
    dragging: bool,
    focused: bool,
}

impl Slider {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        let mut s = Self {
            min,
            max,
            disabled: false,
            step: None,
            value: 0.0,
            hovered: false,
            dragging: false,
            focused: false,
        };
        s.set_value(value);
        s
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, v: f32) {
        let v = if let Some(step) = self.step {
            (v / step).round() * step
        } else {
            v
        };
        self.value = v.clamp(self.min, self.max);
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Keyboard focus, driven by the owning view (plev has no global focus
    /// chain). Disabled sliders refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Normalized position 0.0..=1.0.
    pub fn ratio(&self) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        (self.value - self.min) / (self.max - self.min)
    }

    fn value_at(&self, x: f32, bounds: Rect) -> f32 {
        let usable = (bounds.w - KNOB).max(1.0);
        let t = ((x - bounds.x - KNOB / 2.0) / usable).clamp(0.0, 1.0);
        self.min + t * (self.max - self.min)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        if self.disabled {
            if self.hovered || self.dragging {
                self.hovered = false;
                self.dragging = false;
                return EventResult::changed();
            }
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let mut result = EventResult::IGNORED;
                let inside = bounds.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    result = EventResult::changed();
                }
                if self.dragging {
                    let old = self.value;
                    self.set_value(self.value_at(x, bounds));
                    if self.value != old {
                        result = result.merge(EventResult::changed());
                    }
                    result.merge(EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    })
                } else {
                    result
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if bounds.contains(x, y) {
                    self.dragging = true;
                    self.set_value(self.value_at(x, bounds));
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { .. } => {
                if self.dragging {
                    self.dragging = false;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::Scroll { .. } => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        let ty = bounds.y + (bounds.h - TRACK_H) / 2.0;
        let t = self.ratio();
        let usable = (bounds.w - KNOB).max(1.0);
        let knob_x = bounds.x + usable * t;
        let glass = &theme.glass;
        let text = theme.colors.text;

        if self.focused {
            compositor.push(focus_ring(bounds, bounds.h / 2.0, theme));
        }

        // Track: rgba($n2,.10).
        let track = glass.surface_active;
        compositor.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: ty,
            w: bounds.w,
            h: TRACK_H,
            color: with_alpha(track, track.0[3] * alpha),
            corner_radius: TRACK_H / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });

        // Filled portion: the HOFF progress gradient
        // linear-gradient(90deg, rgba(255,255,255,0) -> .40).
        let fill_w = knob_x + KNOB / 2.0 - bounds.x;
        if fill_w > 1.0 {
            compositor.push(SceneNode::GradientRect {
                x: bounds.x,
                y: ty,
                w: fill_w,
                h: TRACK_H,
                color: [text.0[0], text.0[1], text.0[2], 0.0],
                color2: [text.0[0], text.0[1], text.0[2], 0.40 * alpha],
                // CSS 90deg: transparent stop on the left.
                angle_deg: 90.0,
                corner_radius: TRACK_H / 2.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
        }

        // Knob: the white handle gradient (.90 -> .30, top-lit); grows
        // subtly on hover/drag.
        let grow = if self.dragging || self.hovered {
            1.0
        } else {
            0.0
        };
        let k = KNOB + grow * 2.0;
        let top = glass.knob_gradient[0];
        let bottom = glass.knob_gradient[1];
        compositor.push(SceneNode::GradientRect {
            x: knob_x - grow,
            y: bounds.y + (bounds.h - k) / 2.0,
            w: k,
            h: k,
            color: with_alpha(top, top.0[3] * alpha),
            color2: with_alpha(bottom, bottom.0[3] * alpha),
            angle_deg: 180.0,
            corner_radius: k / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
    }
}
