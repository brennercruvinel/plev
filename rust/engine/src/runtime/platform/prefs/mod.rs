//! The platform's readers of the person's preferences (ADR 0086): each
//! asks its system once, when a window first shows and when it takes the
//! focus again, and fills a field of [`Prefs`]; a reader that cannot
//! answer leaves its field at the default. `Theme::adapted` turns the
//! value into tokens.
//!
//! Reduce motion and more contrast are still read where task-104k put
//! them (`control::accessibility::preference`), and reach the thread's
//! motion and contrast flags; they join `Prefs` when their theme step
//! lands (front M's `Contrast`).

pub mod text_scale;

use crate::styling::theme::{Prefs, TextScale};

/// What the system says of every preference this module reads.
pub fn read() -> Prefs {
    let mut prefs = Prefs::default();
    if let Some(factor) = text_scale::system_text_scale() {
        prefs.text_scale = TextScale::new(factor);
    }
    prefs
}
