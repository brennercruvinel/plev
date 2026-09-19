//! Editable text input: TextBuffer for editing, TextInput for rendering.
//! Cursor <-> pixel mapping is done by `text::TextMeasurer` (real shaping).

mod buffer;
mod component;

pub use buffer::TextBuffer;
pub use component::TextInput;

#[cfg(test)]
mod tests;
