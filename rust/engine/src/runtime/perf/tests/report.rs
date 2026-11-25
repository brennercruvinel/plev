//! The recorder without a GPU: the ring, readings attached by index,
//! tool frames, settling, the layer reports, and the switch.

use super::super::MemoryStats;
use super::super::gpu_time::{GpuPass, GpuReading};
use super::super::report::{FrameReport, HISTORY_FRAMES, LayerReport, Recorder, SETTLING_FRAMES};
use super::super::stages::{self, Stage, StageCost, StageFrame};
use super::super::zones::{self, ZoneFrame};
use crate::drawing::compositor::{Compositor, LayerEffect, LayerId};

/// A report of frame `index` whose app frame took `nanos`.
fn report(index: u64, nanos: u64) -> FrameReport {
    let stages = StageFrame::from_costs(&[(
        Stage::Frame,
        StageCost {
            calls: 1,
            items: 0,
            nanos,
        },
    )]);
    let compositor = Compositor::new();
    FrameReport::new(
        index,
        stages,
        ZoneFrame::default(),
        &compositor,
        MemoryStats::default(),
    )
}

fn reading(frame: u64) -> GpuReading {
    GpuReading {
        frame,
        passes: vec![GpuPass {
            layer: LayerId::DEFAULT,
            nanos: Some(1_000),
        }],
        span: Some(1_000),
        ..GpuReading::default()
    }
}

fn indices(r: &Recorder) -> Vec<u64> {
    r.history().map(|f| f.index).collect()
}

#[test]
fn the_ring_keeps_the_last_frames_oldest_first() {
    let mut r = Recorder::new();
    assert!(r.is_empty() && r.latest().is_none());
    for i in 0..HISTORY_FRAMES as u64 + 10 {
        r.push(report(i, 100), Vec::new(), false);
    }
    assert_eq!(r.len(), HISTORY_FRAMES);
    let kept = indices(&r);
    assert_eq!(kept.first(), Some(&10));
    assert_eq!(kept.last(), Some(&(HISTORY_FRAMES as u64 + 9)));
    assert!(r.get(5).is_none(), "out of the ring");
    assert_eq!(r.get(20).map(|f| f.index), Some(20));
    assert_eq!(r.get(20).map(|f| f.pipeline.frame_nanos), Some(100));
    r.clear();
    assert!(r.is_empty());
}

#[test]
fn a_reading_lands_on_the_frame_it_belongs_to() {
    let mut r = Recorder::new();
    r.push(report(7, 100), Vec::new(), true);
    r.push(report(8, 100), Vec::new(), true);
    assert!(r.get(7).is_some_and(|f| f.gpu.is_none()));
    // frame 9 reads back frame 7's timestamps, and one of a frame long gone
    r.push(report(9, 100), vec![reading(2), reading(7)], true);
    assert_eq!(r.get(7).and_then(|f| f.gpu.clone()), Some(reading(7)));
    assert!(r.get(8).is_some_and(|f| f.gpu.is_none()));
    assert!(r.get(9).is_some_and(|f| f.gpu.is_none()));
    assert!(r.gpu_available());
}

#[test]
fn tool_frames_are_kept_flagged_and_left_out_of_latest_and_slowest() {
    let mut r = Recorder::new();
    r.push(report(1, 900), Vec::new(), false);
    r.push(report(2, 300), Vec::new(), false);
    r.mark_tool();
    r.push(report(3, 5_000), Vec::new(), false);
    r.push(report(4, 200), Vec::new(), false);
    let tools: Vec<_> = r.history().map(|f| f.tool).collect();
    assert_eq!(tools, [false, false, true, false], "only the next frame");
    assert_eq!(r.latest().map(|f| f.index), Some(4));
    assert_eq!(r.slowest().map(|f| f.index), Some(1));
    r.mark_tool();
    r.push(report(5, 100), Vec::new(), false);
    assert_eq!(
        r.latest().map(|f| f.index),
        Some(4),
        "the last non-tool frame"
    );
}

#[test]
fn settling_asks_for_tool_frames_until_the_reading_or_the_limit() {
    let lock = zones::test_lock();
    let mut r = Recorder::new();
    r.set_on(true);
    r.push(report(1, 100), Vec::new(), true);
    assert!(r.settling(), "the latest frame waits for its timestamps");
    for i in 0..SETTLING_FRAMES as u64 {
        assert!(r.settling());
        r.mark_tool();
        r.push(report(2 + i, 100), Vec::new(), true);
    }
    assert!(!r.settling(), "gave up after {SETTLING_FRAMES} tool frames");

    r.push(report(10, 100), Vec::new(), true);
    assert!(r.settling());
    r.mark_tool();
    r.push(report(11, 100), vec![reading(10)], true);
    assert!(!r.settling(), "the reading came back");

    r.push(report(12, 100), Vec::new(), false);
    assert!(!r.settling(), "without timestamps nothing comes back");
    r.set_on(false);
    drop(lock);
}

