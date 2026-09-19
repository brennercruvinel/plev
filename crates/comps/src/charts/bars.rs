//! bar chart geometry: one rect per value plus a measured value label
//! above each bar. absorbed from draw_bar_chart in
//! examples/makepad_charts/charts.rs; the fixed 0..1 normalization is
//! replaced by a nice-tick axis so bar heights reconstruct their values.

use crate::text::TextStyle;
use crate::ui::widgets::Rect;

use super::{Axis, Label, drop_colliding, format_tick, nice_ticks};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    pub rect: Rect,
    pub value: f32,
    /// rounded-corner radius, already clamped to half the bar width.
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct BarChart {
    pub plot: Rect,
    pub axis: Axis,
    pub bars: Vec<Bar>,
    /// effective gap actually used (shrinks in narrow rects).
    pub gap: f32,
    /// decimated value labels, centered above their bars, index order.
    pub value_labels: Vec<Label>,
}

/// Narrow rects shrink the gap before they shrink bars below this.
const MIN_BAR_W: f32 = 2.0;
/// Maximum corner radius (the demo's 3.0), clamped per bar.
const MAX_RADIUS: f32 = 3.0;
/// Vertical clearance between a bar top and its value label.
const VALUE_GAP: f32 = 4.0;
/// Minimum clearance between two kept value labels.
const LABEL_GAP: f32 = 4.0;
const TICK_TARGET: usize = 5;

/// Pure geometry for a vertical bar chart anchored at the axis floor
/// (zero for non-negative data). empty data yields empty primitives.
pub fn bar_chart(data: &[f32], rect: Rect, gap: f32, label_style: TextStyle) -> BarChart {
    let n = data.len();
    if n == 0 {
        return BarChart {
            plot: rect,
            axis: nice_ticks(0.0, 1.0, TICK_TARGET),
            bars: Vec::new(),
            gap,
            value_labels: Vec::new(),
        };
    }
    let mut hi = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let lo = data.iter().copied().fold(f32::INFINITY, f32::min).min(0.0);
    if hi <= lo {
        // flat data (all zeros): keep the floor anchored at the data,
        // not at a range padded around it, so zero stays zero-height.
        hi = lo + 1.0;
    }
    let axis = nice_ticks(lo, hi, TICK_TARGET);

    // shrink the gap before starving bars; the width invariant
    // n * bar_w + (n + 1) * gap_eff == rect.w always holds.
    let nf = n as f32;
    let gap_eff = if (rect.w - gap * (nf + 1.0)) / nf >= MIN_BAR_W {
        gap
    } else {
        ((rect.w - nf * MIN_BAR_W) / (nf + 1.0)).max(0.0)
    };
    let bar_w = (rect.w - gap_eff * (nf + 1.0)) / nf;

    let baseline = rect.y + rect.h;
    let mut bars = Vec::with_capacity(n);
    let mut labels = Vec::with_capacity(n);
    for (i, &v) in data.iter().enumerate() {
        let bx = rect.x + gap_eff + i as f32 * (bar_w + gap_eff);
        let bh = axis.normalize(v) * rect.h;
        let bar = Rect::new(bx, baseline - bh, bar_w, bh);
        bars.push(Bar {
            rect: bar,
            value: v,
            radius: MAX_RADIUS.min(bar_w / 2.0),
        });
        let label = Label::measured(format_tick(v, axis.step), label_style.clone());
        let cy = bar.y - VALUE_GAP - label.h / 2.0;
        labels.push(label.centered_on(bx + bar_w / 2.0, cy));
    }

    BarChart {
        plot: rect,
        axis,
        bars,
        gap: gap_eff,
        value_labels: drop_colliding(labels, LABEL_GAP),
    }
}

#[cfg(test)]
mod tests {
    use super::super::rects_overlap;
    use super::*;

    fn style() -> TextStyle {
        TextStyle::new(11.0).with_weight(500)
    }

