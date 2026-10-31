//! Field page, the hoff core kit's InputsPage for its Field: groups with a
//! title and a bottom rule, cards in rows, one state per card behind a
//! chip that names it (the kit's chips for Email and Password; Disabled
//! and Content are the plev's: a disabled field, and the content a field
//! must survive, a long value, a long label and message, a right-to-left
//! value). The widget is `engine::widgets::form::field::Field`; every
//! visual value comes from the theme, and the page's own numbers are the
//! kit page's (`h-50` cards, `w-60` fields).
//!
//! The page is a [`FormPage`] (`form.rs`): its focus, keys, projection
//! and requests are the form pages', its ids under `field_page`.

mod form;
mod spec;

pub use form::FormPage;
pub(crate) use form::{Group, card};

use engine::widgets::form::field::Field;

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

#[cfg(test)]
mod tests;
