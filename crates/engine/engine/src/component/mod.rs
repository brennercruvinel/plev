//! Lifecycle-based component system with persistent state and caching.
//!
//! **Experimental** -- not the official app pattern (state in structs +
//! retained widgets + explicit invalidation is; the showcase is the
//! template). Do not build new app code on this module without an ADR
//! (docs/adr/official-app-pattern.md).

mod lifecycle_impl;

#[cfg(test)]
mod tests;

pub use self::lifecycle_impl::Component;

use crate::compositor::SceneNode;
use crate::view::ViewContext;

// ---------------------------------------------------------------------------
// Lifecycle trait -- stateful component with mount/update/unmount hooks
// ---------------------------------------------------------------------------

pub trait Lifecycle {
    type State;

    fn initial_state(&self) -> Self::State;

    fn on_mount(&self, _state: &mut Self::State) {}

    fn on_update(&self, _state: &mut Self::State) {}

    fn on_unmount(&self, _state: &mut Self::State) {}

    fn render(&self, state: &Self::State, cx: &mut ViewContext) -> Vec<SceneNode>;
}
