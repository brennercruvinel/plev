//! HOFF slider: 4px glass track with the white 90deg progress gradient
//! and a top-lit 14px knob that grows on hover/drag. The full bounds are
//! the hit area (no 4px precision games); keyboard focus
//! ([`Slider::set_focused`]) rings those bounds with the accent ring.
//!
//! With a [`SemId`] ([`Slider::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`Slider::project`] declares the slider
//! in a semantic projection; the app still owns the value.

use crate::drawing::compositor::{Compositor, SceneNode};
use crate::styling::theme::Theme;
use accord::{Constraints, NodeBuilder, ParamSpec, Role, SemId, SnapshotBuilder, ValueKind};

use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

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
    sem: Option<SemId>,
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
            sem: None,
        };
        s.set_value(value);
        s
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

    /// Declares this slider as `id` in a projection: a `Role::Slider`
    /// named `label`, its `value` as a number, editable within its
    /// `min..=max` range and its step, enabled unless disabled, with a
    /// `set_value` action that takes a number checked against that range.
    /// A stepped slider also declares `increment` and `decrement` (one
    /// step, [`Slider::increment`]); a continuous one has no step to move
    /// by, so it declares neither. Every action reads the `value`, so a
    /// request decided on a value another actor moved goes stale. The
    /// knob's hover and drag are never projected. It only reads the
    /// widget; the app owns it and runs the actions. Pass the id given to
    /// [`Slider::sem`] so the node and the binding name the same control.
    /// Returns the node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Slider, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        let value = format!("{id}:value");
        node.state("value", f64::from(self.value))
            .constraints(Constraints {
                editable: true,
                range: Some((f64::from(self.min), f64::from(self.max))),
                step: self.step.map(f64::from),
                ..Constraints::default()
            })
            .enabled(!self.disabled);
        node.action("set_value")
            .param(ParamSpec {
                kind: ValueKind::Num,
                constrained: true,
            })
            .reads(value.as_str());
        if self.step.is_some() {
            node.action("increment").reads(value.as_str());
            node.action("decrement").reads(value.as_str());
        }
        node
    }

    /// Moves the value one step up, clamped to `max`. `false` when it did
    /// not move: continuous (no step) or already at the top.
    pub fn increment(&mut self) -> bool {
        self.nudge(1.0)
    }

    /// Moves the value one step down, clamped to `min`. `false` when it
    /// did not move: continuous (no step) or already at the bottom.
    pub fn decrement(&mut self) -> bool {
        self.nudge(-1.0)
    }

    fn nudge(&mut self, direction: f32) -> bool {
        let Some(step) = self.step else {
            return false;
        };
        let old = self.value;
        self.set_value(old + direction * step);
        self.value != old
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

    fn value_at(&self, x: f32, bounds: Rect, theme: &Theme) -> f32 {
        let knob = theme.control.slider_knob;
        let usable = (bounds.w - knob).max(1.0);
        let t = ((x - bounds.x - knob / 2.0) / usable).clamp(0.0, 1.0);
        self.min + t * (self.max - self.min)
    }

    /// The knob travel maps through the theme's knob size, so the event
    /// path takes the same `theme` the render path draws with.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
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
                    self.set_value(self.value_at(x, bounds, theme));
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
                    self.set_value(self.value_at(x, bounds, theme));
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
        if let Some(id) = &self.sem {
            compositor.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let glass = &theme.glass;
        let alpha = if self.disabled {
            glass.disabled_alpha
        } else {
            1.0
        };
        let track_h = theme.control.slider_track;
        let knob_d = theme.control.slider_knob;
        let ty = bounds.y + (bounds.h - track_h) / 2.0;
        let t = self.ratio();
        let usable = (bounds.w - knob_d).max(1.0);
        let knob_x = bounds.x + usable * t;
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
            h: track_h,
            color: with_alpha(track, track.0[3] * alpha),
            corner_radius: track_h / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });

        // Filled portion: the HOFF progress gradient
        // linear-gradient(90deg, rgba(255,255,255,0) -> .40).
        let fill_w = knob_x + knob_d / 2.0 - bounds.x;
        if fill_w > 1.0 {
            compositor.push(SceneNode::GradientRect {
                x: bounds.x,
                y: ty,
                w: fill_w,
                h: track_h,
                color: [text.0[0], text.0[1], text.0[2], 0.0],
                color2: with_alpha(glass.text_faint, glass.text_faint.0[3] * alpha),
                // CSS 90deg: transparent stop on the left.
                angle_deg: 90.0,
                corner_radius: track_h / 2.0,
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
        let k = knob_d + grow * 2.0;
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
