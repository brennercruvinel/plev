//! Reactive signal system -- push-pull hybrid (Leptos/Reactively style).
//!
//! Thread-local runtime using `SlotMap` for O(1) node storage with generational keys.
//! Signals, effects, and memos are all `ReactiveNode`s in the same graph.
//!
//! # Borrow Safety
//! The runtime lives in a `RefCell`. To avoid reentrance panics, every function
//! that executes user closures (effects, memos) releases the borrow BEFORE calling
//! the closure, then re-borrows afterward. Closures are stored as `Rc<dyn Fn>`
//! so they can be cloned out of the borrowed runtime.

mod api;
pub(crate) mod execution;
pub(crate) mod runtime;
mod tests;
mod tests_advanced;

pub use api::*;
pub use runtime::NodeId;