    fn demo_data() -> Vec<f32> {
        vec![0.6, 0.8, 0.45, 0.9, 0.55, 0.75, 0.65, 0.85]
    }

    #[test]
    fn bar_widths_and_gaps_sum_to_the_rect_width() {
        let rect = Rect::new(10.0, 10.0, 800.0, 300.0);
        let chart = bar_chart(&demo_data(), rect, 6.0, style());
        assert_eq!(chart.bars.len(), 8);
        let widths: f32 = chart.bars.iter().map(|b| b.rect.w).sum();
        let total = widths + chart.gap * (chart.bars.len() + 1) as f32;
        assert!(
            (total - rect.w).abs() < 1e-3,
            "bars + gaps must tile the rect: {total} vs {}",
            rect.w
        );
        // bars stay inside the rect and stand on the baseline.
        let baseline = rect.y + rect.h;
        for bar in &chart.bars {
            assert!(bar.rect.x >= rect.x && bar.rect.x + bar.rect.w <= rect.x + rect.w + 1e-3);
            assert!((bar.rect.y + bar.rect.h - baseline).abs() < 1e-3);
            assert!(bar.radius <= bar.rect.w / 2.0);
        }
    }

    #[test]
    fn bar_heights_reconstruct_their_values_against_the_axis() {
        let data = demo_data();
        let rect = Rect::new(0.0, 0.0, 600.0, 250.0);
        let chart = bar_chart(&data, rect, 6.0, style());
        let span = chart.axis.max - chart.axis.min;
        let mut reconstructed_sum = 0.0;
        for (bar, &v) in chart.bars.iter().zip(&data) {
            let back = chart.axis.min + bar.rect.h / rect.h * span;
            assert!((back - v).abs() < 1e-4, "bar height must encode {v}");
            reconstructed_sum += back;
        }
        let expected: f32 = data.iter().sum();
        assert!(
            (reconstructed_sum - expected).abs() < 1e-3,
            "geometry sums back to the data range: {reconstructed_sum} vs {expected}"
        );
    }

    #[test]
    fn empty_and_zero_data_stay_safe() {
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let empty = bar_chart(&[], rect, 6.0, style());
        assert!(empty.bars.is_empty());
        assert!(empty.value_labels.is_empty());

        let zeros = bar_chart(&[0.0, 0.0], rect, 6.0, style());
        for bar in &zeros.bars {
            assert_eq!(bar.rect.h, 0.0, "zero value, zero height");
            assert!((bar.rect.y - (rect.y + rect.h)).abs() < 1e-3);
        }
    }

    #[test]
    fn narrow_rect_shrinks_gaps_keeps_bars_and_decollides_labels() {
        // 8 bars at the demo gap of 6 need 54px of gaps alone; 60px
        // forces the gap to shrink so every bar keeps its minimum width.
        let rect = Rect::new(0.0, 0.0, 60.0, 60.0);
        let chart = bar_chart(&demo_data(), rect, 6.0, style());
        assert!(chart.gap < 6.0, "gap must shrink before bars vanish");
        for bar in &chart.bars {
            assert!(bar.rect.w >= MIN_BAR_W - 1e-3);
        }
        let widths: f32 = chart.bars.iter().map(|b| b.rect.w).sum();
        let total = widths + chart.gap * (chart.bars.len() + 1) as f32;
        assert!((total - rect.w).abs() < 1e-3);

        assert!(!chart.value_labels.is_empty());
        assert!(chart.value_labels.len() < chart.bars.len());
        for (i, a) in chart.value_labels.iter().enumerate() {
            for b in chart.value_labels.iter().skip(i + 1) {
                assert!(
                    !rects_overlap(&a.bounds(), &b.bounds(), 0.0),
                    "value labels '{}' and '{}' overlap in a 90px rect",
                    a.text,
                    b.text
                );
            }
        }
    }
}
