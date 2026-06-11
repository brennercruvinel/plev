//! encoder tests: byte determinism, per-field presence (an unchanged
//! field costs zero bytes), custom-curve dedup and error paths. the
//! frozen golden fixture lives in `tests_golden`, which borrows the
//! builders and the header parser from here.

use crate::container::{self, Asset, AssetKind, DeltaOp, Desc, SEC_DELTA};
use crate::easing::{Easing, EasingTable};
use crate::ir::{Keyframe, Node, NodeKind, Prop, Props, Segment, Timeline, Track, Value};
use crate::write::{WriteError, encode};

pub(crate) fn scalar(v: f32) -> Value {
    Value::Scalar(v)
}

pub(crate) fn rect(id: u16, depth: u16, x: f32) -> Node {
    Node {
        id,
        depth,
        kind: NodeKind::Rect,
        props: Props::new()
            .with(Prop::X, scalar(x))
            .with(Prop::Y, scalar(20.0))
            .with(Prop::W, scalar(100.0))
            .with(Prop::H, scalar(50.0))
            .with(Prop::Color, Value::Color([0.2, 0.4, 0.6, 1.0])),
    }
}

fn one_node_timeline(tracks: Vec<Track>) -> Timeline {
    Timeline {
        duration_s: 1.0,
        fps_hint: 60,
        keyframes: vec![Keyframe {
            t: 0.0,
            snapshot: vec![rect(1, 0, 10.0)],
        }],
        tracks,
        ..Timeline::default()
    }
}

pub(crate) fn seg(target: f32, easing: Easing) -> Segment {
    Segment {
        target: scalar(target),
        easing,
        dur_s: 0.5,
    }
}

fn track(prop: Prop, segments: Vec<Segment>) -> Track {
    Track {
        node_id: 1,
        prop,
        start_t: 0.0,
        segments,
    }
}

/// Header fields and section index, parsed back from the bytes; the
/// tests' independent reading of the layout in `container.rs`.
pub(crate) struct Parsed {
    pub(crate) flags: u16,
    pub(crate) asset_count: u16,
    pub(crate) curves: Vec<[u8; 4]>,
    pub(crate) desc_offset: u32,
    /// (tag, offset, len, sha256)
    pub(crate) sections: Vec<(u8, u32, u32, [u8; 32])>,
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

pub(crate) fn parse(bytes: &[u8]) -> Parsed {
    assert_eq!(&bytes[0..4], b"ANM0");
    assert_eq!(u16_at(bytes, 4), 0, "version");
    let flags = u16_at(bytes, 6);
    let mut at = 14;
    let asset_count = u16_at(bytes, at);
    at += 2;
    for _ in 0..asset_count {
        let len = u16_at(bytes, at + 1) as usize;
        at += 3 + len;
    }
    let curve_count = u16_at(bytes, at);
    at += 2;
    let mut curves = Vec::new();
    for _ in 0..curve_count {
        curves.push(bytes[at..at + 4].try_into().unwrap());
        at += 4;
    }
    let desc_offset = u32_at(bytes, at);
    at += 4;
    let sec_count = u16_at(bytes, at);
    at += 2;
    let mut sections = Vec::new();
    for _ in 0..sec_count {
        let digest: [u8; 32] = bytes[at + 9..at + 41].try_into().unwrap();
        sections.push((
            bytes[at],
            u32_at(bytes, at + 1),
            u32_at(bytes, at + 5),
            digest,
        ));
        at += 41;
    }
    Parsed {
        flags,
        asset_count,
        curves,
        desc_offset,
        sections,
    }
}

fn section_payload<'a>(bytes: &'a [u8], parsed: &Parsed, tag: u8, nth: usize) -> &'a [u8] {
    let (_, off, len, _) = *parsed
        .sections
        .iter()
        .filter(|(t, ..)| *t == tag)
        .nth(nth)
        .unwrap_or_else(|| panic!("section {} #{nth} missing", tag as char));
    &bytes[off as usize..(off + len) as usize]
}

