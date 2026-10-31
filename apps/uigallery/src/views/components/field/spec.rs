//! What a card of a form page shows: one widget of the form family, with
//! what the page asks of each the same way (its size in the card, its
//! draw, its pointer, its projection, its ids). The text routes reach
//! the widget's text field through [`Spec::field_mut`].

use accord::{SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::field::{Field, FieldPart};
use engine::widgets::form::label::Label;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

/// A field is the kit page's `w-60` wide, narrower when the card is.
pub(crate) const FIELD_W: f32 = 240.0;

pub(crate) enum Spec {
    Field(Box<Field>),
    Label(Label),
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

impl Spec {
    /// The widget with the id its draw binds to.
    pub(crate) fn sem(self, id: SemId) -> Self {
        match self {
            Spec::Field(f) => Spec::Field(Box::new((*f).sem(id))),
            Spec::Label(l) => Spec::Label(l.sem(id)),
        }
    }

    /// The text field it holds, the one the keys go to.
    pub(crate) fn field(&self) -> Option<&Field> {
        match self {
            Spec::Field(f) => Some(f.as_ref()),
            Spec::Label(_) => None,
        }
    }

    pub(crate) fn field_mut(&mut self) -> Option<&mut Field> {
        match self {
            Spec::Field(f) => Some(f.as_mut()),
            Spec::Label(_) => None,
        }
    }

    /// Its size in a card `card_w` wide with `side` kept clear on either
    /// side: a field `w-60` wide at most, a label wrapped to the room.
    pub(crate) fn size(&self, card_w: f32, side: f32, theme: &Theme) -> (f32, f32) {
        let room = (card_w - side * 2.0).max(0.0);
        match self {
            Spec::Field(f) => {
                let w = FIELD_W.min(room);
                (w, f.height_for(w, theme))
            }
            Spec::Label(l) => l.size(Some(room), theme),
        }
    }

    pub(crate) fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        match self {
            Spec::Field(f) => f.render(c, rect, theme),
            Spec::Label(l) => l.render(c, rect, theme),
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
        }
    }

    /// The part of the widget `id` names: a field's box, eye or link.
    pub(crate) fn part(&self, id: &SemId) -> Option<FieldPart> {
        let f = self.field()?;
        if f.sem_id() == Some(id) {
            Some(FieldPart::Input)
        } else if f.reveal_id().as_ref() == Some(id) {
            Some(FieldPart::Reveal)
        } else if f.link_id().as_ref() == Some(id) {
            Some(FieldPart::Link)
        } else {
            None
        }
    }
}
