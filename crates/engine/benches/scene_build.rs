use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use engine::compositor::{Compositor, TextNodeKey};
use engine::path::PathBuilder;
use engine::signal::create_signal;

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

criterion_group!(
    benches,
    bench_push_rects,
    bench_push_paths,
    bench_dirty_tracking,
    bench_tessellation,
    bench_signals,
    bench_text_hashing,
);
criterion_main!(benches);
