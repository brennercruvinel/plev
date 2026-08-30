//! donut chart geometry: annular slices, measured center label, percent
//! labels on the ring midline and a legend column. absorbed from
//! draw_pie_chart in examples/makepad_charts/charts.rs; the pie becomes a
//! donut, angles close exactly to a full turn, and every label position
//! comes from `TextMeasurer` via `Label::measured` (real width, real
//! weight), never from char counts.

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::text::TextStyle;
use crate::ui::widgets::Rect;

use super::Label;

#[derive(Debug, Clone)]
pub struct DonutSlice {
    /// index into the caller's item list (color mapping).
    pub index: usize,
    /// start angle in radians; the first slice starts at 12 o'clock.
    pub start: f32,
    pub sweep: f32,
    pub fraction: f32,
    /// percent label centered on the ring midline; `None` when the slice
    /// arc cannot fit the measured text (collision avoidance).
    pub percent: Option<Label>,
}

#[derive(Debug, Clone)]
pub struct LegendItem {
    pub index: usize,
    pub swatch: Rect,
    pub label: Label,
}

#[derive(Debug, Clone)]
pub struct Donut {
    pub center: (f32, f32),
    pub outer_r: f32,
    pub inner_r: f32,
    pub slices: Vec<DonutSlice>,
    /// measured and centered in the hole; the style (size, weight)
    /// travels with it so drawing reuses exactly what was measured.
    pub center_label: Label,
    pub legend: Vec<LegendItem>,
}

/// hole radius as a fraction of the outer radius.
const HOLE: f32 = 0.6;
const SWATCH: f32 = 10.0;
/// gap between a swatch and its legend text.
const SWATCH_GAP: f32 = 8.0;
/// gap between the donut and the legend column.
const LEGEND_PAD: f32 = 16.0;
const ROW_GAP: f32 = 6.0;
/// clearance a percent label needs inside its arc and ring thickness.
const FIT_PAD: f32 = 4.0;

/// Pure geometry for a donut with legend. items with non-positive or
/// non-finite values get no slice (a zero share has no angle) but keep
/// their legend row. a non-positive total yields no slices at all.
pub fn donut(
    items: &[(&str, f32)],
    rect: Rect,
    center_text: &str,
    center_style: TextStyle,
    label_style: TextStyle,
) -> Donut {
    // legend column width comes from the widest measured name.
    let legend_labels: Vec<Label> = items
        .iter()
        .map(|(name, _)| Label::measured(*name, label_style.clone()))
        .collect();
    let widest = legend_labels.iter().map(|l| l.w).fold(0.0, f32::max);
    let legend_w = if items.is_empty() {
        0.0
    } else {
        SWATCH + SWATCH_GAP + widest
    };

    let donut_w = (rect.w - legend_w - LEGEND_PAD).max(0.0);
    let outer_r = (donut_w.min(rect.h) / 2.0).max(0.0);
    let inner_r = outer_r * HOLE;
    let center = (rect.x + donut_w / 2.0, rect.y + rect.h / 2.0);

    // legend rows, vertically centered as a block at the right edge.
    let row_h = legend_labels.iter().map(|l| l.h).fold(SWATCH, f32::max);
    let block_h = items.len() as f32 * row_h + (items.len().saturating_sub(1)) as f32 * ROW_GAP;
    let legend_x = rect.x + rect.w - legend_w;
    let mut legend = Vec::with_capacity(items.len());
    for (i, label) in legend_labels.into_iter().enumerate() {
        let row_y = center.1 - block_h / 2.0 + i as f32 * (row_h + ROW_GAP);
        let swatch = Rect::new(legend_x, row_y + (row_h - SWATCH) / 2.0, SWATCH, SWATCH);
        let label_y = row_y + (row_h - label.h) / 2.0;
        let label = label.at(legend_x + SWATCH + SWATCH_GAP, label_y);
        legend.push(LegendItem {
            index: i,
            swatch,
            label,
        });
    }

    // slices: positive finite values only; the last sweep takes the float
    // residual so the ring closes exactly on a full turn.
    let positive: Vec<(usize, f32)> = items
        .iter()
        .enumerate()
        .filter(|(_, (_, v))| v.is_finite() && *v > 0.0)
        .map(|(i, (_, v))| (i, *v))
        .collect();
    let total: f32 = positive.iter().map(|(_, v)| v).sum();
    let mut slices = Vec::with_capacity(positive.len());
    if total > 0.0 {
        let start0 = -FRAC_PI_2;
        let mut start = start0;
        let last = positive.len() - 1;
        for (k, (index, v)) in positive.iter().enumerate() {
            let fraction = v / total;
            let sweep = if k == last {
                start0 + TAU - start
            } else {
                TAU * fraction
            };
            let mid_r = (inner_r + outer_r) / 2.0;
            let label = Label::measured(format!("{:.0}%", fraction * 100.0), label_style.clone());
            // the measured box must fit the arc length and ring thickness.
            let fits = label.w + FIT_PAD <= sweep * mid_r && label.h + FIT_PAD <= outer_r - inner_r;
            let percent = fits.then(|| {
                let mid = start + sweep / 2.0;
                label.centered_on(center.0 + mid_r * mid.cos(), center.1 + mid_r * mid.sin())
            });
            slices.push(DonutSlice {
                index: *index,
                start,
                sweep,
                fraction,
                percent,
            });
            start += sweep;
        }
    }

    Donut {
        center,
        outer_r,
        inner_r,
        slices,
        center_label: Label::measured(center_text, center_style).centered_on(center.0, center.1),
        legend,
    }
}

/// Tessellate an annular slice into a closed polygon: outer arc forward,
/// inner arc back (a pie wedge when `inner_r` is zero). `segments` is the
/// subdivision of each arc, minimum 1.
pub fn slice_polygon(
    center: (f32, f32),
    inner_r: f32,
    outer_r: f32,
    start: f32,
    sweep: f32,
    segments: usize,
) -> Vec<(f32, f32)> {
    let n = segments.max(1);
    let at = |r: f32, a: f32| (center.0 + r * a.cos(), center.1 + r * a.sin());
    let mut points = Vec::with_capacity(if inner_r > 0.0 { 2 * (n + 1) } else { n + 2 });
    for i in 0..=n {
        points.push(at(outer_r, start + sweep * (i as f32 / n as f32)));
    }
    if inner_r > 0.0 {
        for i in (0..=n).rev() {
            points.push(at(inner_r, start + sweep * (i as f32 / n as f32)));
        }
    } else {
        points.push(center);
    }
    points
}
