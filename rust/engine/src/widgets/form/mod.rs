//! Form controls: the things a user sets.

mod checkbox;
mod select;
mod slider;
mod switch;
mod text_field;
pub mod text_input;

// task-104 front A, fields (#154): one module per widget, and the front's
// token slot (`Theme::form`).
pub mod combobox;
pub mod command;
pub mod field;
pub mod input_group;
pub mod label;
pub mod new_field;
pub mod otp;
pub mod textarea;
pub mod tokens;

pub use checkbox::Checkbox;
pub use select::Select;
pub use slider::Slider;
pub use switch::Switch;
pub use text_field::{ClipKey, EditKey, TextField};
pub use text_input::TextInput;
