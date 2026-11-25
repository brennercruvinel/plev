//! The GPU view: what the GPU did for the frame shown. Each pass as a bar
//! with the layer it drew (the engine's own timestamp queries, one pair a
//! pass), then every layer of the frame (redrawn, reused or hidden, with
//! what it holds), the frame's counters and the memory the engine keeps
//! on the GPU.

use super::paint::{Painter, bytes, dim, time};
use crate::drawing::compositor::LayerId;
use crate::runtime::perf::stages::Stage;
use crate::runtime::perf::{FrameReport, Recorder};
use crate::widgets::prelude::Rect;

/// The view in `body`. `dev` is the layer the dev mode itself draws on.
pub(super) fn draw(
    p: &mut Painter<'_>,
    body: Rect,
    recorder: &Recorder,
    shown: Option<&FrameReport>,
    dev: LayerId,
) {
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
        return;
    };
    let gap = p.gap();
    let row = p.row();
    let wide = body.w >= p.width(HEADER, &caption) * 2.0 + gap * 4.0;
    let (left, right) = if wide {
        let w = (body.w - gap * 2.0) / 2.0;
        (
            Rect::new(body.x, body.y, w, body.h),
            Rect::new(body.x + w + gap * 2.0, body.y, w, body.h),
        )
    } else {
        let h = (body.h / 2.0).floor();
        (
            Rect::new(body.x, body.y, body.w, h),
            Rect::new(body.x, body.y + h, body.w, body.h - h),
        )
    };

    let mut y = passes(p, left, recorder, shown, dev);
    y += gap;
    let counters = [
        format!(
            "draw calls {} · glyphs {} · nodes culled {}",
            shown.stats.draw_calls, shown.stats.glyphs, shown.stats.nodes_culled
        ),
        format!(
            "vertices: quad {} · sdf {} · shadow {} · image {}",
            shown.stats.quad_vertices,
            shown.stats.sdf_vertices,
            shown.stats.shadow_vertices,
            shown.stats.image_vertices
        ),
        format!(
            "uploads {} in {} writes",
            bytes(shown.stages.get(Stage::Upload).items),
            shown.stages.get(Stage::Upload).calls
        ),
        format!(
            "memory: atlas {} · pool {} · layers {} · process {}",
            bytes(shown.memory.glyph_atlas_bytes),
            bytes(shown.memory.texture_pool_bytes),
            bytes(shown.memory.layer_bytes),
            shown
                .memory
                .process_rss_bytes
                .map_or_else(|| "-".to_owned(), bytes),
        ),
    ];
    for text in counters {
        if y + row > left.bottom() {
            break;
        }
        p.text(&text, left.x, y, left.w, p.mid(), &caption);
        y += row;
    }

    layers(p, right, shown, dev);
}

/// A layer's name in a table.
fn layer_name(id: LayerId, dev: LayerId) -> String {
    if id == dev {
        format!("layer {} (dev mode)", id.0)
    } else {
        format!("layer {}", id.0)
    }
}

