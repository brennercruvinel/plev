//! Editable text input: TextBuffer for editing, TextInput for rendering.
//! Cursor <-> pixel mapping is done by `text::TextMeasurer` (real shaping),
//! read through `visual::VisualLine` so the caret, the arrow keys and the
//! selection follow the screen in mixed-direction text. `view::Shown`
//! masks a password and `view::LineFit` places a line wider than its box.

mod buffer;
mod component;
pub(crate) mod view;
pub(crate) mod visual;

pub use buffer::TextBuffer;
pub use component::{CURSOR_BLINK_INTERVAL, TextInput, single_line};
pub use view::MASK;

#[cfg(test)]
mod tests;
