//! The stage probes where the engine placed them: each path records the
//! stage it names while stages are on, and nothing while they are off.
//! The frame tail is driven through `headless::Renderer`, the window's
//! own path; those tests skip without a GPU adapter.

use super::stages::{self, Stage, StageFrame};
use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::layout::{LayoutEngine, LayoutItem, LayoutStyle};
use crate::text::{TextMeasurer, TextStyle};

/// Stages on for one test, off again when it ends.
struct On;

impl On {
    fn new() -> Self {
        stages::enable(false);
        stages::enable(true);
        On
    }
}

impl Drop for On {
    fn drop(&mut self) {
        stages::enable(false);
    }
}

fn sem_id(text: &str) -> sem::SemId {
    match sem::SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}

fn leaf() -> LayoutItem {
    LayoutItem {
        style: LayoutStyle {
            width: Some(10.0),
            height: Some(10.0),
            ..Default::default()
        },
        children: vec![],
        text: None,
    }
}

/// Two layers: the default one with two rects, a second with one.
fn two_layers(c: &mut Compositor, second: crate::compositor::LayerId, shade: f32) {
    c.begin_frame();
    c.draw_rect(0.0, 0.0, 10.0, 10.0, [shade, 0.0, 0.0, 1.0]);
    c.draw_rect(20.0, 0.0, 10.0, 10.0, [0.0, 1.0, 0.0, 1.0]);
    c.push_to_layer(
        second,
        SceneNode::Rect {
            x: 0.0,
            y: 20.0,
            w: 10.0,
            h: 10.0,
            color: [0.0, 0.0, 1.0, 1.0],
        },
    );
}

/// What the frame tail does once a layer is encoded.
fn clean(c: &mut Compositor) {
    let ids: Vec<_> = c.layers().iter().map(|l| l.id).collect();
    for id in ids {
        c.mark_layer_clean(id);
    }
}

/// Every probe the CPU side has, once.
fn run_cpu_paths() {
    let mut c = Compositor::new();
    let second = c.create_layer(1);
    two_layers(&mut c, second, 1.0);
    c.bind(&sem_id("probe.a"), [0.0, 0.0, 10.0, 10.0]);
    let _ = c.hit(5.0, 5.0);
    c.resolve_scene((100.0, 100.0));
    let style = TextStyle::new(15.0);
    let _ = TextMeasurer::measure_styled("probes off", &style, None);
    let _ = LayoutEngine::new().compute(&[leaf()], 100.0, 100.0);
}

#[test]
fn off_every_probe_records_nothing() {
    stages::enable(false);
    run_cpu_paths();
    assert!(stages::take().is_empty());
}

#[test]
fn resolve_hashes_every_layer_and_rebuilds_only_the_dirty_ones() {
    let _on = On::new();
    let mut c = Compositor::new();
    let second = c.create_layer(1);

    two_layers(&mut c, second, 1.0);
    c.resolve_scene((100.0, 100.0));
    let first = stages::take();
    let hash = first.get(Stage::LayerHash);
    assert_eq!((hash.calls, hash.items), (2, 3), "two layers, three nodes");
    let geometry = first.get(Stage::Geometry);
    assert_eq!(
        (geometry.calls, geometry.items),
        (2, 3),
        "both new, both built"
    );
    clean(&mut c);

    // the same scene: hashed again, nothing rebuilt
    two_layers(&mut c, second, 1.0);
    c.resolve_scene((100.0, 100.0));
    let same = stages::take();
    assert_eq!(same.get(Stage::LayerHash).calls, 2);
    assert_eq!(same.get(Stage::Geometry).calls, 0);
    clean(&mut c);

    // one rect of the default layer changes: only that layer rebuilds
    two_layers(&mut c, second, 0.5);
    c.resolve_scene((100.0, 100.0));
    let geometry = stages::take().get(Stage::Geometry);
    assert_eq!((geometry.calls, geometry.items), (1, 2));
}

#[test]
fn a_measure_is_a_miss_that_shapes_then_a_hit_and_hit_tests_always_shape() {
    let _on = On::new();
    let style = TextStyle::new(15.0);
    let text = "stage probe text";
    let _ = TextMeasurer::measure_styled(text, &style, None);
    let miss = stages::take();
    assert_eq!(miss.get(Stage::MeasureMiss).calls, 1);
    assert_eq!(miss.get(Stage::MeasureHit).calls, 0);
    let shape = miss.get(Stage::Shape);
    assert_eq!((shape.calls, shape.items), (1, text.len() as u64));
    assert!(miss.get(Stage::MeasureMiss).nanos >= shape.nanos);

    let _ = TextMeasurer::measure_styled(text, &style, None);
    let hit = stages::take();
    assert_eq!(hit.get(Stage::MeasureHit).calls, 1);
    assert_eq!(hit.get(Stage::Shape).calls, 0, "a hit does not shape");

    // the caret and hit-test queries are not cached: they shape each time
    let _ = TextMeasurer::hit_test_styled(text, &style, None, 12.0, 4.0);
    let _ = TextMeasurer::cursor_x_styled(text, &style, None, 3);
    assert_eq!(stages::take().get(Stage::Shape).calls, 2);
}

