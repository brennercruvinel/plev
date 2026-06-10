use crate::animation::Spring;
use crate::compositor::{Compositor, SceneNode};
use crate::theme::{MotionPhysics, Theme};

use super::{EventResult, Rect, WidgetEvent, contrast_text, with_alpha};

/// Track dimensions (the widget centers them inside its bounds).
const TRACK_W: f32 = 36.0;
const TRACK_H: f32 = 20.0;
const KNOB: f32 = 16.0;
const KNOB_PAD: f32 = 2.0;

/// Toggle switch with a spring-animated knob.
///
/// Call [`tick`](Switch::tick) every frame while it returns `true` to
/// drive the knob animation.
#[derive(Clone, Debug)]
pub struct Switch {
    pub on: bool,
    pub disabled: bool,
    hovered: bool,
    pressed: bool,
    /// Knob position progress: 0.0 = off, 1.0 = on.
    knob: Spring<f32>,
}

impl Switch {
    pub fn new(on: bool) -> Self {
        Self {
            on,
            disabled: false,
            hovered: false,
            pressed: false,
            knob: Spring::new(if on { 1.0 } else { 0.0 }),
        }
    }

    /// Use theme motion physics for the knob spring (call once at setup):
    /// `Switch::new(false).with_motion(&theme.motion)`.
    pub fn with_motion(mut self, motion: &MotionPhysics) -> Self {
        self.knob = self.knob.with_motion(motion);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Knob animation progress (0.0..=1.0), exposed for tests.
    pub fn knob_progress(&self) -> f32 {
        self.knob.get().clamp(0.0, 1.0)
    }

    pub fn toggle(&mut self) {
        self.on = !self.on;
        self.knob.set_target(if self.on { 1.0 } else { 0.0 });
    }

    /// Advance the knob spring. Returns `true` while still animating.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.knob.tick(dt);
        self.knob.is_animating()
    }

    pub fn is_animating(&self) -> bool {
        self.knob.is_animating()
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        if self.disabled {
            if self.hovered || self.pressed {
                self.hovered = false;
                self.pressed = false;
                return EventResult::changed();
            }
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let inside = bounds.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if bounds.contains(x, y) {
                    self.pressed = true;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                if !self.pressed {
                    return EventResult::IGNORED;
                }
                self.pressed = false;
                if bounds.contains(x, y) {
                    self.toggle();
                    EventResult::clicked()
                } else {
                    EventResult::changed()
                }
            }
            WidgetEvent::Scroll { .. } => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        let tx = bounds.x + (bounds.w - TRACK_W) / 2.0;
        let ty = bounds.y + (bounds.h - TRACK_H) / 2.0;
        let t = self.knob_progress();

        // Track: blend off-color -> accent with the knob progress so color
        // and position animate together.
        let off = theme.colors.bg_hover.0;
        let on = theme.colors.accent.0;
        let track = [
            off[0] + (on[0] - off[0]) * t,
            off[1] + (on[1] - off[1]) * t,
            off[2] + (on[2] - off[2]) * t,
            alpha,
        ];
        let border = if self.on || self.hovered {
            [0.0; 4]
        } else {
            with_alpha(theme.colors.divider, alpha)
        };
        compositor.push(SceneNode::RoundedRect {
            x: tx,
            y: ty,
            w: TRACK_W,
            h: TRACK_H,
            color: track,
            corner_radius: TRACK_H / 2.0,
            border_width: if border[3] > 0.0 { 1.0 } else { 0.0 },
            border_color: border,
        });

        // Knob slides between the padded ends of the track.
        let kx = tx + KNOB_PAD + (TRACK_W - KNOB - KNOB_PAD * 2.0) * t;
        let knob_color = contrast_text(track);
        compositor.push(SceneNode::RoundedRect {
            x: kx,
            y: ty + KNOB_PAD,
            w: KNOB,
            h: KNOB,
            color: [knob_color[0], knob_color[1], knob_color[2], alpha],
            corner_radius: KNOB / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
    }
}
