//! Editable text input: TextBuffer for editing, TextInput for rendering.

mod buffer;
mod component;
mod cursor_map;

pub use buffer::TextBuffer;
pub use component::TextInput;
pub use cursor_map::{cursor_to_x, x_to_cursor};

#[cfg(test)]
mod tests;
