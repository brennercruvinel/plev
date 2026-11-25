//! The Frame view: where a frame's time went, and what that means. The
//! last frames as bars (CPU under GPU) against the budget, the frame
//! shown as one bar cut into the phases a frame runs (input, tick, scene,
//! resolve, encode, gpu) with what ran inside each, and beside them the
//! findings drawn from it (`insights`): the first thing to read before
//! anything finer is opened. A press on a bar of the history shows that
//! frame; a press on a finding opens the view that holds its numbers.

use super::insights::{self, Insight, Severity};
use super::paint::{BUDGET_NS, ICON, Painter, alpha, dim, percent, time};
use super::{Act, Follow, gpu_span, whole};
use crate::runtime::perf::stages::Stage;
use crate::runtime::perf::{FrameReport, Phase, Recorder};
use crate::widgets::prelude::Rect;

/// The frames the history shows.
const BARS: usize = 120;
/// The history's height, and the pipeline bar's.
const STRIP: f32 = 44.0;
const PIPELINE: f32 = 18.0;
/// The widest a table gets before the rows flow into another column.
const COLUMNS_MOST: usize = 2;
/// The body's width from which the findings sit beside the frame.
const WIDE: f32 = 760.0;
/// The findings' share of a wide body.
const INSIGHTS_SHARE: f32 = 0.36;

/// One line of the table under the pipeline.
struct Row {
    /// A phase's color; `None` for what ran inside one.
    color: Option<[f32; 4]>,
    name: String,
    time: String,
    share: String,
    detail: String,
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

/// The view in `body`.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
) {
    let Some(shown) = shown else {
        let waiting = "Waiting for a frame: move the pointer over the page.";
        p.empty(body, "gauge", waiting);
        return;
    };
    let gap = p.gap();
    let found = insights::of(recorder, shown, p.layer);
    if body.w >= WIDE {
        let side = (body.w * INSIGHTS_SHARE).round();
        let left = Rect::new(body.x, body.y, body.w - side - gap * 3.0, body.h);
        let right = Rect::new(body.right() - side, body.y, side, body.h);
        p.rule_v(right.x - gap * 1.5, body.y, body.h);
        let y = history(p, left, recorder, shown);
        let y = pipeline(p, Rect::new(left.x, y, left.w, left.bottom() - y), shown);
        phases(p, Rect::new(left.x, y, left.w, left.bottom() - y), shown);
        findings(p, right, &found, usize::MAX);
    } else {
        let y = history(p, body, recorder, shown);
        let y = pipeline(p, Rect::new(body.x, y, body.w, body.bottom() - y), shown);
        // a narrow panel reads the findings first, the table in what is left
        let rest = Rect::new(body.x, y, body.w, body.bottom() - y);
        let y = findings(p, rest, &found, 4) + gap;
        phases(p, Rect::new(body.x, y, body.w, body.bottom() - y), shown);
    }
}