#[test]
fn layout_counts_its_items() {
    let _on = On::new();
    let mut root = leaf();
    root.style.width = Some(100.0);
    root.children = vec![1, 2];
    let bounds = LayoutEngine::new().compute(&[root, leaf(), leaf()], 200.0, 200.0);
    assert_eq!(bounds.len(), 3);
    let layout = stages::take().get(Stage::Layout);
    assert_eq!((layout.calls, layout.items), (1, 3));
}

#[test]
fn binds_count_once_per_id_and_a_hit_scans_the_bindings() {
    let _on = On::new();
    let mut c = Compositor::new();
    c.begin_frame();
    c.bind(&sem_id("probe.a"), [0.0, 0.0, 10.0, 10.0]);
    c.bind(&sem_id("probe.b"), [20.0, 0.0, 10.0, 10.0]);
    // a second bind of an id is ignored, and so not counted
    c.bind(&sem_id("probe.a"), [40.0, 0.0, 10.0, 10.0]);
    assert_eq!(c.hit(25.0, 5.0).map(|id| id.as_str()), Some("probe.b"));
    let frame = stages::take();
    assert_eq!(frame.get(Stage::Bind).calls, 2);
    let hit = frame.get(Stage::HitTest);
    assert_eq!((hit.calls, hit.items), (1, 2));
}

#[cfg(not(target_arch = "wasm32"))]
mod frame_tail {
    use super::*;
    use crate::gpu::RenderConfig;
    use crate::headless::{HeadlessError, Renderer, Viewport};

    fn renderer() -> Option<Renderer> {
        match Renderer::new(Viewport::new(160.0, 120.0, 1.0), RenderConfig::default()) {
            Ok(r) => Some(r),
            Err(HeadlessError::NoAdapter) => {
                eprintln!("no GPU adapter; skipping");
                None
            }
            Err(e) => panic!("headless renderer: {e}"),
        }
    }

