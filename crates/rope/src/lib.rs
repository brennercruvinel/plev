//! Core text editing model: rope buffer, multi-cursor selections,
//! transactional edits and undo history.
//!
//! This crate has no UI or GPU dependencies — everything is pure data
//! manipulation, fully testable headless. The design follows Helix:
//! `Document = Rope + Selections + History`, with edits expressed as
//! [`Transaction`]s that can be inverted, composed and used to map
//! positions (and therefore selections) across edits.

pub mod document;
pub mod history;
pub mod movement;
pub mod selection;
pub mod transaction;

pub use document::Document;
pub use history::{CommitKind, History, UndoStep};
pub use movement::GoalColumn;
pub use ropey::{self, Rope};
pub use selection::{Selection, SelectionSet};
pub use transaction::{Bias, Edit, Transaction};
