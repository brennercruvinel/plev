//! View trait and built-in view implementations.

mod context;
#[cfg(test)]
mod tests;
mod trait_def;
mod views;

pub use context::ViewContext;
pub use trait_def::View;
pub use views::{ContainerView, RectView, TextView};
