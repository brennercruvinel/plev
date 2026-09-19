//! Chart drawing: turns the pure geometry from [`crate::charts`] into
//! scene nodes. Every label is drawn with exactly the `TextStyle` it was
//! measured with (the [`Label`](crate::charts::Label) carries it), and the
//! reveal parameter `r` in 0..=1 scales primitives the way the original
//! makepad_charts demo did: the line sweeps open, bars grow, bands rise,
//! the ring sweeps clockwise. `r = 1.0` is the settled state.

use std::f32::consts::TAU;

use crate::charts as geom;
use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::path::PathBuilder;
use crate::text::TextMeasurer;
use crate::theme::Theme;
use crate::ui::widgets::{Rect, rounded_rect};

/// RGBA with overridden alpha (the widgets' `with_alpha` takes a `Color`;
/// chart drawing works in raw RGBA from theme tokens).
fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

/// Monochrome HOFF alpha ramp for donut slices and legend swatches.
const DONUT_ALPHAS: [f32; 5] = [0.90, 0.62, 0.40, 0.24, 0.12];

pub fn line(c: &mut Compositor, data: &[f32], rect: Rect, theme: &Theme, r: f32) {
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

pub fn bars(c: &mut Compositor, data: &[f32], rect: Rect, theme: &Theme, r: f32) {
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

pub fn area(c: &mut Compositor, a: &[f32], b: &[f32], rect: Rect, theme: &Theme, r: f32) {
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
    let fills = [with_alpha(text_c, 0.16), with_alpha(text_c, 0.30)];
    let edges = [with_alpha(text_c, 0.55), with_alpha(green, 0.90)];
    for (i, band) in stack.bands.iter().enumerate() {
        polygon(c, &lift(&band.polygon), fills[i % 2]);
        polyline(c, &lift(&band.top), edges[i % 2], 1.5);
    }
}

pub fn donut(c: &mut Compositor, items: &[(&str, f32)], rect: Rect, theme: &Theme, r: f32) {
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

/// Single score/progress meter: track + proportional fill (the nestui
/// score-bar pattern). `value` is clamped to 0..=1.
pub fn meter(c: &mut Compositor, value: f32, rect: Rect, theme: &Theme) {
    let frac = value.clamp(0.0, 1.0);
    c.push(rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        rect.h / 2.0,
        theme.glass.surface_active.0,
    ));
    if frac > 0.001 {
        c.push(rounded_rect(
            rect.x,
            rect.y,
            rect.w * frac,
            rect.h,
            rect.h / 2.0,
            theme.colors.accent.0,
        ));
    }
}

/// Horizontal labeled bars: name label left, proportional bar, value
/// label right-aligned. Bars scale to the largest value; rows flow from
/// `rect.y` down, `row_h` apart, at most `rect.h` worth of rows. Labels
/// are really measured — the name column width comes from the widest
/// name, never a char count.
pub fn hbars(
    c: &mut Compositor,
    items: &[(String, f64, String)],
    rect: Rect,
    theme: &Theme,
    row_h: f32,
) {
    if items.is_empty() {
        return;
    }
    let style = theme.typography.caption_r();
    let name_w = items
        .iter()
        .map(|(name, _, _)| TextMeasurer::measure_styled(name, &style, None).0)
        .fold(0.0, f32::max)
        + 8.0;
    let value_w = items
        .iter()
        .map(|(_, _, value)| TextMeasurer::measure_styled(value, &style, None).0)
        .fold(0.0, f32::max)
        + 8.0;
    let max = items
        .iter()
        .map(|(_, v, _)| *v)
        .fold(0.0, f64::max)
        .max(1e-9);
    for (i, (name, value, value_label)) in items.iter().enumerate() {
        let y = rect.y + i as f32 * row_h;
        if y + row_h > rect.y + rect.h {
            break;
        }
        let ty = y + (row_h - style.line_height) / 2.0;
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(name, &style, None),
            x: rect.x,
            y: ty,
            color: theme.colors.text_mid.0,
        });
        let bar_x = rect.x + name_w;
        let bar_w = (rect.w - name_w - value_w).max(0.0);
        let bar_h = (row_h * 0.6).min(18.0);
        let bar_y = y + (row_h - bar_h) / 2.0;
        meter(
            c,
            (*value / max) as f32,
            Rect::new(bar_x, bar_y, bar_w, bar_h),
            theme,
        );
        let (vw, _) = TextMeasurer::measure_styled(value_label, &style, None);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(value_label, &style, None),
            x: rect.x + rect.w - vw,
            y: ty,
            color: theme.colors.text_dim.0,
        });
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
