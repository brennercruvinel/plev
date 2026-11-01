//! The kit's range slider (`app/globals.css:22`): a tall well that fills
//! to the value and a raised block thumb, `theme.selection.slider`,
//! painted by the buttons' `paint_surface`. The well takes the token's
//! height inside the bounds; the full bounds are the hit area, and
//! keyboard focus ([`Slider::set_focused`]) rings the well.
//!
//! With a [`SemId`] ([`Slider::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`Slider::project`] declares the slider
//! in a semantic projection; the app still owns the value.

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use accord::{Constraints, NodeBuilder, ParamSpec, Role, SemId, SnapshotBuilder, ValueKind};

use crate::widgets::buttons::paint_surface;
use crate::widgets::recipe::{focus_ring, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// Horizontal slider. Dragging anywhere on the bounds moves the thumb:
/// the full height is the hit area.
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
        let thumb = theme.selection.slider.thumb_w.min(bounds.w);
        let usable = (bounds.w - thumb).max(1.0);
        let t = ((x - bounds.x - thumb / 2.0) / usable).clamp(0.0, 1.0);
        self.min + t * (self.max - self.min)
    }

    /// The thumb travel maps through the theme's thumb width, so the event
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

    /// The well inside `bounds`: the token's height, never taller than
    /// the bounds, centered down them.
    fn well(bounds: Rect, theme: &Theme) -> Rect {
        let h = theme.selection.slider.height.min(bounds.h);
        Rect::new(bounds.x, bounds.y + (bounds.h - h) / 2.0, bounds.w, h)
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            compositor.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let s = &theme.selection.slider;
        let alpha = if self.disabled { s.disabled_alpha } else { 1.0 };
        let well = Self::well(bounds, theme);
        let radius = s.radius.min(well.h / 2.0);
        let t = self.ratio();

        if self.focused {
            compositor.push(focus_ring(well, radius, theme));
        }
        paint_surface(
            compositor,
            LayerId::DEFAULT,
            well,
            radius,
            &s.track,
            alpha,
            false,
            theme,
        );
        // The fill to the value, as the kit's gradient stop at `value%`.
        let fill_w = well.w * t;
        if fill_w > 0.0 {
            compositor.push(rounded_rect(
                well.x,
                well.y,
                fill_w,
                well.h,
                radius.min(fill_w / 2.0),
                with_alpha(s.fill, s.fill.0[3] * alpha),
            ));
        }
        let thumb = Rect::new(
            well.x + (well.w - s.thumb_w).max(0.0) * t,
            well.y,
            s.thumb_w.min(well.w),
            well.h,
        );
        paint_surface(
            compositor,
            LayerId::DEFAULT,
            thumb,
            s.thumb_radius.min(thumb.w / 2.0),
            &s.thumb,
            alpha,
            false,
            theme,
        );
    }
}
