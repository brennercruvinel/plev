//! encoder mode B tests, motion side: linear motion collapses to one
//! segment, discontinuities become keyframes, the cadence inserts
//! random access snapshots, the discovered timeline round-trips the
//! codec, the player reproduces every input sample bit-exactly after
//! quantization, and malformed sequences are typed errors. structural
//! op discovery (place/replace/remove, pins) lives in
//! tests_discover_ops, which borrows the helpers from here.

use crate::discover::{DiscoverConfig, DiscoverError, discover};
use crate::discover_fit::quantize_value;
use crate::easing::Easing;
use crate::ir::{IrError, Node, NodeKind, Prop, Props, Segment, Timeline, Value};
use crate::lower::lower_scene;
use crate::play::AnmPlayer;
use crate::quant;
use crate::read::decode;
use crate::tests::write::rect;
use crate::write::encode;
use plev::compositor::SceneNode;

pub(crate) fn cfg(keyframe_every_s: f32, scalar_jump: f32) -> DiscoverConfig {
    DiscoverConfig {
        keyframe_every_s,
        scalar_jump,
        color_jump: 0.5,
    }
}

pub(crate) fn seg(target: f32, dur_s: f32) -> Segment {
    Segment {
        target: Value::Scalar(target),
        easing: Easing::Linear,
        dur_s,
    }
}

/// Quantize the lowered scene so player f32 noise under half a grid
/// step compares equal ("PartialEq after quantization"). Shared with
/// tests_optimize, whose parity checks measure the same grid.
pub(crate) fn q_scene(scene: Vec<SceneNode>) -> Vec<SceneNode> {
    let qpx = |v: f32| quant::twips_to_px(quant::px_to_twips(v));
    let qc = |c: [f32; 4]| quant::bytes_to_rgba(quant::rgba_to_bytes(c));
    scene
        .into_iter()
        .map(|n| match n {
            SceneNode::Rect { x, y, w, h, color } => SceneNode::Rect {
                x: qpx(x),
                y: qpx(y),
                w: qpx(w),
                h: qpx(h),
                color: qc(color),
            },
            other => other,
        })
        .collect()
}

/// The discovered player must hit every input snapshot at its sample
/// time; between samples it interpolates, at samples it agrees.
pub(crate) fn assert_reproduced(timeline: &Timeline, frames: &[(f32, Vec<Node>)]) {
    let mut player = AnmPlayer::new(timeline.clone()).unwrap();
    for (t, nodes) in frames {
        assert_eq!(
            q_scene(player.scene_at(*t)),
            q_scene(lower_scene(nodes, &[])),
            "scene at t={t}"
        );
    }
}

/// 2 seconds at 4 fps, one rect moving linearly: x = 10 + 20t.
fn linear_frames() -> Vec<(f32, Vec<Node>)> {
    (0..=8)
        .map(|i| {
            let t = i as f32 * 0.25;
            (t, vec![rect(1, 0, 10.0 + 20.0 * t)])
        })
        .collect()
}

#[test]
fn linear_motion_collapses_to_one_segment() {
    let frames = linear_frames();
    let tl = discover(&frames, &cfg(100.0, 8.0)).unwrap();
    assert_eq!(tl.keyframes.len(), 1);
    assert!(tl.places.is_empty() && tl.replaces.is_empty() && tl.removes.is_empty());
    assert_eq!(tl.tracks.len(), 1);
    let track = &tl.tracks[0];
    assert_eq!(
        (track.node_id, track.prop, track.start_t),
        (1, Prop::X, 0.0)
    );
    assert_eq!(track.segments, vec![seg(50.0, 2.0)], "one segment, not N");
    assert_eq!((tl.duration_s, tl.fps_hint), (2.0, 4));
    assert_reproduced(&tl, &frames);
}

