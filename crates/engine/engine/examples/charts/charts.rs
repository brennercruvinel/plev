// Chart data model and chart drawing functions.

use engine::compositor::{Compositor, RoundedRectParams, SceneNode};
use engine::path::PathBuilder;

use crate::palette::{GRID, push_circle};

pub(crate) struct ChartData {
    pub(crate) line_data: Vec<f32>,
    pub(crate) bar_data: Vec<f32>,
    pub(crate) area_data_1: Vec<f32>,
    pub(crate) area_data_2: Vec<f32>,
    pub(crate) pie_values: Vec<f32>,
}

impl ChartData {
    pub(crate) fn new() -> Self {
        Self {
            line_data: vec![
                0.3, 0.5, 0.4, 0.7, 0.6, 0.9, 0.75, 0.85, 0.65, 0.95, 0.8, 0.92,
            ],
            bar_data: vec![0.6, 0.8, 0.45, 0.9, 0.55, 0.75, 0.65, 0.85],
            area_data_1: vec![0.2, 0.35, 0.3, 0.5, 0.45, 0.6, 0.55, 0.7, 0.65, 0.75],
            area_data_2: vec![0.1, 0.2, 0.15, 0.3, 0.25, 0.35, 0.3, 0.4, 0.35, 0.45],
            pie_values: vec![35.0, 25.0, 20.0, 12.0, 8.0],
        }
    }

    pub(crate) fn animate(&mut self, t: f32) {
        for (i, v) in self.line_data.iter_mut().enumerate() {
            *v = (*v + (t * 0.3 + i as f32 * 0.5).sin() * 0.02).clamp(0.1, 1.0);
        }
        for (i, v) in self.bar_data.iter_mut().enumerate() {
            *v = (*v + (t * 0.2 + i as f32 * 0.7).sin() * 0.015).clamp(0.1, 1.0);
        }
    }
}

pub(crate) fn draw_grid(
    comp: &mut Compositor,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    h_lines: u32,
    v_lines: u32,
) {
    for i in 0..=h_lines {
        let ly = y + h * (i as f32 / h_lines as f32);
        comp.push(SceneNode::Rect {
            x,
            y: ly,
            w,
            h: 1.0,
            color: GRID,
        });
    }
    for i in 0..=v_lines {
        let lx = x + w * (i as f32 / v_lines as f32);
        comp.push(SceneNode::Rect {
            x: lx,
            y,
            w: 1.0,
            h,
            color: GRID,
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_line_chart(
    comp: &mut Compositor,
    data: &[f32],
    color: [f32; 4],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    dot_r: f32,
) {
    let n = data.len();
    if n < 2 {
        return;
    }
    let step = w / (n - 1) as f32;

    // Area fill
    let mut b = PathBuilder::new();
    b = b.move_to(x, y + h);
    for (i, v) in data.iter().enumerate() {
        let px = x + i as f32 * step;
        let py = y + h - v * h;
        b = b.line_to(px, py);
    }
    b = b.line_to(x + w, y + h);
    comp.draw_path(b.close().fill([color[0], color[1], color[2], 0.1]));

    // Line stroke
    let stroke_b = {
        let mut sb = PathBuilder::new();
        for (i, v) in data.iter().enumerate() {
            let px = x + i as f32 * step;
            let py = y + h - v * h;
            sb = if i == 0 {
                sb.move_to(px, py)
            } else {
                sb.line_to(px, py)
            };
        }
        sb.stroke(color, 2.0)
    };
    comp.draw_path(stroke_b);

    // Dots
    for (i, v) in data.iter().enumerate() {
        let px = x + i as f32 * step;
        let py = y + h - v * h;
        push_circle(comp, px, py, dot_r, color);
    }
}

pub(crate) fn draw_bar_chart(
    comp: &mut Compositor,
    data: &[f32],
    colors: &[[f32; 4]],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let n = data.len();
    let bar_gap = 6.0;
    let bar_w = (w - bar_gap * (n + 1) as f32) / n as f32;

    for (i, v) in data.iter().enumerate() {
        let bx = x + bar_gap + i as f32 * (bar_w + bar_gap);
        let bh = v * h;
        let by = y + h - bh;
        let color = colors[i % colors.len()];
        comp.draw_rounded_rect(RoundedRectParams {
            x: bx,
            y: by,
            w: bar_w,
            h: bh,
            color,
            corner_radius: 3.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
    }
}

pub(crate) fn draw_pie_chart(
    comp: &mut Compositor,
    values: &[f32],
    colors: &[[f32; 4]],
    cx: f32,
    cy: f32,
    r: f32,
) {
    let total: f32 = values.iter().sum();
    if total <= 0.0 {
        return;
    }
    let n = 48;
    let mut angle = -std::f32::consts::FRAC_PI_2;

    for (vi, val) in values.iter().enumerate() {
        let sweep = std::f32::consts::TAU * (val / total);
        let color = colors[vi % colors.len()];

        let mut b = PathBuilder::new();
        b = b.move_to(cx, cy);
        for i in 0..=n {
            let a = angle + sweep * (i as f32 / n as f32);
            b = b.line_to(cx + r * a.cos(), cy + r * a.sin());
        }
        comp.draw_path(b.close().fill(color));
        angle += sweep;
    }
}
