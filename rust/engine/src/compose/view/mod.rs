//! View trait and built-in view implementations, and the lifecycle-based
//! component system with persistent state and caching.
//!
//! **Experimental** -- not the official app pattern (state in structs +
//! retained widgets + explicit invalidation is; the showcase is the
//! template). Do not build new app code on this module without an ADR
//! (docs/decisoes/estrutura/0023-official-app-pattern.md).

mod context;
mod lifecycle;
#[cfg(test)]
mod tests;
mod views;

pub use context::ViewContext;
pub use lifecycle::Component;
pub use views::{ContainerView, RectView, TextView};

use crate::compose::layout::LayoutStyle;
use crate::drawing::compositor::SceneNode;

/// Produces SceneNodes without touching the compositor directly.
pub trait View {
    fn layout(&self) -> LayoutStyle {
        LayoutStyle::default()
    }

    fn children(&self) -> &[Box<dyn View>] {
        &[]
    }

    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode>;
}

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
