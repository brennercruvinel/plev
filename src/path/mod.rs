//! Path building, tessellation, and types for GPU-ready geometry.

mod builder;
mod tessellation;
#[cfg(test)]
mod tests;
mod types;

use std::sync::atomic::{AtomicU32, Ordering};

pub use builder::PathBuilder;
pub use types::TessellatedPath;

static DEFAULT_TOLERANCE_BITS: AtomicU32 = AtomicU32::new(0.1f32.to_bits());

/// Set the default tessellation tolerance used by [`PathBuilder::fill`] and
/// [`PathBuilder::stroke`]. Configured from `RenderConfig::path_tolerance`
/// when a `GpuContext` is created.
pub fn set_default_tolerance(tolerance: f32) {
    DEFAULT_TOLERANCE_BITS.store(tolerance.to_bits(), Ordering::Relaxed);
}

/// Current default tessellation tolerance (0.1 unless reconfigured).
pub fn default_tolerance() -> f32 {
    f32::from_bits(DEFAULT_TOLERANCE_BITS.load(Ordering::Relaxed))
}
