//! The Memory view: what the frame shown did to the process's heap, and
//! what the engine holds on the GPU. The heap is counted block by block
//! when the app installs `perf::heap::Counting` as its allocator: blocks
//! and bytes asked for and given back in the frame, and what stays live,
//! each as a chart over the frames kept. The GPU's side is the engine's
//! own estimate from capacities (glyph atlas, texture pool, layers) as
//! one bar cut by what holds it, with the process's resident memory as
//! the system reports it.

use super::frames;
use super::paint::{Painter, bytes, count, percent, signed_bytes};
use crate::runtime::perf::FrameReport;
use crate::runtime::perf::Recorder;
use crate::widgets::prelude::Rect;

/// The body's width from which the GPU's side sits beside the heap.
const WIDE: f32 = 760.0;
/// The lines a chart is at most tall, its words included.
const CHART_LINES: f32 = 6.0;

/// The view in `body`.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
) {
    let Some(shown) = shown else {
        p.empty(body, "memory-stick", "Waiting for a frame.");
        return;
    };
    let kept = frames(recorder);
    let marked = kept.iter().position(|f| f.index == shown.index);
    let gap = p.gap();
    let (left, right) = if body.w >= WIDE {
        let w = ((body.w - gap * 3.0) / 2.0).round();
        p.rule_v(body.x + w + gap * 1.5, body.y, body.h);
        (
            Rect::new(body.x, body.y, w, body.h),
            Rect::new(body.x + w + gap * 3.0, body.y, w, body.h),
        )
    } else {
        let h = (body.h / 2.0).floor();
        (
            Rect::new(body.x, body.y, body.w, h - gap),
            Rect::new(body.x, body.y + h, body.w, body.h - h),
        )
    };
    heap(p, left, &kept, shown, marked);
    held(p, right, &kept, shown, marked);
}

/// A figure with its name over it and a second line under: one of the
/// numbers a part of the view opens with. `w` is its room.
fn figure(p: &mut Painter<'_>, x: f32, y: f32, w: f32, name: &str, value: &str, under: &str) {
    let (caption, num, small) = (p.caption(), p.num_strong(), p.small());
    p.text(name, x, y, w, p.faint(), &caption);
    p.text(value, x, y + p.line(), w, p.ink(), &num);
    p.text(under, x, y + p.line() * 2.0, w, p.mid(), &small);
}

/// The heap in the frame shown, and over the frames kept.
fn heap(
    p: &mut Painter<'_>,
    area: Rect,
    kept: &[&FrameReport],
    shown: &FrameReport,
    marked: Option<usize>,
) {
    let caption = p.caption();
    let (gap, row) = (p.gap(), p.row());
    let heap = shown.heap;
    let mut y = p.title("Heap", &format!("frame {}", shown.index), area);
    if !heap.counted {
        for line in [
            "This app does not count its heap.",
            "Install perf::heap::Counting as its global allocator for blocks and bytes per frame.",
        ] {
            p.text(line, area.x, y, area.w, p.mid(), &caption);
            y += row;
        }
        return;
    }
    let kept_blocks = heap.allocs as i64 - heap.frees as i64;
    let figures = [
        (
            "asked for",
            bytes(heap.bytes_in),
            format!("{} blocks", count(heap.allocs)),
        ),
        (
            "given back",
            bytes(heap.bytes_out),
            format!("{} blocks", count(heap.frees)),
        ),
        (
            "kept by the frame",
            signed_bytes(heap.net_bytes()),
            format!("{kept_blocks:+} blocks"),
        ),
        (
            "live since the count",
            signed_bytes(heap.live_bytes),
            String::new(),
        ),
    ];
    let each = area.w / figures.len() as f32;
    for (i, (name, value, under)) in figures.iter().enumerate() {
        figure(
            p,
            area.x + each * i as f32,
            y,
            each - gap,
            name,
            value,
            under,
        );
    }
    y += p.line() * 3.0 + gap;

    let color = p.theme.colors.info.0;
    let live = p.theme.colors.success.0;
    let of = |f: &dyn Fn(&FrameReport) -> f64| -> Vec<f64> { kept.iter().map(|k| f(k)).collect() };
    let most = |values: &[f64]| values.iter().copied().fold(0.0, f64::max);
    let asked = of(&|f| f.heap.allocs as f64);
    let size = of(&|f| f.heap.bytes_in as f64);
    let lives = of(&|f| f.heap.live_bytes as f64);
    let low = lives.iter().copied().fold(f64::INFINITY, f64::min);
    let high = lives.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let charts = [
        (
            "blocks asked for per frame",
            count(heap.allocs),
            format!("up to {}", count(most(&asked) as u64)),
            asked,
            color,
        ),
        (
            "bytes asked for per frame",
            bytes(heap.bytes_in),
            format!("up to {}", bytes(most(&size) as u64)),
            size,
            color,
        ),
        (
            "live bytes",
            signed_bytes(heap.live_bytes),
            format!(
                "{} to {}",
                signed_bytes(low as i64),
                signed_bytes(high as i64)
            ),
            lives,
            live,
        ),
    ];
    let each = ((area.bottom() - y) / charts.len() as f32).min(row * CHART_LINES);
    for (name, value, range, values, color) in charts {
        let cell = Rect::new(area.x, y, area.w, each - gap);
        p.plot(cell, name, &value, &range, &values, color, marked);
        y += each;
    }
}