/// The passes of the frame as bars; where the next thing goes.
fn passes(
    p: &mut Painter<'_>,
    area: Rect,
    recorder: &Recorder,
    shown: &FrameReport,
    dev: LayerId,
) -> f32 {
    let caption = p.caption();
    let strong = p.strong();
    let row = p.row();
    let gap = p.gap();
    let mut y = area.y;
    let Some(gpu) = &shown.gpu else {
        let why = if recorder.gpu_available() {
            "No timestamps for this frame yet: they are read back a frame later."
        } else {
            "This device writes no GPU timestamps (launch with PLEV_DEV=1 on an adapter with TIMESTAMP_QUERY)."
        };
        p.text(why, area.x, y, area.w, p.mid(), &caption);
        return y + row;
    };
    let headline = format!(
        "gpu {} · {} layer passes · {} without a usable pair",
        gpu.span.map_or_else(|| "-".to_owned(), time),
        gpu.passes.len(),
        gpu.incomplete,
    );
    p.text(&headline, area.x, y, area.w, p.ink(), &strong);
    y += row;

    let known = gpu.layers.unwrap_or(0) + gpu.composite.unwrap_or(0);
    let other = gpu.span.map(|s| s.saturating_sub(known)).filter(|o| *o > 0);
    let mut bars: Vec<(String, Option<u64>)> = gpu
        .passes
        .iter()
        .map(|pass| (layer_name(pass.layer, dev), pass.nanos))
        .collect();
    bars.push(("composite".to_owned(), gpu.composite));
    if other.is_some() {
        bars.push(("untimed passes and gaps".to_owned(), other));
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
        + gap;
    let time_w = p.width("000.00 ms", &caption) + gap;
    let track = Rect::new(
        area.x + name_w,
        0.0,
        (area.w - name_w - time_w).max(0.0),
        p.line(),
    );
    let color = p.theme.colors.warning.0;
    for (name, nanos) in bars {
        if y + row > area.bottom() {
            break;
        }
        p.text(&name, area.x, y, name_w - gap, p.mid(), &caption);
        p.track(Rect::new(track.x, y, track.w, track.h));
        match nanos {
            Some(nanos) => {
                let w = (track.w * nanos as f32 / most as f32).max(1.0);
                p.bar(Rect::new(track.x, y, w, track.h), color);
                p.text_right(&time(nanos), area.right(), y, p.ink(), &caption);
            }
            None => p.text_right("no end written", area.right(), y, p.mid(), &caption),
        }
        y += row;
    }
    y
}

/// The table's header, also what sets its least width.
const HEADER: &str =
    "layer 000 (dev mode)   z 0000   redrawn   0000 nodes   00000 glyphs   000 draws   0 fx";

/// Every layer of the frame, the top one first.
fn layers(p: &mut Painter<'_>, area: Rect, shown: &FrameReport, dev: LayerId) {
    let caption = p.caption();
    let strong = p.strong();
    let row = p.row();
    let gap = p.gap();
    let columns = [
        p.width("layer 000 (dev mode)", &caption) + gap,
        p.width("z 0000000", &caption) + gap,
        p.width("redrawn", &caption) + gap,
        p.width("00000 nodes", &caption) + gap,
        p.width("000000 glyphs", &caption) + gap,
        p.width("0000 draws", &caption) + gap,
        p.width("00 fx", &caption) + gap,
    ];
    let mut y = area.y;
    let redrawn = shown.layers.iter().filter(|l| l.redrawn).count();
    let title = format!(
        "{} layers · {} redrawn · {} reused",
        shown.layers.len(),
        redrawn,
        shown.layers.len() - redrawn
    );
    p.text(&title, area.x, y, area.w, p.ink(), &strong);
    y += row;
    let mut sorted: Vec<_> = shown.layers.iter().collect();
    sorted.sort_by_key(|l| std::cmp::Reverse(l.z_order));
    for layer in sorted {
        if y + row > area.bottom() {
            break;
        }
        let (state, ink) = match (layer.visible, layer.redrawn) {
            (false, _) => ("hidden", dim(p.mid())),
            (true, true) => ("redrawn", p.ink()),
            (true, false) => ("reused", p.mid()),
        };
        let cells = [
            layer_name(layer.id, dev),
            format!("z {}", layer.z_order),
            state.to_owned(),
            format!("{} nodes", layer.nodes),
            format!("{} glyphs", layer.glyphs),
            format!("{} draws", layer.draws),
            format!("{} fx", layer.effects),
        ];
        let mut x = area.x;
        for (cell, w) in cells.iter().zip(columns) {
            if x + w > area.right() + gap {
                break;
            }
            p.text(cell, x, y, w - gap, ink, &caption);
            x += w;
        }
        y += row;
    }
}
