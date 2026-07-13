//! monster codec hot paths: encode a snapshot scene, decode it back, and run
//! the encoder-side optimizer over it. the scene is a flat list of rect nodes
//! with quantizable scalar props, the cheapest representative payload.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use monster::{
    Keyframe, Node, NodeKind, OptimizeCfg, Prop, Props, Timeline, Value, decode, encode, optimize,
};

fn scene(n: u16) -> Timeline {
    let snapshot = (0..n)
        .map(|i| Node {
            id: i,
            depth: i,
            kind: NodeKind::Rect,
            props: Props::new()
                .with(Prop::X, Value::Scalar(i as f32))
                .with(Prop::Y, Value::Scalar(2.0 * i as f32))
                .with(Prop::W, Value::Scalar(10.0))
                .with(Prop::H, Value::Scalar(20.0))
                .with(Prop::Color, Value::Color([0.1, 0.2, 0.3, 1.0])),
        })
        .collect();
    Timeline {
        duration_s: 5.0,
        fps_hint: 60,
        keyframes: vec![Keyframe { t: 0.0, snapshot }],
        tracks: Vec::new(),
        ..Timeline::default()
    }
}

fn bench_codec(c: &mut Criterion) {
    let tl = scene(256);
    let bytes = encode(&tl, &[], &[]).expect("scene encodes");

    c.bench_function("monster_encode_256_nodes", |b| {
        b.iter(|| encode(black_box(&tl), &[], &[]).expect("encode"));
    });
    c.bench_function("monster_decode_256_nodes", |b| {
        b.iter(|| decode(black_box(&bytes)).expect("decode"));
    });
    c.bench_function("monster_optimize_256_nodes", |b| {
        b.iter(|| optimize(black_box(&tl), &OptimizeCfg::default()).expect("optimize"));
    });
}

criterion_group!(benches, bench_codec);
criterion_main!(benches);
