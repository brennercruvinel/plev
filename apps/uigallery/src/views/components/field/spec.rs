//! What a card of a form page shows: one widget of the form family, with
//! what the page asks of each the same way (its size in the card, its
//! draw, its pointer, its projection, its parts and their ids). The text
//! routes reach the widget's text field through [`Spec::text_mut`].

use accord::{SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::form::field::{Field, FieldPart};
use engine::widgets::form::input_group::InputGroup;
use engine::widgets::form::label::Label;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

/// A field is the kit page's `w-60` wide, narrower when the card is.
pub(crate) const FIELD_W: f32 = 240.0;
/// An input group is the kit's Search, `w-54`.
pub(crate) const GROUP_W: f32 = 216.0;

pub(crate) enum Spec {
    Field(Box<Field>),
    Label(Label),
    Group(Box<InputGroup>),
}

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
}

impl From<Field> for Spec {
    fn from(f: Field) -> Self {
        Spec::Field(Box::new(f))
    }
}

impl From<Label> for Spec {
    fn from(l: Label) -> Self {
        Spec::Label(l)
    }
}

impl From<InputGroup> for Spec {
    fn from(g: InputGroup) -> Self {
        Spec::Group(Box::new(g))
    }
}

impl Spec {
    /// The widget with the id its draw binds to.
    pub(crate) fn sem(self, id: SemId) -> Self {
        match self {
            Spec::Field(f) => Spec::Field(Box::new((*f).sem(id))),
            Spec::Label(l) => Spec::Label(l.sem(id)),
            Spec::Group(g) => Spec::Group(Box::new((*g).sem(id))),
        }
    }

    /// The field, for what only a field has (its layout, its eye).
    #[cfg(test)]
    pub(crate) fn field(&self) -> Option<&Field> {
        match self {
            Spec::Field(f) => Some(f.as_ref()),
            _ => None,
        }
    }

    /// The text field the keys go to.
    pub(crate) fn text(&self) -> Option<&TextField> {
        match self {
            Spec::Field(f) => Some(&f.input),
            Spec::Group(g) => Some(&g.input),
            Spec::Label(_) => None,
        }
    }

    pub(crate) fn text_mut(&mut self) -> Option<&mut TextField> {
        match self {
            Spec::Field(f) => Some(&mut f.input),
            Spec::Group(g) => Some(&mut g.input),
            Spec::Label(_) => None,
        }
    }

    /// Its size in a card `card_w` wide with `side` kept clear on either
    /// side: a field `w-60` and a group `w-54` wide at most, a label
    /// wrapped to the room.
    pub(crate) fn size(&self, card_w: f32, side: f32, theme: &Theme) -> (f32, f32) {
        let room = (card_w - side * 2.0).max(0.0);
        match self {
            Spec::Field(f) => {
                let w = FIELD_W.min(room);
                (w, f.height_for(w, theme))
            }
            Spec::Label(l) => l.size(Some(room), theme),
            Spec::Group(g) => (GROUP_W.min(room), g.height(theme)),
        }
    }

    pub(crate) fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        match self {
            Spec::Field(f) => f.render(c, rect, theme),
            Spec::Label(l) => l.render(c, rect, theme),
            Spec::Group(g) => g.render(c, rect, theme),
        }
    }

    pub(crate) fn handle_event(
        &mut self,
        event: &WidgetEvent,
        rect: Rect,
        theme: &Theme,
    ) -> EventResult {
        match self {
            Spec::Field(f) => f.handle_event(event, rect, theme),
            Spec::Label(_) => EventResult::IGNORED,
            Spec::Group(g) => g.handle_event(event, rect, theme),
        }
    }

    /// Declares the widget under `parent`, as the comps projects it.
    pub(crate) fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        match self {
            Spec::Field(f) => {
                if let Some(id) = f.sem_id() {
                    f.project(b, id, Some(parent));
                }
            }
            Spec::Label(l) => {
                if let Some(id) = l.sem_id() {
                    l.project(b, id, Some(parent));
                }
            }
            Spec::Group(g) => {
                if let Some(id) = g.sem_id() {
                    let name = g.input.input.placeholder.clone();
                    g.project(b, id, name, Some(parent));
                }
            }
        }
    }

    /// The id of `part` of the widget.
    pub(crate) fn part_id(&self, part: Part) -> Option<SemId> {
        match (self, part) {
            (Spec::Field(f), Part::Box) => f.sem_id().cloned(),
            (Spec::Field(f), Part::Reveal) => f.reveal_id(),
            (Spec::Field(f), Part::Link) => f.link_id(),
            (Spec::Group(g), Part::Box) => g.sem_id().cloned(),
            (Spec::Group(g), Part::Action) => g.action_id(),
            _ => None,
        }
    }

    /// The part of the widget `id` names.
    pub(crate) fn part(&self, id: &SemId) -> Option<Part> {
        [Part::Box, Part::Reveal, Part::Link, Part::Action]
            .into_iter()
            .find(|p| self.part_id(*p).as_ref() == Some(id))
    }

    /// The keyboard focus on a part other than the box (whose focus is the
    /// caret), or off them.
    pub(crate) fn set_part_focus(&mut self, part: Option<Part>) {
        match self {
            Spec::Field(f) => f.set_part_focus(match part {
                Some(Part::Reveal) => Some(FieldPart::Reveal),
                Some(Part::Link) => Some(FieldPart::Link),
                _ => None,
            }),
            Spec::Group(g) => g.set_action_focus(part == Some(Part::Action)),
            Spec::Label(_) => {}
        }
    }

    pub(crate) fn part_focus(&self) -> Option<Part> {
        match self {
            Spec::Field(f) => match f.part_focus() {
                Some(FieldPart::Reveal) => Some(Part::Reveal),
                Some(FieldPart::Link) => Some(Part::Link),
                _ => None,
            },
            Spec::Group(g) => g.has_action_focus().then_some(Part::Action),
            Spec::Label(_) => None,
        }
    }

    /// Whether `(x, y)` is on the text box, laid out in `rect`.
    pub(crate) fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        match self {
            Spec::Field(f) => f.part_at(x, y, rect, theme) == Some(FieldPart::Input),
            Spec::Group(g) => {
                g.layout(rect, theme).input.contains(x, y) && !g.on_action(x, y, rect, theme)
            }
            Spec::Label(_) => false,
        }
    }

    /// Where the input method's window goes for the focused box.
    pub(crate) fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        match self {
            Spec::Field(f) => Some(f.input.ime_area(f.layout(rect, theme).input, theme)),
            Spec::Group(g) => Some(g.ime_area(rect, theme)),
            Spec::Label(_) => None,
        }
    }

    /// Runs `action` on `part`: the box's `set_value` types, the eye's
    /// `toggle` reveals, a link's or an icon button's `activate` presses.
    /// `false` when the widget has no such thing.
    pub(crate) fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Box, "set_value", Some(Value::Text(text))) => {
                self.text_mut().map(|t| t.set_text(text)).is_some()
            }
            (Part::Reveal, "toggle", None) => match self {
                Spec::Field(f) => f.toggle_reveal(),
                _ => false,
            },
            (Part::Link, "activate", None) => matches!(self, Spec::Field(_)),
            (Part::Action, "activate", None) => matches!(self, Spec::Group(_)),
            _ => false,
        }
    }
}
