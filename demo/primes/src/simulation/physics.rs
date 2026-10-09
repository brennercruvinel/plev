//! the per-step physics, transcribed from the reference demo: a grid-local
//! neighbor scan accumulates short-range repulsion, coherence-weighted bond
//! springs (with phase-resonant strength), boids alignment, central gravity,
//! and Kuramoto phase sync; then a steering integration turns force into
//! thrust (along the heading) and steer (across it), so particles swim.
//!
//! phase sync is collected as a delta and applied in `integrate` (Jacobi),
//! rather than mutated mid-scan (the demo's Gauss-Seidel); visually identical
//! and free of aliasing.

use crate::sim::coherence::CoherenceMatrix;
use crate::sim::grid::Grid;
use crate::sim::params as p;
use crate::sim::particle::Particle;
use std::f32::consts::{PI, TAU};

/// a drawn bond between two particles for this frame (the demo's lineBuffer).
#[derive(Clone, Copy)]
pub struct Bond {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    /// stroke strength in [0, 1], from force magnitude and coherence.
    pub strength: f32,
    /// coherence in [0, 1], drives the link lightness.
    pub coherence: f32,
}

/// read every particle, writing per-particle force, phase delta, and bond
/// count into the scratch buffers and pushing drawn bonds into `bonds`.
#[allow(clippy::too_many_arguments)]
pub fn accumulate(
    particles: &[Particle],
    forces: &mut [(f32, f32)],
    phase_delta: &mut [f32],
    link_counts: &mut [u32],
    bonds: &mut Vec<Bond>,
    matrix: &CoherenceMatrix,
    grid: &Grid,
    w: f32,
    h: f32,
) {
    bonds.clear();
    let (cx, cy) = (w * 0.5, h * 0.5);

    for (i, pi) in particles.iter().enumerate() {
        let mut fx = 0.0f32;
        let mut fy = 0.0f32;
        let mut pdelta = 0.0f32;
        let mut links = 0u32;

        // central gravity as a steering force (the demo boosts it x20).
        let (gdx, gdy) = (cx - pi.x, cy - pi.y);
        let gd = (gdx * gdx + gdy * gdy).sqrt();
        if gd > 0.0 {
            fx += gdx / gd * p::GRAVITY * 20.0;
            fy += gdy / gd * p::GRAVITY * 20.0;
        }

        grid.for_each_neighbor(pi.x, pi.y, |j| {
            let j = j as usize;
            if j == i {
                return;
            }
            let n = &particles[j];
            let dx = n.x - pi.x;
            let dy = n.y - pi.y;
            let d2 = dx * dx + dy * dy;

            let coh = matrix.get(pi.prime_index, n.prime_index);
            let range_scale = 1.0 + (pi.mass + n.mass) * 0.15 + coh * 0.5;
            let max_dist = p::BOND_DIST * range_scale;
            if d2 > max_dist * max_dist || d2 == 0.0 {
                return;
            }
            let dist = d2.sqrt();
            let (nx, ny) = (dx / dist, dy / dist);
            let skin = (pi.mass + n.mass) * 6.0 + 10.0;

            if dist < skin {
                let repel = (1.0 - dist / skin) * p::REPULSION * 0.2;
                fx -= nx * repel;
                fy -= ny * repel;
            } else if coh > 0.1 {
                let phase_diff = (pi.phase - n.phase).abs();
                let phase_factor = (phase_diff.cos() + 1.0) * 0.5;
                let saturation = 1.0 / (1.0 + links as f32 * 0.2);

                let activity = pi.phase.sin() * p::MUSCLE;
                let target = max_dist * (0.4 + activity * 0.2);
                let dyn_str =
                    p::STIFFNESS * (coh * p::COH_FORCE + 0.2) * (0.5 + phase_factor) * saturation;
                let force = (dist - target) * dyn_str;
                fx += nx * force / pi.mass;
                fy += ny * force / pi.mass;

                if force.abs() > 0.01 {
                    links += 1;
                    if i < j {
                        let strength = (force.abs() * 10.0).min(1.0) * coh;
                        bonds.push(Bond {
                            x1: pi.x,
                            y1: pi.y,
                            x2: n.x,
                            y2: n.y,
                            strength,
                            coherence: coh,
                        });
                    }
                }

                if p::ALIGN > 0.0 && dist < max_dist * 0.8 {
                    fx += (n.vx - pi.vx) * p::ALIGN * coh * saturation;
                    fy += (n.vy - pi.vy) * p::ALIGN * coh * saturation;
                }

                // Kuramoto sync with a velocity-dependent lag.
                let dot = pi.vx * nx + pi.vy * ny;
                let lag_dir = if dot > 0.0 { -p::LAG } else { p::LAG };
                let mut pdiff = (n.phase + lag_dir) - pi.phase;
                if pdiff > PI {
                    pdiff -= TAU;
                }
                if pdiff < -PI {
                    pdiff += TAU;
                }
                pdelta += pdiff * p::SYNC_RATE * coh;
            }
        });

        forces[i] = (fx, fy);
        phase_delta[i] = pdelta;
        link_counts[i] = links;
    }
}

/// apply the accumulated force as steering: decompose it into thrust (along
/// the current heading) and steer (across it), update speed and angle, move,
/// bounce off the walls, advance the phase, and record the trail.
pub fn integrate(
    particles: &mut [Particle],
    forces: &[(f32, f32)],
    phase_delta: &[f32],
    link_counts: &[u32],
    w: f32,
    h: f32,
) {
    let pad = p::BOUNCE_PAD;
    for (i, particle) in particles.iter_mut().enumerate() {
        let (fx, fy) = forces[i];
        let speed = (particle.vx * particle.vx + particle.vy * particle.vy).sqrt();

        if speed > 0.001 {
            let (hx, hy) = (particle.vx / speed, particle.vy / speed);
            let f_thrust = fx * hx + fy * hy;
            let f_steer = fx * -hy + fy * hx;

            let mut new_speed = (speed + f_thrust) * (1.0 - p::VISCOSITY / particle.mass);
            if new_speed < 0.0 {
                new_speed = 0.0;
            }
            let angle = particle.vy.atan2(particle.vx) + f_steer * (0.5 / particle.mass);
            particle.vx = angle.cos() * new_speed;
            particle.vy = angle.sin() * new_speed;
        } else {
            particle.vx += fx;
            particle.vy += fy;
        }

        particle.x += particle.vx;
        particle.y += particle.vy;

        if particle.x < pad {
            particle.x = pad;
            particle.vx *= -p::RESTITUTION;
        } else if particle.x > w - pad {
            particle.x = w - pad;
            particle.vx *= -p::RESTITUTION;
        }
        if particle.y < pad {
            particle.y = pad;
            particle.vy *= -p::RESTITUTION;
        } else if particle.y > h - pad {
            particle.y = h - pad;
            particle.vy *= -p::RESTITUTION;
        }

        particle.phase += 0.1 * particle.phase_rate + phase_delta[i];
        if particle.phase > TAU {
            particle.phase -= TAU;
        } else if particle.phase < 0.0 {
            particle.phase += TAU;
        }
        particle.links = link_counts[i];
        particle.push_trail();
    }
}
