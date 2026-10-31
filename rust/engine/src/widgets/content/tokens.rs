//! Disclosure tokens: the token slot of task-104 front D, disclosure and
//! display (#157), for the accordion, collapsible, item, kbd, aspect
//! ratio, carousel, pagination and table.
//!
//! `DisclosureTokens::hoff_core` copies the kit as it is written;
//! [`DisclosureTokens::derive`] serves every other theme from its palette, so
//! the widgets render under all of them (the shape of
//! `crate::widgets::buttons::tokens`). Empty until the front fills it.

use crate::styling::theme::tokens::ColorTokens;

#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureTokens {}

impl DisclosureTokens {
    /// The hoff core kit, values as written in its class lists.
    pub fn hoff_core() -> Self {
        Self {}
    }

    /// The recipe of any other theme, from its palette.
    pub fn derive(_colors: &ColorTokens) -> Self {
        Self {}
    }
}
