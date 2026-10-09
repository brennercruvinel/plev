//! The Apps Demo group: complete small apps and the embedded ones (the
//! editor, the ide's git workspace), the template of the official app
//! pattern, a surface materialized from verified plev IR over the todo's
//! capabilities (the second pattern), the silences app of the video
//! editor scenario (the fixed screen of task-69 phase 4a), the files
//! page, the file dialog and the file drop as capabilities (task-76), and
//! the capture page, the camera and the microphone a person turns on
//! (task-76 item 10).

pub mod builder;
pub mod capture;
pub mod dock;
pub mod editor;
pub mod files;
pub mod git;
pub mod silences;
pub mod surface;
pub mod todo;

pub(crate) use super::{EditKey, group_label, panel, text, with_alpha};
