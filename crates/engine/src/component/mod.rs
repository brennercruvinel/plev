//! Lifecycle-based component system with persistent state and caching.

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
