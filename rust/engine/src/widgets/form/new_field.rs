//! NewField: a value edited in place, the kit's NewField
//! (`components/NewField/index.tsx`). At rest it is the value with a
//! pencil after it, faded while hovered; a press turns it into a bare box
//! (the text alone, [`Look::Bare`]) with a confirm after it, green while
//! there is something to save. The box starts empty, the value kept: the
//! confirm or Enter saves what was typed, or the value again when nothing
//! was; Escape keeps the value as it was. Every value is
//! `theme.form.edit`'s.
//!
//! The box is a [`TextField`] (`edit.input`): typing, the editing keys,
//! the clipboard and the input method reach it as any field.
//!
//! Projected ([`NewField::project`]): one text field, named `label`, its
//! `text` the value saved (never the draft), `editing` while the box is
//! out; `set_value` saves a value as the confirm does. Beside it the
//! pencil (`<id>.edit`, `activate`) at rest, the confirm (`<id>.save`,
//! `activate`) while editing.

use accord::{Constraints, ParamSpec, Role, SemId, SnapshotBuilder, ValueKind};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::form::text_field::TextField;
use crate::widgets::form::tokens::Look;
use crate::widgets::icons;
use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// Where the parts go: at rest the value and the pencil, editing the box
/// and the confirm; `whole` is what a press on it edits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewFieldLayout {
    pub whole: Rect,
    pub text: Rect,
    pub icon: Rect,
}

/// A value edited in place.
#[derive(Clone, Debug)]
pub struct NewField {
    pub input: TextField,
    value: String,
    editing: bool,
    hovered: bool,
    /// The keyboard focus on the pencil or the confirm.
    icon_focus: bool,
    sem: Option<SemId>,
}

impl NewField {
    pub fn new(value: &str, placeholder: &str) -> Self {
        Self {
            input: TextField::new(placeholder).look(Look::Bare),
            value: value.to_owned(),
            editing: false,
            hovered: false,
            icon_focus: false,
            sem: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.input.disabled = disabled;
        self
    }

    /// The id it projects as; the box binds to it while editing; the
    /// pencil and the confirm are `<id>.edit` and `<id>.save`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.input = self.input.sem(id.clone());
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// The id of the pencil at rest, of the confirm while editing.
    pub fn icon_id(&self) -> Option<SemId> {
        let name = if self.editing { "save" } else { "edit" };
        self.sem.as_ref()?.child(name).ok()
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn set_icon_focus(&mut self, on: bool) {
        self.icon_focus = on && !self.input.disabled;
    }

    pub fn has_icon_focus(&self) -> bool {
        self.icon_focus
    }

    /// Start editing: the box out, empty and focused, the value kept.
    /// `false` when disabled.
    pub fn edit(&mut self) -> bool {
        if self.input.disabled {
            return false;
        }
        self.editing = true;
        self.input.set_text("");
        self.input.focus();
        true
    }

    /// Save what was typed, or keep the value when nothing was; the box
    /// goes. `false` when not editing.
    pub fn save(&mut self) -> bool {
        if !self.editing {
            return false;
        }
        let typed = self.input.text().trim();
        if !typed.is_empty() {
            self.value = typed.to_owned();
        }
        self.close();
        true
    }

    /// Leave the box, the value as it was. `false` when not editing.
    pub fn cancel(&mut self) -> bool {
        if !self.editing {
            return false;
        }
        self.close();
        true
    }

    fn close(&mut self) {
        self.editing = false;
        self.input.set_text("");
        self.input.unfocus();
    }

    /// The value set whole (an agent's, a reader's): saved as the confirm
    /// saves it. `false` for an empty one or disabled.
    pub fn set_value(&mut self, value: &str) -> bool {
        let value = value.trim();
        if value.is_empty() || self.input.disabled {
            return false;
        }
        self.value = value.to_owned();
        self.close();
        true
    }

    /// Where the parts go, from the top left of `bounds`, as wide as they
    /// measure, never past its width.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> NewFieldLayout {
        let e = &theme.form.edit;
        let h = e.box_h.max(e.icon);
        let room = (bounds.w - e.gap - e.icon).max(0.0);
        let text_w = if self.editing {
            let shown = [self.input.text(), self.input.input.placeholder.as_str()]
                .map(|t| TextMeasurer::measure_styled(t, &e.text, None).0.ceil())
                .into_iter()
                .fold(e.box_w, f32::max);
            shown + theme.control.focus_ring_width
        } else {
            TextMeasurer::measure_styled(&self.value, &e.text, None)
                .0
                .ceil()
        }
        .min(room);
        let text = Rect::new(bounds.x, bounds.y + (h - e.box_h) / 2.0, text_w, e.box_h);
        let icon = Rect::new(
            text.right() + e.gap,
            bounds.y + (h - e.icon) / 2.0,
            e.icon,
            e.icon,
        );
        let whole = Rect::new(bounds.x, bounds.y, icon.right() - bounds.x, h);
        NewFieldLayout { whole, text, icon }
    }

    /// Its size in a room `w` wide.
    pub fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let l = self.layout(Rect::new(0.0, 0.0, w, 0.0), theme);
        (l.whole.w, l.whole.h)
    }

