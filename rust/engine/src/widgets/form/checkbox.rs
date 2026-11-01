//! The checkbox, derived from the kit's switch (the kit has none): the
//! `control.box_size` box at `shape.micro` is the switch's well, ringed
//! off and dark on with the knob's paper as the check, plus an optional
//! base-2r trailing label; `theme.selection.checkbox`, painted by the
//! buttons' `paint_surface`. The whole bounds are the hit area;
//! keyboard focus ([`Checkbox::set_focused`]) rings the box itself.
//!
//! With a [`SemId`] ([`Checkbox::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`Checkbox::project`] declares the
//! checkbox in a semantic projection; the app still owns `checked`.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::TypeRole;
use crate::styling::theme::{IconSize, Theme};
use crate::widgets::hit::HitArea;
use crate::widgets::icons;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::paint_surface;
use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// Retained checkbox with optional trailing label. The whole bounds act
/// as the hit area (box + label), like every native toolkit.
#[derive(Clone, Debug)]
pub struct Checkbox {
    pub checked: bool,
    pub label: Option<String>,
    pub disabled: bool,
    hovered: bool,
    pressed: bool,
    focused: bool,
    sem: Option<SemId>,
    /// The press area: the bounds grown to the minimum target
    /// (`widgets::hit`), an 18 px box pressing in 24.
    hit: HitArea,
}

impl Checkbox {
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            label: None,
            disabled: false,
            hovered: false,
            pressed: false,
            focused: false,
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

    /// Declares this checkbox as `id` in a projection: a
    /// `Role::Checkbox` named `label`, its `checked` state, enabled unless
    /// disabled, and a `toggle` action that reads its own `checked`. It
    /// only reads the widget; the app owns it and runs `toggle`. Pass the
    /// id given to [`Checkbox::sem`] so the node and the binding name the
    /// same control. Returns the node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Checkbox, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("checked", self.checked).enabled(!self.disabled);
        node.action("toggle").reads(format!("{id}:checked"));
        node
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
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
    /// chain). Disabled checkboxes refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
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
                    self.checked = !self.checked;
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
        let t = &theme.selection.checkbox;
        let alpha = if self.disabled { t.disabled_alpha } else { 1.0 };
        let box_size = theme.control.box_size;
        let radius = theme.shape.micro;
        let r = Rect::new(
            bounds.x,
            bounds.y + (bounds.h - box_size) / 2.0,
            box_size,
            box_size,
        );

        if self.focused {
            // Ring the box, not the label: that is where the action is.
            compositor.push(focus_ring(r, radius, theme));
        }

        let surface = match (self.checked, self.hovered) {
            (true, _) => &t.on,
            (false, true) => &t.off_hover,
            (false, false) => &t.off,
        };
        paint_surface(
            compositor,
            LayerId::DEFAULT,
            r,
            radius,
            surface,
            alpha,
            false,
            theme,
        );
        if self.checked {
            // The check glyph sits centered in the box at the small icon
            // size, so it keeps its margin at every box size.
            // disabled, the box fades and the check takes a gray that
            // still reads on it, drawn whole
            let mark = if self.disabled {
                t.mark_disabled.0
            } else {
                t.mark.0
            };
            let glyph = theme.control.icon(IconSize::Sm).min(box_size);
            let inset = (box_size - glyph) / 2.0;
            if let Some(node) = icons::icon_at("check", glyph, mark, r.x + inset, r.y + inset) {
                compositor.push(node);
            }
        }

        if let Some(label) = &self.label {
            // Label: base-2r.
            let style = theme.typography.role(TypeRole::Body);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: r.right() + theme.control.inline_gap,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: with_alpha(t.label, t.label.0[3] * alpha),
            });
        }
    }
}
