//! Performance instrumentation built on the engine's existing per-frame
//! sources: [`AnimationTick`] (frame clock) and [`RenderStats`]
//! (compositor counters). Pure and testable; no GPU, no clocks of its own.
//!
//! Per frame, the render loop calls [`PerfMonitor::record_frame`] (and
//! optionally [`PerfMonitor::record_memory`]); consumers read an immutable
//! [`PerfSnapshot`] for logging ([`PerfSnapshot::log_line`]) or the visual
//! overlay ([`PerfHud`]).

mod hud;
mod memory;
mod window;

#[cfg(test)]
mod tests;

pub use hud::PerfHud;
pub use memory::{MemoryStats, process_rss_bytes};
pub use window::RollingWindow;

use crate::animation::AnimationTick;
use crate::compositor::RenderStats;

/// Rolling window length used by [`PerfMonitor::new`]: two seconds of
/// frames at 60 fps.
pub const DEFAULT_WINDOW_SAMPLES: usize = 120;

/// Aggregates per-frame timing and counters over rolling windows.
pub struct PerfMonitor {
    dt_seconds: RollingWindow,
    encode_micros: RollingWindow,
    resolve_micros: RollingWindow,
    last_stats: RenderStats,
    memory: MemoryStats,
    gpu_micros: Option<u64>,
    frames: u64,
}

impl Default for PerfMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfMonitor {
    pub fn new() -> Self {
        Self::with_window(DEFAULT_WINDOW_SAMPLES)
    }

    /// Monitor with a custom rolling-window length (in frames).
    pub fn with_window(samples: usize) -> Self {
        Self {
            dt_seconds: RollingWindow::new(samples),
            encode_micros: RollingWindow::new(samples),
            resolve_micros: RollingWindow::new(samples),
            last_stats: RenderStats::default(),
            memory: MemoryStats::default(),
            gpu_micros: None,
            frames: 0,
        }
    }

    /// Record one rendered frame. Non-positive `dt` samples (the very
    /// first tick) are excluded from the fps/dt windows but still count
    /// as a frame.
    pub fn record_frame(&mut self, tick: AnimationTick, stats: RenderStats) {
        if tick.dt > 0.0 {
            self.dt_seconds.push(f64::from(tick.dt));
        }
        self.encode_micros.push(stats.encode_micros as f64);
        self.resolve_micros.push(stats.resolve_micros as f64);
        self.last_stats = stats;
        self.frames += 1;
    }

    /// Record the latest memory readings (see [`MemoryStats`]).
    pub fn record_memory(&mut self, memory: MemoryStats) {
        self.memory = memory;
    }

    /// Feed a measured GPU frame time in microseconds.
    ///
    /// Known gap: wgpu 28 timestamp queries (`Features::TIMESTAMP_QUERY`)
    /// are not wired into the render loop yet. Pass-level
    /// `timestamp_writes` would thread a `QuerySet` through the public
    /// `encode_layer_passes`/`encode_composite_pass` signatures plus an
    /// async map-readback ring, and encoder-level `write_timestamp`
    /// requires `TIMESTAMP_QUERY_INSIDE_ENCODERS` (native-only; webgpu
    /// adapters deny it). Until that integration lands, callers with their
    /// own timing can feed this; `PerfSnapshot::gpu_micros` stays `None`
    /// otherwise.
    pub fn record_gpu_micros(&mut self, micros: u64) {
        self.gpu_micros = Some(micros);
    }

    /// Total frames recorded since creation (drives log cadence).
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Counters of the most recent recorded frame.
    pub fn last_stats(&self) -> RenderStats {
        self.last_stats
    }

    /// Immutable view of the current windows; all-zero before the first
    /// recorded frame.
    pub fn snapshot(&self) -> PerfSnapshot {
        let fps = if self.dt_seconds.is_empty() {
            0.0
        } else {
            (self.dt_seconds.len() as f64 / self.dt_seconds.sum()) as f32
        };
        PerfSnapshot {
            fps,
            dt_p50_ms: (self.dt_seconds.percentile(50.0) * 1000.0) as f32,
            dt_p95_ms: (self.dt_seconds.percentile(95.0) * 1000.0) as f32,
            dt_p99_ms: (self.dt_seconds.percentile(99.0) * 1000.0) as f32,
            encode_avg_micros: self.encode_micros.mean().round() as u64,
            resolve_avg_micros: self.resolve_micros.mean().round() as u64,
            gpu_micros: self.gpu_micros,
            draw_calls: self.last_stats.draw_calls,
            glyphs: self.last_stats.glyphs,
            memory: self.memory,
            frames: self.frames,
        }
    }
}

/// One coherent reading of the monitor: rolling aggregates plus the last
/// frame's counters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerfSnapshot {
    /// Effective fps over the window: samples / sum(dt).
    pub fps: f32,
    pub dt_p50_ms: f32,
    pub dt_p95_ms: f32,
    pub dt_p99_ms: f32,
    /// Mean CPU encode+submit time over the window.
    pub encode_avg_micros: u64,
    /// Mean `Compositor::resolve` time over the window.
    pub resolve_avg_micros: u64,
    /// GPU frame time when fed externally (see
    /// [`PerfMonitor::record_gpu_micros`]); `None` until timestamp
    /// queries are integrated.
    pub gpu_micros: Option<u64>,
    /// Draw calls of the last frame.
    pub draw_calls: u32,
    /// Glyph quads of the last frame.
    pub glyphs: u32,
    pub memory: MemoryStats,
    pub frames: u64,
}

impl PerfSnapshot {
    /// Compact single-line summary for periodic logging.
    pub fn log_line(&self) -> String {
        let mut line = format!(
            "perf fps {:.1} dt p50 {:.2}ms p95 {:.2}ms p99 {:.2}ms enc {}us res {}us \
             draw {} glyphs {} gpu-mem {:.1}MB",
            self.fps,
            self.dt_p50_ms,
            self.dt_p95_ms,
            self.dt_p99_ms,
            self.encode_avg_micros,
            self.resolve_avg_micros,
            self.draw_calls,
            self.glyphs,
            mb(self.memory.gpu_total_bytes()),
        );
        if let Some(gpu) = self.gpu_micros {
            line.push_str(&format!(" gpu {gpu}us"));
        }
        if let Some(rss) = self.memory.process_rss_bytes {
            line.push_str(&format!(" rss {:.0}MB", mb(rss)));
        }
        line
    }
}

pub(crate) fn mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}
