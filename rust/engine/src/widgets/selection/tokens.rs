//! Selection tokens: the token slot of task-104 front E, selection and
//! dates (#158), for the toggle, toggle group, button group, radio group,
//! calendar and date picker.
//!
//! `SelectionTokens::hoff_core` copies the kit as it is written;
//! [`SelectionTokens::derive`] serves every other theme from its palette, so
//! the widgets render under all of them (the shape of
//! `crate::widgets::buttons::tokens`). Empty until the front fills it.

use crate::styling::theme::tokens::ColorTokens;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionTokens {}

impl SelectionTokens {
    /// The hoff core kit, values as written in its class lists.
    pub fn hoff_core() -> Self {
        Self {}
    }

    /// The recipe of any other theme, from its palette.
    pub fn derive(_colors: &ColorTokens) -> Self {
        Self {}
    }
}
