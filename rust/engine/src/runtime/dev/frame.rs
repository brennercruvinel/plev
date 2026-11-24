//! The Frame view: the whole frame at a high level. The last frames as
//! bars (CPU under GPU), the frame shown as one bar cut into the phases a
//! frame runs (input, tick, scene, resolve, encode, gpu), and under it
//! each phase with what ran inside it. It answers where a frame's time
//! went before anything finer is opened.

use super::paint::{BUDGET_NS, Painter, alpha, dim, percent, time};
use super::{Strip, gpu_span};
use crate::runtime::perf::stages::Stage;
use crate::runtime::perf::{FrameReport, Phase, Recorder};
use crate::widgets::prelude::Rect;

/// The frames the history shows.
const BARS: usize = 120;
/// The history's height, in caption lines.
const STRIP_LINES: f32 = 3.0;
/// The widest a table gets before the rows flow into another column.
const COLUMNS_MOST: usize = 3;

/// One line of the table under the pipeline.
struct Row {
    /// A phase's color; `None` for what ran inside one.
    color: Option<[f32; 4]>,
    name: String,
    time: String,
    share: String,
    detail: String,
}

/// A frame's whole time: its CPU phases and the GPU's span.
fn whole(frame: &FrameReport) -> u64 {
    frame.pipeline.cpu_nanos + gpu_span(frame).unwrap_or(0)
}

/// What a phase's items count.
fn unit(phase: Phase) -> &'static str {
    match phase {
        Phase::Input => "events",
        Phase::Scene => "nodes",
        Phase::Resolve => "glyphs",
        Phase::Encode => "draw calls",
        Phase::Tick | Phase::Gpu => "",
    }
}

/// The view in `body`; the history's place, for a click to choose a frame.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
) -> Option<Strip> {
    let caption = p.caption();
    let Some(shown) = shown else {
        let waiting = "Waiting for a frame: move the pointer over the page.";
        p.text(waiting, body.x, body.y, body.w, p.mid(), &caption);
        return None;
    };
    let line = p.line();
    let gap = p.gap();

    // the history, the latest at the right
    let mut frames: Vec<&FrameReport> = recorder
        .history()
        .rev()
        .filter(|f| !f.tool)
        .take(BARS)
        .collect();
    frames.reverse();
    let strip = Rect::new(body.x, body.y, body.w, (line * STRIP_LINES).round());
    let most = frames.iter().map(|f| whole(f)).max().unwrap_or(0).max(1);
    let bar_w = strip.w / BARS as f32;
    let first = BARS - frames.len();
    let cpu = p.phase_color(Phase::Scene);
    let gpu = p.phase_color(Phase::Gpu);
    for (i, f) in frames.iter().enumerate() {
        let x = strip.x + (first + i) as f32 * bar_w;
        let w = (bar_w - 1.0).max(1.0);
        let on = f.index == shown.index;
        let cpu_h = (strip.h * f.pipeline.cpu_nanos as f32 / most as f32).max(1.0);
        let gpu_h = strip.h * gpu_span(f).unwrap_or(0) as f32 / most as f32;
        let (cpu_c, gpu_c) = if on { (cpu, gpu) } else { (dim(cpu), dim(gpu)) };
        p.fill(Rect::new(x, strip.bottom() - cpu_h, w, cpu_h), cpu_c);
        p.fill(
            Rect::new(x, strip.bottom() - cpu_h - gpu_h, w, gpu_h),
            gpu_c,
        );
    }
    if most >= BUDGET_NS {
        let y = strip.bottom() - strip.h * BUDGET_NS as f32 / most as f32;
        p.fill(Rect::new(strip.x, y, strip.w, 1.0), p.theme.colors.danger.0);
    }
    let place = Strip {
        rect: strip,
        first,
        frames: frames.iter().map(|f| f.index).collect(),
    };

    // the frame shown, in one line
    let mut y = strip.bottom() + gap;
    let total = whole(shown);
    let redrawn = shown.layers.iter().filter(|l| l.redrawn).count();
    let gpu_text = gpu_span(shown).map_or_else(|| "-".to_owned(), time);
    let headline = format!(
        "frame {} · cpu {} · gpu {} · {}% of 16.6 ms · {} of {} layers redrawn · {} draw calls",
        shown.index,
        time(shown.pipeline.cpu_nanos),
        gpu_text,
        percent(total, BUDGET_NS),
        redrawn,
        shown.layers.len(),
        shown.stats.draw_calls,
    );
    p.text(&headline, body.x, y, body.w, p.ink(), &p.strong());
    y += line + gap;

    // the pipeline: one bar, a segment a phase
    let nanos = |phase: Phase| match phase {
        Phase::Gpu => gpu_span(shown).unwrap_or(0),
        _ => shown.pipeline.phase(phase).nanos,
    };
    let bar = Rect::new(body.x, y, body.w, line + p.theme.spacing.xs);
    p.track(bar);
    let mut x = bar.x;
    for phase in Phase::ALL {
        let w = bar.w * nanos(phase) as f32 / total.max(1) as f32;
        if w <= 0.0 {
            continue;
        }
        let color = p.phase_color(phase);
        p.bar(Rect::new(x, bar.y, (w - 1.0).max(1.0), bar.h), color);
        let label = phase.name();
        if w > p.width(label, &caption) + gap {
            let ink = p.theme.colors.bg.0;
            let at = bar.y + (bar.h - line) / 2.0;
            p.text(label, x + gap / 2.0, at, w - gap, ink, &caption);
        }
        x += w;
    }
    y += bar.h + gap;

    // each phase, and what ran inside it
    let mut rows = Vec::new();
    for phase in Phase::ALL {
        let cost = shown.pipeline.phase(phase);
        let items = match (cost.items, unit(phase)) {
            (0, _) | (_, "") => String::new(),
            (n, unit) => format!("{n} {unit}"),
        };
        rows.push(Row {
            color: Some(p.phase_color(phase)),
            name: phase.name().to_owned(),
            time: time(nanos(phase)),
            share: format!("{}%", percent(nanos(phase), total)),
            detail: items,
        });
        if phase == Phase::Gpu {
            gpu_rows(shown, total, &mut rows);
            continue;
        }
        for (stage, part) in &cost.parts {
            let mut detail = format!("{} calls", part.calls);
            if let (Some(unit), true) = (stage.items(), part.items > 0) {
                detail = format!("{detail} · {} {unit}", part.items);
            }
            rows.push(Row {
                color: None,
                name: stage.name().to_owned(),
                time: time(part.nanos),
                share: format!("{}%", percent(part.nanos, total)),
                detail,
            });
        }
    }
    if shown.pipeline.other_nanos > 0 {
        rows.push(Row {
            color: Some(alpha(p.ink(), 0.25)),
            name: "outside the phases".to_owned(),
            time: time(shown.pipeline.other_nanos),
            share: String::new(),
            detail: stage_note(shown),
        });
    }
    table(p, Rect::new(body.x, y, body.w, body.bottom() - y), &rows);
    Some(place)
}

