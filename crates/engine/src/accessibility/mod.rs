//! Accessibility integration via AccessKit.

pub mod action;
#[cfg(not(target_arch = "wasm32"))]
pub mod bridge;
pub mod focus;
pub(crate) mod id_map;
pub mod ids;
pub mod role;
pub mod state;
pub mod tree;

#[cfg(test)]
mod tests;

/// The AccessKit types a shell or a test names (an `ActionRequest`, a
/// `TreeUpdate`), so an app does not pin its own copy of the crate.
pub use accesskit;
pub use action::{Assist, Unmapped, primary, translate};
#[cfg(not(target_arch = "wasm32"))]
pub use bridge::{Bridge, BridgeError, Wake};
pub use focus::Direction;
pub use focus::{FocusDirection, FocusGraph};
pub use ids::{NodeIds, ROOT};
pub use state::AccessibilityState;
pub use tree::{Chrome, Shown, build};
