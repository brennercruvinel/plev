//! The Zones view: the CPU's named zones of the frame shown
//! (`engine::zone!`), thread by thread on one time axis, each zone a bar
//! as long as it took with the zones it holds under it and the lines code
//! said (`engine::message!`) as marks across them; the table of every
//! zone (how many times it ran, its whole time, the time that was its
//! own, its mean and its longest run); and, for the zone chosen in the
//! table, how its runs spread over the frames kept.

use super::paint::{Painter, alpha, count, time};
use crate::runtime::perf::{FrameReport, Histogram, Recorder};
use crate::widgets::prelude::Rect;

/// How deep a thread's zones are drawn.
const DEPTH_MOST: u32 = 6;
/// A zone's opacity at the top, and what each level under it loses.
const TOP: f32 = 0.9;
const STEP: f32 = 0.12;
/// The steps a zone's runs are counted in.
const BINS: usize = 32;
/// The timeline's share of its column; the spread takes the rest.
const TIMELINE_SHARE: f32 = 0.5;

/// A row of the table, for a press to choose its zone.
pub(super) type Rows = Vec<(Rect, &'static str)>;

/// The view in `body`, with `chosen` as the zone whose spread shows (the
/// one with the most time of its own when none is, or when the frame has
/// no such zone). Returns the table's rows.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
    chosen: Option<&'static str>,
) -> Rows {
    let caption = p.caption();
    let Some(shown) = shown else {
        p.text(
            "Waiting for a frame.",
            body.x,
            body.y,
            body.w,
            p.mid(),
            &caption,
        );
        return Rows::new();
    };
    if shown.zones.is_empty() {
        let none = "No zone closed in this frame.";
        p.text(none, body.x, body.y, body.w, p.mid(), &caption);
        return Rows::new();
    }
    let stats = shown.zones.stats();
    let chosen = chosen
        .filter(|name| stats.iter().any(|stat| stat.name == *name))
        .or(stats.first().map(|stat| stat.name));
    let gap = p.gap();
    let wide = body.w >= table_least(p) * 2.0 + gap * 2.0;
    let (lanes, runs, rows) = if wide {
        let w = (body.w - gap * 2.0) / 2.0;
        let h = (body.h * TIMELINE_SHARE).floor();
        (
            Rect::new(body.x, body.y, w, h - gap),
            Rect::new(body.x, body.y + h, w, body.h - h),
            Rect::new(body.x + w + gap * 2.0, body.y, w, body.h),
        )
    } else {
        let h = (body.h / 3.0).floor();
        (
            Rect::new(body.x, body.y, body.w, h - gap),
            Rect::new(body.x, body.y + h * 2.0, body.w, body.h - h * 2.0),
            Rect::new(body.x, body.y + h, body.w, h - gap),
        )
    };
    timeline(p, lanes, shown);
    if let Some(name) = chosen {
        spread(p, runs, recorder, name);
    }
    table(p, rows, shown, chosen)
}

/// Every thread's zones on one time axis.
fn timeline(p: &mut Painter<'_>, area: Rect, shown: &FrameReport) {
    let caption = p.caption();
    let strong = p.strong();
    let gap = p.gap();
    let row = p.row();
    let all = || shown.zones.threads.iter().flat_map(|t| t.zones.iter());
    let start = all().map(|z| z.start_ns).min().unwrap_or(0);
    let end = all().map(|z| z.end_ns()).max().unwrap_or(start);
    let span = (end - start).max(1);
    let title = format!(
        "{} zones on {} threads over {}",
        shown.zones.len(),
        shown.zones.threads.len(),
        time(span)
    );
    p.text(&title, area.x, area.y, area.w, p.ink(), &strong);
    let mut y = area.y + row;
    let label_w = p.width("thread-0000", &caption) + gap;
    let lane = Rect::new(area.x + label_w, y, (area.w - label_w).max(0.0), 0.0);
    let per_ns = lane.w / span as f32;
    let color = p.theme.colors.accent.0;
    let ink = p.theme.colors.bg.0;
    for thread in &shown.zones.threads {
        if y + row > area.bottom() {
            break;
        }
        p.text(&thread.name, area.x, y, label_w - gap, p.mid(), &caption);
        let mut deepest = 0;
        for zone in &thread.zones {
            if zone.depth >= DEPTH_MOST {
                continue;
            }
            let top = y + row * zone.depth as f32;
            if top + row > area.bottom() {
                continue;
            }
            deepest = deepest.max(zone.depth);
            let x = lane.x + (zone.start_ns - start) as f32 * per_ns;
            let w = (zone.nanos as f32 * per_ns).max(1.0).min(lane.right() - x);
            let fill = alpha(color, TOP - STEP * zone.depth as f32);
            p.bar(Rect::new(x, top, w, p.line()), fill);
            let label = format!("{} {}", zone.name, time(zone.nanos));
            if w > p.width(&label, &caption) + gap {
                p.text(&label, x + gap / 2.0, top, w - gap, ink, &caption);
            } else if w > p.width(zone.name, &caption) + gap {
                p.text(zone.name, x + gap / 2.0, top, w - gap, ink, &caption);
            }
        }
        y += row * (deepest + 1) as f32 + gap / 2.0;
    }
    // what code said in the frame, at its moment across every lane
    let mark = p.theme.colors.warning.0;
    let thin = p.theme.spacing.xs / 2.0;
    for message in &shown.messages {
        if (start..=end).contains(&message.at_ns) {
            let x = lane.x + (message.at_ns - start) as f32 * per_ns;
            p.fill(Rect::new(x, lane.y, thin, y - lane.y), mark);
        }
    }
    if shown.zones.dropped > 0 && y + row <= area.bottom() {
        let lost = format!(
            "{} zones over a thread's limit were not kept",
            shown.zones.dropped
        );
        p.text(&lost, area.x, y, area.w, p.theme.colors.danger.0, &caption);
    }
}