/// The GPU's rows: the reading of this very frame, or why there is none.
fn gpu_rows(shown: &FrameReport, total: u64, rows: &mut Vec<Row>) {
    let Some(gpu) = &shown.gpu else {
        rows.push(Row {
            color: None,
            name: "no timestamps for this frame".to_owned(),
            time: String::new(),
            share: String::new(),
            detail: String::new(),
        });
        return;
    };
    let mut part = |name: &str, nanos: Option<u64>, detail: String| {
        if let Some(nanos) = nanos {
            rows.push(Row {
                color: None,
                name: name.to_owned(),
                time: time(nanos),
                share: format!("{}%", percent(nanos, total)),
                detail,
            });
        }
    };
    part(
        Stage::GpuLayers.name(),
        gpu.layers,
        format!("{} passes", gpu.layer_passes),
    );
    part(Stage::GpuComposite.name(), gpu.composite, String::new());
    let known = gpu.layers.unwrap_or(0) + gpu.composite.unwrap_or(0);
    let other = gpu.span.map(|span| span.saturating_sub(known));
    part(
        "untimed passes and gaps",
        other.filter(|o| *o > 0),
        String::new(),
    );
}

/// What the frame's own span says, when the app recorded it.
fn stage_note(shown: &FrameReport) -> String {
    let frame = shown.stages.get(Stage::Frame);
    if frame.calls == 0 {
        String::new()
    } else {
        format!("frame {} as the app's loop saw it", time(frame.nanos))
    }
}

/// `rows` in `area`, flowing into as many columns as fit.
fn table(p: &mut Painter<'_>, area: Rect, rows: &[Row]) {
    let caption = p.caption();
    let strong = p.strong();
    let gap = p.gap();
    let row_h = p.row();
    let per_column = (area.h / row_h).floor() as usize;
    if per_column == 0 || rows.is_empty() {
        return;
    }
    let swatch = (p.line() / 2.0).round();
    let indent = swatch + gap * 1.5;
    let longest = ["text measure, cache miss", "untimed passes and gaps"]
        .map(|name| p.width(name, &caption))
        .into_iter()
        .fold(0.0, f32::max);
    let name_w = indent + longest + gap;
    let time_w = p.width("000.00 ms", &caption) + gap;
    let share_w = p.width("100%", &caption) + gap;
    let detail_w = p.width("000 calls · 000000 layout items", &caption);
    let least = name_w + time_w + share_w + detail_w + gap;
    let wanted = rows.len().div_ceil(per_column);
    let fit = ((area.w / least).floor() as usize).clamp(1, COLUMNS_MOST);
    let columns = wanted.clamp(1, fit);
    let column_w = area.w / columns as f32;
    for (i, row) in rows.iter().take(per_column * columns).enumerate() {
        let x = area.x + (i / per_column) as f32 * column_w;
        let y = area.y + (i % per_column) as f32 * row_h;
        let (name_x, style, ink) = match row.color {
            Some(color) => {
                let at = y + (p.line() - swatch) / 2.0;
                p.bar(Rect::new(x, at, swatch, swatch), color);
                (x + swatch + gap / 2.0, &strong, p.ink())
            }
            None => (x + indent, &caption, p.mid()),
        };
        p.text(&row.name, name_x, y, x + name_w - name_x, ink, style);
        p.text_right(&row.time, x + name_w + time_w - gap, y, ink, &caption);
        p.text_right(
            &row.share,
            x + name_w + time_w + share_w - gap,
            y,
            p.mid(),
            &caption,
        );
        let detail_x = x + name_w + time_w + share_w;
        p.text(
            &row.detail,
            detail_x,
            y,
            x + column_w - detail_x - gap,
            p.mid(),
            &caption,
        );
    }
}
