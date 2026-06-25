//! prime number creatures: pure, gpu-free backend modules unit tested ahead
//! of the scene and controls that live in the bin target. backend before ui.
//!
//! `sim` is the deterministic simulation core (seeded rng, prime generation,
//! the prime coherence matrix, shared params); `color` maps a prime to its
//! particle color. nothing here touches gpu, winit, or the compositor.

pub mod color;
pub mod sim;
