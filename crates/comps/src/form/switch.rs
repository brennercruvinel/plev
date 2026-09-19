//! HOFF toggle switch: 44x24 pill track with a 16px spring-animated knob
//! (graphite track fading to the white handle gradient as it travels).
//! Tick it every frame while animating; keyboard focus
//! ([`Switch::set_focused`]) rings the track with the accent focus ring.

use engine::animation::Spring;
use engine::compositor::{Compositor, SceneNode};
use engine::theme::{MotionPhysics, Theme};

use crate::core::{EventResult, Rect, WidgetEvent, mix};
use crate::recipe::focus_ring;

// HOFF switch: `control.switch_size` track (pill), `control.switch_knob`
// knob inset by the derived pad. The widget centers the track inside its
// bounds.

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
    focused: bool,
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
            focused: false,
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

    /// Keyboard focus, driven by the owning view (plev has no global focus
    /// chain). Disabled switches refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
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
        let glass = &theme.glass;
        let alpha = if self.disabled {
            glass.disabled_alpha
        } else {
            1.0
        };
        let [track_w, track_h] = theme.control.switch_size;
        let knob = theme.control.switch_knob;
        let knob_pad = theme.control.switch_knob_pad();
        let tx = bounds.x + (bounds.w - track_w) / 2.0;
        let ty = bounds.y + (bounds.h - track_h) / 2.0;
        let t = self.knob_progress();

        if self.focused {
            // Ring the visible track, not the (larger) hit bounds.
            compositor.push(focus_ring(
                Rect::new(tx, ty, track_w, track_h),
                track_h / 2.0,
                theme,
            ));
        }

        // Track: rgba($n2,.05) off -> rgba(40,40,40,.5) checked, blended
        // with the knob progress so color and position animate together.
        let off = glass.field.0;
        let on = {
            let b = glass.button.0;
            [b[0], b[1], b[2], 0.5]
        };
        let mut track = mix(off, on, t);
        track[3] *= alpha;
        compositor.push(SceneNode::RoundedRect {
            x: tx,
            y: ty,
            w: track_w,
            h: track_h,
            color: track,
            corner_radius: track_h / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });

        // Knob slides 20px between the padded ends. Off: flat
        // rgba($n2,.3); on: the HOFF white handle gradient (.90 -> .30,
        // top-lit) — blended along the same progress.
        let kx = tx + knob_pad + (track_w - knob - knob_pad * 2.0) * t;
        let flat = glass.knob_gradient[1].0;
        let mut top = mix(flat, glass.knob_gradient[0].0, t);
        let mut bottom = flat;
        top[3] *= alpha;
        bottom[3] *= alpha;
        compositor.push(SceneNode::GradientRect {
            x: kx,
            y: ty + knob_pad,
            w: knob,
            h: knob,
            color: top,
            color2: bottom,
            // CSS 180deg: bright stop at the top.
            angle_deg: 180.0,
            corner_radius: knob / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
    }
}
