//! The kit's switch (`components/Switch/index.tsx`): a pill well, light
//! off and dark on, and a raised knob that springs across it; the
//! surfaces are `theme.selection.switch` and the measures
//! `theme.control.switch_size` and `switch_knob`, painted by the
//! buttons' `paint_surface`. Tick it every frame while animating;
//! keyboard focus ([`Switch::set_focused`]) rings the track with the
//! accent focus ring.
//!
//! With a [`SemId`] ([`Switch::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`Switch::project`] declares the switch
//! in a semantic projection; the app still owns `on`.

use crate::compose::animation::Spring;
use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::color::Color;
use crate::styling::theme::{ButtonSurface, MotionPhysics, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::paint_surface;
use crate::widgets::hit::HitArea;
use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, mix};

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
    sem: Option<SemId>,
    /// The press area: the bounds grown to the minimum target
    /// (`widgets::hit`), the 40x22 track pressing in 40x24.
    hit: HitArea,
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
            sem: None,
            hit: HitArea::default(),
        }
    }

    /// The logical identity the render binds its bounds to
    /// (`Compositor::bind`).
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this switch as `id` in a projection: a `Role::Toggle`
    /// named `label`, its `on` state, enabled unless disabled, and a
    /// `toggle` action that reads its own `on`. The knob's travel is
    /// animation and is never projected. It only reads the widget; the
    /// app owns it and runs `toggle` ([`Switch::toggle`]). Pass the id
    /// given to [`Switch::sem`] so the node and the binding name the same
    /// control. Returns the node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Toggle, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("on", self.on).enabled(!self.disabled);
        node.action("toggle").reads(format!("{id}:on"));
        node
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
        let bounds = self.hit.grow(bounds);
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
        self.hit.remember(theme);
        if let Some(id) = &self.sem {
            let hit = self.hit.grow(bounds);
            compositor.bind(id, [hit.x, hit.y, hit.w, hit.h]);
        }
        let s = &theme.selection.switch;
        let alpha = if self.disabled { s.disabled_alpha } else { 1.0 };
        let [track_w, track_h] = theme.control.switch_size;
        let knob = theme.control.switch_knob;
        let knob_pad = theme.control.switch_knob_pad();
        let track = Rect::new(
            bounds.x + (bounds.w - track_w) / 2.0,
            bounds.y + (bounds.h - track_h) / 2.0,
            track_w,
            track_h,
        );
        let t = self.knob_progress();

        if self.focused {
            // Ring the visible track, not the (larger) hit bounds.
            compositor.push(focus_ring(track, track_h / 2.0, theme));
        }

        // The well's fill blends off to on with the knob's travel, so
        // color and position move together; its shades are the state the
        // knob is nearer.
        let near = if t < 0.5 { &s.off } else { &s.on };
        let well = ButtonSurface {
            fill: [0, 1].map(|i| Color(mix(s.off.fill[i].0, s.on.fill[i].0, t))),
            ..near.clone()
        };
        paint_surface(
            compositor,
            LayerId::DEFAULT,
            track,
            track_h / 2.0,
            &well,
            alpha,
            false,
            theme,
        );
        let kx = track.x + knob_pad + (track_w - knob - knob_pad * 2.0) * t;
        paint_surface(
            compositor,
            LayerId::DEFAULT,
            Rect::new(kx, track.y + knob_pad, knob, knob),
            knob / 2.0,
            &s.knob,
            alpha,
            false,
            theme,
        );
    }
}
