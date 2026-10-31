//! Selection: the things a user picks, one or several, and dates
//! (task-104 front E, selection and dates, #158): one module per widget,
//! and the front's token slot (`Theme::selection`).

pub mod button_group;
pub mod calendar;
pub mod date_picker;
pub mod radio_group;
pub mod toggle;
pub mod toggle_group;
pub mod tokens;

pub use button_group::{ButtonGroup, GroupItem};
pub use toggle::Toggle;
pub use toggle_group::{SegmentSize, ToggleGroup};
