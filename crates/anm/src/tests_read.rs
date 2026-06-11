//! decoder tests: golden fixture decode and structural round-trip over
//! every node kind and value type. malformed-input and fuzz coverage
//! lives in tests_read_malformed.rs.

use crate::container::{Asset, AssetKind, Desc};
use crate::easing::{Easing, quantize_curve};
use crate::ir::{Keyframe, Node, NodeKind, Prop, Props, Segment, Timeline, Track, Value};
use crate::quant;
use crate::read::{ReadError, decode};
use crate::tests_golden::golden_doc;
use crate::tests_write::parse;
use crate::write::encode;

const GOLDEN: &[u8] = include_bytes!("../fixtures/golden_v0_minimal.anm");

/// What the decoder must hand back for a value that went through the
/// wire: the quantized-grid form of the authored value.
fn quantize_value(prop: Prop, v: Value) -> Value {
    match (prop, v) {
        (Prop::AngleDeg, Value::Scalar(d)) => {
            Value::Scalar(quant::u16_to_angle_deg(quant::angle_deg_to_u16(d)))
        }
        (_, Value::Scalar(px)) => Value::Scalar(quant::twips_to_px(quant::px_to_twips(px))),
        (_, Value::Color(c)) => Value::Color(quant::bytes_to_rgba(quant::rgba_to_bytes(c))),
    }
}

fn quantize_easing(e: Easing) -> Easing {
    match e {
        Easing::CustomBezier { x1, y1, x2, y2 } => {
            let [a, b, c, d] = quantize_curve(x1, y1, x2, y2);
            Easing::CustomBezier {
                x1: quant::u8_to_bezier_x(a),
                y1: quant::u8_to_bezier_y(b),
                x2: quant::u8_to_bezier_x(c),
                y2: quant::u8_to_bezier_y(d),
            }
        }
        preset => preset,
    }
}

fn quantize_timeline(mut tl: Timeline) -> Timeline {
    for kf in &mut tl.keyframes {
        for node in &mut kf.snapshot {
            let entries: Vec<(Prop, Value)> = node.props.iter().copied().collect();
            for (p, v) in entries {
                node.props.set(p, quantize_value(p, v));
            }
        }
    }
    for track in &mut tl.tracks {
        for seg in &mut track.segments {
            seg.target = quantize_value(track.prop, seg.target);
            seg.easing = quantize_easing(seg.easing);
        }
    }
    tl
}

#[test]
fn golden_fixture_decodes_to_the_expected_document() {
    let (timeline, assets, descs) = golden_doc();
    let doc = decode(GOLDEN).expect("frozen fixture must decode");
    assert_eq!(doc.timeline, quantize_timeline(timeline));
    assert_eq!(doc.assets, assets);
    assert_eq!(doc.descs, descs);
}

/// Grid-aligned timeline exercising all six node kinds, twips, rgba8
/// and angle values, presets and a custom curve: decode(encode(t)) is
/// t exactly. Tracks are listed in the canonical wire order (keyframe,
/// at_s, node id, prop wire id) because Timeline equality is ordered.
#[test]
fn round_trip_covers_every_node_kind_and_value_type() {
    let gray = Value::Color(quant::bytes_to_rgba([10, 20, 30, 255]));
    let snapshot = vec![
        Node {
            id: 1,
            depth: 0,
            kind: NodeKind::Rect,
            props: Props::new()
                .with(Prop::X, Value::Scalar(1.05))
                .with(Prop::Color, gray),
        },
        Node {
            id: 2,
            depth: 1,
            kind: NodeKind::RoundedRect,
            props: Props::new()
                .with(Prop::BorderWidth, Value::Scalar(2.5))
                .with(Prop::BorderColor, gray),
        },
        Node {
            id: 3,
            depth: 2,
            kind: NodeKind::GradientRect,
            props: Props::new()
                .with(Prop::Color2, gray)
                .with(Prop::AngleDeg, Value::Scalar(quant::u16_to_angle_deg(8192))),
        },
        Node {
            id: 4,
            depth: 3,
            kind: NodeKind::Text { style: 0 },
            props: Props::new().with(Prop::Y, Value::Scalar(-3.2)),
        },
        Node {
            id: 5,
            depth: 4,
            kind: NodeKind::Image { image: 1 },
            props: Props::new().with(Prop::CornerRadius, Value::Scalar(4.0)),
        },
        Node {
            id: 6,
            depth: 5,
            kind: NodeKind::Path { path: 2 },
            props: Props::new(),
        },
    ];
    let curve = Easing::CustomBezier {
        x1: quant::u8_to_bezier_x(84),
        y1: quant::u8_to_bezier_y(64),
        x2: quant::u8_to_bezier_x(51),
        y2: quant::u8_to_bezier_y(242),
    };
    let seg = |target, easing| Segment {
        target,
        easing,
        dur_s: 0.25,
    };
    let timeline = Timeline {
        duration_s: 2.0,
        fps_hint: 30,
        keyframes: vec![
            Keyframe {
                t: 0.0,
                snapshot: snapshot.clone(),
            },
            Keyframe { t: 1.5, snapshot },
        ],
        tracks: vec![
            Track {
                node_id: 1,
                prop: Prop::X,
                start_t: 0.0,
                segments: vec![
                    seg(Value::Scalar(8.05), curve),
                    seg(Value::Scalar(0.0), Easing::Hold),
                ],
            },
            Track {
                node_id: 3,
                prop: Prop::AngleDeg,
                start_t: 0.5,
                segments: vec![seg(
                    Value::Scalar(quant::u16_to_angle_deg(40000)),
                    Easing::Step,
                )],
            },
            Track {
                node_id: 2,
                prop: Prop::BorderColor,
                start_t: 1.5,
                segments: vec![seg(gray, Easing::EaseInOutBounce)],
            },
        ],
    };
    let assets = vec![
        Asset {
            kind: AssetKind::TextStyle,
            data: b"ts".to_vec(),
        },
        Asset {
            kind: AssetKind::Image,
            data: vec![0, 1, 2],
        },
        Asset {
            kind: AssetKind::Path,
            data: vec![],
        },
    ];
    let descs = vec![
        Desc {
            keyframe: 0,
            text: "abre".into(),
        },
        Desc {
            keyframe: 1,
            text: "assenta".into(),
        },
    ];
    let bytes = encode(&timeline, &assets, &descs).unwrap();
    let doc = decode(&bytes).unwrap();
    assert_eq!(doc.timeline, timeline);
    assert_eq!(doc.assets, assets);
    assert_eq!(doc.descs, descs);
}
