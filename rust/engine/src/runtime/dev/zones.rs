//! The Zones view: the CPU's named zones of the frame shown
//! (`engine::zone!`), thread by thread on one time axis, each zone a bar
//! as long as it took with the zones it holds under it and the lines code
//! said (`engine::message!`) as marks across them; the table of every
//! zone (how many times it ran, its whole time, the time that was its
//! own, its mean and its longest run); and, for the zone chosen in the
//! table, how its runs spread over the frames kept.
//!
//! A zone keeps one color wherever it shows (`Painter::name_color`), so
//! the bar in the timeline, the row in the table and the spread read as
//! the same thing.

use super::Act;
use super::paint::{Painter, alpha, count, percent, time};
use crate::runtime::perf::{FrameReport, Histogram, Recorder};
use crate::widgets::prelude::Rect;
use crate::widgets::widget::contrast_text;

/// How deep a thread's zones are drawn.
const DEPTH_MOST: u32 = 6;
/// The steps a zone's runs are counted in.
const BINS: usize = 32;
/// The timeline's share of its column; the spread takes the rest.
const TIMELINE_SHARE: f32 = 0.55;
/// The body's width from which the table sits beside the timeline.
const WIDE: f32 = 760.0;
/// The marks of the time axis.
const TICKS: u32 = 4;

/// The view in `body`, with `chosen` as the zone whose spread shows (the
/// one with the most time of its own when none is, or when the frame has
/// no such zone).
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
    chosen: Option<&'static str>,
) {
    let Some(shown) = shown else {
        p.empty(body, "flame", "Waiting for a frame.");
        return;
    };
    if shown.zones.is_empty() {
        p.empty(
            body,
            "flame",
            "No zone closed in this frame. Code names one with engine::zone!.",
        );
        return;
    }
    let stats = shown.zones.stats();
    let chosen = chosen
        .filter(|name| stats.iter().any(|stat| stat.name == *name))
        .or(stats.first().map(|stat| stat.name));
    let gap = p.gap();
    let (lanes, runs, rows) = if body.w >= WIDE {
        let w = ((body.w - gap * 3.0) / 2.0).round();
        let h = (body.h * TIMELINE_SHARE).floor();
        p.rule_v(body.x + w + gap * 1.5, body.y, body.h);
        (
            Rect::new(body.x, body.y, w, h - gap),
            Rect::new(body.x, body.y + h, w, body.h - h),
            Rect::new(body.x + w + gap * 3.0, body.y, w, body.h),
        )
    } else {
        let h = (body.h / 3.0).floor();
        (
            Rect::new(body.x, body.y, body.w, h - gap),
            Rect::new(body.x, body.y + h * 2.0, body.w, body.h - h * 2.0),
            Rect::new(body.x, body.y + h, body.w, h - gap),
        )
    };
    timeline(p, lanes, shown, chosen);
    if let Some(name) = chosen {
        spread(p, runs, recorder, name);
    }
    table(p, rows, shown, chosen);
}

