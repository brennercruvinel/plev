//! the pure simulation core: a deterministic seeded rng, prime generation,
//! the prime coherence matrix, a uniform spatial grid, the per-step physics,
//! and the `Simulation` that ties them together. no gpu, no winit, no
//! compositor; the logic is unit tested ahead of the pixels.

pub mod coherence;
pub mod grid;
pub mod params;
pub mod particle;
pub mod physics;
pub mod primes;
pub mod rng;

use crate::color::particle_color;
use coherence::CoherenceMatrix;
use grid::Grid;
use particle::Particle;
use physics::Bond;
use rng::Rng;
use std::f32::consts::TAU;

/// the whole simulation: the swarm, the prime tables, the coherence matrix,
/// the bonds drawn this frame, and the scratch buffers the physics reuses.
/// world coordinates are in logical pixels, sized to the window.
pub struct Simulation {
    particles: Vec<Particle>,
    primes: Vec<u32>,
    matrix: CoherenceMatrix,
    mode: params::CohMode,
    modulus: u32,
    rng: Rng,
    grid: Grid,
    bonds: Vec<Bond>,
    forces: Vec<(f32, f32)>,
    phase_delta: Vec<f32>,
    link_counts: Vec<u32>,
    width: f32,
    height: f32,
}

impl Simulation {
    /// a new simulation filling a `width` x `height` field, seeded for
    /// reproducibility. spawns `params::RESET_COUNT` particles.
    pub fn new(width: f32, height: f32, seed: u64) -> Self {
        let primes = primes::generate_primes(params::MAX_PRIME_INDEX);
        let mode = params::CohMode::default();
        let modulus = params::DEFAULT_MODULUS;
        let matrix = CoherenceMatrix::build(&primes, mode, modulus);
        let mut sim = Self {
            particles: Vec::new(),
            primes,
            matrix,
            mode,
            modulus,
            rng: Rng::new(seed),
            grid: Grid::new(),
            bonds: Vec::new(),
            forces: Vec::new(),
            phase_delta: Vec::new(),
            link_counts: Vec::new(),
            width: width.max(1.0),
            height: height.max(1.0),
        };
        sim.reset(width, height);
        sim
    }

    /// clear and respawn the swarm to fill a `width` x `height` field.
    pub fn reset(&mut self, width: f32, height: f32) {
        self.width = width.max(1.0);
        self.height = height.max(1.0);
        self.particles.clear();
        let pad = params::SPAWN_PAD;
        for _ in 0..params::RESET_COUNT {
            let x = self.rng.range(pad, (self.width - pad).max(pad + 1.0));
            let y = self.rng.range(pad, (self.height - pad).max(pad + 1.0));
            let particle = self.make_particle(x, y);
            self.particles.push(particle);
        }
        self.resize_scratch();
    }

    /// paint one particle at `(x, y)` (the brush).
    pub fn spawn_at(&mut self, x: f32, y: f32) {
        let particle = self.make_particle(x, y);
        self.particles.push(particle);
        self.resize_scratch();
    }

    /// clear every particle (the demo's Clear).
    pub fn clear(&mut self) {
        self.particles.clear();
        self.resize_scratch();
    }

    /// the painting brush: roughly half the time, spawn a particle somewhere
    /// inside the brush radius of `(x, y)` (the demo's per-frame loop spawn).
    pub fn brush(&mut self, x: f32, y: f32) {
        if self.rng.next_f32() > 0.5 {
            let angle = self.rng.range(0.0, TAU);
            let r = self.rng.range(0.0, params::BRUSH_SIZE);
            self.spawn_at(x + angle.cos() * r, y + angle.sin() * r);
        }
    }

    fn make_particle(&mut self, x: f32, y: f32) -> Particle {
        let prime_index = self.rng.below(params::MAX_PRIME_INDEX);
        let prime_value = self.primes[prime_index];
        let phase = self.rng.range(0.0, TAU);
        let phase_rate = 0.01 + self.rng.range(0.0, 0.01);
        let color = particle_color(self.mode, prime_index, prime_value, self.modulus);
        Particle::new(
            x,
            y,
            prime_index,
            prime_value,
            params::MASS,
            phase,
            phase_rate,
            color,
        )
    }

    fn resize_scratch(&mut self) {
        let n = self.particles.len();
        self.forces.resize(n, (0.0, 0.0));
        self.phase_delta.resize(n, 0.0);
        self.link_counts.resize(n, 0);
    }

    /// resize the field, clamping every particle into the new bounds.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width.max(1.0);
        self.height = height.max(1.0);
        for particle in self.particles.iter_mut() {
            particle.x = particle
                .x
                .clamp(params::BOUNCE_PAD, self.width - params::BOUNCE_PAD);
            particle.y = particle
                .y
                .clamp(params::BOUNCE_PAD, self.height - params::BOUNCE_PAD);
        }
    }

    /// advance one fixed step. `_dt` is the caller's accumulator unit; the
    /// physics is frame-based (the demo's model), so a step is a step.
    pub fn step(&mut self, _dt: f32) {
        self.grid
            .rebuild(&self.particles, self.width, self.height, params::GRID_SIZE);
        physics::accumulate(
            &self.particles,
            &mut self.forces,
            &mut self.phase_delta,
            &mut self.link_counts,
            &mut self.bonds,
            &self.matrix,
            &self.grid,
            self.width,
            self.height,
        );
        physics::integrate(
            &mut self.particles,
            &self.forces,
            &self.phase_delta,
            &self.link_counts,
            self.width,
            self.height,
        );
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// the bonds drawn this frame (the demo's lineBuffer).
    pub fn bonds(&self) -> &[Bond] {
        &self.bonds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_the_reset_count() {
        let sim = Simulation::new(900.0, 700.0, 1);
        assert_eq!(sim.particles().len(), params::RESET_COUNT);
    }

    #[test]
    fn stays_in_bounds_and_finite_after_many_steps() {
        let mut sim = Simulation::new(900.0, 700.0, 7);
        for _ in 0..600 {
            sim.step(params::FIXED_DT);
        }
        for particle in sim.particles() {
            assert!(
                particle.x.is_finite() && particle.y.is_finite(),
                "position went non-finite"
            );
            assert!(
                particle.x >= 0.0
                    && particle.x <= 900.0
                    && particle.y >= 0.0
                    && particle.y <= 700.0,
                "particle escaped: ({}, {})",
                particle.x,
                particle.y
            );
        }
    }

    #[test]
    fn same_seed_is_reproducible() {
        let run = |seed| {
            let mut sim = Simulation::new(900.0, 700.0, seed);
            for _ in 0..120 {
                sim.step(params::FIXED_DT);
            }
            sim.particles()
                .iter()
                .map(|p| (p.x, p.y))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(42), run(42));
        assert_ne!(run(42), run(43));
    }

    #[test]
    fn the_swarm_moves_and_bonds_form() {
        let mut sim = Simulation::new(900.0, 700.0, 3);
        let start: Vec<_> = sim.particles().iter().map(|p| (p.x, p.y)).collect();
        let mut saw_bonds = false;
        for _ in 0..120 {
            sim.step(params::FIXED_DT);
            saw_bonds |= !sim.bonds().is_empty();
        }
        let moved = sim
            .particles()
            .iter()
            .zip(&start)
            .filter(|(p, s)| (p.x - s.0).abs() + (p.y - s.1).abs() > 1.0)
            .count();
        assert!(
            moved > sim.particles().len() / 2,
            "swarm barely moved: {moved}"
        );
        assert!(saw_bonds, "no bonds ever formed; links would be empty");
    }
}
