//! Icon Button page: the icon-only groups of the hoff core kit's
//! ButtonsPage (the ghost, large and small icon buttons, `:186`, `:219`,
//! `:252`, each live and disabled; the toolbar button at rest, disabled
//! and active, `:320`), in the Button page's mold and with its cards
//! ([`ButtonPage::icons`]). Projected as the Button page is, under its own
//! root: `icon_button_page.group[<g>].card[<c>]`, each button named by its
//! icon and described by its chip.

pub use super::button::ButtonPage as IconButtonPage;