#[test]
fn curved_motion_splits_per_sample_and_stays_faithful() {
    // x = 20t^2: no two consecutive steps are collinear, so v0 (no
    // easing recovery) keeps one linear segment per sample step.
    let frames: Vec<(f32, Vec<Node>)> = (0..=4)
        .map(|i| {
            let t = i as f32 * 0.25;
            (t, vec![rect(1, 0, 20.0 * t * t)])
        })
        .collect();
    let tl = discover(&frames, &cfg(100.0, 20.0)).unwrap();
    assert_eq!(tl.keyframes.len(), 1);
    assert_eq!(tl.tracks.len(), 1);
    assert_eq!(tl.tracks[0].segments.len(), 4);
    assert_reproduced(&tl, &frames);
}

#[test]
fn discontinuity_becomes_a_keyframe() {
    let frames: Vec<(f32, Vec<Node>)> = (0..=8)
        .map(|i| {
            let t = i as f32 * 0.25;
            let x = if t < 1.0 {
                10.0 + 20.0 * t
            } else {
                100.0 + 20.0 * (t - 1.0)
            };
            (t, vec![rect(1, 0, x)])
        })
        .collect();
    let tl = discover(&frames, &cfg(100.0, 8.0)).unwrap();
    let kf_ts: Vec<f32> = tl.keyframes.iter().map(|k| k.t).collect();
    assert_eq!(
        kf_ts,
        vec![0.0, 1.0],
        "the 75px jump snapshots, cadence is off"
    );
    assert_eq!(
        tl.keyframes[1].snapshot[0].props.get(Prop::X),
        Some(Value::Scalar(100.0))
    );
    assert_eq!(tl.tracks.len(), 2);
    assert_eq!(
        tl.tracks[0].segments,
        vec![seg(25.0, 0.75)],
        "jump never lands"
    );
    assert_eq!(tl.tracks[1].start_t, 1.0);
    assert_eq!(tl.tracks[1].segments, vec![seg(120.0, 1.0)]);
    assert_reproduced(&tl, &frames);
}

#[test]
fn cadence_inserts_keyframes_and_motion_lands_on_them() {
    let frames: Vec<(f32, Vec<Node>)> = (0..=16)
        .map(|i| {
            let t = i as f32 * 0.25;
            (t, vec![rect(1, 0, 10.0 + 20.0 * t)])
        })
        .collect();
    let tl = discover(&frames, &cfg(1.0, 8.0)).unwrap();
    let kf_ts: Vec<f32> = tl.keyframes.iter().map(|k| k.t).collect();
    assert_eq!(
        kf_ts,
        vec![0.0, 1.0, 2.0, 3.0, 4.0],
        "random access cadence"
    );
    assert_eq!(tl.tracks.len(), 4);
    for (w, track) in tl.tracks.iter().enumerate() {
        assert_eq!(track.start_t, w as f32);
        let next_x = 10.0 + 20.0 * (w + 1) as f32;
        assert_eq!(
            track.segments,
            vec![seg(next_x, 1.0)],
            "continuous motion lands exactly on the next snapshot"
        );
    }
    assert_reproduced(&tl, &frames);
}

/// Everything at once: linear motion plus a color fade on node 1, an
/// intermittent node 2, a placed node 3 replaced by node 4, and the
/// 1-second cadence splitting it into three windows.
fn rich_frames() -> Vec<(f32, Vec<Node>)> {
    let node1 = |t: f32| {
        let c = 0.8 * t.min(1.0);
        Node {
            id: 1,
            depth: 0,
            kind: NodeKind::Rect,
            props: Props::new()
                .with(Prop::X, Value::Scalar(10.0 + 20.0 * t))
                .with(Prop::Y, Value::Scalar(20.0))
                .with(Prop::W, Value::Scalar(100.0))
                .with(Prop::H, Value::Scalar(50.0))
                .with(Prop::Color, Value::Color([c, c, c, 1.0])),
        }
    };
    (0..=8)
        .map(|i| {
            let t = i as f32 * 0.25;
            let mut scene = vec![node1(t)];
            if (1..=2).contains(&i) {
                scene.push(rect(2, 1, 20.0));
            }
            if (5..=6).contains(&i) {
                scene.push(rect(3, 2, 30.0));
            }
            if i >= 7 {
                scene.push(rect(4, 2, 35.0));
            }
            (t, scene)
        })
        .collect()
}

