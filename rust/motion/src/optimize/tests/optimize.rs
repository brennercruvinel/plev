//! optimizer tests, per-pass side: each pass on constructed redundancy
//! with an exact measured reduction. pipeline-level coverage
//! (idempotence, codec round-trip, tolerance parity, pin safety, error
//! path) lives in tests_optimize_pipe, which borrows the builders and
//! assertion helpers from here.

use crate::easing::Easing;
use crate::ir::{Keyframe, Prop, Segment, Timeline, Track, Value};
use crate::optimize::{OptimizeCfg, optimize};
use crate::play::MonsterPlayer;
use crate::tests::discover::q_scene;
use crate::tests::write::rect;

pub(crate) fn seg(target: f32, dur_s: f32, easing: Easing) -> Segment {
    Segment {
        target: Value::Scalar(target),
        easing,
        dur_s,
    }
}

pub(crate) fn lin(target: f32, dur_s: f32) -> Segment {
    seg(target, dur_s, Easing::Linear)
}

pub(crate) fn col(target: [f32; 4], dur_s: f32) -> Segment {
    Segment {
        target: Value::Color(target),
        easing: Easing::Linear,
        dur_s,
    }
}

pub(crate) fn track(node_id: u16, prop: Prop, start_t: f32, segments: Vec<Segment>) -> Track {
    Track {
        node_id,
        prop,
        start_t,
        segments,
    }
}

/// One rect at x=10 y=20 (the tests_write builder), 2 seconds.
fn one_rect(tracks: Vec<Track>) -> Timeline {
    Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![Keyframe {
            t: 0.0,
            snapshot: vec![rect(1, 0, 10.0)],
        }],
        tracks,
        ..Timeline::default()
    }
}

/// Segment passes only, collapse off, conservative tolerances.
fn only(reduce_rdp: bool, fuse_collinear: bool) -> OptimizeCfg {
    OptimizeCfg {
        collapse_static: false,
        reduce_rdp,
        fuse_collinear,
        ..OptimizeCfg::default()
    }
}

/// Both players quantize to identical scenes on a 50 ms grid: the
/// optimization is invisible on the wire grid.
pub(crate) fn assert_quantized_parity(a: &Timeline, b: &Timeline) {
    let mut pa = MonsterPlayer::new(a.clone()).unwrap();
    let mut pb = MonsterPlayer::new(b.clone()).unwrap();
    let steps = (a.duration_s / 0.05).round() as u32;
    for k in 0..=steps {
        let t = k as f32 * 0.05;
        assert_eq!(
            q_scene(pa.scene_at(t)),
            q_scene(pb.scene_at(t)),
            "scene at t={t}"
        );
    }
}

#[test]
fn constant_and_empty_chains_collapse_under_the_default() {
    let tl = one_rect(vec![
        track(1, Prop::X, 0.0, vec![lin(10.0, 0.5), lin(10.0, 0.5)]),
        track(1, Prop::Y, 0.5, vec![]),
    ]);
    let opt = optimize(&tl, &OptimizeCfg::default()).unwrap();
    assert!(
        opt.tracks.is_empty(),
        "pure redundancy gone: {:?}",
        opt.tracks
    );
}

#[test]
fn wiggle_collapses_under_a_loose_tolerance_and_survives_the_default() {
    let wiggle = vec![lin(10.5, 0.5), lin(9.5, 0.5), lin(10.0, 0.5)];
    let motion = track(1, Prop::Y, 0.0, vec![lin(60.0, 1.0)]);
    let tl = one_rect(vec![track(1, Prop::X, 0.0, wiggle.clone()), motion.clone()]);
    // half a px is ten grid steps: static under a 12-step tolerance.
    let loose = OptimizeCfg {
        static_tol: 12.0,
        ..OptimizeCfg::default()
    };
    let opt = optimize(&tl, &loose).unwrap();
    assert_eq!(opt.tracks, vec![motion], "wiggle collapsed, motion kept");
    // the conservative default keeps any wiggle the wire can show.
    let opt = optimize(&tl, &OptimizeCfg::default()).unwrap();
    assert_eq!(opt.tracks.len(), 2);
    assert_eq!(opt.tracks[0].segments, wiggle);
}