#[test]
fn same_timeline_encodes_to_identical_bytes() {
    let build = || {
        let tl = one_node_timeline(vec![track(
            Prop::X,
            vec![
                seg(150.0, Easing::EaseOutCubic),
                seg(
                    200.0,
                    Easing::CustomBezier {
                        x1: 0.3,
                        y1: 0.0,
                        x2: 0.2,
                        y2: 1.4,
                    },
                ),
            ],
        )]);
        let assets = vec![Asset {
            kind: AssetKind::TextStyle,
            data: b"s".to_vec(),
        }];
        let descs = vec![Desc {
            keyframe: 0,
            text: "slide".into(),
        }];
        encode(&tl, &assets, &descs).unwrap()
    };
    let first = build();
    assert_eq!(first, build(), "independent builds must agree");
    let tl = one_node_timeline(vec![track(Prop::X, vec![seg(150.0, Easing::Linear)])]);
    assert_eq!(
        encode(&tl, &[], &[]).unwrap(),
        encode(&tl, &[], &[]).unwrap()
    );
}

#[test]
fn modify_of_one_field_is_minimal() {
    let one = encode(
        &one_node_timeline(vec![track(Prop::X, vec![seg(150.0, Easing::EaseOutCubic)])]),
        &[],
        &[],
    )
    .unwrap();
    let parsed = parse(&one);
    let d = section_payload(&one, &parsed, SEC_DELTA, 0);
    // op_count + (op_code, at_s, node_id, presence) + seg_count +
    // (easing byte, dur f32, target i32): nothing else on the wire.
    let mut expect = Vec::new();
    expect.extend_from_slice(&1u16.to_le_bytes()); // op_count
    expect.push(1); // OP_MODIFY
    expect.extend_from_slice(&0.0f32.to_le_bytes()); // at_s
    expect.extend_from_slice(&1u16.to_le_bytes()); // node_id
    expect.extend_from_slice(&0b1u16.to_le_bytes()); // presence: X only
    expect.extend_from_slice(&1u16.to_le_bytes()); // seg_count
    expect.push(0x06); // EaseOutCubic
    expect.extend_from_slice(&0.5f32.to_le_bytes()); // dur_s
    expect.extend_from_slice(&3000i32.to_le_bytes()); // 150 px in twips
    assert_eq!(d, expect.as_slice());
    assert_eq!(d.len(), 22);

    // a second animated field adds exactly one chain (11 bytes here);
    // the unchanged fields still cost zero.
    let two = encode(
        &one_node_timeline(vec![
            track(Prop::X, vec![seg(150.0, Easing::EaseOutCubic)]),
            track(Prop::Y, vec![seg(80.0, Easing::EaseOutCubic)]),
        ]),
        &[],
        &[],
    )
    .unwrap();
    let parsed2 = parse(&two);
    let d2 = section_payload(&two, &parsed2, SEC_DELTA, 0);
    assert_eq!(d2.len(), d.len() + 11);
    assert_eq!(u16_at(d2, 9), 0b11, "presence mask X|Y");
}

