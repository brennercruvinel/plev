//! What a card of a form page shows: one widget of the form family, with
//! what the page asks of each the same way ([`CardWidget`]: its size in
//! the card, its draw, its pointer, its projection, its parts and their
//! ids). A widget plugs in by implementing the trait, here for the field,
//! the label and the input group, in its own page's file for the others.
//! The text routes reach the widget's text field through
//! [`CardWidget::text_mut`].

#[cfg(test)]
use std::any::Any;

use accord::{SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::form::field::{Field, FieldPart};
use engine::widgets::form::input_group::InputGroup;
use engine::widgets::form::label::Label;
use engine::widgets::form::{EditKey, TextField};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

/// A field is the kit page's `w-60` wide, narrower when the card is.
pub(crate) const FIELD_W: f32 = 240.0;
/// An input group is the kit's Search, `w-54`.
pub(crate) const GROUP_W: f32 = 216.0;
/// A select is the kit's DropdownPage's, `w-44`.
pub(crate) const SELECT_W: f32 = 176.0;

/// A part of a widget a request or a reader's focus names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    /// The text box.
    Box,
    /// A password's eye.
    Reveal,
    /// A field's link.
    Link,
    /// An input group's icon button.
    Action,
    /// An open list's option, by its index in the options.
    Option(usize),
}

/// A widget of a form page's card. Only the size, the draw and the
/// projection are its own; the rest has the answer of a widget without
/// a text box or parts.
pub(crate) trait CardWidget: 'static {
    /// The widget with the id its draw binds to.
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget>;

    /// Its size with `room` across.
    fn size(&self, room: f32, theme: &Theme) -> (f32, f32);

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme);

    /// Declares the widget under `parent`, as the comps projects it.
    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId);

    /// Draws what floats over the page (an open list) on `layer`.
    fn render_overlay(&self, _c: &mut Compositor, _layer: LayerId, _rect: Rect, _theme: &Theme) {}

    /// Whether `(x, y)` is on what floats over the page: the pointer
    /// there is the widget's alone.
    fn over_overlay(&self, _x: f32, _y: f32, _rect: Rect, _theme: &Theme) -> bool {
        false
    }

    /// The caret to the widget's box, or off it: the text field's by
    /// default.
    fn set_focused(&mut self, focused: bool) {
        if let Some(t) = self.text_mut() {
            t.set_focused(focused);
        }
    }

    /// The part `id` names: the box, the eye, the link or the icon button
    /// by default.
    fn part(&self, id: &SemId) -> Option<Part> {
        [Part::Box, Part::Reveal, Part::Link, Part::Action]
            .into_iter()
            .find(|p| self.part_id(*p).as_ref() == Some(id))
    }

    fn handle_event(&mut self, _event: &WidgetEvent, _rect: Rect, _theme: &Theme) -> EventResult {
        EventResult::IGNORED
    }

    /// The text field the keys go to.
    fn text(&self) -> Option<&TextField> {
        None
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        None
    }

    /// An editing key for the widget's focused text: the text field's
    /// own by default.
    fn edit(&mut self, key: EditKey) -> bool {
        self.text_mut().is_some_and(|t| t.edit(key))
    }

    /// Enter on the focused widget. `false`, the default: the page's
    /// focus walk presses.
    fn enter(&mut self) -> bool {
        false
    }

    /// Up or Down on the focused widget. `false`, the default: the
    /// page's focus walk moves.
    fn nav(&mut self, _down: bool) -> bool {
        false
    }

    /// The id of `part` of the widget.
    fn part_id(&self, _part: Part) -> Option<SemId> {
        None
    }

    /// The keyboard focus on a part other than the box (whose focus is
    /// the caret), or off them.
    fn set_part_focus(&mut self, _part: Option<Part>) {}

    fn part_focus(&self) -> Option<Part> {
        None
    }

    /// Whether `(x, y)` is on the text box, laid out in `rect`.
    fn on_box(&self, _x: f32, _y: f32, _rect: Rect, _theme: &Theme) -> bool {
        false
    }

    /// Where the input method's window goes for the focused box.
    fn ime_area(&self, _rect: Rect, _theme: &Theme) -> Option<Rect> {
        None
    }

    /// Runs `action` on `part`; `false` when the widget has no such
    /// thing. The box's `set_value` sets the text whole, refused where
    /// the field does not take it ([`TextField::set_value`]).
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Box, "set_value", Some(Value::Text(text))) => {
                self.text_mut().is_some_and(|t| t.set_value(text))
            }
            _ => false,
        }
    }

    /// The widget as itself, for a test to read what only it has.
    #[cfg(test)]
    fn as_any(&self) -> &dyn Any;
}

