//! line chart geometry: polyline, area-fill polygon, dots, grid lines and
//! measured y-axis tick labels. absorbed from draw_line_chart plus
//! draw_grid in examples/makepad_charts/charts.rs, with the fixed 0..1
//! normalization and constant grid counts replaced by a nice-tick axis and
//! a gutter sized from real text measurement.

use crate::text::TextStyle;
use crate::ui::widgets::Rect;

use super::{Axis, Label, drop_colliding, format_tick, nice_ticks};

/// Marker dot at a data point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dot {
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

/// Everything a view needs to draw a line chart inside a rect.
#[derive(Debug, Clone)]
pub struct LineChart {
    /// inner plot area: the input rect minus the measured label gutter.
    pub plot: Rect,
    pub axis: Axis,
    /// the data polyline, one vertex per sample.
    pub points: Vec<(f32, f32)>,
    /// closed polygon for the soft fill under the line (baseline included).
    pub area: Vec<(f32, f32)>,
    pub dots: Vec<Dot>,
    /// y coordinate of one horizontal grid line per axis tick.
    pub grid_h: Vec<f32>,
    /// x coordinate of one vertical grid line per sample.
    pub grid_v: Vec<f32>,
    /// decimated y tick labels, right-aligned into the gutter.
    pub tick_labels: Vec<Label>,
}

/// Gap between the tick label column and the plot edge.
const GUTTER_GAP: f32 = 8.0;
/// Minimum clearance between two kept tick labels.
const LABEL_GAP: f32 = 4.0;
const TICK_TARGET: usize = 5;

