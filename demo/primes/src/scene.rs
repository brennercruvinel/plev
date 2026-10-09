//! the field renderer, faithful to the demo's draw order: motion trails first
//! (faded tails reconstructed from each particle's position history, since the
//! engine clears every frame instead of accumulating), then cyan bond links,
//! then glow halos for active particles, then the particle cores on top.
//!
//! colors are sRGB; the gpu linearizes them in the shader. push order is
//! preserved by the engine's layer encoder, so the layering reads correctly.

use engine::compositor::{Compositor, RoundedRectParams};
use engine::path::PathBuilder;
use prime::color::hsl_to_srgb;
use prime::sim::Simulation;
use prime::sim::particle::Particle;

/// the demo's particle radius: max(0.5, mass*1.5) breathing with the phase.
fn core_radius(p: &Particle) -> f32 {
    (p.mass * 1.5).max(0.5) + p.phase.sin() * 0.5
}

fn circle(compositor: &mut Compositor, cx: f32, cy: f32, r: f32, color: [f32; 4]) {
    let r = r.max(0.3);
    compositor.draw_rounded_rect(RoundedRectParams {
        x: cx - r,
        y: cy - r,
        w: r * 2.0,
        h: r * 2.0,
        color,
        corner_radius: r,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
}

pub fn field_scene(compositor: &mut Compositor, sim: &Simulation) {
    // 1. trails: faded tail circles, oldest (faint) to newest (brighter).
    for p in sim.particles() {
        let base = core_radius(p);
        for (pos, frac) in p.trail_iter() {
            let alpha = frac * frac * 0.45;
            if alpha < 0.02 {
                continue;
            }
            let r = base * (0.25 + 0.6 * frac);
            circle(
                compositor,
                pos[0],
                pos[1],
                r,
                [p.color[0], p.color[1], p.color[2], alpha],
            );
        }
    }

    // 2. links: cyan lines, brighter and more opaque with coherence/strength
    // (the demo's hsla(180, 100%, 50+coh*50%, str*0.3)).
    for b in sim.bonds() {
        let lightness = 0.5 + b.coherence * 0.5;
        let mut color = hsl_to_srgb(180.0, 1.0, lightness);
        color[3] = (b.strength * 0.3).clamp(0.0, 1.0);
        if color[3] < 0.01 {
            continue;
        }
        let path = PathBuilder::new()
            .move_to(b.x1, b.y1)
            .line_to(b.x2, b.y2)
            .stroke(color, 1.0);
        compositor.draw_path(path);
    }

    // 3. glow: a soft translucent halo behind active particles (links > 2),
    // standing in for the demo's shadowBlur.
    for p in sim.particles() {
        if p.links > 2 {
            let r = core_radius(p) * 2.6;
            circle(
                compositor,
                p.x,
                p.y,
                r,
                [p.color[0], p.color[1], p.color[2], 0.16],
            );
        }
    }

    // 4. cores on top.
    for p in sim.particles() {
        circle(compositor, p.x, p.y, core_radius(p), p.color);
    }
}