#[test]
fn rdp_collapses_redundant_collinear_keyframes_to_one_segment() {
    let segments: Vec<Segment> = (1..=10)
        .map(|i| lin(10.0 + 9.0 * i as f32, 0.125))
        .collect();
    let tl = one_rect(vec![track(1, Prop::X, 0.0, segments)]);
    let opt = optimize(&tl, &only(true, false)).unwrap();
    assert_eq!(
        opt.tracks[0].segments,
        vec![lin(100.0, 1.25)],
        "ten authored keyframes on one line, one segment"
    );
}

#[test]
fn rdp_preserves_extremes() {
    let up = (1..=4).map(|i| lin(10.0 + 22.5 * i as f32, 0.125));
    let down = (1..=4).map(|i| lin(100.0 - 22.5 * i as f32, 0.125));
    let tl = one_rect(vec![track(1, Prop::X, 0.0, up.chain(down).collect())]);
    let opt = optimize(&tl, &only(true, false)).unwrap();
    assert_eq!(
        opt.tracks[0].segments,
        vec![lin(100.0, 0.5), lin(10.0, 0.5)],
        "the peak survives every chord"
    );
}

#[test]
fn non_linear_segments_bound_the_reduction_runs() {
    let chain = vec![
        lin(30.0, 0.25),
        lin(50.0, 0.25),
        seg(80.0, 0.5, Easing::EaseOutCubic),
        lin(60.0, 0.25),
        lin(40.0, 0.25),
    ];
    let tl = one_rect(vec![track(1, Prop::X, 0.0, chain)]);
    let opt = optimize(&tl, &only(true, true)).unwrap();
    assert_eq!(
        opt.tracks[0].segments,
        vec![
            lin(50.0, 0.5),
            seg(80.0, 0.5, Easing::EaseOutCubic),
            lin(40.0, 0.5)
        ],
        "each collinear pair merges, the eased segment is a fixed wall"
    );
}

#[test]
fn sub_grid_noise_reduces_losslessly_on_the_wire() {
    // interior targets sit 0.4 of a grid step (0.02 px) off the line:
    // under half a step, the wire could never show the difference.
    let noisy = vec![lin(32.52, 0.25), lin(54.98, 0.25), lin(100.0, 0.5)];
    let tl = one_rect(vec![track(1, Prop::X, 0.0, noisy)]);
    let opt = optimize(&tl, &OptimizeCfg::default()).unwrap();
    assert_eq!(opt.tracks[0].segments, vec![lin(100.0, 1.0)]);
    assert_quantized_parity(&tl, &opt);
}

#[test]
fn fusion_merges_collinear_linear_pairs_only() {
    let fuse_only = only(false, true);
    // three collinear segments chain-fuse into one.
    let tl = one_rect(vec![track(
        1,
        Prop::X,
        0.0,
        vec![lin(30.0, 0.25), lin(50.0, 0.25), lin(90.0, 0.5)],
    )]);
    let opt = optimize(&tl, &fuse_only).unwrap();
    assert_eq!(opt.tracks[0].segments, vec![lin(90.0, 1.0)]);
    // a bend past the tolerance stays.
    let bent = vec![lin(30.0, 0.25), lin(40.0, 0.25)];
    let tl = one_rect(vec![track(1, Prop::X, 0.0, bent.clone())]);
    assert_eq!(optimize(&tl, &fuse_only).unwrap().tracks[0].segments, bent);
    // same easing is not enough: only linear survives re-timing.
    let eased = vec![
        seg(30.0, 0.25, Easing::EaseInOut),
        seg(50.0, 0.25, Easing::EaseInOut),
    ];
    let tl = one_rect(vec![track(1, Prop::X, 0.0, eased.clone())]);
    assert_eq!(optimize(&tl, &fuse_only).unwrap().tracks[0].segments, eased);
}
