//! shared parameters and tuning, transcribed from the reference demo's GENOME
//! and State so the simulation behaves identically.

/// number of primes generated; particles index into [0, MAX_PRIME_INDEX).
pub const MAX_PRIME_INDEX: usize = 250;

/// default modulus for the modular coherence and hue modes (the demo default).
pub const DEFAULT_MODULUS: u32 = 432;

/// particles spawned on reset (the demo's 400).
pub const RESET_COUNT: usize = 400;

// --- GENOME (shared by every particle, like the demo) ---
pub const MASS: f32 = 1.5;
pub const MUSCLE: f32 = 0.2;
pub const LAG: f32 = 0.5;
pub const BOND_DIST: f32 = 30.0;
pub const STIFFNESS: f32 = 0.05;
pub const ALIGN: f32 = 0.05;
pub const REPULSION: f32 = 0.6;

// --- STATE (environment) ---
pub const VISCOSITY: f32 = 0.06;
pub const GRAVITY: f32 = 0.001;
pub const GRID_SIZE: f32 = 100.0;
pub const COH_FORCE: f32 = 0.8;
pub const SYNC_RATE: f32 = 0.1;

/// edge margin where particles bounce (the demo's pad = 20).
pub const BOUNCE_PAD: f32 = 20.0;
/// wall bounce energy retained (the demo's -0.8).
pub const RESTITUTION: f32 = 0.8;
/// spawn inset on reset (the demo's pad = 50).
pub const SPAWN_PAD: f32 = 50.0;

/// brush radius for painting new particles.
pub const BRUSH_SIZE: f32 = 50.0;

/// how many past positions each particle keeps for its motion trail. the demo
/// gets trails free from canvas accumulation; the engine clears every frame,
/// so the tail is drawn explicitly from this history.
pub const TRAIL_LEN: usize = 9;

/// fixed physics timestep, in seconds; the accumulator steps in these.
pub const FIXED_DT: f32 = 1.0 / 60.0;

/// how a prime pair maps to a coherence value, and how a single prime maps to
/// a hue. mirrors the demo's `cohMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CohMode {
    /// affinity from raw prime proximity: 1 - |p-q|/(p+q). the demo default.
    #[default]
    Proximity,
    /// affinity from circular distance of p and q modulo `modulus`.
    Modular,
    /// affinity from the log-ratio of p and q (harmonic closeness).
    Harmonic,
    /// affinity from shared bits: (p & q) / (p | q).
    Bitwise,
}
