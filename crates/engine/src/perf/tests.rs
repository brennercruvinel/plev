//! Pure tests: rolling window semantics, percentiles, fps from synthetic
//! dts, snapshot/log formatting, memory aggregation and HUD scene output.

use super::*;
use crate::animation::AnimationTick;
use crate::compositor::{Compositor, LayerId, RenderStats, SceneNode};

fn tick(dt: f32) -> AnimationTick {
    AnimationTick { dt, elapsed: 0.0 }
}

fn stats(draw_calls: u32, glyphs: u32, encode: u64, resolve: u64) -> RenderStats {
    RenderStats {
        draw_calls,
        glyphs,
        encode_micros: encode,
        resolve_micros: resolve,
        ..RenderStats::default()
    }
}

// -- RollingWindow ----------------------------------------------------------

#[test]
fn rolling_window_keeps_only_last_capacity_samples() {
    let mut w = RollingWindow::new(4);
    for v in 1..=10 {
        w.push(v as f64);
    }
    // Window holds 7, 8, 9, 10.
    assert_eq!(w.len(), 4);
    assert_eq!(w.sum(), 34.0);
    assert_eq!(w.mean(), 8.5);
}

#[test]
fn rolling_window_empty_is_zero() {
    let w = RollingWindow::new(8);
    assert!(w.is_empty());
    assert_eq!(w.mean(), 0.0);
    assert_eq!(w.percentile(99.0), 0.0);
}

#[test]
fn rolling_window_capacity_clamped_to_one() {
    let mut w = RollingWindow::new(0);
    w.push(3.0);
    w.push(5.0);
    assert_eq!(w.len(), 1);
    assert_eq!(w.mean(), 5.0);
}

#[test]
fn percentile_nearest_rank_on_known_distribution() {
    let mut w = RollingWindow::new(100);
    for v in 1..=100 {
        w.push(v as f64);
    }
    assert_eq!(w.percentile(50.0), 50.0);
    assert_eq!(w.percentile(95.0), 95.0);
    assert_eq!(w.percentile(99.0), 99.0);
    assert_eq!(w.percentile(100.0), 100.0);
    assert_eq!(w.percentile(0.0), 1.0);
}

#[test]
fn percentile_single_sample() {
    let mut w = RollingWindow::new(8);
    w.push(7.5);
    assert_eq!(w.percentile(50.0), 7.5);
    assert_eq!(w.percentile(99.0), 7.5);
}

// -- PerfMonitor ------------------------------------------------------------

#[test]
fn fps_from_synthetic_60hz_dts() {
    let mut m = PerfMonitor::new();
    for _ in 0..120 {
        m.record_frame(tick(1.0 / 60.0), stats(10, 100, 500, 200));
    }
    let s = m.snapshot();
    assert!((s.fps - 60.0).abs() < 0.01, "fps = {}", s.fps);
    assert!((s.dt_p50_ms - 16.666).abs() < 0.1);
    assert_eq!(s.frames, 120);
}

#[test]
fn dt_p99_catches_frame_spikes() {
    let mut m = PerfMonitor::with_window(100);
    for i in 0..100 {
        // Two 50ms hitches among 16.6ms frames: nearest-rank p99 over 100
        // samples reads the 99th sorted value, the first hitch. A single
        // hitch (1 in 100) sits exactly past that rank by definition.
        let dt = if i == 40 || i == 80 {
            0.050
        } else {
            1.0 / 60.0
        };
        m.record_frame(tick(dt), stats(1, 0, 100, 50));
    }
    let s = m.snapshot();
    assert!((s.dt_p50_ms - 16.666).abs() < 0.1);
    assert!((s.dt_p95_ms - 16.666).abs() < 0.1, "p95 = {}", s.dt_p95_ms);
    assert!((s.dt_p99_ms - 50.0).abs() < 0.01, "p99 = {}", s.dt_p99_ms);
    // The hitches drag effective fps below the nominal 60.
    assert!(s.fps < 60.0);
}

#[test]
fn cpu_micros_average_and_last_frame_counters() {
    let mut m = PerfMonitor::with_window(4);
    m.record_frame(tick(0.016), stats(8, 50, 100, 40));
    m.record_frame(tick(0.016), stats(24, 512, 300, 80));
    let s = m.snapshot();
    assert_eq!(s.encode_avg_micros, 200);
    assert_eq!(s.resolve_avg_micros, 60);
    // draw_calls/glyphs reflect the last frame, not an average.
    assert_eq!(s.draw_calls, 24);
    assert_eq!(s.glyphs, 512);
}

#[test]
fn zero_dt_first_tick_excluded_from_fps_window() {
    let mut m = PerfMonitor::new();
    m.record_frame(tick(0.0), stats(1, 0, 10, 10));
    let s = m.snapshot();
    assert_eq!(s.fps, 0.0);
    assert_eq!(s.frames, 1);
    m.record_frame(tick(0.02), stats(1, 0, 10, 10));
    assert!((m.snapshot().fps - 50.0).abs() < 0.01);
}

#[test]
fn empty_monitor_snapshot_is_all_zero() {
    let s = PerfMonitor::new().snapshot();
    assert_eq!(s.fps, 0.0);
    assert_eq!(s.dt_p99_ms, 0.0);
    assert_eq!(s.draw_calls, 0);
    assert_eq!(s.gpu_micros, None);
}

