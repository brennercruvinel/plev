//! The GPU view: what the GPU did for the frame shown. Each pass as a bar
//! with the layer it drew (the engine's own timestamp queries, one pair a
//! pass), the frame's counters under them, and beside them every layer of
//! the frame as a table: redrawn, reused or hidden, what it holds, and
//! the GPU's time for it.

use super::paint::{Painter, alpha, bytes, count, time};
use crate::drawing::compositor::LayerId;
use crate::runtime::perf::stages::Stage;
use crate::runtime::perf::{FrameReport, LayerReport, Recorder};
use crate::widgets::prelude::Rect;

/// The body's width from which the layers sit beside the passes.
const WIDE: f32 = 760.0;

/// The view in `body`.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
) {
    let Some(shown) = shown else {
        p.empty(body, "cpu", "Waiting for a frame.");
        return;
    };
    let gap = p.gap();
    let (left, right) = if body.w >= WIDE {
        let w = ((body.w - gap * 3.0) * 0.44).round();
        p.rule_v(body.x + w + gap * 1.5, body.y, body.h);
        (
            Rect::new(body.x, body.y, w, body.h),
            Rect::new(
                body.x + w + gap * 3.0,
                body.y,
                body.w - w - gap * 3.0,
                body.h,
            ),
        )
    } else {
        let h = (body.h / 2.0).floor();
        (
            Rect::new(body.x, body.y, body.w, h - gap),
            Rect::new(body.x, body.y + h, body.w, body.h - h),
        )
    };
    let y = passes(p, left, recorder, shown) + gap;
    counters(p, Rect::new(left.x, y, left.w, left.bottom() - y), shown);
    layers(p, right, shown);
}

/// A layer's name in a table.
fn layer_name(id: LayerId, dev: LayerId) -> String {
    if id == dev {
        format!("layer {} · dev mode", id.0)
    } else {
        format!("layer {}", id.0)
    }
}

/// The passes of the frame as bars; the y under them.
fn passes(p: &mut Painter<'_>, area: Rect, recorder: &Recorder, shown: &FrameReport) -> f32 {
    let (caption, num) = (p.caption(), p.num());
    let row = p.row();
    let gap = p.gap();
    let dev = p.layer;
    let Some(gpu) = &shown.gpu else {
        let y = p.title("Passes", "", area);
        let why = if recorder.gpu_available() {
            "This frame wrote no timestamps: every query slot was in flight."
        } else {
            "This device writes no GPU timestamps (no TIMESTAMP_QUERY on the adapter)."
        };
        p.text(why, area.x, y, area.w, p.mid(), &caption);
        return y + row;
    };
    let note = format!(
        "{} · {} passes",
        gpu.span.map_or_else(|| "no span".to_owned(), time),
        gpu.passes.len() + 1
    );
    let mut y = p.title("Passes", &note, area);

    let known = gpu.layers.unwrap_or(0) + gpu.composite.unwrap_or(0);
    let other = gpu.span.map(|s| s.saturating_sub(known)).filter(|o| *o > 0);
    let mut bars: Vec<(String, Option<u64>)> = gpu
        .passes
        .iter()
        .map(|pass| (layer_name(pass.layer, dev), pass.nanos))
        .collect();
    bars.push(("composite".to_owned(), gpu.composite));
    if other.is_some() {
        bars.push(("untimed and gaps".to_owned(), other));
    }
    let most = bars
        .iter()
        .filter_map(|(_, n)| *n)
        .max()
        .unwrap_or(0)
        .max(1);
    let name_w = bars
        .iter()
        .map(|(name, _)| p.width(name, &caption))
        .fold(0.0, f32::max)
        .min(area.w / 2.0)
        + gap;
    let time_w = p.width("000.00 ms", &num) + gap;
    let track_x = area.x + name_w;
    let track_w = (area.w - name_w - time_w).max(0.0);
    let bar_h = p.line() - 4.0;
    let color = p.theme.colors.warning.0;
    for (name, nanos) in bars {
        if y + row > area.bottom() {
            break;
        }
        p.text(&name, area.x, y, name_w - gap, p.mid(), &caption);
        let track = Rect::new(track_x, y + 2.0, track_w, bar_h);
        p.track(track);
        match nanos {
            Some(nanos) => {
                let w = (track.w * nanos as f32 / most as f32).max(1.0);
                p.bar(Rect::new(track.x, track.y, w, track.h), color);
                p.text_right(&time(nanos), area.right(), y, p.ink(), &num);
            }
            None => p.text_right("no end", area.right(), y, p.faint(), &num),
        }
        y += row;
    }
    y
}

