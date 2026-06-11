//! structural delta op tests, codec side: the frozen ops fixture, wire
//! layout of an ops-only delta block, round-trip with place, replace
//! and remove mid-timeline, canonical op ordering and the op-specific
//! validation errors. player behavior lives in tests_play_ops.

use crate::container::SEC_DELTA;
use crate::easing::Easing;
use crate::ir::{
    IrError, Keyframe, PlaceNode, Prop, RemoveNode, ReplaceNode, Timeline, Track, Value,
};
use crate::read::decode;
use crate::tests_write::{parse, rect, seg};
use crate::write::encode;

const GOLDEN_OPS: &[u8] = include_bytes!("../fixtures/golden_v0_ops.anm");

/// One rect in the snapshot, a second life at depth 1: placed at 0.5,
/// replaced at 1.0, removed at 1.5, plus one modify chain on node 1.
/// Times are dyadic and values grid-aligned, so decode is exact.
/// FROZEN with the ops fixture; tests_play_ops replays this document.
pub(crate) fn ops_timeline() -> Timeline {
    Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![Keyframe {
            t: 0.0,
            snapshot: vec![rect(1, 0, 10.0)],
        }],
        tracks: vec![Track {
            node_id: 1,
            prop: Prop::X,
            start_t: 0.25,
            segments: vec![seg(150.0, Easing::Linear)],
        }],
        places: vec![PlaceNode {
            t: 0.5,
            node: rect(2, 1, 40.0),
        }],
        replaces: vec![ReplaceNode {
            t: 1.0,
            depth: 1,
            node: rect(3, 1, 80.0),
        }],
        removes: vec![RemoveNode { t: 1.5, depth: 1 }],
    }
}

/// FROZEN: fixtures/golden_v0_ops.anm was generated once when the
/// structural ops became decodable and committed. If this fails, the
/// encoder changed the ops wire; that requires a version bump and a new
/// spec entry, never a fixture refresh.
#[test]
fn ops_fixture_is_frozen_byte_for_byte() {
    let bytes = encode(&ops_timeline(), &[], &[]).unwrap();
    assert_eq!(bytes.as_slice(), GOLDEN_OPS);
}

#[test]
fn ops_fixture_decodes_to_the_authored_timeline() {
    let doc = decode(GOLDEN_OPS).expect("frozen ops fixture must decode");
    assert_eq!(doc.timeline, ops_timeline());
}

#[test]
fn ops_only_delta_block_has_the_spec_wire_layout() {
    let mut tl = ops_timeline();
    tl.tracks.clear();
    tl.replaces.clear();
    tl.removes.clear();
    tl.places = vec![PlaceNode {
        t: 0.5,
        node: rect(2, 1, 40.0),
    }];
    let bytes = encode(&tl, &[], &[]).unwrap();
    let parsed = parse(&bytes);
    let (_, off, len, _) = *parsed
        .sections
        .iter()
        .find(|(t, ..)| *t == SEC_DELTA)
        .expect("a placed node alone must still emit a delta block");
    let d = &bytes[off as usize..(off + len) as usize];
    let mut expect = Vec::new();
    expect.extend_from_slice(&1u16.to_le_bytes()); // op_count
    expect.push(0); // OP_PLACE
    expect.extend_from_slice(&0.5f32.to_le_bytes()); // at_s
    // node: id, depth, kind rect, presence X|Y|W|H|Color, values
    expect.extend_from_slice(&[2, 0, 1, 0, 0, 0x1F, 0]);
    for twips in [800i32, 400, 2000, 1000] {
        expect.extend_from_slice(&twips.to_le_bytes());
    }
    expect.extend_from_slice(&[51, 102, 153, 255]); // rgba8 of rect()
    assert_eq!(d, expect.as_slice());
}

