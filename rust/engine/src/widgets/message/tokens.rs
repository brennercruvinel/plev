//! Message tokens: the token slot of task-104 front C, conversation and
//! feedback (#156), for the message, bubble, message scroller, alert,
//! attachment, marker and questionnaire.
//!
//! `MessageTokens::hoff_core` copies the kit as it is written;
//! [`MessageTokens::derive`] serves every other theme from its palette, so
//! the widgets render under all of them (the shape of
//! `crate::widgets::buttons::tokens`). Empty until the front fills it.

use crate::styling::theme::tokens::ColorTokens;

#[derive(Clone, Debug, PartialEq)]
pub struct MessageTokens {}

impl MessageTokens {
    /// The hoff core kit, values as written in its class lists.
    pub fn hoff_core() -> Self {
        Self {}
    }

    /// The recipe of any other theme, from its palette.
    pub fn derive(_colors: &ColorTokens) -> Self {
        Self {}
    }
}
