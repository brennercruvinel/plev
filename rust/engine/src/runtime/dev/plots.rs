//! The Plots view: what each frame kept counted, as a chart over the
//! frames. First what the engine counts for every frame: the CPU's time
//! and the GPU's, then the page's draw calls, scene nodes, glyphs and
//! layers redrawn (the dev mode's own layer is left out of those four: a
//! panel that draws more as its history grows would be the curve). Then
//! every value the app's own code gave under a name (`engine::plot!`).
//! The column of the frame shown is drawn in the text color, and its
//! value is the one written beside the name.

use super::paint::{Painter, count, time};
use super::{frames, gpu_span};
use crate::drawing::compositor::LayerId;
use crate::runtime::perf::{FrameReport, LayerReport, Recorder};
use crate::widgets::prelude::Rect;

/// The lines a chart cell is at most tall, its name and its range
/// included.
const CELL_LINES_MOST: f32 = 12.0;
/// The charts a line of them holds at most.
const COLUMNS_MOST: usize = 4;
/// The lines a chart cell is at most tall.
const CELL_LINES_TALL: f32 = 8.0;
/// The lines a chart cell needs at least.
const CELL_LINES_LEAST: f32 = 4.0;

/// How a plot's values read.
#[derive(Clone, Copy)]
enum Unit {
    Time,
    Count,
    /// A value the app gave: as it is.
    Plain,
}

struct Series {
    name: String,
    unit: Unit,
    color: [f32; 4],
    values: Vec<f64>,
}

fn say(unit: Unit, value: f64) -> String {
    match unit {
        Unit::Time => time(value.max(0.0) as u64),
        Unit::Count => count(value.max(0.0).round() as u64),
        Unit::Plain if value.fract() == 0.0 => format!("{value:.0}"),
        Unit::Plain => format!("{value:.2}"),
    }
}

/// The layers of `frame` that are the page's: all but the dev mode's.
fn page(frame: &FrameReport, dev: LayerId) -> impl Iterator<Item = &LayerReport> {
    frame.layers.iter().filter(move |layer| layer.id != dev)
}

/// What the engine counts for every frame.
fn counted(p: &Painter<'_>, kept: &[&FrameReport], gpu: bool) -> Vec<Series> {
    let colors = &p.theme.colors;
    let dev = p.layer;
    let series = |name: &str, unit, color, of: &dyn Fn(&FrameReport) -> f64| Series {
        name: name.to_owned(),
        unit,
        color,
        values: kept.iter().map(|frame| of(frame)).collect(),
    };
    let mut all = vec![series("cpu", Unit::Time, colors.accent.0, &|f| {
        f.pipeline.cpu_nanos as f64
    })];
    if gpu {
        all.push(series("gpu", Unit::Time, colors.warning.0, &|f| {
            gpu_span(f).unwrap_or(0) as f64
        }));
    }
    all.push(series("draw calls", Unit::Count, colors.success.0, &|f| {
        page(f, dev).map(|l| f64::from(l.draws)).sum()
    }));
    all.push(series("scene nodes", Unit::Count, colors.accent.0, &|f| {
        page(f, dev).map(|l| f64::from(l.nodes)).sum()
    }));
    all.push(series("glyphs", Unit::Count, colors.info.0, &|f| {
        page(f, dev).map(|l| f64::from(l.glyphs)).sum()
    }));
    all.push(series(
        "layers redrawn",
        Unit::Count,
        colors.success.0,
        &|f| page(f, dev).filter(|l| l.redrawn).count() as f64,
    ));
    all
}

/// What the app's code gave under a name, in the order the names first
/// came. A frame that gave nothing under a name keeps the value before it.
fn said(p: &Painter<'_>, kept: &[&FrameReport]) -> Vec<Series> {
    let mut names: Vec<&'static str> = Vec::new();
    for point in kept.iter().flat_map(|frame| frame.plots.iter()) {
        if !names.contains(&point.name) {
            names.push(point.name);
        }
    }
    names
        .into_iter()
        .map(|name| {
            let mut last = 0.0;
            let values = kept
                .iter()
                .map(|frame| {
                    if let Some(point) = frame.plots.iter().find(|point| point.name == name) {
                        last = point.value;
                    }
                    last
                })
                .collect();
            Series {
                name: name.to_owned(),
                unit: Unit::Plain,
                color: p.theme.colors.info.0,
                values,
            }
        })
        .collect()
}

/// The view in `body`.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
) {
    let caption = p.caption();
    let kept = frames(recorder);
    if kept.is_empty() {
        p.empty(body, "chart-line", "Waiting for a frame.");
        return;
    }
    let marked = shown.and_then(|shown| kept.iter().position(|f| f.index == shown.index));
    let mut all = counted(p, &kept, recorder.gpu_available());
    all.extend(said(p, &kept));

    let (gap, row) = (p.gap(), p.row());
    let between = gap * 3.0;
    let least_w = p.width("layers redrawn  000.00 ms", &caption) + between;
    let columns = (((body.w + between) / (least_w + between)).floor() as usize)
        .clamp(1, all.len().min(COLUMNS_MOST));
    let rows = all.len().div_ceil(columns);
    let cell_w = (body.w - between * (columns - 1) as f32) / columns as f32;
    let cell_h = (body.h / rows as f32).clamp(
        row * CELL_LINES_LEAST,
        row * CELL_LINES_MOST.min(CELL_LINES_TALL),
    );
    for (i, series) in all.iter().enumerate() {
        let x = body.x + (i % columns) as f32 * (cell_w + between);
        let y = body.y + (i / columns) as f32 * cell_h;
        if y + cell_h - gap > body.bottom() {
            break;
        }
        let value = marked
            .and_then(|at| series.values.get(at))
            .or(series.values.last())
            .copied()
            .unwrap_or(0.0);
        let low = series.values.iter().copied().fold(f64::INFINITY, f64::min);
        let high = series
            .values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let range = format!("{} to {}", say(series.unit, low), say(series.unit, high));
        p.plot(
            Rect::new(x, y, cell_w, cell_h - gap * 1.5),
            &series.name,
            &say(series.unit, value),
            &range,
            &series.values,
            series.color,
            marked,
        );
    }
}