/// A card's widget.
pub(crate) struct Spec(Box<dyn CardWidget>);

impl<W: CardWidget> From<W> for Spec {
    fn from(w: W) -> Self {
        Spec(Box::new(w))
    }
}

impl std::ops::Deref for Spec {
    type Target = dyn CardWidget;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl std::ops::DerefMut for Spec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl Spec {
    pub(crate) fn sem(self, id: SemId) -> Self {
        Spec(self.0.with_sem(id))
    }

    /// The widget as a `W`, for what only it has (a field's layout, its
    /// eye).
    #[cfg(test)]
    pub(crate) fn get<W: CardWidget>(&self) -> Option<&W> {
        self.0.as_any().downcast_ref()
    }

    /// The field, when the card shows one.
    #[cfg(test)]
    pub(crate) fn field(&self) -> Option<&Field> {
        self.get()
    }

    /// Its size in a card `card_w` wide with `side` kept clear on either
    /// side.
    pub(crate) fn size_in(&self, card_w: f32, side: f32, theme: &Theme) -> (f32, f32) {
        self.0.size((card_w - side * 2.0).max(0.0), theme)
    }
}

impl CardWidget for Field {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        let w = FIELD_W.min(room);
        (w, self.height_for(w, theme))
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        Field::render(self, c, rect, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            Field::project(self, b, id, Some(parent));
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        Field::handle_event(self, event, rect, theme)
    }

    fn text(&self) -> Option<&TextField> {
        Some(&self.input)
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        Some(&mut self.input)
    }

    fn part_id(&self, part: Part) -> Option<SemId> {
        match part {
            Part::Box => self.sem_id().cloned(),
            Part::Reveal => self.reveal_id(),
            Part::Link => self.link_id(),
            Part::Action | Part::Option(_) => None,
        }
    }

    fn set_part_focus(&mut self, part: Option<Part>) {
        Field::set_part_focus(
            self,
            match part {
                Some(Part::Reveal) => Some(FieldPart::Reveal),
                Some(Part::Link) => Some(FieldPart::Link),
                _ => None,
            },
        );
    }

    fn part_focus(&self) -> Option<Part> {
        match Field::part_focus(self) {
            Some(FieldPart::Reveal) => Some(Part::Reveal),
            Some(FieldPart::Link) => Some(Part::Link),
            _ => None,
        }
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        self.part_at(x, y, rect, theme) == Some(FieldPart::Input)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        Some(self.input.ime_area(self.layout(rect, theme).input, theme))
    }

    /// The eye's `toggle` reveals; a link's `activate` does what a click
    /// does (nothing more).
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Box, "set_value", Some(Value::Text(text))) => self.input.set_value(text),
            (Part::Reveal, "toggle", None) => self.toggle_reveal(),
            (Part::Link, "activate", None) => true,
            _ => false,
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CardWidget for Label {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    /// Wrapped to the room.
    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        Label::size(self, Some(room), theme)
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        Label::render(self, c, rect, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            Label::project(self, b, id, Some(parent));
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CardWidget for InputGroup {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        (GROUP_W.min(room), self.height(theme))
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        InputGroup::render(self, c, rect, theme);
    }

    /// Named by its placeholder.
    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            let name = self.input.input.placeholder.clone();
            InputGroup::project(self, b, id, name, Some(parent));
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        InputGroup::handle_event(self, event, rect, theme)
    }

    fn text(&self) -> Option<&TextField> {
        Some(&self.input)
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        Some(&mut self.input)
    }

    fn part_id(&self, part: Part) -> Option<SemId> {
        match part {
            Part::Box => self.sem_id().cloned(),
            Part::Action => self.action_id(),
            _ => None,
        }
    }

    fn set_part_focus(&mut self, part: Option<Part>) {
        self.set_action_focus(part == Some(Part::Action));
    }

    fn part_focus(&self) -> Option<Part> {
        self.has_action_focus().then_some(Part::Action)
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        self.layout(rect, theme).input.contains(x, y) && !self.on_action(x, y, rect, theme)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        Some(InputGroup::ime_area(self, rect, theme))
    }

    /// The icon button's `activate` does what a click does.
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Box, "set_value", Some(Value::Text(text))) => self.input.set_value(text),
            (Part::Action, "activate", None) => self.action().is_some(),
            _ => false,
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}
