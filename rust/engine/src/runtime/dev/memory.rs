//! The Memory view: what the frame shown did to the process's heap, and
//! what the engine holds on the GPU. The heap is counted block by block
//! when the app installs `perf::heap::Counting` as its allocator: blocks
//! and bytes asked for and given back in the frame, and what stays live,
//! each as a chart over the frames kept. The GPU's side is the engine's
//! own estimate from capacities (glyph atlas, texture pool, layers), with
//! the process's resident memory as the system reports it.

use super::frames;
use super::paint::{Painter, bytes, count, signed_bytes};
use crate::runtime::perf::FrameReport;
use crate::runtime::perf::Recorder;
use crate::widgets::prelude::Rect;

/// The view in `body`.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
) {
    let caption = p.caption();
    let Some(shown) = shown else {
        let waiting = "Waiting for a frame.";
        p.text(waiting, body.x, body.y, body.w, p.mid(), &caption);
        return;
    };
    let kept = frames(recorder);
    let marked = kept.iter().position(|f| f.index == shown.index);
    let gap = p.gap();
    let wide = body.w >= p.width(WIDEST, &caption) * 2.0 + gap * 4.0;
    let (left, right) = if wide {
        let w = (body.w - gap * 2.0) / 2.0;
        (
            Rect::new(body.x, body.y, w, body.h),
            Rect::new(body.x + w + gap * 2.0, body.y, w, body.h),
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

/// The widest line of a column, what sets its least width.
const WIDEST: &str = "given back   0,000,000 blocks   0000.0 MB";

/// The heap in the frame shown, and over the frames kept.
fn heap(
    p: &mut Painter<'_>,
    area: Rect,
    kept: &[&FrameReport],
    shown: &FrameReport,
    marked: Option<usize>,
) {
    let (caption, strong) = (p.caption(), p.strong());
    let (gap, row) = (p.gap(), p.row());
    let title = format!("Heap in frame {}", shown.index);
    p.text(&title, area.x, area.y, area.w, p.ink(), &strong);
    let mut y = area.y + row;
    let heap = shown.heap;
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
    let label_w = p.width("given back", &caption) + gap * 2.0;
    let blocks_right = area.x + label_w + p.width("0,000,000 blocks", &caption);
    let bytes_right = blocks_right + gap * 2.0 + p.width("+0000.0 MB", &caption);
    let kept_blocks = heap.allocs as i64 - heap.frees as i64;
    let lines = [
        (
            "asked for",
            format!("{} blocks", count(heap.allocs)),
            bytes(heap.bytes_in),
        ),
        (
            "given back",
            format!("{} blocks", count(heap.frees)),
            bytes(heap.bytes_out),
        ),
        (
            "kept",
            format!("{kept_blocks:+} blocks"),
            signed_bytes(heap.net_bytes()),
        ),
    ];
    for (label, blocks, size) in lines {
        p.text(label, area.x, y, label_w, p.mid(), &caption);
        p.text_right(&blocks, blocks_right, y, p.ink(), &caption);
        p.text_right(&size, bytes_right, y, p.ink(), &caption);
        y += row;
    }
    y += gap / 2.0;

    let color = p.theme.colors.accent.0;
    let live = p.theme.colors.info.0;
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
            "live bytes since the count began",
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
    let each = ((area.bottom() - y) / charts.len() as f32).min(row * 6.0);
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
    let (caption, strong) = (p.caption(), p.strong());
    let (gap, row) = (p.gap(), p.row());
    let memory = shown.memory;
    let title = format!(
        "GPU memory in frame {}: {} (from capacities)",
        shown.index,
        bytes(memory.gpu_total_bytes())
    );
    p.text(&title, area.x, area.y, area.w, p.ink(), &strong);
    let mut y = area.y + row;
    let label_w = p.width("texture pool", &caption) + gap * 2.0;
    let value_w = p.width("0000.0 MB", &caption) + gap;
    let lane = Rect::new(
        area.x + label_w,
        y,
        (area.w - label_w - value_w).max(0.0),
        p.line(),
    );
    let total = memory.gpu_total_bytes().max(1);
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
    for (name, size, color) in parts {
        p.text(name, area.x, y, label_w, p.mid(), &caption);
        p.track(Rect::new(lane.x, y, lane.w, lane.h));
        let w = lane.w * (size as f32 / total as f32);
        p.bar(Rect::new(lane.x, y, w, lane.h), color);
        p.text_right(&bytes(size), area.right(), y, p.ink(), &caption);
        y += row;
    }
    y += gap / 2.0;

    let color = p.theme.colors.accent.0;
    let totals: Vec<f64> = kept
        .iter()
        .map(|f| f.memory.gpu_total_bytes() as f64)
        .collect();
    let high = totals.iter().copied().fold(0.0, f64::max);
    let mut charts = vec![(
        "GPU memory over the frames",
        bytes(memory.gpu_total_bytes()),
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
    let each = ((area.bottom() - y) / charts.len() as f32).min(row * 6.0);
    for (name, value, range, values, color) in charts {
        let cell = Rect::new(area.x, y, area.w, each - gap);
        p.plot(cell, name, &value, &range, &values, color, marked);
        y += each;
    }
}
