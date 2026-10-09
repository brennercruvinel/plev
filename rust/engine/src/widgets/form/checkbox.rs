//! HOFF checkbox: 18px radius-6 box, white fill with a dark check when
//! on, `rgba($n2,.05)` glass with a `.25` border when off, plus an
//! optional base-2r trailing label. The whole bounds are the hit area;
//! keyboard focus ([`Checkbox::set_focused`]) rings the box itself.
//!
//! With a [`SemId`] ([`Checkbox::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`Checkbox::project`] declares the
//! checkbox in a semantic projection; the app still owns `checked`.

use crate::widgets::icons;
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::{IconSize, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::widget::{EventResult, Rect, WidgetEvent, contrast_text, with_alpha};
use crate::widgets::recipe::focus_ring;

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
        if let Some(id) = &self.sem {
            compositor.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let glass = &theme.glass;
        let alpha = if self.disabled {
            glass.disabled_alpha
        } else {
            1.0
        };
        let box_size = theme.control.box_size;
        let radius = theme.shape.micro;
        let bx = bounds.x;
        let by = bounds.y + (bounds.h - box_size) / 2.0;

        if self.focused {
            // Ring the box, not the label: that is where the action is.
            compositor.push(focus_ring(
                Rect::new(bx, by, box_size, box_size),
                radius,
                theme,
            ));
        }

        if self.checked {
            // Filled white ($text-primary) with a dark check on top (push
            // order is preserved across primitive types).
            let fill = theme.colors.text;
            compositor.push(crate::widgets::recipe::rounded_rect(
                bx,
                by,
                box_size,
                box_size,
                radius,
                with_alpha(fill, fill.0[3] * alpha),
            ));
            let mark = contrast_text(fill.0);
            let mark = [mark[0], mark[1], mark[2], alpha];
            // The check glyph sits centered in the box at the small icon
            // size, so it keeps its margin at every box size.
            let glyph = theme.control.icon(IconSize::Sm).min(box_size);
            let inset = (box_size - glyph) / 2.0;
            if let Some(node) = icons::icon_at("check", glyph, mark, bx + inset, by + inset) {
                compositor.push(node);
            }
        } else {
            // Unchecked: rgba($n2,.05) glass with a .25 border (hover .1 bg).
            let bg = if self.hovered {
                glass.surface_active
            } else {
                glass.field
            };
            let border = glass.field_focus_border;
            compositor.push(SceneNode::RoundedRect {
                x: bx,
                y: by,
                w: box_size,
                h: box_size,
                color: with_alpha(bg, bg.0[3] * alpha),
                corner_radius: radius,
                border_width: theme.control.edge_width,
                border_color: with_alpha(border, border.0[3] * alpha),
            });
        }

        if let Some(label) = &self.label {
            // Label: base-2r.
            let style = theme.typography.base_2r();
            let text = theme.colors.text_mid;
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: bx + box_size + theme.control.inline_gap,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: with_alpha(text, text.0[3] * alpha),
            });
        }
    }
}