/// What the engine holds on the GPU, and the process's resident memory.
fn held(
    p: &mut Painter<'_>,
    area: Rect,
    kept: &[&FrameReport],
    shown: &FrameReport,
    marked: Option<usize>,
) {
    let (caption, num) = (p.caption(), p.num());
    let (gap, row) = (p.gap(), p.row());
    let memory = shown.memory;
    let total = memory.gpu_total_bytes();
    let note = format!("{} from capacities", bytes(total));
    let mut y = p.title("GPU memory", &note, area);
    let parts = [
        (
            "glyph atlas",
            memory.glyph_atlas_bytes,
            p.theme.colors.info.0,
        ),
        (
            "texture pool",
            memory.texture_pool_bytes,
            p.theme.colors.warning.0,
        ),
        ("layers", memory.layer_bytes, p.theme.colors.accent.0),
    ];
    // one bar, cut by what holds the memory
    let bar = Rect::new(area.x, y, area.w, p.line());
    p.track(bar);
    let mut x = bar.x;
    for (_, size, color) in parts {
        let w = bar.w * (size as f32 / total.max(1) as f32);
        if w >= 1.0 {
            p.bar(Rect::new(x, bar.y, (w - 1.0).max(1.0), bar.h), color);
        }
        x += w;
    }
    y += bar.h + gap;
    let each = area.w / parts.len() as f32;
    let swatch = (p.line() / 2.0).round();
    for (i, (name, size, color)) in parts.iter().enumerate() {
        let x = area.x + each * i as f32;
        p.bar(
            Rect::new(x, y + (p.line() - swatch) / 2.0, swatch, swatch),
            *color,
        );
        let text_x = x + swatch + gap / 2.0;
        p.text(name, text_x, y, each - swatch - gap, p.mid(), &caption);
        let value = format!("{} · {}%", bytes(*size), percent(*size, total));
        p.text(
            &value,
            text_x,
            y + p.line(),
            each - swatch - gap,
            p.ink(),
            &num,
        );
    }
    y += p.line() * 2.0 + gap * 1.5;

    let color = p.theme.colors.accent.0;
    let totals: Vec<f64> = kept
        .iter()
        .map(|f| f.memory.gpu_total_bytes() as f64)
        .collect();
    let high = totals.iter().copied().fold(0.0, f64::max);
    let mut charts = vec![(
        "GPU memory over the frames",
        bytes(total),
        format!("up to {}", bytes(high as u64)),
        totals,
        color,
    )];
    if let Some(rss) = memory.process_rss_bytes {
        let resident: Vec<f64> = kept
            .iter()
            .map(|f| f.memory.process_rss_bytes.unwrap_or(0) as f64)
            .collect();
        let high = resident.iter().copied().fold(0.0, f64::max);
        charts.push((
            "process resident memory",
            bytes(rss),
            format!("up to {}", bytes(high as u64)),
            resident,
            p.theme.colors.success.0,
        ));
    }
    let each = ((area.bottom() - y) / charts.len() as f32).min(row * CHART_LINES);
    for (name, value, range, values, color) in charts {
        let cell = Rect::new(area.x, y, area.w, each - gap);
        p.plot(cell, name, &value, &range, &values, color, marked);
        y += each;
    }
}