/// Pure geometry for a line chart. fewer than two samples yields empty
/// primitives (a line needs two points); the rect is never split then.
pub fn line_chart(data: &[f32], rect: Rect, dot_r: f32, label_style: TextStyle) -> LineChart {
    if data.len() < 2 {
        return LineChart {
            plot: rect,
            axis: nice_ticks(0.0, 1.0, TICK_TARGET),
            points: Vec::new(),
            area: Vec::new(),
            dots: Vec::new(),
            grid_h: Vec::new(),
            grid_v: Vec::new(),
            tick_labels: Vec::new(),
        };
    }
    let lo = data.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let axis = nice_ticks(lo, hi, TICK_TARGET);

    // gutter from the widest measured tick label, not from char counts.
    let labels: Vec<Label> = axis
        .ticks
        .iter()
        .map(|t| Label::measured(format_tick(*t, axis.step), label_style.clone()))
        .collect();
    let widest = labels.iter().map(|l| l.w).fold(0.0, f32::max);
    let gutter = (widest + GUTTER_GAP).min(rect.w);
    let plot = Rect::new(rect.x + gutter, rect.y, (rect.w - gutter).max(0.0), rect.h);

    let y_of = |v: f32| plot.y + plot.h * (1.0 - axis.normalize(v));
    let step = plot.w / (data.len() - 1) as f32;
    let points: Vec<(f32, f32)> = data
        .iter()
        .enumerate()
        .map(|(i, v)| (plot.x + i as f32 * step, y_of(*v)))
        .collect();

    let baseline = plot.y + plot.h;
    let mut area = Vec::with_capacity(points.len() + 2);
    area.push((plot.x, baseline));
    area.extend(points.iter().copied());
    area.push((plot.x + plot.w, baseline));

    let dots = points
        .iter()
        .map(|&(x, y)| Dot { x, y, r: dot_r })
        .collect();
    let grid_h: Vec<f32> = axis.ticks.iter().map(|t| y_of(*t)).collect();
    let grid_v: Vec<f32> = points.iter().map(|p| p.0).collect();

    // position labels at their tick line, then decimate collisions
    // (short rects keep a readable subset instead of overprinting).
    let placed: Vec<Label> = labels
        .into_iter()
        .zip(grid_h.iter())
        .map(|(l, y)| l.right_aligned(plot.x - GUTTER_GAP, *y))
        .collect();
    let tick_labels = drop_colliding(placed, LABEL_GAP);

    LineChart {
        plot,
        axis,
        points,
        area,
        dots,
        grid_h,
        grid_v,
        tick_labels,
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
        vec![
            0.3, 0.5, 0.4, 0.7, 0.6, 0.9, 0.75, 0.85, 0.65, 0.95, 0.8, 0.92,
        ]
    }

    #[test]
    fn polyline_spans_the_plot_and_maps_values_monotonically() {
        let rect = Rect::new(10.0, 20.0, 800.0, 300.0);
        let chart = line_chart(&demo_data(), rect, 3.0, style());

        assert_eq!(chart.points.len(), 12);
        let first = chart.points.first().unwrap();
        let last = chart.points.last().unwrap();
        assert!((first.0 - chart.plot.x).abs() < 0.01);
        assert!((last.0 - (chart.plot.x + chart.plot.w)).abs() < 0.01);
        for &(x, y) in &chart.points {
            assert!(x >= chart.plot.x - 0.01 && x <= chart.plot.x + chart.plot.w + 0.01);
            assert!(y >= chart.plot.y - 0.01 && y <= chart.plot.y + chart.plot.h + 0.01);
        }
        // bigger value, smaller y: 0.95 (index 9) above 0.3 (index 0).
        assert!(chart.points[9].1 < chart.points[0].1);
        // dots ride exactly on the polyline.
        assert_eq!(chart.dots.len(), chart.points.len());
        for (dot, p) in chart.dots.iter().zip(&chart.points) {
            assert_eq!((dot.x, dot.y), *p);
            assert_eq!(dot.r, 3.0);
        }
    }

    #[test]
    fn area_polygon_closes_on_the_baseline() {
        let rect = Rect::new(0.0, 0.0, 600.0, 200.0);
        let chart = line_chart(&demo_data(), rect, 2.0, style());
        let baseline = chart.plot.y + chart.plot.h;
        let first = chart.area.first().unwrap();
        let last = chart.area.last().unwrap();
        assert_eq!(first.1, baseline);
        assert_eq!(last.1, baseline);
        assert_eq!(chart.area.len(), chart.points.len() + 2);
    }

    #[test]
    fn grid_follows_ticks_and_samples() {
        let rect = Rect::new(0.0, 0.0, 600.0, 200.0);
        let chart = line_chart(&demo_data(), rect, 2.0, style());
        assert_eq!(chart.grid_h.len(), chart.axis.ticks.len());
        assert_eq!(chart.grid_v.len(), 12);
        // axis bounds land on the plot edges.
        assert!((chart.grid_h.first().unwrap() - (chart.plot.y + chart.plot.h)).abs() < 0.01);
        assert!((chart.grid_h.last().unwrap() - chart.plot.y).abs() < 0.01);
    }

    #[test]
    fn too_few_samples_yield_empty_primitives_not_panics() {
        for data in [vec![], vec![0.5]] {
            let chart = line_chart(&data, Rect::new(0.0, 0.0, 100.0, 50.0), 2.0, style());
            assert!(chart.points.is_empty());
            assert!(chart.area.is_empty());
            assert!(chart.dots.is_empty());
            assert!(chart.tick_labels.is_empty());
        }
    }

    #[test]
    fn tick_labels_do_not_collide_in_a_narrow_short_rect() {
        let rect = Rect::new(0.0, 0.0, 120.0, 60.0);
        let chart = line_chart(&demo_data(), rect, 2.0, style());
        assert!(!chart.tick_labels.is_empty());
        for (i, a) in chart.tick_labels.iter().enumerate() {
            for b in chart.tick_labels.iter().skip(i + 1) {
                assert!(
                    !rects_overlap(&a.bounds(), &b.bounds(), 0.0),
                    "labels '{}' and '{}' overlap in a 120x60 rect",
                    a.text,
                    b.text
                );
            }
        }
        assert!(chart.plot.w >= 0.0);
    }
}
