//! Path building, tessellation, and types for GPU-ready geometry.

mod builder;
mod tessellation;
#[cfg(test)]
mod tests;
mod types;

pub use builder::PathBuilder;
pub use types::TessellatedPath;
