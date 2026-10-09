//! Chart geometry core: pure functions that turn data plus a target `Rect`
//! into drawable primitives (polylines, polygons, rects, dot centers,
//! measured label positions). No `Compositor`, no GPU; every output is
//! testable headless. The [`draw`] module emits the scene nodes; widgets
//! and app views compose the two.
//!
//! Math absorbed from the makepad_charts demo (line with axes, grid and
//! dots; bars; stacked area; donut) and rebuilt content-driven: value
//! ranges come from nice-number ticks (the 1-2-5 granularity family, same
//! idea as plotters' `key_points`), gutters come from label widths
//! measured via `TextMeasurer`, never from char-count heuristics.
//!
//! Charts carry no widget state of their own: the owning view keeps the
//! (optional) reveal `Tween<f32>` and routes clicks. That is the
//! deliberate contract — a chart is a pure function of (data, rect,
//! reveal), so the same chart animates in a gallery and sits still in a
//! dashboard.

mod area;
mod axis;
mod bars;
mod donut;
mod line;

pub mod draw;
#[cfg(test)]
mod tests;

pub use area::{AreaBand, StackedArea, stacked_area};
pub use axis::{Axis, format_tick, nice_ticks};
pub use bars::{Bar, BarChart, bar_chart};
pub use donut::{Donut, DonutSlice, LegendItem, donut, slice_polygon};
pub use line::{Dot, LineChart, line_chart};

use crate::widgets::widget::Rect;
use crate::drawing::text::{TextMeasurer, TextStyle};

/// A measured text primitive. The `TextStyle` used for measurement travels
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
/// label by `gap` px. Order encodes priority; the first label always
/// survives. Used wherever a narrow rect cannot fit every tick or value.
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