/// Every thread's zones on one time axis.
fn timeline(p: &mut Painter<'_>, area: Rect, shown: &FrameReport, chosen: Option<&'static str>) {
    let (caption, small) = (p.caption(), p.small());
    let gap = p.gap();
    let row = p.row();
    let all = || shown.zones.threads.iter().flat_map(|t| t.zones.iter());
    let start = all().map(|z| z.start_ns).min().unwrap_or(0);
    let end = all().map(|z| z.end_ns()).max().unwrap_or(start);
    let span = (end - start).max(1);
    let note = format!(
        "{} zones · {} {} · {}",
        shown.zones.len(),
        shown.zones.threads.len(),
        if shown.zones.threads.len() == 1 {
            "thread"
        } else {
            "threads"
        },
        time(span)
    );
    let mut y = p.title("Timeline", &note, area);
    let label_w = p.width("thread-00", &caption) + gap;
    let lane_x = area.x + label_w;
    let lane_w = (area.w - label_w).max(0.0);
    let per_ns = lane_w / span as f32;

    // the axis: where the time is, in equal steps
    let axis_y = y;
    for i in 0..=TICKS {
        let x = lane_x + lane_w * i as f32 / TICKS as f32;
        let label = time(span * u64::from(i) / u64::from(TICKS));
        if i == TICKS {
            p.text_right(&label, x, axis_y, p.faint(), &small);
        } else {
            p.text(
                &label,
                x + 2.0,
                axis_y,
                lane_w / TICKS as f32,
                p.faint(),
                &small,
            );
        }
    }
    y += small.line_height + 2.0;
    let lanes_top = y;

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
            let x = lane_x + (zone.start_ns - start) as f32 * per_ns;
            let w = (zone.nanos as f32 * per_ns)
                .max(1.0)
                .min(lane_x + lane_w - x);
            let at = Rect::new(x, top, w, row - 1.0);
            let hot = w >= 3.0 && p.region(at, Act::Zone(zone.name));
            let color = p.name_color(zone.name);
            let on = chosen == Some(zone.name);
            let strength = if on || hot { 1.0 } else { 0.72 };
            p.bar(at, alpha(color, strength));
            let ink = contrast_text(color);
            let text_y = top + (at.h - p.line()) / 2.0;
            let label = format!("{} {}", zone.name, time(zone.nanos));
            if w > p.width(&label, &caption) + gap {
                p.text(&label, x + gap / 2.0, text_y, w - gap, ink, &caption);
            } else if w > p.width(zone.name, &caption) + gap {
                p.text(zone.name, x + gap / 2.0, text_y, w - gap, ink, &caption);
            }
        }
        y += row * (deepest + 1) as f32 + gap / 2.0;
    }
    // the axis' lines down the lanes, and what code said at its moment
    for i in 0..=TICKS {
        let x = lane_x + lane_w * i as f32 / TICKS as f32;
        p.fill(
            Rect::new(x.round(), lanes_top, 1.0, y - lanes_top),
            alpha(p.ink(), 0.06),
        );
    }
    let mark = p.theme.colors.danger.0;
    for message in &shown.messages {
        if (start..=end).contains(&message.at_ns) {
            let x = lane_x + (message.at_ns - start) as f32 * per_ns;
            p.fill(Rect::new(x, lanes_top, 1.0, y - lanes_top), mark);
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

/// Every zone of the frame, the one with the most time of its own first,
/// the `chosen` one washed. A press on a row chooses its zone.
fn table(p: &mut Painter<'_>, area: Rect, shown: &FrameReport, chosen: Option<&'static str>) {
    let (caption, strong, num) = (p.caption(), p.strong(), p.num());
    let gap = p.gap();
    let row = p.row() + 2.0;
    let number_w = p.width("000.00 ms", &num) + gap * 1.5;
    let calls_w = p.width("0000", &num) + gap * 1.5;
    let share_w = p.width("100%", &num) + gap * 1.5;
    let swatch = (p.line() / 2.0).round();
    let name_x = area.x + gap / 2.0 + swatch + gap / 2.0;
    let numbers = calls_w + number_w * 4.0 + share_w;
    let name_w = (area.right() - numbers - name_x).max(0.0);
    let heads = ["calls", "total", "self", "self %", "mean", "max"];
    let widths = [calls_w, number_w, number_w, share_w, number_w, number_w];
    let mut rights = [0.0; 6];
    let mut right = name_x + name_w;
    for (slot, w) in rights.iter_mut().zip(widths) {
        right += w;
        *slot = right - gap / 2.0;
    }
    let mut y = area.y;
    p.table_head(Rect::new(area.x, y, area.w, row));
    let head_y = y + (row - p.line()) / 2.0;
    p.text("zone", name_x, head_y, name_w, p.mid(), &strong);
    for (head, right) in heads.iter().zip(rights) {
        p.text_right(head, right, head_y, p.mid(), &strong);
    }
    y += row;
    let stats = shown.zones.stats();
    let whole: u64 = stats.iter().map(|stat| stat.self_ns).sum();
    for stat in stats {
        if y + row > area.bottom() {
            break;
        }
        let line = Rect::new(area.x, y, area.w, row);
        let hot = p.region(line, Act::Zone(stat.name));
        let on = chosen == Some(stat.name);
        if on {
            p.wash(line, 2.0);
        } else if hot {
            p.wash(line, 1.0);
        }
        let text_y = y + (row - p.line()) / 2.0;
        let color = p.name_color(stat.name);
        p.bar(
            Rect::new(
                area.x + gap / 2.0,
                text_y + (p.line() - swatch) / 2.0,
                swatch,
                swatch,
            ),
            color,
        );
        let mean = stat.total_ns / stat.calls.max(1);
        let cells = [
            count(stat.calls),
            time(stat.total_ns),
            time(stat.self_ns),
            format!("{}%", percent(stat.self_ns, whole)),
            time(mean),
            time(stat.max_ns),
        ];
        let style = if on { &strong } else { &caption };
        p.text(stat.name, name_x, text_y, name_w - gap, p.ink(), style);
        for (i, (cell, right)) in cells.iter().zip(rights).enumerate() {
            let ink = if i == 2 { p.ink() } else { p.mid() };
            p.text_right(cell, right, text_y, ink, &num);
        }
        y += row;
    }
}

/// How the runs of the zone `name` spread over the frames kept: its runs
/// counted in equal steps from the shortest to the longest.
fn spread(p: &mut Painter<'_>, area: Rect, recorder: &Recorder, name: &'static str) {
    let (num, small) = (p.num(), p.small());
    let row = p.row();
    let gap = p.gap();
    let runs = Histogram::of(recorder.zone_runs(name), BINS);
    if runs.is_empty() || area.h < row * 4.0 {
        return;
    }
    p.rule_h(area.x, area.y - gap / 2.0, area.w);
    let note = format!("{} runs in the frames kept", count(runs.count));
    let y = p.title(name, &note, area);
    let summary = [
        ("median", runs.median_ns),
        ("mean", runs.mean_ns()),
        ("p95", runs.p95_ns),
    ];
    let mut x = area.x;
    for (label, nanos) in summary {
        let caption = p.caption();
        p.text(label, x, y, area.w, p.faint(), &caption);
        x += p.width(label, &caption) + gap / 2.0;
        let value = time(nanos);
        p.text(&value, x, y, area.w, p.ink(), &num);
        x += p.width(&value, &num) + gap * 1.5;
    }
    let bars = Rect::new(
        area.x,
        y + row,
        area.w,
        area.bottom() - y - row - small.line_height - 2.0,
    );
    p.track(bars);
    let step = bars.w / runs.bins.len() as f32;
    let tallest = runs.tallest().max(1) as f32;
    let color = p.name_color(name);
    for (i, &n) in runs.bins.iter().enumerate() {
        if n > 0 {
            let h = (bars.h * n as f32 / tallest).max(2.0);
            let x = bars.x + i as f32 * step;
            let bar = Rect::new(x, bars.bottom() - h, (step - 1.0).max(1.0), h);
            p.bar(bar, color);
        }
    }
    let under = area.bottom() - small.line_height;
    p.text(
        &time(runs.min_ns),
        area.x,
        under,
        area.w / 2.0,
        p.faint(),
        &small,
    );
    p.text_right(&time(runs.max_ns), area.right(), under, p.faint(), &small);
}