#[test]
fn discovered_timeline_round_trips_the_codec() {
    let tl = discover(&rich_frames(), &cfg(1.0, 8.0)).unwrap();
    let kf_ts: Vec<f32> = tl.keyframes.iter().map(|k| k.t).collect();
    assert_eq!(kf_ts, vec![0.0, 1.0, 2.0]);
    assert_eq!(
        (tl.places.len(), tl.replaces.len(), tl.removes.len()),
        (2, 1, 1)
    );
    assert_eq!(tl.tracks.len(), 3, "x and color in window 0, x in window 1");
    assert!(tl.tracks.iter().all(|t| t.segments.len() == 1));
    let doc = decode(&encode(&tl, &[], &[]).unwrap()).unwrap();
    assert_eq!(doc.timeline, tl);
}

#[test]
fn player_reproduces_the_sampled_sequence_through_the_full_pipeline() {
    let frames = rich_frames();
    let tl = discover(&frames, &cfg(1.0, 8.0)).unwrap();
    assert_reproduced(&tl, &frames);
    // and again after a trip through the wire.
    let doc = decode(&encode(&tl, &[], &[]).unwrap()).unwrap();
    assert_reproduced(&doc.timeline, &frames);
}

#[test]
fn input_quantizes_to_the_wire_grid() {
    assert_eq!(
        quantize_value(Prop::X, Value::Scalar(10.013)),
        Value::Scalar(10.0),
        "twentieths of a px"
    );
    assert_eq!(
        quantize_value(Prop::AngleDeg, Value::Scalar(360.0)),
        Value::Scalar(0.0),
        "angles wrap the turn"
    );
    let tl = discover(
        &[(0.0, vec![rect(1, 0, 10.013)])],
        &DiscoverConfig::default(),
    )
    .unwrap();
    assert_eq!(
        tl.keyframes[0].snapshot[0].props.get(Prop::X),
        Some(Value::Scalar(10.0))
    );
    assert_eq!(tl.duration_s, 0.0);
    assert!(tl.tracks.is_empty());
}

#[test]
fn malformed_sequences_are_typed_errors() {
    let c = DiscoverConfig::default();
    assert_eq!(discover(&[], &c), Err(DiscoverError::Empty));
    assert_eq!(
        discover(&[(0.5, vec![])], &c),
        Err(DiscoverError::FirstFrameNotZero { t: 0.5 })
    );
    assert_eq!(
        discover(&[(0.0, vec![]), (0.0, vec![])], &c),
        Err(DiscoverError::FrameOutOfOrder { t: 0.0 })
    );
    assert_eq!(
        discover(&[(0.0, vec![]), (f32::NAN, vec![])], &c),
        Err(DiscoverError::NonFiniteTime { at: 1 })
    );
    assert_eq!(
        discover(&[(0.0, vec![rect(1, 0, 10.0), rect(2, 0, 20.0)])], &c),
        Err(DiscoverError::DuplicateDepth { t: 0.0, depth: 0 })
    );
    assert_eq!(
        discover(&[(0.0, vec![rect(1, 0, 10.0), rect(1, 1, 20.0)])], &c),
        Err(DiscoverError::DuplicateId { t: 0.0, id: 1 })
    );
    let bad = Node {
        id: 1,
        depth: 0,
        kind: NodeKind::Rect,
        props: Props::new().with(Prop::CornerRadius, Value::Scalar(4.0)),
    };
    assert!(matches!(
        discover(&[(0.0, vec![bad])], &c),
        Err(DiscoverError::Ir(IrError::PropNotAnimatable { .. }))
    ));
}
