//! a single particle: position, velocity, force accumulator, prime identity,
//! oscillator phase, and a short ring buffer of past positions for the motion
//! trail. coordinates are in logical pixels (the demo's css-pixel world).

use crate::sim::params::TRAIL_LEN;

/// one swarm particle. prime fields are fixed at spawn; the rest evolve.
#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// index into the prime table and the coherence matrix.
    pub prime_index: usize,
    /// the prime value itself (drives the hue under some modes).
    pub prime_value: u32,
    pub mass: f32,
    /// oscillator phase in radians.
    pub phase: f32,
    pub phase_rate: f32,
    /// bonds formed this step; drives glow and the drawn radius.
    pub links: u32,
    /// base color as sRGB [r, g, b, a] in [0, 1], precomputed from the prime.
    pub color: [f32; 4],
    /// ring buffer of past positions, oldest..newest by age; `trail_head` is
    /// the next slot to overwrite.
    trail: [[f32; 2]; TRAIL_LEN],
    trail_head: usize,
}

impl Particle {
    /// a particle seeded at `(x, y)`. the trail starts collapsed at the spawn
    /// point so no stray tail flashes on the first frames.
    #[allow(clippy::too_many_arguments)] // cohesive spawn fields; a builder would obscure them
    pub fn new(
        x: f32,
        y: f32,
        prime_index: usize,
        prime_value: u32,
        mass: f32,
        phase: f32,
        phase_rate: f32,
        color: [f32; 4],
    ) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            prime_index,
            prime_value,
            mass,
            phase,
            phase_rate,
            links: 0,
            color,
            trail: [[x, y]; TRAIL_LEN],
            trail_head: 0,
        }
    }

    /// record the current position into the trail ring.
    pub fn push_trail(&mut self) {
        self.trail[self.trail_head] = [self.x, self.y];
        self.trail_head = (self.trail_head + 1) % TRAIL_LEN;
    }

    /// trail positions from oldest to newest, paired with an age fraction in
    /// (0, 1] (1 is the most recent), for fading the tail.
    pub fn trail_iter(&self) -> impl Iterator<Item = ([f32; 2], f32)> + '_ {
        (0..TRAIL_LEN).map(move |k| {
            let idx = (self.trail_head + k) % TRAIL_LEN;
            let frac = (k + 1) as f32 / TRAIL_LEN as f32;
            (self.trail[idx], frac)
        })
    }
}
