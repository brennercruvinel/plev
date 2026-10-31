//! The date picker: a field that shows the chosen day, and a popover
//! with the calendar to choose it. The kit has none, so it is composed of
//! the parts the fronts built from it: the box, its label and message are
//! front A's [`Field`] (`theme.form`), the panel is front B's [`Popover`]
//! (`theme.overlay.popover`), and the month is the [`Calendar`]
//! (`theme.selection.calendar`). It owns no value of its own.
//!
//! A press on the box opens the panel below it, and a second press, a
//! press outside or Escape (the app's, [`DatePicker::close`]) closes it;
//! a day chosen in the panel fills the box and closes it. The box shows
//! the day as a person reads it, and is not typed into: the calendar is
//! the one way in.
//!
//! The panel is placed inside the `container` the app gives
//! ([`Popover::placement`]), so it answers the room it has, never the
//! window, and it draws after the field on the default layer: the app
//! gives it a container that nothing else draws over (the calendar has no
//! layer of its own to draw on yet).
//!
//! With a [`SemId`] ([`DatePicker::sem`]) the render binds the box to
//! `id` and the calendar to `<id>.calendar`, and [`DatePicker::project`]
//! declares the box as a button named by the label, described by the day
//! chosen (or its message), with its `expanded` and an `activate` that
//! opens and closes the panel; while open, the panel `<id>.panel`, a
//! non-modal dialog with a `close`, holds the calendar.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::form::field::{Field, FieldLayout};
use crate::widgets::overlay::popover::{Popover, render_panel};
use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::calendar::{Calendar, Date};

/// The box's icon: a calendar (lucide).
const ICON: &str = "calendar";

#[derive(Clone, Debug)]
pub struct DatePicker {
    /// The box, its label and message; its text is the day chosen.
    pub field: Field,
    pub calendar: Calendar,
    pub popover: Popover,
    disabled: bool,
    /// The keyboard focus on the box (a reader's, or Tab): its ring.
    focused: bool,
    sem: Option<SemId>,
}

/// Where a picker's parts sit: the field's parts, and the panel and the
/// calendar in it while open.
#[derive(Clone, Copy, Debug)]
pub struct DatePickerLayout {
    pub field: FieldLayout,
    pub panel: Rect,
    pub calendar: Rect,
}

impl DatePicker {
    /// A picker labeled `label` over `calendar`, its box showing the day
    /// the calendar has chosen, or `placeholder` while there is none.
    pub fn new(label: &str, placeholder: &str, calendar: Calendar) -> Self {
        let field = Field::new(placeholder).label(label).icon(ICON);
        let mut picker = Self {
            field,
            calendar,
            popover: Popover::new(),
            disabled: false,
            focused: false,
            sem: None,
        };
        picker.show_chosen();
        picker
    }

