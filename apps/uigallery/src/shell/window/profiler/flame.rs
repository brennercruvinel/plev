//! Puffin's part of the profiler: the frames recorded since it opened,
//! kept by a `GlobalFrameView` while Puffin's scopes are on. The CPU's
//! scopes are `engine::cpu_scope!`'s (the gallery's `view.render`, the
//! engine's `draw_frame`); the GPU's are wgpu-profiler's passes, a thread named "GPU" on
//! the GPU's clock (`engine::runtime::perf::profiler`). It draws the last
//! frames' times as bars, then the latest frame thread by thread, each
//! scope a bar as long as it took, the scopes it holds under it.

use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::runtime::perf::stages::{Stage, StageCost};
use engine::runtime::perf::stages::StageFrame;
use engine::widgets::prelude::{Rect, rounded_rect};
use puffin::{FrameData, FrameView, GlobalFrameView, Reader, Scope, Stream};

use super::{caption_style, text};
use crate::views::with_alpha;

/// The frames the bars show.
const BARS: usize = 120;
/// A frame at 60 Hz: the bars' least scale.
const FRAME_NS: i64 = 16_666_667;
/// How deep a thread's scopes are drawn.
const DEPTH: usize = 6;
/// The thread wgpu-profiler reports the passes on, on the GPU's clock.
const GPU: &str = "GPU";

/// A frame's CPU time, first scope to last, the GPU's thread aside: its
/// clock is not the CPU's, and Puffin's own range of the frame mixes them.
fn cpu_span(frame: &FrameData) -> Option<(i64, i64)> {
    let unpacked = frame.unpacked().ok()?;
    let cpu = unpacked
        .thread_streams
        .iter()
        .filter(|(t, _)| t.name != GPU);
    cpu.map(|(_, s)| s.range_ns)
        .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
}

fn cpu_ns(frame: &FrameData) -> i64 {
    cpu_span(frame).map_or(0, |(a, b)| b - a)
}



/// The name of a scope id, or an empty string.
fn view_scope_name(view: &FrameView, id: &puffin::ScopeId) -> String {
    view.scope_collection()
        .fetch_by_id(id)
        .map_or_else(String::new, |d| d.name().to_string())
}

pub(super) struct Recorder {
    view: GlobalFrameView,
    stage_frame: Option<StageFrame>,
}

impl Recorder {
    /// Puffin's scopes on, and the frames kept from the next one on.
    pub(super) fn start() -> Self {
        puffin::set_scopes_on(true);
        Self {
            view: GlobalFrameView::default(),
            stage_frame: None,
        }
    }

    /// Feeds the latest 22-stage frame, captured by the app's frame loop.
    pub(super) fn set_stage_frame(&mut self, frame: StageFrame) {
        self.stage_frame = Some(frame);
    }

    /// "Puffin · frame 812 · 3.41 ms", the latest frame's.
    pub(super) fn title(&self) -> String {
        match self.view.lock().latest_frame() {
            Some(f) => {
                let ms = cpu_ns(&f) as f64 / 1e6;
                format!(
                    "Puffin · frame {} · CPU {:.2} ms · {:.0}% of 16.6 ms",
                    f.frame_index(),
                    ms,
                    ms / 16.666 * 100.0
                )
            }
            None => "Puffin".to_owned(),
        }
    }

