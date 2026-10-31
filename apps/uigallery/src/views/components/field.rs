//! Field page, the hoff core kit's InputsPage for its Field: groups with a
//! title and a bottom rule, cards in rows, one state per card behind a
//! chip that names it (the kit's chips for Email and Password; Disabled
//! and Content are the plev's: a disabled field, and the content a field
//! must survive, a long value, a long label and message, a right-to-left
//! value). The widget is `engine::widgets::form::field::Field`; every
//! visual value comes from the theme, and the page's own numbers are the
//! kit page's (`h-50` cards, `w-60` fields).
//!
//! The Username group is the kit's NewField, a value edited in place
//! (`engine::widgets::form::new_field::NewField`), as the kit's
//! InputsPage shows it after the passwords.
//!
//! The page is a [`FormPage`] (`form.rs`): its focus, keys, projection
//! and requests are the form pages', its ids under `field_page`.

mod form;
pub(crate) mod spec;

pub use form::FormPage;
pub(crate) use form::{Group, card};

#[cfg(test)]
use std::any::Any;

use accord::{SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::form::field::Field;
use engine::widgets::form::new_field::NewField;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use spec::{CardWidget, Part};

/// The live card's chip, as the kit names it.
const LIVE: &str = "Default, Click on, Focus, Hover, Typing";

fn email() -> Field {
    Field::new("Enter email").label("Email")
}

fn password() -> Field {
    Field::new("Enter password")
        .label("Password")
        .link("Forgot password?")
        .password(true)
}

/// The Field tab.
pub fn page() -> FormPage {
    let groups = vec![
        Group {
            title: "Email",
            columns: 3,
            cards: vec![
                card(LIVE, email()),
                card(
                    "Typed",
                    email().with_text("sophie@hoff.dev").validated(true),
                ),
                card(
                    "Message, With icon",
                    email().icon("scan").error("This email is already taken."),
                ),
            ],
        },
        Group {
            title: "Password",
            columns: 3,
            cards: vec![
                card(LIVE, password()),
                card(
                    "Typed",
                    password().with_text("correct horse").validated(true),
                ),
                card("Message", password().error("Confirm password is incorrect")),
            ],
        },
        Group {
            title: "Username",
            columns: 1,
            cards: vec![card(
                "Default, Hover, Edit, Confirm",
                NewField::new("sophie", "Enter new username"),
            )],
        },
        Group {
            title: "Disabled",
            columns: 2,
            cards: vec![
                card("Disabled", email().disabled(true)),
                card(
                    "Disabled, Typed",
                    email().with_text("sophie@hoff.dev").disabled(true),
                ),
            ],
        },
        Group {
            title: "Content",
            columns: 3,
            cards: vec![
                card(
                    "Long value",
                    email().with_text("sophie.marguerite.delacroix-oyelaran@research.hoff.example"),
                ),
                card(
                    "Long label, Long message",
                    Field::new("Geben Sie Ihre geschäftliche E-Mail-Adresse ein")
                        .label("Geschäftliche E-Mail-Adresse für Rechnungen und Belege")
                        .helper(
                            "An diese Adresse senden wir Rechnungen, Belege und \
                             Hinweise zu Ihrem Arbeitsbereich.",
                        ),
                ),
                card(
                    "Right to left",
                    Field::new("הזינו כתובת דוא״ל")
                        .label("דוא״ל")
                        .with_text("שלום עולם"),
                ),
            ],
        },
    ];
    FormPage::new("field_page", groups)
}

impl CardWidget for NewField {
    fn with_sem(self: Box<Self>, id: SemId) -> Box<dyn CardWidget> {
        Box::new((*self).sem(id))
    }

    fn size(&self, room: f32, theme: &Theme) -> (f32, f32) {
        NewField::size(self, room, theme)
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        NewField::render(self, c, rect, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        if let Some(id) = self.sem_id() {
            NewField::project(self, b, id, "Username", Some(parent));
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        NewField::handle_event(self, event, rect, theme)
    }

    fn text(&self) -> Option<&TextField> {
        Some(&self.input)
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        Some(&mut self.input)
    }

    /// The focus edits (the box out); its loss saves.
    fn set_focused(&mut self, focused: bool) {
        if focused {
            self.edit();
        } else {
            self.save();
        }
    }

    /// Enter saves.
    fn enter(&mut self) -> bool {
        self.save()
    }

    /// Escape keeps the value as it was.
    fn escape(&mut self) -> bool {
        self.cancel()
    }

    fn part_id(&self, part: Part) -> Option<SemId> {
        match part {
            Part::Box => self.sem_id().cloned(),
            Part::Action => self.icon_id(),
            _ => None,
        }
    }

    fn set_part_focus(&mut self, part: Option<Part>) {
        self.set_icon_focus(part == Some(Part::Action));
    }

    fn part_focus(&self) -> Option<Part> {
        self.has_icon_focus().then_some(Part::Action)
    }

    fn on_box(&self, x: f32, y: f32, rect: Rect, theme: &Theme) -> bool {
        self.is_editing() && self.layout(rect, theme).text.contains(x, y)
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        Some(NewField::ime_area(self, rect, theme))
    }

    /// `set_value` saves a value as the confirm does; the pencil's
    /// `activate` edits, the confirm's saves.
    fn run(&mut self, part: Part, action: &str, param: Option<&Value>) -> bool {
        match (part, action, param) {
            (Part::Box, "set_value", Some(Value::Text(text))) => self.set_value(text),
            (Part::Action, "activate", None) if self.is_editing() => self.save(),
            (Part::Action, "activate", None) => self.edit(),
            _ => false,
        }
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests;