    /// The pointer: at rest, hover fades it and a press anywhere on it
    /// edits; editing, a press on the confirm saves, the box takes its
    /// own, and a press anywhere else saves too.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        if self.input.disabled {
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let over = l.whole.contains(x, y);
                let mut r = self.input.handle_event(event, l.text, theme);
                if over != self.hovered {
                    self.hovered = over;
                    r = r.merge(EventResult::changed());
                }
                r
            }
            WidgetEvent::MouseDown { x, y } if !self.editing => {
                if l.whole.contains(x, y) {
                    self.edit();
                    EventResult::clicked()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if l.icon.contains(x, y) {
                    self.save();
                    EventResult::clicked()
                } else if l.text.contains(x, y) {
                    self.input.handle_event(event, l.text, theme)
                } else {
                    self.save();
                    EventResult::changed()
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    /// Where the input method's window goes: the box's caret.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        self.input.ime_area(self.layout(bounds, theme).text, theme)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let layer = LayerId::DEFAULT;
        let l = self.layout(bounds, theme);
        let e = &theme.form.edit;
        let faded = if self.input.disabled {
            theme.form.input.disabled_alpha
        } else if self.hovered && !self.editing {
            e.hover_alpha
        } else {
            1.0
        };
        let fade = |color: Color| with_alpha(color, color.0[3] * faded);
        if let Some(id) = self.icon_id() {
            let r = l.icon;
            c.bind_to_layer(layer, &id, [r.x, r.y, r.w, r.h]);
        }
        let (name, tint) = if self.editing {
            let lit = if self.input.is_empty() {
                e.confirm_empty
            } else {
                e.confirm
            };
            ("circle-check", lit)
        } else {
            ("pencil", e.pencil_color)
        };
        if self.editing {
            self.input.render(c, l.text, theme);
        } else {
            if let Some(id) = &self.sem {
                let r = l.whole;
                c.bind_to_layer(layer, id, [r.x, r.y, r.w, r.h]);
            }
            let shown = TextMeasurer::truncate_to_width(&self.value, &e.text, l.text.w);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &e.text, None),
                x: l.text.x,
                y: l.text.y + TextMeasurer::vertical_center(&e.text, l.text.h),
                color: fade(e.color),
            });
        }
        if let Some(node) = icons::icon_at(name, l.icon.w, fade(tint), l.icon.x, l.icon.y) {
            c.push(node);
        }
        if self.icon_focus {
            let r = l.icon;
            c.push(focus_ring(r, theme.shape.micro.min(r.h / 2.0), theme));
        }
    }

    /// Declares it as `id` (see the module docs). It only reads the
    /// widget: the owner runs `set_value` ([`NewField::set_value`]) and
    /// the icons' `activate` ([`NewField::edit`], [`NewField::save`]).
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::TextInput, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("text", self.value.as_str())
            .state("editing", self.editing)
            .constraints(Constraints {
                editable: true,
                ..Constraints::default()
            })
            .enabled(!self.input.disabled);
        node.action("set_value")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            })
            .reads(format!("{id}:text"));
        if let Some(icon) = self.icon_id() {
            let name = if self.editing { "save" } else { "edit" };
            let mut node = b.node(&icon, Role::Button, name);
            node.parent(parent.unwrap_or(id));
            node.enabled(!self.input.disabled);
            node.action("activate");
        }
    }
}
