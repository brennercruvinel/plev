//! the frozen golden fixture (spec benchmark gate: golden fixture
//! frozen at first release of the codec; byte-identical builds
//! thereafter). builders and the header parser come from `tests_write`.

use crate::container::{Asset, AssetKind, Desc, SEC_DELTA, SEC_DESC, SEC_KEYFRAME};
use crate::easing::Easing;
use crate::ir::{Keyframe, Node, NodeKind, Prop, Props, Segment, Timeline, Track, Value};
use crate::tests::write::{parse, rect, scalar, seg};
use crate::write::encode;
use sha2::{Digest, Sha256};

/// The golden document: 2 nodes, 2 keyframes, 3 segments, 1 asset,
/// 1 description, 1 deduped custom curve. FROZEN with the fixture.
/// `tests_read` decodes the fixture back against this same document.
pub(crate) fn golden_doc() -> (Timeline, Vec<Asset>, Vec<Desc>) {
    let text = |color: [f32; 4]| Node {
        id: 2,
        depth: 1,
        kind: NodeKind::Text { style: 0 },
        props: Props::new()
            .with(Prop::X, scalar(12.0))
            .with(Prop::Y, scalar(24.0))
            .with(Prop::Color, Value::Color(color)),
    };
    let curve = Easing::CustomBezier {
        x1: 0.33,
        y1: 0.0,
        x2: 0.2,
        y2: 1.4,
    };
    let timeline = Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![
            Keyframe {
                t: 0.0,
                snapshot: vec![rect(1, 0, 10.0), text([1.0, 1.0, 1.0, 1.0])],
            },
            Keyframe {
                t: 1.0,
                snapshot: vec![rect(1, 0, 200.0), text([1.0, 0.5, 0.0, 1.0])],
            },
        ],
        tracks: vec![
            Track {
                node_id: 1,
                prop: Prop::X,
                start_t: 0.0,
                segments: vec![seg(150.0, Easing::EaseOutCubic), seg(200.0, curve)],
            },
            Track {
                node_id: 2,
                prop: Prop::Color,
                start_t: 1.0,
                segments: vec![Segment {
                    target: Value::Color([1.0, 0.5, 0.0, 1.0]),
                    easing: curve,
                    dur_s: 1.0,
                }],
            },
        ],
        ..Timeline::default()
    };
    let assets = vec![Asset {
        kind: AssetKind::TextStyle,
        data: b"style-0".to_vec(),
    }];
    let descs = vec![Desc {
        keyframe: 0,
        text: "two nodes settle into place".into(),
    }];
    (timeline, assets, descs)
}

/// FROZEN: fixtures/golden_v0_minimal.monster was generated once at the
/// codec's first release and committed. If this fails, the encoder
/// changed the wire format; that requires a version bump and a new
/// spec entry, never a fixture refresh.
#[test]
fn golden_fixture_is_frozen_byte_for_byte() {
    let golden = include_bytes!("../../fixtures/golden_v0_minimal.monster");
    let (timeline, assets, descs) = golden_doc();
    let bytes = encode(&timeline, &assets, &descs).unwrap();
    assert_eq!(bytes.as_slice(), golden.as_slice());
}

#[test]
fn golden_header_sections_and_checksums_hold() {
    let (timeline, assets, descs) = golden_doc();
    let bytes = encode(&timeline, &assets, &descs).unwrap();
    let parsed = parse(&bytes);
    assert_eq!(parsed.flags, 0, "zstd flag stays reserved in v0");
    assert_eq!(parsed.asset_count, 1);
    assert_eq!(parsed.curves.len(), 1, "the shared curve dedups");
    let tags: Vec<u8> = parsed.sections.iter().map(|(t, ..)| *t).collect();
    assert_eq!(
        tags,
        [SEC_KEYFRAME, SEC_DELTA, SEC_KEYFRAME, SEC_DELTA, SEC_DESC]
    );
    let (_, t_off, ..) = parsed.sections[4];
    assert_eq!(parsed.desc_offset, t_off, "header points at the T payload");
    for (tag, off, len, digest) in &parsed.sections {
        let payload = &bytes[*off as usize..(*off + *len) as usize];
        let fresh: [u8; 32] = Sha256::digest(payload).into();
        assert_eq!(&fresh, digest, "sha256 of section {}", *tag as char);
    }
    let last = parsed.sections.last().unwrap();
    assert_eq!((last.1 + last.2) as usize, bytes.len(), "no trailing bytes");
}