#[test]
fn the_switch_turns_every_source_on_and_off_together() {
    let lock = zones::test_lock();
    let mut r = Recorder::new();
    assert!(!r.is_on());
    r.set_on(true);
    assert!(r.is_on() && stages::is_enabled() && zones::is_enabled());
    let mut waited = false;
    stages::gpu_wait(|| waited = true);
    assert!(waited, "the gpu sync is on");
    r.set_on(false);
    assert!(!r.is_on() && !stages::is_enabled() && !zones::is_enabled());
    stages::gpu_wait(|| panic!("the gpu sync is off"));
    assert!(!r.settling(), "never while off");
    drop(lock);
}

#[test]
fn layer_reports_follow_the_compositor_and_what_the_frame_drew() {
    let mut c = Compositor::new();
    let top = c.create_layer(5);
    c.set_layer_opacity(top, 0.5);
    c.set_layer_visible(top, false);
    c.set_layer_effects(top, vec![LayerEffect::Blur { sigma: 2.0 }]);
    c.draw_rect(0.0, 0.0, 10.0, 10.0, [1.0; 4]);
    c.draw_rect(0.0, 20.0, 10.0, 10.0, [1.0; 4]);
    c.drawn = vec![(LayerId::DEFAULT, 4)];

    let layers = LayerReport::of(&c);
    assert_eq!(layers.len(), 2);
    let base = layers[0];
    assert_eq!(
        (base.id, base.redrawn, base.draws),
        (LayerId::DEFAULT, true, 4)
    );
    assert_eq!(base.nodes, 2);
    let other = layers[1];
    assert_eq!((other.id, other.z_order, other.visible), (top, 5, false));
    assert_eq!((other.opacity, other.effects), (0.5, 1));
    assert_eq!((other.redrawn, other.draws, other.nodes), (false, 0, 0));
}

/// The recorder inside a window host's frame path, on a headless host;
/// skips without a GPU adapter.
mod host {
    use super::super::super::pipeline::Phase;
    use super::*;
    use crate::drawing::gpu::RenderConfig;
    use crate::runtime::window::WindowHost;

    fn zone_names(frame: &FrameReport) -> Vec<&'static str> {
        let name = std::thread::current().name().map(str::to_owned);
        frame
            .zones
            .threads
            .iter()
            .filter(|t| Some(&t.name) == name.as_ref())
            .flat_map(|t| t.zones.iter().map(|z| z.name))
            .collect()
    }

    #[test]
    fn a_host_reports_each_frame_with_its_layers_zones_and_gpu_passes() {
        let _lock = zones::test_lock();
        let config = RenderConfig {
            gpu_timestamps: true,
            ..RenderConfig::default()
        };
        let Some(mut host) = WindowHost::headless(160, 120, 1.0, config) else {
            eprintln!("no GPU adapter; skipping");
            return;
        };
        host.dev_mut().set_on(true);
        for i in 0..3 {
            let c = host.compositor_mut();
            c.begin_frame();
            c.draw_rect(10.0 + i as f32, 10.0, 40.0, 20.0, [0.9, 0.2, 0.2, 1.0]);
            assert!(host.present([0.0, 0.0, 0.0, 1.0], "dev").is_some());
        }
        host.dev_mut().set_on(false);

        let dev = host.dev();
        let indices: Vec<_> = dev.history().map(|f| f.index).collect();
        assert_eq!(indices, [0, 1, 2]);
        let Some(first) = dev.get(0) else {
            panic!("frame 0 kept");
        };
        let base = first.layers[0];
        assert!(base.redrawn && base.draws > 0, "{base:?}");
        assert_eq!(first.stats.draw_calls, base.draws + 1, "plus the composite");
        assert!(first.pipeline.phase(Phase::Encode).nanos > 0);
        let zones = zone_names(first);
        for step in [
            "draw_frame",
            "resolve",
            "encode layers",
            "submit",
            "gpu wait",
        ] {
            assert!(zones.contains(&step), "{step} in {zones:?}");
        }
        if dev.gpu_available() {
            let Some(gpu) = first.gpu.as_ref() else {
                panic!("frame 0's timestamps came back with the sync on");
            };
            assert_eq!(gpu.frame, 0);
            assert_eq!(gpu.passes.first().map(|p| p.layer), Some(LayerId::DEFAULT));
        } else {
            assert!(dev.history().all(|f| f.gpu.is_none()));
        }
        eprintln!("gpu available: {}", dev.gpu_available());
    }
}