/// The history, the latest at the right; the y under it.
fn history(p: &mut Painter<'_>, area: Rect, recorder: &Recorder, shown: &FrameReport) -> f32 {
    let gap = p.gap();
    let small = p.small();
    let mut frames: Vec<&FrameReport> = recorder
        .history()
        .rev()
        .filter(|f| !f.tool)
        .take(BARS)
        .collect();
    frames.reverse();
    let note = format!("{} frames · budget 16.6 ms", frames.len());
    let y = p.title("Timeline", &note, area);
    let strip = Rect::new(area.x, y, area.w, STRIP);
    p.track(strip);
    let most = frames.iter().map(|f| whole(f)).max().unwrap_or(0).max(1);
    // the budget always shows: a history all under it reads as low bars
    let scale = most.max(BUDGET_NS + BUDGET_NS / 4);
    let bar_w = strip.w / BARS as f32;
    let first = BARS - frames.len();
    let cpu = p.phase_color(Phase::Scene);
    let gpu = p.phase_color(Phase::Gpu);
    let mut said = None;
    for (i, f) in frames.iter().enumerate() {
        let x = strip.x + (first + i) as f32 * bar_w;
        let w = (bar_w - 1.0).max(1.0);
        let slot = Rect::new(x, strip.y, bar_w, strip.h);
        let hot = p.region(slot, Act::Follow(Follow::Frame(f.index)));
        let on = f.index == shown.index;
        if hot {
            p.wash(slot, 2.0);
            said = Some(format!("frame {} · {}", f.index, time(whole(f))));
        }
        let cpu_h = (strip.h * f.pipeline.cpu_nanos as f32 / scale as f32).max(1.0);
        let gpu_h = strip.h * gpu_span(f).unwrap_or(0) as f32 / scale as f32;
        let (cpu_c, gpu_c) = if on || hot {
            (cpu, gpu)
        } else {
            (dim(cpu), dim(gpu))
        };
        p.fill(Rect::new(x, strip.bottom() - cpu_h, w, cpu_h), cpu_c);
        p.fill(
            Rect::new(x, strip.bottom() - cpu_h - gpu_h, w, gpu_h),
            gpu_c,
        );
        if on {
            let mark = Rect::new(x, strip.bottom() + 1.0, w, 2.0);
            p.fill(mark, p.ink());
        }
    }
    let budget_y = strip.bottom() - strip.h * BUDGET_NS as f32 / scale as f32;
    let danger = p.theme.colors.danger.0;
    p.fill(
        Rect::new(strip.x, budget_y, strip.w, 1.0),
        alpha(danger, 0.7),
    );
    let label = said.unwrap_or_else(|| "16.6 ms".to_owned());
    let label_y = (budget_y - small.line_height).max(strip.y);
    p.text(
        &label,
        strip.x + gap / 2.0,
        label_y,
        strip.w - gap,
        p.mid(),
        &small,
    );
    strip.bottom() + gap * 1.5
}

/// The frame shown as one bar, a segment a phase; the y under it.
fn pipeline(p: &mut Painter<'_>, area: Rect, shown: &FrameReport) -> f32 {
    let gap = p.gap();
    let caption = p.caption();
    let total = whole(shown);
    let gpu_text =
        gpu_span(shown).map_or_else(|| "no gpu time".to_owned(), |n| format!("gpu {}", time(n)));
    let note = format!("cpu {} · {gpu_text}", time(shown.pipeline.cpu_nanos));
    let y = p.title(&format!("Frame {}", shown.index), &note, area);
    let bar = Rect::new(area.x, y, area.w, PIPELINE);
    p.track(bar);
    let mut x = bar.x;
    for phase in Phase::ALL {
        let w = bar.w * phase_nanos(shown, phase) as f32 / total.max(1) as f32;
        if w <= 0.0 {
            continue;
        }
        let color = p.phase_color(phase);
        p.bar(Rect::new(x, bar.y, (w - 1.0).max(1.0), bar.h), color);
        let label = phase.name();
        if w > p.width(label, &caption) + gap {
            let ink = p.theme.colors.bg.0;
            let at = bar.y + (bar.h - p.line()) / 2.0;
            p.text(label, x + gap / 2.0, at, w - gap, ink, &caption);
        }
        x += w;
    }
    bar.bottom() + gap
}

/// A phase's time in `shown`: the GPU's is its own span.
fn phase_nanos(shown: &FrameReport, phase: Phase) -> u64 {
    match phase {
        Phase::Gpu => gpu_span(shown).unwrap_or(0),
        _ => shown.pipeline.phase(phase).nanos,
    }
}

