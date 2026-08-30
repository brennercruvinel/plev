//! View trait and built-in view implementations.
//!
//! **Experimental** -- not the official app pattern (state in structs +
//! retained widgets + explicit invalidation is; the showcase is the
//! template). Do not build new app code on this module without an ADR
//! (docs/adr/official-app-pattern.md).

mod context;
#[cfg(test)]
mod tests;
mod trait_def;
mod views;

pub use context::ViewContext;
pub use trait_def::View;
pub use views::{ContainerView, RectView, TextView};
