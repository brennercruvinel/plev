//! Overlay tokens: the token slot of task-104 front B, overlays (#155),
//! for the popover, hover card, dropdown menu, alert dialog, menubar,
//! sheet and drawer.
//!
//! `OverlayTokens::hoff_core` copies the kit as it is written;
//! [`OverlayTokens::derive`] serves every other theme from the theme already
//! built (its palette and its glass, which the hoff themes write by hand),
//! so the widgets render under all of them (the shape of
//! `crate::widgets::buttons::tokens`). Empty until the front fills it.

use crate::styling::theme::Theme;

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayTokens {}

impl OverlayTokens {
    /// The hoff core kit, values as written in its class lists.
    pub fn hoff_core() -> Self {
        Self {}
    }

    /// The recipe of any other theme, from the theme built without it
    /// (`Theme::derive_front_slots` fills the slots last).
    pub fn derive(_theme: &Theme) -> Self {
        Self {}
    }
}
