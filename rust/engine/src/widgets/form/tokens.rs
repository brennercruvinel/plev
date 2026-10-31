//! Form tokens: the token slot of task-104 front A, fields (#154), for
//! the field, label, input group, input OTP, textarea, combobox and
//! command.
//!
//! `FormTokens::hoff_core` copies the kit as it is written;
//! [`FormTokens::derive`] serves every other theme from the theme already
//! built (its palette and its glass, which the hoff themes write by hand),
//! so the widgets render under all of them (the shape of
//! `crate::widgets::buttons::tokens`). Empty until the front fills it.

use crate::styling::theme::Theme;

#[derive(Clone, Debug, PartialEq)]
pub struct FormTokens {}

impl FormTokens {
    /// The hoff core kit, values as written in its class lists.
    pub fn hoff_core() -> Self {
        Self {}
    }

    /// The recipe of any other theme, from the theme built without it:
    /// a constructor fills the slot last (`theme.form = FormTokens::derive(&theme)`).
    pub fn derive(_theme: &Theme) -> Self {
        Self {}
    }
}