/// What the frame counted, a name and a number a line, in two columns.
fn counters(p: &mut Painter<'_>, area: Rect, shown: &FrameReport) {
    let (caption, num) = (p.caption(), p.num());
    let row = p.row();
    let gap = p.gap();
    if area.h < row * 2.0 {
        return;
    }
    let mut y = p.title("Counters", "", area);
    let stats = &shown.stats;
    let upload = shown.stages.get(Stage::Upload);
    let pairs = [
        ("draw calls", count(u64::from(stats.draw_calls))),
        ("glyphs", count(u64::from(stats.glyphs))),
        ("nodes culled", count(u64::from(stats.nodes_culled))),
        ("quad vertices", count(u64::from(stats.quad_vertices))),
        ("sdf vertices", count(u64::from(stats.sdf_vertices))),
        ("shadow vertices", count(u64::from(stats.shadow_vertices))),
        ("image vertices", count(u64::from(stats.image_vertices))),
        (
            "buffer uploads",
            format!("{} in {} writes", bytes(upload.items), upload.calls),
        ),
        ("glyph atlas", bytes(shown.memory.glyph_atlas_bytes)),
        ("texture pool", bytes(shown.memory.texture_pool_bytes)),
        ("layer textures", bytes(shown.memory.layer_bytes)),
    ];
    let columns = if area.w >= 420.0 { 2 } else { 1 };
    let column_w = (area.w - gap * 2.0 * (columns - 1) as f32) / columns as f32;
    let per_column = pairs.len().div_ceil(columns);
    let top = y;
    for (i, (name, value)) in pairs.iter().enumerate() {
        let x = area.x + (i / per_column) as f32 * (column_w + gap * 2.0);
        y = top + (i % per_column) as f32 * row;
        if y + row > area.bottom() {
            continue;
        }
        p.text(name, x, y, column_w / 2.0, p.mid(), &caption);
        p.text_right(value, x + column_w, y, p.ink(), &num);
    }
}

/// How a layer stands in the frame: its word and its color.
fn state(p: &Painter<'_>, layer: &LayerReport) -> (&'static str, [f32; 4]) {
    match (layer.visible, layer.redrawn) {
        (false, _) => ("hidden", p.faint()),
        (true, true) => ("redrawn", p.theme.colors.warning.0),
        (true, false) => ("reused", p.theme.colors.success.0),
    }
}

/// Every layer of the frame, the top one first.
fn layers(p: &mut Painter<'_>, area: Rect, shown: &FrameReport) {
    let (caption, strong, num) = (p.caption(), p.strong(), p.num());
    let row = p.row() + 2.0;
    let gap = p.gap();
    let dev = p.layer;
    let redrawn = shown.layers.iter().filter(|l| l.redrawn).count();
    let note = format!(
        "{} redrawn · {} reused",
        redrawn,
        shown.layers.len() - redrawn
    );
    let mut y = p.title("Layers", &note, area);

    // the numbers' columns end at these, from the right
    let heads = ["gpu", "fx", "draws", "glyphs", "nodes", "z"];
    let widths = ["000.00 ms", "00", "0000", "000000", "00000", "00000"]
        .map(|sample| p.width(sample, &num) + gap * 1.5);
    let state_w = p.width("redrawn", &p.small()) + gap * 3.0;
    let numbers: f32 = widths.iter().sum();
    let name_w = (area.w - numbers - state_w).max(p.width("layer 00", &caption));
    let mut rights = [0.0; 6];
    let mut right = area.right() - gap / 2.0;
    for (slot, w) in rights.iter_mut().zip(widths) {
        *slot = right;
        right -= w;
    }
    let head = Rect::new(area.x, y, area.w, row);
    p.table_head(head);
    let head_y = y + (row - p.line()) / 2.0;
    p.text(
        "layer",
        area.x + gap / 2.0,
        head_y,
        name_w,
        p.mid(),
        &strong,
    );
    p.text("state", area.x + name_w, head_y, state_w, p.mid(), &strong);
    for (name, right) in heads.iter().zip(rights) {
        p.text_right(name, right, head_y, p.mid(), &strong);
    }
    y += row;

    let mut sorted: Vec<_> = shown.layers.iter().collect();
    sorted.sort_by_key(|l| std::cmp::Reverse(l.z_order));
    for (i, layer) in sorted.into_iter().enumerate() {
        if y + row > area.bottom() {
            break;
        }
        let line = Rect::new(area.x, y, area.w, row);
        if i % 2 == 1 {
            p.fill(line, alpha(p.ink(), 0.02));
        }
        let text_y = y + (row - p.line()) / 2.0;
        let (word, color) = state(p, layer);
        let ink = if layer.visible { p.ink() } else { p.faint() };
        p.text(
            &layer_name(layer.id, dev),
            area.x + gap / 2.0,
            text_y,
            name_w - gap,
            ink,
            &caption,
        );
        p.badge(word, area.x + name_w, text_y, color);
        let gpu: u64 = shown
            .gpu
            .iter()
            .flat_map(|g| g.passes.iter())
            .filter(|pass| pass.layer == layer.id)
            .filter_map(|pass| pass.nanos)
            .sum();
        let cells = [
            if gpu > 0 { time(gpu) } else { "-".to_owned() },
            layer.effects.to_string(),
            layer.draws.to_string(),
            count(u64::from(layer.glyphs)),
            count(u64::from(layer.nodes)),
            layer.z_order.to_string(),
        ];
        for (cell, right) in cells.iter().zip(rights) {
            let quiet = cell == "0" || cell == "-";
            let ink = if quiet { p.faint() } else { p.ink() };
            p.text_right(cell, right, text_y, ink, &num);
        }
        y += row;
    }
}
