use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::layout::{LayoutEngine, LayoutItem, LayoutStyle};
use engine::path::PathBuilder;
use engine::signal::create_signal;
use engine::text::{TextMeasurer, TextStyle};

// ---------------------------------------------------------------------------
// Group 1: Scene construction (push_rects)
// ---------------------------------------------------------------------------

fn bench_push_rects(c: &mut Criterion) {
    let mut group = c.benchmark_group("push_rects");
    for count in [100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &n| {
            b.iter(|| {
                let mut comp = Compositor::new();
                for i in 0..n {
                    let f = i as f32;
                    comp.draw_rect(f, f, 100.0, 50.0, [0.5, 0.5, 0.5, 1.0]);
                }
                black_box(&comp);
            });
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Group 2: Path construction (push_paths)
// ---------------------------------------------------------------------------

fn bench_push_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("push_paths");

    for count in [100, 1_000] {
        group.bench_with_input(BenchmarkId::new("circles", count), &count, |b, &n| {
            b.iter(|| {
                let mut comp = Compositor::new();
                for i in 0..n {
                    let f = i as f32;
                    let path =
                        PathBuilder::circle(f * 10.0, f * 10.0, 50.0).fill([1.0, 0.0, 0.0, 1.0]);
                    comp.draw_path(path);
                }
                black_box(&comp);
            });
        });

        group.bench_with_input(BenchmarkId::new("rrects", count), &count, |b, &n| {
            b.iter(|| {
                let mut comp = Compositor::new();
                for i in 0..n {
                    let f = i as f32;
                    let path = PathBuilder::rounded_rect(f * 10.0, f * 10.0, 200.0, 100.0, 12.0)
                        .fill([0.0, 0.0, 1.0, 1.0]);
                    comp.draw_path(path);
                }
                black_box(&comp);
            });
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Group 3: Dirty tracking (static scene, steady state)
// ---------------------------------------------------------------------------

fn bench_dirty_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("dirty_tracking");

    group.bench_function("static_1000_rects", |b| {
        let mut comp = Compositor::new();
        for i in 0..1_000 {
            let f = i as f32;
            comp.draw_rect(f, f, 100.0, 50.0, [0.5, 0.5, 0.5, 1.0]);
        }

        b.iter(|| {
            comp.begin_frame();
            for i in 0..1_000 {
                let f = i as f32;
                comp.draw_rect(f, f, 100.0, 50.0, [0.5, 0.5, 0.5, 1.0]);
            }
            black_box(&comp);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 4: Lyon tessellation (per-shape cost)
// ---------------------------------------------------------------------------

fn bench_tessellation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tessellation");

    group.bench_function("circle_r50", |b| {
        b.iter(|| {
            black_box(PathBuilder::circle(100.0, 100.0, 50.0).fill([1.0, 0.0, 0.0, 1.0]));
        });
    });

    group.bench_function("rounded_rect_200x100_r12", |b| {
        b.iter(|| {
            black_box(
                PathBuilder::rounded_rect(0.0, 0.0, 200.0, 100.0, 12.0).fill([0.0, 1.0, 0.0, 1.0]),
            );
        });
    });

    group.bench_function("star_5pt", |b| {
        b.iter(|| {
            let outer = 80.0_f32;
            let inner = 35.0_f32;
            let mut pb = PathBuilder::new();
            for i in 0..5 {
                let angle_outer =
                    std::f32::consts::FRAC_PI_2 + (i as f32) * std::f32::consts::TAU / 5.0;
                let angle_inner = angle_outer + std::f32::consts::TAU / 10.0;
                let ox = 100.0 + outer * angle_outer.cos();
                let oy = 100.0 - outer * angle_outer.sin();
                let ix = 100.0 + inner * angle_inner.cos();
                let iy = 100.0 - inner * angle_inner.sin();
                if i == 0 {
                    pb = pb.move_to(ox, oy);
                } else {
                    pb = pb.line_to(ox, oy);
                }
                pb = pb.line_to(ix, iy);
            }
            pb = pb.close();
            black_box(pb.fill([1.0, 1.0, 0.0, 1.0]));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 5: Signals (create + get + set cycle)
// ---------------------------------------------------------------------------

fn bench_signals(c: &mut Criterion) {
    let mut group = c.benchmark_group("signals");

    group.bench_function("create_get_set_x1000", |b| {
        b.iter(|| {
            for i in 0..1_000 {
                let (read, write) = create_signal(i as f32);
                let _ = black_box(read.get());
                write.set((i as f32) * 2.0);
                let _ = black_box(read.get());
            }
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 6: Text node hashing
// ---------------------------------------------------------------------------

fn bench_text_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_hashing");

    group.bench_function("1000_unique_keys", |b| {
        let keys: Vec<TextNodeKey> = (0..1_000)
            .map(|i| TextNodeKey::new(&format!("Label {i}"), 16.0, 20.8, None))
            .collect();

        b.iter(|| {
            let mut comp = Compositor::new();
            for (i, key) in keys.iter().enumerate() {
                let f = i as f32;
                comp.draw_text(key.clone(), f * 10.0, 0.0, [1.0, 1.0, 1.0, 1.0]);
            }
            black_box(&comp);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 7: The paths the stage probes sit on (task-73), with stages off
// ---------------------------------------------------------------------------
//
// Only APIs that predate the probes, so the same file runs on the commit
// before them: the difference between the two runs is what an idle probe
// costs.

/// Two layers, 200 rects and 20 text runs, 20 ids bound, the frame a small
/// page builds.
fn probed_scene(comp: &mut Compositor, second: LayerId, ids: &[sem::SemId], shade: f32) {
    comp.begin_frame();
    for i in 0..200 {
        let f = i as f32;
        comp.draw_rect(f, f, 100.0, 20.0, [shade, 0.5, 0.5, 1.0]);
    }
    for (i, id) in ids.iter().enumerate() {
        let f = i as f32 * 24.0;
        comp.push_to_layer(
            second,
            SceneNode::Text {
                key: TextNodeKey::new(&format!("row {i}"), 14.0, 20.0, None),
                x: 0.0,
                y: f,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        );
        comp.bind_to_layer(second, id, [0.0, f, 200.0, 24.0]);
    }
}

fn bench_probed_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("probed_paths");
    let ids: Vec<sem::SemId> = (0..20)
        .filter_map(|i| sem::SemId::parse(&format!("bench.row{i}")).ok())
        .collect();

    // build, bind, hit ten times, resolve: hashes both layers, rebuilds none
    group.bench_function("frame_static_220_nodes", |b| {
        let mut comp = Compositor::new();
        let second = comp.create_layer(1);
        probed_scene(&mut comp, second, &ids, 0.5);
        comp.resolve_scene((1200.0, 800.0));
        comp.mark_layer_clean(LayerId::DEFAULT);
        comp.mark_layer_clean(second);
        b.iter(|| {
            probed_scene(&mut comp, second, &ids, 0.5);
            for i in 0..10 {
                black_box(comp.hit(50.0, i as f32 * 24.0 + 4.0));
            }
            comp.resolve_scene((1200.0, 800.0));
            black_box(&comp);
        });
    });

    // the same with the default layer changed every frame: one rebuild
    group.bench_function("frame_dirty_220_nodes", |b| {
        let mut comp = Compositor::new();
        let second = comp.create_layer(1);
        let mut shade = 0.0;
        b.iter(|| {
            shade = if shade > 0.5 { 0.0 } else { 1.0 };
            probed_scene(&mut comp, second, &ids, shade);
            comp.resolve_scene((1200.0, 800.0));
            comp.mark_layer_clean(LayerId::DEFAULT);
            comp.mark_layer_clean(second);
            black_box(&comp);
        });
    });

    // a page's labels, measured every frame: all cache hits
    group.bench_function("measure_100_cached", |b| {
        let style = TextStyle::new(14.0);
        let labels: Vec<String> = (0..100).map(|i| format!("label {i}")).collect();
        for l in &labels {
            TextMeasurer::measure_styled(l, &style, None);
        }
        b.iter(|| {
            for l in &labels {
                black_box(TextMeasurer::measure_styled(l, &style, None));
            }
        });
    });

    // one flex column of 50 fixed rows
    group.bench_function("layout_50_items", |b| {
        let mut items = vec![LayoutItem {
            style: LayoutStyle {
                width: Some(400.0),
                ..Default::default()
            },
            children: (1..=50).collect(),
            text: None,
        }];
        items.extend((0..50).map(|_| LayoutItem {
            style: LayoutStyle {
                height: Some(20.0),
                ..Default::default()
            },
            children: vec![],
            text: None,
        }));
        let mut engine = LayoutEngine::new();
        b.iter(|| black_box(engine.compute(&items, 1200.0, 800.0)));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_push_rects,
    bench_push_paths,
    bench_dirty_tracking,
    bench_tessellation,
    bench_signals,
    bench_text_hashing,
    bench_probed_paths,
);
criterion_main!(benches);
