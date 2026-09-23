//! Accessibility integration via AccessKit.

pub mod action;
pub mod focus;
pub(crate) mod id_map;
pub mod ids;
pub mod role;
pub mod state;
pub mod tree;

#[cfg(test)]
mod tests;

pub use focus::{FocusDirection, FocusGraph};
pub use ids::{NodeIds, ROOT};
pub use state::AccessibilityState;
pub use tree::{Chrome, build};
