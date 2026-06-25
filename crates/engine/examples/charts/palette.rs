// Color palette and visual helpers for the charts demo.

use engine::compositor::{Compositor, SceneNode};
use engine::path::PathBuilder;

pub(crate) const BG: [f32; 4] = [0.05, 0.05, 0.08, 1.0];
pub(crate) const SURFACE: [f32; 4] = [0.09, 0.09, 0.14, 1.0];
pub(crate) const ACCENT: [f32; 4] = [0.30, 0.55, 1.0, 1.0];
pub(crate) const GREEN: [f32; 4] = [0.20, 0.80, 0.45, 1.0];
pub(crate) const RED: [f32; 4] = [1.0, 0.30, 0.25, 1.0];
pub(crate) const YELLOW: [f32; 4] = [1.0, 0.85, 0.20, 1.0];
pub(crate) const PURPLE: [f32; 4] = [0.60, 0.30, 0.90, 1.0];
pub(crate) const CYAN: [f32; 4] = [0.0, 0.85, 0.95, 1.0];
pub(crate) const ORANGE: [f32; 4] = [1.0, 0.55, 0.10, 1.0];
pub(crate) const PINK: [f32; 4] = [1.0, 0.40, 0.70, 1.0];
pub(crate) const TEXT: [f32; 4] = [0.93, 0.93, 0.96, 1.0];
pub(crate) const TEXT_DIM: [f32; 4] = [0.50, 0.50, 0.60, 1.0];
pub(crate) const DIVIDER: [f32; 4] = [0.16, 0.16, 0.22, 1.0];
pub(crate) const GRID: [f32; 4] = [0.12, 0.12, 0.18, 0.5];

pub(crate) fn push_circle(comp: &mut Compositor, cx: f32, cy: f32, r: f32, color: [f32; 4]) {
    comp.draw_path(PathBuilder::circle(cx, cy, r).fill(color));
}

pub(crate) fn push_shadow(comp: &mut Compositor, x: f32, y: f32, w: f32, h: f32) {
    for i in 0..5u32 {
        let s = (5 - i) as f32 * 2.0;
        comp.push(SceneNode::Rect {
            x: x - s + 2.0,
            y: y - s + 3.0,
            w: w + s * 2.0,
            h: h + s * 2.0,
            color: [0.0, 0.0, 0.0, 0.01 + i as f32 * 0.006],
        });
    }
}