#[test]
fn gpu_micros_is_none_until_fed() {
    let mut m = PerfMonitor::new();
    m.record_frame(tick(0.016), stats(1, 0, 10, 10));
    assert_eq!(m.snapshot().gpu_micros, None);
    m.record_gpu_micros(1234);
    assert_eq!(m.snapshot().gpu_micros, Some(1234));
}

// -- Memory -----------------------------------------------------------------

#[test]
fn memory_stats_gpu_total_sums_components() {
    let mem = MemoryStats {
        glyph_atlas_bytes: 512 * 512,
        texture_pool_bytes: 1024,
        layer_bytes: 2048,
        process_rss_bytes: Some(1 << 30),
    };
    assert_eq!(mem.gpu_total_bytes(), 512 * 512 + 1024 + 2048);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn process_rss_reports_resident_bytes_on_native() {
    let rss = process_rss_bytes().expect("rss available on this target");
    // A running test binary holds at least 1 MB resident.
    assert!(rss > 1024 * 1024, "rss = {rss}");
}

#[test]
fn log_line_is_compact_and_complete() {
    let mut m = PerfMonitor::new();
    m.record_frame(tick(1.0 / 60.0), stats(24, 512, 300, 80));
    m.record_memory(MemoryStats {
        glyph_atlas_bytes: 1 << 20,
        texture_pool_bytes: 0,
        layer_bytes: 0,
        process_rss_bytes: Some(100 << 20),
    });
    let line = m.snapshot().log_line();
    assert!(line.starts_with("perf fps 60.0"), "{line}");
    for token in [
        "p50",
        "p99",
        "enc",
        "res",
        "draw 24",
        "glyphs 512",
        "rss 100MB",
    ] {
        assert!(line.contains(token), "missing {token} in {line}");
    }
    assert!(!line.contains('\n'));
}

// -- HUD --------------------------------------------------------------------

fn hud_snapshot() -> PerfSnapshot {
    let mut m = PerfMonitor::new();
    for _ in 0..60 {
        m.record_frame(tick(1.0 / 60.0), stats(24, 512, 300, 80));
    }
    m.record_memory(MemoryStats {
        glyph_atlas_bytes: 512 * 512,
        texture_pool_bytes: 0,
        layer_bytes: 4 << 20,
        process_rss_bytes: Some(150 << 20),
    });
    m.snapshot()
}

#[test]
fn hud_draws_panel_and_text_on_own_high_z_layer() {
    let mut c = Compositor::new();
    let mut hud = PerfHud::new();
    hud.draw(&mut c, &hud_snapshot(), 800.0);

    let id = hud.layer().expect("layer created on first draw");
    assert_ne!(id, LayerId::DEFAULT);
    let layer = c.layer(id).expect("hud layer exists");
    assert_eq!(layer.z_order, PerfHud::Z_ORDER);
    assert!(c.layer(LayerId::DEFAULT).unwrap().nodes().is_empty());

    let nodes = layer.nodes();
    let SceneNode::RoundedRect { x, w, .. } = nodes[0] else {
        panic!("first node must be the panel background");
    };
    // Anchored top-right: panel ends before the right edge, starts inside.
    assert!(x + w <= 800.0);
    assert!(x > 400.0, "panel x = {x}");

    let texts: Vec<_> = nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key),
            _ => None,
        })
        .collect();
    assert_eq!(texts.len(), 4);
    for key in texts {
        // One TextStyle for measure and draw: from_style carries the family.
        assert_eq!(key.font_family.as_deref(), Some("Inclusive Sans"));
    }
}

#[test]
fn hud_clamps_to_viewport_narrower_than_panel() {
    let mut c = Compositor::new();
    let mut hud = PerfHud::new();
    hud.draw(&mut c, &hud_snapshot(), 50.0);
    let layer = c.layer(hud.layer().unwrap()).unwrap();
    let SceneNode::RoundedRect { x, .. } = layer.nodes()[0] else {
        panic!("panel missing");
    };
    assert_eq!(x, 0.0);
}

#[test]
fn hud_clear_removes_layer_and_redraw_recreates() {
    let mut c = Compositor::new();
    let mut hud = PerfHud::new();
    hud.draw(&mut c, &hud_snapshot(), 800.0);
    assert_eq!(c.layers().len(), 2);

    hud.clear(&mut c);
    assert_eq!(hud.layer(), None);
    assert_eq!(c.layers().len(), 1);
    // Clearing twice is a no-op.
    hud.clear(&mut c);
    assert_eq!(c.layers().len(), 1);

    hud.draw(&mut c, &hud_snapshot(), 800.0);
    assert_eq!(c.layers().len(), 2);
}

#[test]
fn hud_shows_gpu_time_line_only_when_fed() {
    let mut c = Compositor::new();
    let mut hud = PerfHud::new();
    let mut snap = hud_snapshot();
    snap.gpu_micros = Some(2500);
    hud.draw(&mut c, &snap, 800.0);
    let layer = c.layer(hud.layer().unwrap()).unwrap();
    let texts = layer
        .nodes()
        .iter()
        .filter(|n| matches!(n, SceneNode::Text { .. }))
        .count();
    assert_eq!(texts, 5);
}
