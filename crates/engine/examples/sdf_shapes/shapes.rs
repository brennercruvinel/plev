//! Shape primitives, color palette and typography for the SDF shapes demo.

use engine::compositor::{Compositor, ShadowParams};
use engine::path::PathBuilder;
use engine::text::TextStyle;

// ---------------------------------------------------------------------------
// Color palette
// ---------------------------------------------------------------------------

pub const BG: [f32; 4] = [0.05, 0.05, 0.08, 1.0];
pub const SURFACE: [f32; 4] = [0.09, 0.09, 0.14, 1.0];
pub const SURFACE_2: [f32; 4] = [0.12, 0.12, 0.18, 1.0];
pub const ACCENT: [f32; 4] = [0.30, 0.55, 1.0, 1.0];
pub const GREEN: [f32; 4] = [0.20, 0.80, 0.45, 1.0];
pub const RED: [f32; 4] = [1.0, 0.30, 0.25, 1.0];
pub const YELLOW: [f32; 4] = [1.0, 0.85, 0.20, 1.0];
pub const PURPLE: [f32; 4] = [0.60, 0.30, 0.90, 1.0];
pub const CYAN: [f32; 4] = [0.0, 0.85, 0.95, 1.0];
pub const ORANGE: [f32; 4] = [1.0, 0.55, 0.10, 1.0];
pub const PINK: [f32; 4] = [1.0, 0.40, 0.70, 1.0];
pub const TEXT: [f32; 4] = [0.93, 0.93, 0.96, 1.0];
pub const TEXT_DIM: [f32; 4] = [0.50, 0.50, 0.60, 1.0];
pub const DIVIDER: [f32; 4] = [0.16, 0.16, 0.22, 1.0];

// ---------------------------------------------------------------------------
// Typography: one TextStyle per run, shared by measurement and drawing
// (docs/adr/one-text-style-for-measurement-and-drawing.md). Weights are
// semantic: 700 page title, 600 card title, 500 label, 400 body; code
// references render in JetBrains Mono.
// ---------------------------------------------------------------------------

pub fn title_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_weight(700)
}

pub fn card_title_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_weight(600)
}

pub fn label_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_weight(500)
}

pub fn body_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size).with_line_height(line_height)
}

pub fn code_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_family("JetBrains Mono")
}

// ---------------------------------------------------------------------------
// Shape helpers
// ---------------------------------------------------------------------------

pub fn push_circle(comp: &mut Compositor, cx: f32, cy: f32, r: f32, color: [f32; 4]) {
    comp.draw_path(PathBuilder::circle(cx, cy, r).fill(color));
}

pub fn push_ring(comp: &mut Compositor, cx: f32, cy: f32, r_out: f32, r_in: f32, color: [f32; 4]) {
    let n = 64;
    let mut b = PathBuilder::new();
    for i in 0..=n {
        let a = std::f32::consts::TAU * (i as f32 / n as f32);
        b = if i == 0 {
            b.move_to(cx + r_out * a.cos(), cy + r_out * a.sin())
        } else {
            b.line_to(cx + r_out * a.cos(), cy + r_out * a.sin())
        };
    }
    for i in (0..=n).rev() {
        let a = std::f32::consts::TAU * (i as f32 / n as f32);
        b = b.line_to(cx + r_in * a.cos(), cy + r_in * a.sin());
    }
    comp.draw_path(b.close().fill(color));
}

/// Analytic soft shadow (`Compositor::draw_shadow`, Evan Wallace
/// approximation — no blur pass). `layers` is a leftover of the old
/// stacked-translucent-rects POC and now maps to elevation: more layers,
/// larger blur and drop. `0` means flat (no shadow).
pub fn push_shadow(comp: &mut Compositor, x: f32, y: f32, w: f32, h: f32, layers: u32) {
    if layers == 0 {
        return;
    }
    comp.draw_shadow(ShadowParams {
        x,
        y,
        w,
        h,
        corner_radius: 6.0,
        blur_radius: layers as f32 * 3.0,
        offset: [0.0, layers as f32 * 0.8],
        color: [0.0, 0.0, 0.0, 0.45],
        inset: false,
    });
}

pub fn push_star(
    comp: &mut Compositor,
    cx: f32,
    cy: f32,
    r_out: f32,
    r_in: f32,
    points: u32,
    color: [f32; 4],
) {
    let mut b = PathBuilder::new();
    let total = points * 2;
    for i in 0..total {
        let a = std::f32::consts::TAU * (i as f32 / total as f32) - std::f32::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { r_out } else { r_in };
        let px = cx + r * a.cos();
        let py = cy + r * a.sin();
        b = if i == 0 {
            b.move_to(px, py)
        } else {
            b.line_to(px, py)
        };
    }
    comp.draw_path(b.close().fill(color));
}

pub fn push_polygon(comp: &mut Compositor, cx: f32, cy: f32, r: f32, sides: u32, color: [f32; 4]) {
    let mut b = PathBuilder::new();
    for i in 0..sides {
        let a = std::f32::consts::TAU * (i as f32 / sides as f32) - std::f32::consts::FRAC_PI_2;
        let px = cx + r * a.cos();
        let py = cy + r * a.sin();
        b = if i == 0 {
            b.move_to(px, py)
        } else {
            b.line_to(px, py)
        };
    }
    comp.draw_path(b.close().fill(color));
}