/// The least width the table reads at: a long zone name and its five
/// numbers. Under twice this the view stacks its parts.
fn table_least(p: &Painter<'_>) -> f32 {
    let caption = p.caption();
    let gap = p.gap();
    let number_w = p.width("000.00 ms", &caption) + gap;
    let calls_w = p.width("calls 0000", &caption) + gap;
    p.width("encode composite", &caption) + gap + calls_w + number_w * 4.0
}

/// Every zone of the frame, the one with the most time of its own first,
/// the `chosen` one on a track. Returns each row's place and zone.
fn table(
    p: &mut Painter<'_>,
    area: Rect,
    shown: &FrameReport,
    chosen: Option<&'static str>,
) -> Rows {
    let caption = p.caption();
    let strong = p.strong();
    let gap = p.gap();
    let row = p.row();
    let number_w = p.width("000.00 ms", &caption) + gap;
    let calls_w = p.width("calls 0000", &caption) + gap;
    let name_w = (area.w - calls_w - number_w * 4.0).max(0.0);
    let heads = ["calls", "total", "self", "mean", "max"];
    let rights: Vec<f32> = (0..heads.len())
        .map(|i| area.x + name_w + calls_w + number_w * i as f32)
        .collect();
    let mut y = area.y;
    p.text("zone", area.x, y, name_w, p.mid(), &strong);
    for (head, right) in heads.iter().zip(&rights) {
        p.text_right(head, *right, y, p.mid(), &strong);
    }
    y += row;
    let mut rows = Rows::new();
    for stat in shown.zones.stats() {
        if y + row > area.bottom() {
            break;
        }
        let place = Rect::new(area.x, y, area.w, row);
        if chosen == Some(stat.name) {
            p.track(Rect::new(area.x, y, area.w, p.line()));
        }
        rows.push((place, stat.name));
        let mean = stat.total_ns / stat.calls.max(1);
        let cells = [
            stat.calls.to_string(),
            time(stat.total_ns),
            time(stat.self_ns),
            time(mean),
            time(stat.max_ns),
        ];
        p.text(stat.name, area.x, y, name_w - gap, p.ink(), &caption);
        for (cell, right) in cells.iter().zip(&rights) {
            p.text_right(cell, *right, y, p.mid(), &caption);
        }
        y += row;
    }
    rows
}

/// How the runs of the zone `name` spread over the frames kept: its runs
/// counted in equal steps from the shortest to the longest.
fn spread(p: &mut Painter<'_>, area: Rect, recorder: &Recorder, name: &'static str) {
    let (caption, strong) = (p.caption(), p.strong());
    let row = p.row();
    let runs = Histogram::of(recorder.zone_runs(name), BINS);
    if runs.is_empty() || area.h < row * 4.0 {
        return;
    }
    let title = format!("{name}: {} runs in the frames kept", count(runs.count));
    p.text(&title, area.x, area.y, area.w, p.ink(), &strong);
    let summary = format!(
        "median {} · mean {} · p95 {}",
        time(runs.median_ns),
        time(runs.mean_ns()),
        time(runs.p95_ns)
    );
    p.text(&summary, area.x, area.y + row, area.w, p.mid(), &caption);
    let bars = Rect::new(area.x, area.y + row * 2.0, area.w, area.h - row * 3.0);
    p.track(bars);
    let step = bars.w / runs.bins.len() as f32;
    let tallest = runs.tallest().max(1) as f32;
    let color = p.theme.colors.accent.0;
    for (i, &n) in runs.bins.iter().enumerate() {
        if n > 0 {
            let h = (bars.h * n as f32 / tallest).max(p.theme.spacing.xs / 2.0);
            let x = bars.x + i as f32 * step;
            let bar = Rect::new(x, bars.bottom() - h, (step - 1.0).max(1.0), h);
            p.bar(bar, color);
        }
    }
    let under = area.bottom() - p.line();
    p.text(
        &time(runs.min_ns),
        area.x,
        under,
        area.w / 2.0,
        p.mid(),
        &caption,
    );
    p.text_right(&time(runs.max_ns), area.right(), under, p.mid(), &caption);
}
