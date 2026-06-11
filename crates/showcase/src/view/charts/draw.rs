//! Chart drawing: turns the pure geometry from `showcase::model::charts`
//! into scene nodes. Every label is drawn with exactly the `TextStyle` it
//! was measured with (the `Label` carries it), and the reveal parameter
//! `r` in 0..=1 scales primitives the way the old makepad_charts demo did:
//! the line sweeps open, bars grow, bands rise, the ring sweeps clockwise.

use std::f32::consts::TAU;

use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::path::PathBuilder;
use plev::theme::Theme;
use plev::ui::widgets::{Rect, rounded_rect};
use showcase::model::charts as geom;

use crate::view::with_alpha;

/// Monochrome HOFF alpha ramp for donut slices and legend swatches.
const DONUT_ALPHAS: [f32; 5] = [0.90, 0.62, 0.40, 0.24, 0.12];

pub(super) fn line(c: &mut Compositor, data: &[f32], rect: Rect, theme: &Theme, r: f32) {
    let chart = geom::line_chart(data, rect, 3.0, theme.typography.small_sm());
    let divider = theme.colors.divider.0;
    let plot = chart.plot;
    for y in &chart.grid_h {
        line_rect(c, plot.x, *y, plot.w, 1.0, divider);
    }
    let v_color = with_alpha(divider, divider[3] * 0.6);
    for x in &chart.grid_v {
        line_rect(c, *x, plot.y, 1.0, plot.h, v_color);
    }
    for l in &chart.tick_labels {
        label(c, l, theme.glass.text_faint.0);
    }
    // The reveal sweeps the plot open left to right; the 4px margin keeps
    // the dot caps from being shaved at the plot edges.
    let accent = theme.colors.accent.0;
    c.push(SceneNode::PushClip {
        x: plot.x - 4.0,
        y: plot.y - 4.0,
        w: (plot.w + 8.0) * r,
        h: plot.h + 8.0,
    });
    polygon(c, &chart.area, with_alpha(accent, 0.08));
    polyline(c, &chart.points, accent, 2.0);
    for d in &chart.dots {
        c.draw_path(PathBuilder::circle(d.x, d.y, d.r).fill(accent));
    }
    c.push(SceneNode::PopClip);
}

pub(super) fn bars(c: &mut Compositor, data: &[f32], rect: Rect, theme: &Theme, r: f32) {
    let chart = geom::bar_chart(data, rect, 8.0, theme.typography.small_sm());
    let baseline = rect.y + rect.h;
    line_rect(c, rect.x, baseline, rect.w, 1.0, theme.glass.edge.0);
    let tallest = (chart.bars.iter().enumerate())
        .max_by(|a, b| a.1.value.total_cmp(&b.1.value))
        .map(|(i, _)| i);
    for (i, bar) in chart.bars.iter().enumerate() {
        let h = bar.rect.h * r; // bars grow out of the baseline
        if h <= 0.01 {
            continue;
        }
        let color = if Some(i) == tallest {
            with_alpha(theme.colors.success.0, 0.90)
        } else {
            with_alpha(theme.colors.text.0, 0.28)
        };
        c.push(rounded_rect(
            bar.rect.x,
            baseline - h,
            bar.rect.w,
            h,
            bar.radius,
            color,
        ));
    }
    let faint = theme.glass.text_faint.0;
    for l in &chart.value_labels {
        label(c, l, with_alpha(faint, faint[3] * r * r));
    }
}

pub(super) fn area(c: &mut Compositor, a: &[f32], b: &[f32], rect: Rect, theme: &Theme, r: f32) {
    let stack = geom::stacked_area(&[a, b], rect);
    for t in &stack.axis.ticks {
        let y = rect.y + rect.h * (1.0 - stack.axis.normalize(*t));
        line_rect(c, rect.x, y, rect.w, 1.0, theme.colors.divider.0);
    }
    // Bands rise from the baseline: thickness scales with the reveal.
    let baseline = rect.y + rect.h;
    let lift = |pts: &[(f32, f32)]| -> Vec<(f32, f32)> {
        pts.iter()
            .map(|&(x, y)| (x, baseline - (baseline - y) * r))
            .collect()
    };
    let (text_c, green) = (theme.colors.text.0, theme.colors.success.0);
    let fills = [with_alpha(text_c, 0.16), with_alpha(green, 0.30)];
    let edges = [with_alpha(text_c, 0.55), with_alpha(green, 0.90)];
    for (i, band) in stack.bands.iter().enumerate() {
        polygon(c, &lift(&band.polygon), fills[i % 2]);
        polyline(c, &lift(&band.top), edges[i % 2], 1.5);
    }
}

pub(super) fn donut(c: &mut Compositor, items: &[(&str, f32)], rect: Rect, theme: &Theme, r: f32) {
    let ty = &theme.typography;
    let d = geom::donut(items, rect, "100%", ty.title(), ty.caption_r());
    let text_c = theme.colors.text.0;
    let alpha_of = |index: usize| DONUT_ALPHAS[index % DONUT_ALPHAS.len()];
    // The ring sweeps open clockwise from 12 o'clock, demo style; the
    // percent labels (fixed at their final spots) fade in with r^2.
    let start0 = d.slices.first().map(|s| s.start).unwrap_or(0.0);
    let fade = r * r;
    for s in &d.slices {
        let (sweep, start) = (s.sweep * r, start0 + (s.start - start0) * r);
        if sweep <= 1e-4 {
            continue;
        }
        let segs = ((sweep / TAU) * 64.0).ceil().max(2.0) as usize;
        let poly = geom::slice_polygon(d.center, d.inner_r, d.outer_r, start, sweep, segs);
        polygon(c, &poly, with_alpha(text_c, alpha_of(s.index)));
        if let Some(percent) = &s.percent {
            // Bright (high-alpha) slices need dark text for contrast.
            let color = if alpha_of(s.index) >= 0.5 {
                with_alpha(theme.colors.bg.0, fade)
            } else {
                with_alpha(text_c, text_c[3] * fade)
            };
            label(c, percent, color);
        }
    }
    label(c, &d.center_label, text_c);
    for item in &d.legend {
        let sw = item.swatch;
        let color = with_alpha(text_c, alpha_of(item.index));
        c.push(rounded_rect(sw.x, sw.y, sw.w, sw.h, 3.0, color));
        label(c, &item.label, theme.colors.text_mid.0);
    }
}

/// Draw a measured label with exactly the style it was measured with
/// (one TextStyle per run: measurement = drawing).
fn label(c: &mut Compositor, l: &geom::Label, color: [f32; 4]) {
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&l.text, &l.style, None),
        x: l.x,
        y: l.y,
        color,
    });
}

/// 1px-thin grid or baseline segment.
fn line_rect(c: &mut Compositor, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
    c.push(SceneNode::Rect { x, y, w, h, color });
}

fn poly_builder(pts: &[(f32, f32)]) -> PathBuilder {
    let mut b = PathBuilder::new().move_to(pts[0].0, pts[0].1);
    for p in &pts[1..] {
        b = b.line_to(p.0, p.1);
    }
    b
}

fn polygon(c: &mut Compositor, pts: &[(f32, f32)], color: [f32; 4]) {
    if pts.len() >= 3 {
        c.draw_path(poly_builder(pts).close().fill(color));
    }
}

fn polyline(c: &mut Compositor, pts: &[(f32, f32)], color: [f32; 4], width: f32) {
    if pts.len() >= 2 {
        c.draw_path(poly_builder(pts).end_open().stroke_round(color, width));
    }
}