    fn scene(c: &mut Compositor) {
        c.begin_frame();
        c.draw_rect(10.0, 10.0, 40.0, 20.0, [0.9, 0.2, 0.2, 1.0]);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style("stages", &TextStyle::new(16.0), None),
            x: 12.0,
            y: 40.0,
            color: [1.0, 1.0, 1.0, 1.0],
        });
    }

    fn draw(r: &mut Renderer, c: &mut Compositor) -> StageFrame {
        scene(c);
        let _ = stages::take();
        r.draw(c, [0.0, 0.0, 0.0, 1.0]);
        stages::take()
    }

    #[test]
    fn a_new_frame_records_the_whole_tail_and_a_clean_one_uploads_nothing() {
        let Some(mut r) = renderer() else {
            return;
        };
        let _on = On::new();
        let mut c = Compositor::new();

        let first = draw(&mut r, &mut c);
        for stage in [
            Stage::Draw,
            Stage::Resolve,
            Stage::TextResolve,
            Stage::Encode,
        ] {
            assert_eq!(first.get(stage).calls, 1, "{stage:?}");
        }
        assert_eq!(first.get(Stage::Geometry).calls, 1);
        assert!(
            first.get(Stage::Upload).items > 0,
            "vertices and text went up"
        );
        assert!(first.get(Stage::TextResolve).items > 0, "glyphs emitted");
        assert!(first.get(Stage::Encode).items > 0, "draw calls issued");
        assert_eq!(first.get(Stage::GpuWait).calls, 0, "no sync asked");

        // nothing changed: the layer is hashed, not rebuilt, not uploaded
        let clean = draw(&mut r, &mut c);
        assert_eq!(clean.get(Stage::LayerHash).calls, 1);
        assert_eq!(clean.get(Stage::Geometry).calls, 0);
        assert_eq!(clean.get(Stage::Upload), Default::default());
        assert_eq!(clean.get(Stage::TextResolve).items, 0);
    }

    #[test]
    fn the_gpu_sync_waits_after_submit_and_records_the_wait() {
        let Some(mut r) = renderer() else {
            return;
        };
        let _on = On::new();
        stages::set_gpu_sync(true);
        let mut c = Compositor::new();
        let frame = draw(&mut r, &mut c);
        assert_eq!(frame.get(Stage::GpuWait).calls, 1);
        assert_eq!(frame.get(Stage::Draw).calls, 1);
    }

    #[test]
    fn stages_on_draw_the_same_pixels_as_stages_off() {
        let Some(mut off) = renderer() else {
            return;
        };
        let Some(mut on) = renderer() else {
            return;
        };
        stages::enable(false);
        let mut c = Compositor::new();
        scene(&mut c);
        let plain = match off.render(&mut c, [0.0, 0.0, 0.0, 1.0]) {
            Ok(f) => f,
            Err(e) => panic!("{e}"),
        };
        let _on = On::new();
        stages::set_gpu_sync(true);
        let mut c = Compositor::new();
        scene(&mut c);
        let measured = match on.render(&mut c, [0.0, 0.0, 0.0, 1.0]) {
            Ok(f) => f,
            Err(e) => panic!("{e}"),
        };
        assert!(plain.rgba == measured.rgba, "the probes changed a pixel");
    }

    /// A renderer whose context has GPU timestamps, or `None` (skip) when
    /// there is no adapter or it lacks `TIMESTAMP_QUERY`.
    fn timed_renderer(viewport: Viewport) -> Option<Renderer> {
        let config = RenderConfig {
            gpu_timestamps: true,
            ..RenderConfig::default()
        };
        let probe = pollster::block_on(crate::gpu::GpuContext::new_headless(1, 1, config.clone()));
        if !probe.is_some_and(|gpu| gpu.timestamps.is_some()) {
            eprintln!("no GPU adapter with timestamp queries; skipping");
            return None;
        }
        match Renderer::new(viewport, config) {
            Ok(r) => Some(r),
            Err(e) => panic!("headless renderer: {e}"),
        }
    }

    const GPU_STAGES: [Stage; 3] = [Stage::GpuSpan, Stage::GpuLayers, Stage::GpuComposite];

    #[test]
    fn without_timestamps_asked_no_gpu_stage_is_recorded() {
        let Some(mut r) = renderer() else {
            return;
        };
        let _on = On::new();
        stages::set_gpu_sync(true);
        let mut c = Compositor::new();
        let frame = draw(&mut r, &mut c);
        assert_eq!(frame.get(Stage::GpuWait).calls, 1);
        for stage in GPU_STAGES {
            assert_eq!(frame.get(stage), Default::default(), "{stage:?}");
        }
        assert_eq!(frame.gpu_micros(), None);
    }

    #[test]
    fn timestamps_read_back_one_frame_later_and_feed_the_monitor() {
        let Some(mut r) = timed_renderer(Viewport::new(800.0, 600.0, 2.0)) else {
            return;
        };
        let _on = On::new();
        stages::set_gpu_sync(true);
        let mut c = Compositor::new();

        // the first frame writes its timestamps; nothing finished before it
        let first = draw(&mut r, &mut c);
        for stage in GPU_STAGES {
            assert_eq!(first.get(stage).calls, 0, "{stage:?}");
        }

        // the next frame resolves the first, finished, and reads it back
        let second = draw(&mut r, &mut c);
        let (span, layers, composite) = (
            second.get(Stage::GpuSpan),
            second.get(Stage::GpuLayers),
            second.get(Stage::GpuComposite),
        );
        assert_eq!(
            (span.calls, layers.calls, composite.calls),
            (1, 1, 1),
            "{second:?}"
        );
        assert_eq!(
            layers.items, 1,
            "the first frame: one dirty layer, one pass"
        );
        assert!(layers.nanos > 0 && composite.nanos > 0, "{second:?}");
        assert!(
            span.nanos >= layers.nanos.max(composite.nanos),
            "{second:?}"
        );
        assert!(span.nanos < 1_000_000_000, "a plausible frame: {span:?}");

        let mut perf = crate::perf::PerfMonitor::new();
        let Some(micros) = second.gpu_micros() else {
            panic!("a timed frame reads a gpu time: {second:?}");
        };
        perf.record_gpu_micros(micros);
        assert!(
            perf.snapshot().gpu_micros.is_some_and(|us| us > 0),
            "a 1600x1200 msaa layer and its composite take a microsecond or more"
        );

        // the second frame changed nothing: no layer pass, the composite alone
        let third = draw(&mut r, &mut c);
        assert_eq!(third.get(Stage::GpuLayers).items, 0);
        assert_eq!(third.get(Stage::GpuComposite).calls, 1);
        assert!(third.get(Stage::GpuComposite).nanos > 0);
    }

    #[test]
    fn timestamps_are_written_only_by_frames_drawn_with_stages_on() {
        let Some(mut r) = timed_renderer(Viewport::new(160.0, 120.0, 1.0)) else {
            return;
        };
        stages::enable(false);
        let mut c = Compositor::new();
        for _ in 0..3 {
            scene(&mut c);
            r.draw(&mut c, [0.0, 0.0, 0.0, 1.0]);
        }
        let _on = On::new();
        stages::set_gpu_sync(true);
        let first_on = draw(&mut r, &mut c);
        assert_eq!(
            first_on.get(Stage::GpuSpan).calls,
            0,
            "the frames drawn with stages off wrote nothing to read: {first_on:?}"
        );
        assert_eq!(draw(&mut r, &mut c).get(Stage::GpuSpan).calls, 1);
    }

    #[test]
    fn timestamps_draw_the_same_pixels_as_a_plain_context() {
        let Some(mut plain) = renderer() else {
            return;
        };
        let Some(mut timed) = timed_renderer(Viewport::new(160.0, 120.0, 1.0)) else {
            return;
        };
        stages::enable(false);
        let mut c = Compositor::new();
        scene(&mut c);
        let before = match plain.render(&mut c, [0.0, 0.0, 0.0, 1.0]) {
            Ok(f) => f,
            Err(e) => panic!("{e}"),
        };
        let _on = On::new();
        stages::set_gpu_sync(true);
        let mut c = Compositor::new();
        let mut frames = Vec::new();
        for _ in 0..2 {
            scene(&mut c);
            match timed.render(&mut c, [0.0, 0.0, 0.0, 1.0]) {
                Ok(f) => frames.push(f),
                Err(e) => panic!("{e}"),
            }
        }
        assert_eq!(stages::take().get(Stage::GpuSpan).calls, 1, "timed");
        for measured in frames {
            assert!(
                before.rgba == measured.rgba,
                "the timestamps changed a pixel"
            );
        }
    }

    #[test]
    fn without_the_sync_readings_arrive_later_and_nothing_waits() {
        let Some(mut r) = timed_renderer(Viewport::new(160.0, 120.0, 1.0)) else {
            return;
        };
        let _on = On::new();
        let mut c = Compositor::new();
        let mut readings = 0;
        for _ in 0..20 {
            readings += draw(&mut r, &mut c).get(Stage::GpuSpan).calls;
        }
        // two synced frames: the first sees every frame in flight done,
        // the second reads them back
        stages::set_gpu_sync(true);
        for _ in 0..2 {
            readings += draw(&mut r, &mut c).get(Stage::GpuSpan).calls;
        }
        assert!((1..=21).contains(&readings), "{readings}");
    }

    #[test]
    fn a_frame_with_more_layer_passes_than_a_set_holds_times_the_first_ones() {
        let Some(mut r) = timed_renderer(Viewport::new(160.0, 120.0, 1.0)) else {
            return;
        };
        let _on = On::new();
        stages::set_gpu_sync(true);
        let mut c = Compositor::new();
        let layers: Vec<_> = (1..=70).map(|z| c.create_layer(z)).collect();
        c.begin_frame();
        // the default layer draws too: an empty pass reads no end on Metal
        c.draw_rect(0.0, 20.0, 10.0, 10.0, [0.6, 0.4, 0.2, 1.0]);
        for (i, layer) in layers.iter().enumerate() {
            c.push_to_layer(
                *layer,
                SceneNode::Rect {
                    x: i as f32,
                    y: 0.0,
                    w: 10.0,
                    h: 10.0,
                    color: [0.2, 0.4, 0.6, 1.0],
                },
            );
        }
        r.draw(&mut c, [0.0, 0.0, 0.0, 1.0]);
        let next = draw(&mut r, &mut c);
        assert_eq!(next.get(Stage::GpuLayers).items, 63, "{next:?}");
        assert_eq!(next.get(Stage::GpuComposite).calls, 1);
    }

    #[test]
    fn an_empty_layer_pass_does_not_break_the_reading() {
        let Some(mut r) = timed_renderer(Viewport::new(160.0, 120.0, 1.0)) else {
            return;
        };
        let _on = On::new();
        stages::set_gpu_sync(true);
        let mut c = Compositor::new();
        let _empty = c.create_layer(1);
        draw(&mut r, &mut c);
        let next = draw(&mut r, &mut c);
        let layers = next.get(Stage::GpuLayers);
        // two dirty layers, two passes; Metal samples no end for the one
        // that only clears
        assert!((1..=2).contains(&layers.items), "{next:?}");
        assert!(layers.nanos > 0);
        assert_eq!(next.get(Stage::GpuSpan).calls, 1);
        assert_eq!(next.get(Stage::GpuComposite).calls, 1);
    }
}
