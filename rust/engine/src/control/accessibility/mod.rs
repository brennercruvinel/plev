//! Accessibility as a projection of the semantic contract
//! (docs/adr/interface/0038-accessibility-projects-the-semantic-snapshot.md).
//!
//! The tree assistive technology reads is built from the same
//! `sem::Snapshot` agents and tests read (`tree`), placed by the
//! compositor's binding table, with node ids that follow each node's
//! `SemId` across frames (`ids`); the actions an item's own node does not
//! reach are presented inside it as controls (`present`), since no
//! adapter implements custom actions. What a screen reader asks comes back as
//! a `sem::Request` with `Actor::Assistive`, for the app's one dispatcher
//! (`action`). Keyboard focus walks the same snapshot (`focus`). The
//! system's "reduce motion" is read here (`motion`) and ends every
//! transition at once (`animation::set_reduced_motion`). An app shell
//! wires all of it through one `Bridge` (native only).

pub mod action;
#[cfg(not(target_arch = "wasm32"))]
pub mod bridge;
pub mod focus;
pub mod ids;
pub mod present;
pub mod reduced_motion;
pub mod role;
pub mod tree;

/// The AccessKit types a shell or a test names (an `ActionRequest`, a
/// `TreeUpdate`), so an app does not pin its own copy of the crate.
pub use accesskit;
pub use action::{Assist, Unmapped, primary, setter, translate};
#[cfg(not(target_arch = "wasm32"))]
pub use bridge::{Bridge, BridgeError, Wake};
pub use focus::Direction;
pub use ids::{NodeIds, ROOT};
pub use present::presented;
pub use tree::{Chrome, Shown, build};