#[test]
fn round_trip_keeps_ops_in_both_segments() {
    // two keyframes; each segment owns a place and a remove, the
    // second also a replace; all times dyadic so kf.t + at_s is exact.
    let mut snapshot = vec![rect(1, 0, 10.0)];
    let timeline = Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![
            Keyframe {
                t: 0.0,
                snapshot: snapshot.clone(),
            },
            Keyframe {
                t: 1.0,
                snapshot: {
                    snapshot[0].props.set(Prop::X, Value::Scalar(60.0));
                    snapshot
                },
            },
        ],
        tracks: vec![Track {
            node_id: 1,
            prop: Prop::X,
            start_t: 0.0,
            segments: vec![seg(60.0, Easing::EaseInOut)],
        }],
        places: vec![
            PlaceNode {
                t: 0.25,
                node: rect(2, 1, 20.0),
            },
            PlaceNode {
                t: 1.25,
                node: rect(4, 2, 30.0),
            },
        ],
        replaces: vec![ReplaceNode {
            t: 1.5,
            depth: 2,
            node: rect(5, 2, 35.0),
        }],
        removes: vec![
            RemoveNode { t: 0.75, depth: 1 },
            RemoveNode { t: 1.75, depth: 2 },
        ],
    };
    let bytes = encode(&timeline, &[], &[]).unwrap();
    let doc = decode(&bytes).unwrap();
    assert_eq!(doc.timeline, timeline);
}

#[test]
fn encoder_emits_ops_in_canonical_order() {
    // authored out of order: the encoder sorts each block by
    // (at_s, place < replace < remove, depth); the decoder hands the
    // lists back in that wire order.
    let mut tl = ops_timeline();
    tl.replaces.clear();
    tl.removes.clear();
    tl.places = vec![
        PlaceNode {
            t: 0.75,
            node: rect(3, 2, 30.0),
        },
        PlaceNode {
            t: 0.25,
            node: rect(2, 1, 20.0),
        },
        PlaceNode {
            t: 0.25,
            node: rect(4, 3, 40.0),
        },
    ];
    let bytes = encode(&tl, &[], &[]).unwrap();
    let got: Vec<(f32, u16)> = decode(&bytes)
        .unwrap()
        .timeline
        .places
        .iter()
        .map(|p| (p.t, p.node.depth))
        .collect();
    assert_eq!(got, vec![(0.25, 1), (0.25, 3), (0.75, 2)]);
    // and the encoding itself is deterministic for the unsorted input.
    assert_eq!(bytes, encode(&tl, &[], &[]).unwrap());
}

#[test]
fn op_validation_errors_are_typed() {
    // op outside [0, duration].
    let mut tl = ops_timeline();
    tl.removes[0].t = 99.0;
    assert_eq!(
        tl.validate(),
        Err(IrError::OpOutOfRange {
            t: 99.0,
            duration_s: 2.0
        })
    );
    // replace whose node sits at another depth.
    let mut tl = ops_timeline();
    tl.replaces[0].node.depth = 5;
    assert_eq!(
        tl.validate(),
        Err(IrError::ReplaceDepthMismatch {
            t: 1.0,
            depth: 1,
            node_depth: 5
        })
    );
    // placed node carrying a prop outside its kind's surface.
    let mut tl = ops_timeline();
    tl.places[0]
        .node
        .props
        .set(Prop::CornerRadius, Value::Scalar(4.0));
    assert_eq!(
        tl.validate(),
        Err(IrError::PropNotAnimatable {
            kind: "rect",
            prop: Prop::CornerRadius
        })
    );
    // a track may target a node that only a place introduces.
    let mut tl = ops_timeline();
    tl.tracks.push(Track {
        node_id: 2,
        prop: Prop::X,
        start_t: 0.5,
        segments: vec![seg(60.0, Easing::Linear)],
    });
    assert_eq!(tl.validate(), Ok(()));
    // but a node no snapshot, place or replace knows stays unknown.
    tl.tracks[1].node_id = 77;
    assert_eq!(tl.validate(), Err(IrError::UnknownNode { node_id: 77 }));
}