#[test]
fn custom_curves_dedup_to_one_table_entry() {
    let curve = Easing::CustomBezier {
        x1: 0.33,
        y1: 0.0,
        x2: 0.2,
        y2: 1.4,
    };
    // quantization-equal control points must dedup too (u8 step 1/255).
    let near = Easing::CustomBezier {
        x1: 0.3305,
        y1: 0.0,
        x2: 0.2,
        y2: 1.4,
    };
    let bytes = encode(
        &one_node_timeline(vec![
            track(Prop::X, vec![seg(150.0, curve)]),
            track(Prop::Y, vec![seg(80.0, near)]),
        ]),
        &[],
        &[],
    )
    .unwrap();
    let parsed = parse(&bytes);
    assert_eq!(parsed.curves.len(), 1, "two segments, one deduped curve");
    let d = section_payload(&bytes, &parsed, SEC_DELTA, 0);
    // both segments carry 0xFF plus index 0 into the shared table.
    let hits: Vec<usize> = (0..d.len()).filter(|&i| d[i] == 0xFF).collect();
    assert_eq!(hits.len(), 2);
    for at in hits {
        assert_eq!(u16_at(d, at + 1), 0);
    }

    let distinct = encode(
        &one_node_timeline(vec![
            track(Prop::X, vec![seg(150.0, curve)]),
            track(
                Prop::Y,
                vec![seg(
                    80.0,
                    Easing::CustomBezier {
                        x1: 0.9,
                        y1: 0.1,
                        x2: 0.1,
                        y2: 0.9,
                    },
                )],
            ),
        ]),
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(parse(&distinct).curves.len(), 2);
}

#[test]
fn duplicate_track_on_same_field_and_start_is_rejected() {
    let tl = one_node_timeline(vec![
        track(Prop::X, vec![seg(150.0, Easing::Linear)]),
        track(Prop::X, vec![seg(90.0, Easing::Linear)]),
    ]);
    assert_eq!(
        encode(&tl, &[], &[]),
        Err(WriteError::DuplicateTrack {
            node_id: 1,
            prop: Prop::X
        })
    );
    // same field at a different start is two ops, not a duplicate.
    let staggered = one_node_timeline(vec![
        track(Prop::X, vec![seg(150.0, Easing::Linear)]),
        Track {
            node_id: 1,
            prop: Prop::X,
            start_t: 0.5,
            segments: vec![seg(90.0, Easing::Linear)],
        },
    ]);
    assert!(encode(&staggered, &[], &[]).is_ok());
}

#[test]
fn description_track_errors_are_rejected() {
    let tl = one_node_timeline(vec![]);
    assert_eq!(
        encode(
            &tl,
            &[],
            &[Desc {
                keyframe: 3,
                text: "x".into()
            }]
        ),
        Err(WriteError::DescOutOfRange {
            keyframe: 3,
            keyframes: 1
        })
    );
    let twice = [
        Desc {
            keyframe: 0,
            text: "a".into(),
        },
        Desc {
            keyframe: 0,
            text: "b".into(),
        },
    ];
    assert_eq!(
        encode(&tl, &[], &twice),
        Err(WriteError::DuplicateDesc { keyframe: 0 })
    );
}

/// Pin the wire bytes of the structural ops; encoder coverage of their
/// timeline lists lives in tests_ops.
#[test]
fn place_replace_remove_ops_serialize_per_spec() {
    let table = EasingTable::default();
    let node = rect(1, 0, 10.0);
    // id, depth, kind rect, presence X|Y|W|H|Color, twips, rgba8.
    let mut node_bytes: Vec<u8> = vec![1, 0, 0, 0, 0, 0x1F, 0];
    for twips in [200i32, 400, 2000, 1000] {
        node_bytes.extend_from_slice(&twips.to_le_bytes());
    }
    node_bytes.extend_from_slice(&[51, 102, 153, 255]);

    let mut buf = Vec::new();
    let place = DeltaOp::Place {
        at_s: 0.25,
        node: node.clone(),
    };
    container::put_op(&mut buf, &place, &table);
    let mut expect = vec![0u8]; // OP_PLACE
    expect.extend_from_slice(&0.25f32.to_le_bytes());
    expect.extend_from_slice(&node_bytes);
    assert_eq!(buf, expect);

    buf.clear();
    container::put_op(&mut buf, &DeltaOp::Replace { at_s: 0.25, node }, &table);
    expect[0] = 2; // OP_REPLACE, same body as place
    assert_eq!(buf, expect);

    buf.clear();
    let remove = DeltaOp::Remove {
        at_s: 0.25,
        depth: 7,
    };
    container::put_op(&mut buf, &remove, &table);
    let mut expect = vec![3u8]; // OP_REMOVE
    expect.extend_from_slice(&0.25f32.to_le_bytes());
    expect.extend_from_slice(&7u16.to_le_bytes());
    assert_eq!(buf, expect);
}
