//! The Apps Demo group: complete small apps and the embedded ones (the
//! editor, the ide's git workspace), the template of the official app
//! pattern, and a surface materialized from verified plev IR over the
//! todo's capabilities (the second pattern).

pub mod app;
pub mod builder;
pub mod dock;
pub mod editor;
pub mod git;
pub mod surface_page;

pub(crate) use super::{EditKey, group_label, panel, text, with_alpha};
