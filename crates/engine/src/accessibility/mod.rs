//! Accessibility integration via AccessKit.

pub mod focus;
pub(crate) mod id_map;
pub mod ids;
pub mod state;

#[cfg(test)]
mod tests;

pub use focus::{FocusDirection, FocusGraph};
pub use ids::{NodeIds, ROOT};
pub use state::AccessibilityState;
