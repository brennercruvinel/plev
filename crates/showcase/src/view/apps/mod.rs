//! The Apps Demo group: complete small apps and the embedded ones (the
//! editor, the ide's git workspace), the template of the official app
//! pattern.

pub mod app;
pub mod builder;
pub mod dock;
pub mod editor;
pub mod git;

pub(crate) use super::{EditKey, group_label, panel, text, with_alpha};
