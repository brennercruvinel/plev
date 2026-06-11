//! chart geometry core: pure functions that turn data plus a target `Rect`
//! into drawable primitives (polylines, polygons, rects, dot centers,
//! measured label positions). no `Compositor`, no gpu; every output is
//! testable headless.
//!
//! math absorbed from examples/makepad_charts/charts.rs (line with axes,
//! grid and dots; bars; stacked area; donut) and rebuilt content-driven:
//! value ranges come from nice-number ticks (the 1-2-5 granularity family,
//! same idea as plotters' `key_points` in ref/vis/plotters), gutters come
//! from label widths measured via `TextMeasurer`, never from char-count
//! heuristics.

mod area;
mod axis;
mod bars;
mod donut;
mod line;
#[cfg(test)]
mod tests_donut;

pub use area::{AreaBand, StackedArea, stacked_area};
pub use axis::{Axis, format_tick, nice_ticks};
pub use bars::{Bar, BarChart, bar_chart};
pub use donut::{Donut, DonutSlice, LegendItem, donut, slice_polygon};
pub use line::{Dot, LineChart, line_chart};

use plev::text::{TextMeasurer, TextStyle};
use plev::ui::widgets::Rect;

/// A measured text primitive. the `TextStyle` used for measurement travels
/// with the label so the draw site provably reuses the same style (one
/// style per run, measurement = drawing).
#[derive(Debug, Clone)]
pub struct Label {
    pub text: String,
    pub style: TextStyle,
    /// top-left draw position, logical px.
    pub x: f32,
    pub y: f32,
    /// measured extent via `TextMeasurer::measure_styled`.
    pub w: f32,
    pub h: f32,
}

impl Label {
    /// Measure `text` with `style`; position defaults to the origin.
    pub fn measured(text: impl Into<String>, style: TextStyle) -> Self {
        let text = text.into();
        let (w, h) = TextMeasurer::measure_styled(&text, &style, None);
        Self {
            text,
            style,
            x: 0.0,
            y: 0.0,
            w,
            h,
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Top-left such that the measured box centers on `(cx, cy)`.
    pub fn centered_on(self, cx: f32, cy: f32) -> Self {
        let (w, h) = (self.w, self.h);
        self.at(cx - w / 2.0, cy - h / 2.0)
    }

    /// Top-left such that the right edge lands on `right`, vertically
    /// centered on `cy` (axis tick labels left of a plot).
    pub fn right_aligned(self, right: f32, cy: f32) -> Self {
        let (w, h) = (self.w, self.h);
        self.at(right - w, cy - h / 2.0)
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h)
    }
}

/// Whether two rects, each inflated by `gap / 2`, intersect.
pub fn rects_overlap(a: &Rect, b: &Rect, gap: f32) -> bool {
    a.x < b.x + b.w + gap && b.x < a.x + a.w + gap && a.y < b.y + b.h + gap && b.y < a.y + a.h + gap
}

/// Greedy decimation: keep a label only when its bounds clear every kept
/// label by `gap` px. order encodes priority, the first label always
/// survives. used wherever a narrow rect cannot fit every tick or value.
pub fn drop_colliding(labels: Vec<Label>, gap: f32) -> Vec<Label> {
    let mut kept: Vec<Label> = Vec::with_capacity(labels.len());
    for label in labels {
        let b = label.bounds();
        if kept.iter().all(|k| !rects_overlap(&k.bounds(), &b, gap)) {
            kept.push(label);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> TextStyle {
        TextStyle::new(12.0).with_weight(500)
    }

    #[test]
    fn measured_label_has_real_extent_and_carries_its_style() {
        let label = Label::measured("1.000", style());
        assert!(label.w > 0.0 && label.h > 0.0);
        // the exact style used to measure is the one the draw site gets.
        assert_eq!(label.style, style());
        // wider text measures wider (real shaping, not a heuristic).
        let wider = Label::measured("1.000.000", style());
        assert!(wider.w > label.w);
    }

    #[test]
    fn centered_and_right_aligned_anchor_the_measured_box() {
        let c = Label::measured("42", style()).centered_on(100.0, 50.0);
        let b = c.bounds();
        assert!((b.x + b.w / 2.0 - 100.0).abs() < 0.01);
        assert!((b.y + b.h / 2.0 - 50.0).abs() < 0.01);

        let r = Label::measured("42", style()).right_aligned(80.0, 20.0);
        assert!((r.x + r.w - 80.0).abs() < 0.01);
    }

    #[test]
    fn drop_colliding_keeps_first_and_leaves_no_overlap() {
        // labels stacked nearly on top of each other: only spaced survivors.
        let labels: Vec<Label> = (0..10)
            .map(|i| Label::measured("ovrlap", style()).at(0.0, i as f32 * 4.0))
            .collect();
        let kept = drop_colliding(labels, 2.0);
        assert!(!kept.is_empty(), "first label always survives");
        assert!(kept.len() < 10, "dense stack must be decimated");
        for (i, a) in kept.iter().enumerate() {
            for b in kept.iter().skip(i + 1) {
                assert!(
                    !rects_overlap(&a.bounds(), &b.bounds(), 2.0),
                    "kept labels must not collide"
                );
            }
        }
    }

    #[test]
    fn drop_colliding_keeps_everything_when_spaced() {
        let labels: Vec<Label> = (0..5)
            .map(|i| Label::measured("ok", style()).at(0.0, i as f32 * 40.0))
            .collect();
        assert_eq!(drop_colliding(labels, 2.0).len(), 5);
    }
}
