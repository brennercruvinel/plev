//! stacked area geometry: cumulative bands between consecutive series.
//! the makepad demo faked stacking by overdrawing two translucent fills;
//! here each band is a real polygon between cumulative sums, so bands
//! tile exactly and every thickness reconstructs its value.

use crate::ui::widgets::Rect;

use super::{Axis, nice_ticks};

/// One band of the stack, bottom-up order.
#[derive(Debug, Clone, PartialEq)]
pub struct AreaBand {
    /// lower edge, left to right (band 0 sits on the baseline).
    pub bottom: Vec<(f32, f32)>,
    /// upper edge, left to right.
    pub top: Vec<(f32, f32)>,
    /// closed fill polygon: bottom edge forward, top edge reversed.
    pub polygon: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackedArea {
    pub plot: Rect,
    pub axis: Axis,
    pub bands: Vec<AreaBand>,
}

const TICK_TARGET: usize = 5;

/// Pure geometry for a stacked area chart. series of unequal length are
/// truncated to the shortest; fewer than two shared samples (or no
/// series) yields no bands.
pub fn stacked_area(series: &[&[f32]], rect: Rect) -> StackedArea {
    let n = series.iter().map(|s| s.len()).min().unwrap_or(0);
    if series.is_empty() || n < 2 {
        return StackedArea {
            plot: rect,
            axis: nice_ticks(0.0, 1.0, TICK_TARGET),
            bands: Vec::new(),
        };
    }

    // the axis must hold the tallest stacked total, not the tallest series.
    let max_total = (0..n)
        .map(|i| series.iter().map(|s| s[i]).sum::<f32>())
        .fold(f32::NEG_INFINITY, f32::max);
    let axis = nice_ticks(0.0, max_total, TICK_TARGET);

    let step = rect.w / (n - 1) as f32;
    let y_of = |v: f32| rect.y + rect.h * (1.0 - axis.normalize(v));
    let edge = |cumulative: &[f32]| -> Vec<(f32, f32)> {
        cumulative
            .iter()
            .enumerate()
            .map(|(i, v)| (rect.x + i as f32 * step, y_of(*v)))
            .collect()
    };

    let mut cumulative = vec![0.0_f32; n];
    let mut bottom_edge = edge(&cumulative);
    let mut bands = Vec::with_capacity(series.len());
    for s in series {
        for (acc, v) in cumulative.iter_mut().zip(s.iter()) {
            *acc += v;
        }
        let top_edge = edge(&cumulative);
        let mut polygon = bottom_edge.clone();
        polygon.extend(top_edge.iter().rev().copied());
        bands.push(AreaBand {
            bottom: bottom_edge,
            top: top_edge.clone(),
            polygon,
        });
        bottom_edge = top_edge;
    }

    StackedArea {
        plot: rect,
        axis,
        bands,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_series() -> (Vec<f32>, Vec<f32>) {
        (
            vec![0.2, 0.35, 0.3, 0.5, 0.45, 0.6, 0.55, 0.7, 0.65, 0.75],
            vec![0.1, 0.2, 0.15, 0.3, 0.25, 0.35, 0.3, 0.4, 0.35, 0.45],
        )
    }

    #[test]
    fn bands_tile_without_gaps_and_stay_in_the_plot() {
        let (a, b) = demo_series();
        let rect = Rect::new(10.0, 20.0, 600.0, 200.0);
        let stack = stacked_area(&[&a, &b], rect);
        assert_eq!(stack.bands.len(), 2);
        // band 0 sits on the baseline; band 1 starts where band 0 ends.
        let baseline = rect.y + rect.h;
        for &(_, y) in &stack.bands[0].bottom {
            assert!((y - baseline).abs() < 1e-3);
        }
        assert_eq!(
            stack.bands[0].top, stack.bands[1].bottom,
            "no gap, no overlap"
        );
        for band in &stack.bands {
            assert_eq!(band.polygon.len(), band.top.len() + band.bottom.len());
            for &(x, y) in &band.polygon {
                assert!(x >= rect.x - 1e-3 && x <= rect.x + rect.w + 1e-3);
                assert!(y >= rect.y - 1e-3 && y <= baseline + 1e-3);
            }
        }
    }

    #[test]
    fn band_thickness_reconstructs_each_value() {
        let (a, b) = demo_series();
        let rect = Rect::new(0.0, 0.0, 450.0, 180.0);
        let stack = stacked_area(&[&a, &b], rect);
        let span = stack.axis.max - stack.axis.min;
        for (band, series) in stack.bands.iter().zip([&a, &b]) {
            for ((bottom, top), &v) in band.bottom.iter().zip(&band.top).zip(series.iter()) {
                let thickness = bottom.1 - top.1;
                let back = thickness / rect.h * span;
                assert!((back - v).abs() < 1e-4, "thickness must encode {v}");
            }
        }
    }

    #[test]
    fn axis_covers_the_stacked_total_not_one_series() {
        let (a, b) = demo_series();
        let stack = stacked_area(&[&a, &b], Rect::new(0.0, 0.0, 100.0, 100.0));
        // max total is 0.75 + 0.45 = 1.2; one series alone peaks at 0.75.
        assert!(stack.axis.max >= 1.2);
    }

    #[test]
    fn degenerate_input_yields_no_bands() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(stacked_area(&[], rect).bands.is_empty());
        let short: Vec<f32> = vec![1.0];
        let full: Vec<f32> = vec![1.0, 2.0, 3.0];
        // one series is shorter than two samples: truncation empties all.
        assert!(stacked_area(&[&full, &short], rect).bands.is_empty());
    }

    #[test]
    fn unequal_series_truncate_to_the_shortest() {
        let long: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let short: Vec<f32> = vec![1.0, 1.0, 1.0];
        let stack = stacked_area(&[&long, &short], Rect::new(0.0, 0.0, 90.0, 90.0));
        assert_eq!(stack.bands.len(), 2);
        for band in &stack.bands {
            assert_eq!(band.top.len(), 3);
        }
    }
}