    pub fn helper(mut self, helper: &str) -> Self {
        self.field = self.field.helper(helper);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.field = self.field.disabled(disabled);
        self.calendar = self.calendar.disabled(disabled);
        self
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// The keyboard focus on the box, driven by the owning view; a
    /// disabled picker takes none.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// The box binds to `id`, the calendar to `<id>.calendar`.
    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(cal) = id.child("calendar") {
            self.calendar = self.calendar.sem(cal);
        }
        self.field = self.field.sem(id.clone());
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn chosen(&self) -> Option<Date> {
        self.calendar.selected
    }

    /// The box's text: the day chosen as a person reads it.
    fn show_chosen(&mut self) {
        let text = self.calendar.selected.map(Date::spoken).unwrap_or_default();
        self.field.input.set_text(&text);
    }

    /// Choose `date`, as a click on it in the panel does: the box shows
    /// it and the panel closes. Whether the choice moved.
    pub fn choose(&mut self, date: Date) -> bool {
        if !self.calendar.select(date) {
            return false;
        }
        self.show_chosen();
        self.popover.close();
        true
    }

    /// Open when closed, closed when open, as a press on the box does; a
    /// disabled picker stays closed. Whether it changed.
    pub fn toggle(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        self.popover.toggle();
        true
    }

    /// Ask it open (a forced state of a showcase card); a disabled picker
    /// stays closed.
    pub fn open(&mut self) -> bool {
        !self.disabled && self.popover.open()
    }

    pub fn close(&mut self) -> bool {
        self.popover.close()
    }

    pub fn is_open(&self) -> bool {
        self.popover.is_open()
    }

    /// The field's height at width `w`.
    pub fn field_height(&self, w: f32, theme: &Theme) -> f32 {
        self.field.height_for(w, theme)
    }

    /// The panel's size: the calendar within `max_w` less the panel's
    /// padding, and that padding around it.
    pub fn panel_size(&self, max_w: f32, theme: &Theme) -> (f32, f32) {
        let pad = theme.overlay.popover.pad;
        let room = (max_w - pad * 2.0).max(Calendar::min_width(theme));
        let (w, h) = self.calendar.size_within(room, theme);
        (w + pad * 2.0, h + pad * 2.0)
    }

    /// The field in `bounds`, and the panel placed by its box inside
    /// `container`.
    pub fn layout(&self, bounds: Rect, container: Rect, theme: &Theme) -> DatePickerLayout {
        let field = self.field.layout(bounds, theme);
        let size = self.panel_size(container.w, theme);
        let panel = self
            .popover
            .placement(field.input, size, container, theme)
            .rect;
        let calendar = panel.inset(theme.overlay.popover.pad);
        DatePickerLayout {
            field,
            panel,
            calendar,
        }
    }

    /// A press on the box opens and closes the panel; while open the
    /// calendar in it takes the pointer, a day chosen fills the box and
    /// closes it, and a press outside both closes it. The box's hover is
    /// the field's.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        container: Rect,
        theme: &Theme,
    ) -> EventResult {
        if self.disabled {
            return EventResult::IGNORED;
        }
        let l = self.layout(bounds, container, theme);
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseMove { .. }) {
            result = self.field.handle_event(event, bounds, theme);
        }
        if self.popover.is_open() {
            if self.popover.dismiss(event, l.field.input, l.panel) {
                return result.merge(EventResult::changed());
            }
            let before = self.calendar.selected;
            result = result.merge(self.calendar.handle_event(event, l.calendar, theme));
            if self.calendar.selected != before {
                self.show_chosen();
                self.popover.close();
                return result.merge(EventResult::clicked());
            }
        }
        if let WidgetEvent::MouseDown { x, y } = *event
            && l.field.input.contains(x, y)
        {
            self.popover.toggle();
            return result.merge(EventResult::clicked());
        }
        result
    }

    /// The field, and while open the panel and its calendar after it.
    pub fn render(&self, c: &mut Compositor, bounds: Rect, container: Rect, theme: &Theme) {
        self.field.render(c, bounds, theme);
        let l = self.layout(bounds, container, theme);
        if self.focused {
            let input = l.field.input;
            let radius = theme.form.input.radius.min(input.h / 2.0);
            c.push(focus_ring(input, radius, theme));
        }
        if !self.popover.is_open() {
            return;
        }
        render_panel(c, LayerId::DEFAULT, l.panel, theme);
        self.calendar.render(c, l.calendar, theme);
    }

    /// Declares the picker as `id`: the box a `Role::Button` named by the
    /// label and described by the day chosen (or the message), with an
    /// `activate`; the popover's `expanded` on it, and while open its
    /// panel `<id>.panel` with the calendar `<id>.calendar` in it.
    #[track_caller]
    pub fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: Option<&SemId>) {
        let name = self
            .field
            .label_text()
            .unwrap_or(self.field.input.input.placeholder.as_str())
            .to_owned();
        let mut node = b.node(id, Role::Button, name.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        let chosen = self.calendar.selected.map(Date::spoken);
        if let Some(text) = chosen.or_else(|| self.field.message().map(|m| m.text().to_owned())) {
            node.description(text);
        }
        node.state(
            "value",
            self.calendar.selected.map(Date::key).unwrap_or_default(),
        )
        .enabled(!self.disabled);
        node.action("activate");
        if self.popover.project(b, id, &name, parent).is_none() {
            return;
        }
        let (Some(cal), Some(panel)) = (self.calendar.sem_id(), Popover::panel_id(id)) else {
            return;
        };
        self.calendar.project(b, cal, name.as_str(), Some(&panel));
    }
}
