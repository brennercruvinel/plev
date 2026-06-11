//! IR model tests: minimal timeline construction (happy path), every
//! validation error path, and the per-kind animatable surfaces.

use crate::easing::Easing;
use crate::ir::*;

fn rect_props(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Props {
    Props::new()
        .with(Prop::X, Value::Scalar(x))
        .with(Prop::Y, Value::Scalar(y))
        .with(Prop::W, Value::Scalar(w))
        .with(Prop::H, Value::Scalar(h))
        .with(Prop::Color, Value::Color(color))
}

/// One red rect sliding right over half the timeline.
fn minimal_timeline() -> Timeline {
    let rect = Node {
        id: 1,
        depth: 1,
        kind: NodeKind::Rect,
        props: rect_props(10.0, 20.0, 100.0, 50.0, [1.0, 0.0, 0.0, 1.0]),
    };
    Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![Keyframe {
            t: 0.0,
            snapshot: vec![rect],
        }],
        tracks: vec![Track {
            node_id: 1,
            prop: Prop::X,
            start_t: 0.0,
            segments: vec![
                Segment {
                    target: Value::Scalar(200.0),
                    easing: Easing::EaseInOut,
                    dur_s: 0.5,
                },
                Segment {
                    target: Value::Scalar(150.0),
                    easing: Easing::Linear,
                    dur_s: 0.5,
                },
            ],
        }],
        ..Timeline::default()
    }
}

#[test]
fn minimal_timeline_validates_and_roundtrips_structurally() {
    let tl = minimal_timeline();
    assert_eq!(tl.validate(), Ok(()));
    // PartialEq across the whole tree: the codec round-trip contract
    assert_eq!(tl.clone(), tl);
    assert_eq!(tl.tracks[0].end_t(), 1.0);
}

#[test]
fn every_kind_accepts_exactly_its_spec_surface() {
    assert_eq!(
        NodeKind::Rect.animatable_props(),
        &[Prop::X, Prop::Y, Prop::W, Prop::H, Prop::Color]
    );
    assert_eq!(NodeKind::RoundedRect.animatable_props().len(), 8);
    assert_eq!(NodeKind::GradientRect.animatable_props().len(), 10);
    assert_eq!(
        NodeKind::Text { style: 0 }.animatable_props(),
        &[Prop::X, Prop::Y, Prop::Color]
    );
    assert_eq!(
        NodeKind::Image { image: 0 }.animatable_props(),
        &[Prop::X, Prop::Y, Prop::W, Prop::H, Prop::CornerRadius]
    );
    assert!(NodeKind::Path { path: 0 }.animatable_props().is_empty());
    assert!(NodeKind::GradientRect.allows(Prop::AngleDeg));
    assert!(!NodeKind::Rect.allows(Prop::CornerRadius));
}

#[test]
fn props_set_replaces_in_place() {
    let mut p = Props::new();
    p.set(Prop::X, Value::Scalar(1.0));
    p.set(Prop::X, Value::Scalar(2.0));
    assert_eq!(p.len(), 1);
    assert_eq!(p.get(Prop::X), Some(Value::Scalar(2.0)));
    assert_eq!(p.get(Prop::Y), None);
}

#[test]
fn empty_or_late_opening_keyframe_is_rejected() {
    let mut tl = minimal_timeline();
    tl.keyframes.clear();
    assert_eq!(tl.validate(), Err(IrError::MissingOpeningKeyframe));
    let mut tl = minimal_timeline();
    tl.keyframes[0].t = 0.5;
    assert_eq!(tl.validate(), Err(IrError::MissingOpeningKeyframe));
}

#[test]
fn unordered_or_out_of_range_keyframes_are_rejected() {
    let mut tl = minimal_timeline();
    tl.keyframes.push(Keyframe {
        t: 0.0,
        snapshot: vec![],
    });
    assert_eq!(tl.validate(), Err(IrError::KeyframeOutOfOrder { t: 0.0 }));
    let mut tl = minimal_timeline();
    tl.keyframes.push(Keyframe {
        t: 99.0,
        snapshot: vec![],
    });
    assert_eq!(tl.validate(), Err(IrError::KeyframeOutOfOrder { t: 99.0 }));
}

#[test]
fn duplicate_depth_or_id_in_snapshot_is_rejected() {
    let mut tl = minimal_timeline();
    let mut clone = tl.keyframes[0].snapshot[0].clone();
    clone.id = 2; // same depth, new id
    tl.keyframes[0].snapshot.push(clone);
    assert_eq!(
        tl.validate(),
        Err(IrError::DuplicateDepth { t: 0.0, depth: 1 })
    );

    let mut tl = minimal_timeline();
    let mut clone = tl.keyframes[0].snapshot[0].clone();
    clone.depth = 2; // new depth, same id
    tl.keyframes[0].snapshot.push(clone);
    assert_eq!(
        tl.validate(),
        Err(IrError::DuplicateNodeId { t: 0.0, id: 1 })
    );
}

#[test]
fn prop_outside_the_kind_surface_is_rejected() {
    let mut tl = minimal_timeline();
    tl.keyframes[0].snapshot[0]
        .props
        .set(Prop::CornerRadius, Value::Scalar(4.0));
    assert_eq!(
        tl.validate(),
        Err(IrError::PropNotAnimatable {
            kind: "rect",
            prop: Prop::CornerRadius
        })
    );
    // and on a track too
    let mut tl = minimal_timeline();
    tl.tracks[0].prop = Prop::AngleDeg;
    assert_eq!(
        tl.validate(),
        Err(IrError::PropNotAnimatable {
            kind: "rect",
            prop: Prop::AngleDeg
        })
    );
}

#[test]
fn value_kind_mismatch_is_rejected() {
    let mut tl = minimal_timeline();
    tl.keyframes[0].snapshot[0]
        .props
        .set(Prop::Color, Value::Scalar(1.0));
    assert_eq!(
        tl.validate(),
        Err(IrError::ValueKindMismatch {
            prop: Prop::Color,
            expected: "color"
        })
    );
    let mut tl = minimal_timeline();
    tl.tracks[0].segments[0].target = Value::Color([1.0; 4]);
    assert_eq!(
        tl.validate(),
        Err(IrError::ValueKindMismatch {
            prop: Prop::X,
            expected: "scalar"
        })
    );
}

#[test]
fn track_on_unknown_node_is_rejected() {
    let mut tl = minimal_timeline();
    tl.tracks[0].node_id = 77;
    assert_eq!(tl.validate(), Err(IrError::UnknownNode { node_id: 77 }));
}

#[test]
fn track_timing_is_bounded() {
    let mut tl = minimal_timeline();
    tl.tracks[0].segments[0].dur_s = 0.0;
    assert_eq!(
        tl.validate(),
        Err(IrError::NonPositiveDuration { node_id: 1 })
    );
    let mut tl = minimal_timeline();
    tl.tracks[0].start_t = 1.75; // 1.75 + 1.0 > 2.0
    assert_eq!(
        tl.validate(),
        Err(IrError::TrackPastEnd {
            node_id: 1,
            end_t: 2.75,
            duration_s: 2.0
        })
    );
}
