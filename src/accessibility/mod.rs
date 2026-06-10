//! Accessibility integration via AccessKit.

pub mod focus;
pub(crate) mod id_map;
pub mod state;

#[cfg(test)]
mod tests;

pub use focus::{FocusDirection, FocusGraph};
pub use state::AccessibilityState;