/// Each phase, and what ran inside it.
fn phases(p: &mut Painter<'_>, area: Rect, shown: &FrameReport) {
    let total = whole(shown);
    let mut rows = Vec::new();
    for phase in Phase::ALL {
        let cost = shown.pipeline.phase(phase);
        let nanos = phase_nanos(shown, phase);
        let items = match (cost.items, unit(phase)) {
            (0, _) | (_, "") => String::new(),
            (n, unit) => format!("{n} {unit}"),
        };
        rows.push(Row {
            color: Some(p.phase_color(phase)),
            name: phase.name().to_owned(),
            time: time(nanos),
            share: format!("{}%", percent(nanos, total)),
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
    table(p, area, &rows);
}

/// The GPU's rows: the reading of this very frame, or why there is none.
fn gpu_rows(shown: &FrameReport, total: u64, rows: &mut Vec<Row>) {
    let Some(gpu) = &shown.gpu else {
        rows.push(Row {
            color: None,
            name: "no timestamps".to_owned(),
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
    let (caption, strong, num) = (p.caption(), p.strong(), p.num());
    let gap = p.gap();
    let row_h = p.row();
    let per_column = (area.h / row_h).floor() as usize;
    if per_column == 0 || rows.is_empty() {
        return;
    }
    let swatch = (p.line() / 2.0).round();
    let indent = swatch + gap;
    let longest = ["text measure, cache miss", "untimed passes and gaps"]
        .map(|name| p.width(name, &caption))
        .into_iter()
        .fold(0.0, f32::max);
    let name_w = indent + longest + gap;
    let time_w = p.width("000.00 ms", &num) + gap;
    let share_w = p.width("100%", &num) + gap;
    // the detail is cut to what is left of a column: it sets no width
    let least = name_w + time_w + share_w + p.width("000 calls", &num) + gap;
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
        p.text_right(&row.time, x + name_w + time_w - gap, y, ink, &num);
        p.text_right(
            &row.share,
            x + name_w + time_w + share_w - gap,
            y,
            p.mid(),
            &num,
        );
        let detail_x = x + name_w + time_w + share_w;
        p.text(
            &row.detail,
            detail_x,
            y,
            x + column_w - detail_x - gap,
            p.faint(),
            &num,
        );
    }
}

/// The icon and the color a finding of `severity` is said with.
fn mark(p: &Painter<'_>, severity: Severity) -> (&'static str, [f32; 4]) {
    let colors = &p.theme.colors;
    match severity {
        Severity::Bad => ("circle-x", colors.danger.0),
        Severity::Warn => ("alert-triangle", colors.warning.0),
        Severity::Note => ("info", colors.info.0),
        Severity::Good => ("circle-check", colors.success.0),
    }
}

/// The findings in `area`, `most` of them at most; the y under them.
fn findings(p: &mut Painter<'_>, area: Rect, found: &[Insight], most: usize) -> f32 {
    let (caption, small) = (p.caption(), p.small());
    let gap = p.gap();
    let count_of = |severity| found.iter().filter(|i| i.severity == severity).count();
    let (bad, warn) = (count_of(Severity::Bad), count_of(Severity::Warn));
    let note = match (bad, warn) {
        (0, 0) => "nothing to fix".to_owned(),
        (0, w) => format!("{w} to look at"),
        (b, 0) => format!("{b} to fix"),
        (b, w) => format!("{b} to fix · {w} to look at"),
    };
    let mut y = p.title("Insights", &note, area);
    let each = p.line() * 2.0 + gap;
    let room = ((area.bottom() - y) / each).floor() as usize;
    let fits = room.min(most);
    let shown = if found.len() > fits {
        fits.saturating_sub(1)
    } else {
        found.len()
    };
    for insight in &found[..shown] {
        let at = Rect::new(area.x, y, area.w, each);
        let hot = p.region(at, Act::Tab(insight.tab));
        if hot {
            p.wash(Rect::new(at.x - gap / 2.0, at.y, at.w + gap, at.h), 1.0);
        }
        let (icon, color) = mark(p, insight.severity);
        let top = y + gap / 2.0;
        p.icon(icon, area.x, top + (p.line() - ICON) / 2.0, ICON, color);
        let x = area.x + ICON + gap;
        p.text(&insight.title, x, top, area.right() - x, p.ink(), &caption);
        let detail = top + p.line();
        p.text(
            &insight.detail,
            x,
            detail,
            area.right() - x,
            p.mid(),
            &caption,
        );
        y += each;
        p.rule_h(x, y - 1.0, area.right() - x);
    }
    if shown < found.len() && fits > 0 {
        let more = format!("{} more in a taller panel", found.len() - shown);
        p.text(
            &more,
            area.x + ICON + gap,
            y + gap / 2.0,
            area.w,
            p.faint(),
            &small,
        );
        y += each;
    }
    y
}