    pub(super) fn draw(&self, c: &mut Compositor, layer: LayerId, body: Rect, theme: &Theme) {
        let view = self.view.lock();
        let style = caption_style(theme);
        let line = style.line_height;
        let mid = theme.colors.text_mid.0;
        let frames: Vec<_> = view.latest_frames(BARS).cloned().collect();
        let Some(latest) = frames.last() else {
            let waiting = "Waiting for a frame: move the pointer over the page.";
            text(c, layer, waiting, body.x, body.y, body.w, mid, &style);
            return;
        };
        // the bars: one per frame, the latest at the right
        let strip = (line * 2.5).round();
        let most = frames
            .iter()
            .map(|f| cpu_ns(f))
            .max()
            .unwrap_or(0)
            .max(FRAME_NS);
        let bar_w = body.w / BARS as f32;
        let start = BARS - frames.len();
        for (i, f) in frames.iter().enumerate() {
            let bh = (strip * cpu_ns(f) as f32 / most as f32).max(1.0);
            let x = body.x + (start + i) as f32 * bar_w;
            let color = if i + 1 == frames.len() {
                theme.colors.accent.0
            } else {
                with_alpha(theme.colors.accent.0, 0.35)
            };
            let w = (bar_w - 1.0).max(1.0);
            c.push_to_layer(
                layer,
                rounded_rect(x, body.y + strip - bh, w, bh, 0.0, color),
            );
        }
        // summary: total, budget, hottest scope, peak over the bars
        let peak_ms = frames
            .iter()
            .map(|f| cpu_ns(f))
            .max()
            .unwrap_or(0) as f64
            / 1e6;
        let mut hottest: Option<(String, f64)> = None;
        #[allow(clippy::match_result_ok)]
        if let Some(unpacked_peek) = latest.unpacked().ok() {
            for stream in unpacked_peek.thread_streams.values() {
                if let Ok(top) = Reader::from_start(&stream.stream).read_top_scopes() {
                    for sc in top {
                        let dur = sc.record.duration_ns as f64 / 1e6;
                        if hottest.as_ref().is_none_or(|(_, d)| dur > *d) {
                            let name = view_scope_name(&view, &sc.id);
                            hottest = Some((name, dur));
                        }
                    }
                }
            }
        }
        let mut summary_y = body.y + strip + theme.spacing.sm;
        let mut summary = format!(
            "CPU {:.2} ms · {:.0}% of 16.6 ms · peak {:.2} ms / {} bars",
            cpu_ns(latest) as f64 / 1e6,
            cpu_ns(latest) as f64 / 1e6 / 16.666 * 100.0,
            peak_ms,
            frames.len()
        );
        if let Some((name, dur)) = &hottest {
            summary = format!("{summary} · hottest: {name} {dur:.2} ms");
        }
        text(c, layer, &summary, body.x, summary_y, body.w, mid, &style);
        summary_y += line + theme.spacing.sm;

        // the latest frame, thread by thread
        let Some(unpacked) = latest.unpacked().ok() else {
            return;
        };
        let label_w = (body.w * 0.15).clamp(48.0, 120.0);
        let lane = Rect::new(body.x + label_w, 0.0, body.w - label_w, 0.0);
        let (cpu_start, cpu_end) = cpu_span(latest).unwrap_or_default();
        let px_per_ns = lane.w / (cpu_end - cpu_start).max(1) as f32;
        let row = line + 4.0;
        let mut y = summary_y;
        for (thread, stream) in &unpacked.thread_streams {
            if y + row > body.bottom() {
                break;
            }
            text(
                c,
                layer,
                &thread.name,
                body.x,
                y,
                label_w - 4.0,
                mid,
                &style,
            );
            // the GPU's clock is its own: its scopes start at its first,
            // on the CPU's scale
            let origin = if thread.name == GPU {
                stream.range_ns.0
            } else {
                cpu_start
            };
            let at = Lane {
                x: lane.x,
                w: lane.w,
                origin,
                px_per_ns,
                row,
                bottom: body.bottom(),
            };
            let tops = Reader::from_start(&stream.stream).read_top_scopes();
            let depth = tops.map_or(0, |s| {
                at.scopes(c, layer, &view, &stream.stream, &s, y, 0, theme)
            });
            y += row * depth.max(1) as f32 + theme.spacing.xs;
        }
        // the 22 stages of the last frame, heaviest first (stage costs come
        // from engine::runtime::perf::stages, taken by the frame loop)
        if let Some(stage_frame) = &self.stage_frame {
            if !stage_frame.is_empty() {
                let mut costs: Vec<(Stage, StageCost)> = Stage::ALL
                    .iter()
                    .map(|&st| (st, stage_frame.get(st)))
                    .filter(|(_, c)| c.calls > 0)
                    .collect();
                costs.sort_by_key(|e| std::cmp::Reverse(e.1.nanos));
                let stage_line = caption_style(theme).line_height;
                for (st, cost) in costs.into_iter().take(8) {
                    if y + stage_line > body.bottom() { break; }
                    let ms = cost.nanos as f64 / 1e6;
                    let line = format!(
                        "{} {:.2} ms ({} calls, {} items)",
                        st.name(),
                        ms,
                        cost.calls,
                        cost.items
                    );
                    text(c, layer, &line, body.x, y, body.w, mid, &style);
                    y += stage_line + 2.0;
                }
            }
        }

    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        puffin::set_scopes_on(false);
    }
}

/// Where a thread's scopes go: from `x`, `w` wide, `origin` at `x`.
struct Lane {
    x: f32,
    w: f32,
    origin: i64,
    px_per_ns: f32,
    row: f32,
    bottom: f32,
}

impl Lane {
    /// `scopes` at `depth`, a row each depth, those they hold under
    /// them; how many rows were drawn.
    #[allow(clippy::too_many_arguments)] // the recursion carries its place
    fn scopes(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        view: &FrameView,
        stream: &Stream,
        scopes: &[Scope<'_>],
        y: f32,
        depth: usize,
        theme: &Theme,
    ) -> usize {
        let top = y + self.row * depth as f32;
        if depth == DEPTH || top + self.row > self.bottom {
            return depth;
        }
        let style = super::caption_style(theme);
        let mut deepest = depth + 1;
        for s in scopes {
            let x0 = self.x + (s.record.start_ns - self.origin) as f32 * self.px_per_ns;
            let w = (s.record.duration_ns as f32 * self.px_per_ns).max(1.0);
            let x0 = x0.clamp(self.x, self.x + self.w);
            let w = w.min(self.x + self.w - x0);
            let fill = with_alpha(theme.colors.accent.0, 0.85 - 0.12 * depth as f32);
            c.push_to_layer(layer, rounded_rect(x0, top, w, self.row - 2.0, 3.0, fill));
            if w > 32.0 {
                let name = view
                    .scope_collection()
                    .fetch_by_id(&s.id)
                    .map_or_else(String::new, |d| d.name().to_string());
                let label = format!("{name} {:.2} ms", s.record.duration_ns as f64 / 1e6);
                let ink = theme.colors.bg.0;
                text(c, layer, &label, x0 + 3.0, top + 1.0, w - 6.0, ink, &style);
            }
            let held = Reader::with_offset(stream, s.child_begin_position)
                .and_then(Reader::read_top_scopes);
            if let Some(held) = held.ok().filter(|h| !h.is_empty()) {
                let d = self.scopes(c, layer, view, stream, &held, y, depth + 1, theme);
                deepest = deepest.max(d);
            }
        }
        deepest
    }
}
