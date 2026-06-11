//! property tests for the spec's round-trip gate: decode(encode(t)) is
//! structurally identical, 100 percent, over generated grid-aligned
//! timelines (values on the twips/rgba8/bezier-u8 quantization grid, so
//! the wire is lossless), plus byte determinism of the encoder.

use crate::container::{Asset, AssetKind, Desc};
use crate::easing::{Easing, PRESETS};
use crate::ir::{
    Keyframe, Node, NodeKind, PlaceNode, Prop, Props, RemoveNode, ReplaceNode, Segment, Timeline,
    Track, Value,
};
use crate::quant;
use crate::read::decode;
use crate::write::encode;
use proptest::collection::vec as pvec;
use proptest::prelude::*;

/// Scalars on the twips grid round-trip exactly.
fn grid_px() -> impl Strategy<Value = f32> {
    (-200_000i32..200_000).prop_map(quant::twips_to_px)
}

/// Colors on the rgba8 grid round-trip exactly.
fn grid_color() -> impl Strategy<Value = [f32; 4]> {
    proptest::array::uniform4(any::<u8>()).prop_map(quant::bytes_to_rgba)
}

/// Any preset, or a custom bezier already on the u8 grid.
fn arb_easing() -> impl Strategy<Value = Easing> {
    prop_oneof![
        (0..PRESETS.len()).prop_map(|i| PRESETS[i]),
        proptest::array::uniform4(any::<u8>()).prop_map(|[a, b, c, d]| Easing::CustomBezier {
            x1: quant::u8_to_bezier_x(a),
            y1: quant::u8_to_bezier_y(b),
            x2: quant::u8_to_bezier_x(c),
            y2: quant::u8_to_bezier_y(d),
        }),
    ]
}

fn arb_segs(color: bool) -> impl Strategy<Value = Vec<Segment>> {
    let target = if color {
        grid_color().prop_map(Value::Color).boxed()
    } else {
        grid_px().prop_map(Value::Scalar).boxed()
    };
    pvec(
        (target, arb_easing(), 1..=30u32).prop_map(|(target, easing, d)| Segment {
            target,
            easing,
            // durations on a centisecond grid; raw f32 on the wire
            dur_s: d as f32 / 100.0,
        }),
        1..=3,
    )
}

/// Per-node spec: rect geometry plus optional X and Color chains.
type NodeSpec = (
    f32,
    f32,
    f32,
    f32,
    [f32; 4],
    Option<Vec<Segment>>,
    Option<Vec<Segment>>,
);

fn arb_node_spec() -> impl Strategy<Value = NodeSpec> {
    (
        grid_px(),
        grid_px(),
        grid_px(),
        grid_px(),
        grid_color(),
        proptest::option::of(arb_segs(false)),
        proptest::option::of(arb_segs(true)),
    )
}

fn rect(id: u16, depth: u16, x: f32, y: f32, w: f32, h: f32, c: [f32; 4]) -> Node {
    Node {
        id,
        depth,
        kind: NodeKind::Rect,
        // insertion in wire id order, matching the decoder's rebuild
        props: Props::new()
            .with(Prop::X, Value::Scalar(x))
            .with(Prop::Y, Value::Scalar(y))
            .with(Prop::W, Value::Scalar(w))
            .with(Prop::H, Value::Scalar(h))
            .with(Prop::Color, Value::Color(c)),
    }
}

/// One structural delta op: kind 0 place, 1 replace, 2 remove, at a
/// centisecond t in [0,1] (owned by the opening keyframe like the
/// chains), on a depth disjoint from the snapshot's (so removes always
/// leave the tracked nodes alone).
type OpSpec = (u8, u32, u16, (f32, f32, [f32; 4]));

fn arb_op_spec() -> impl Strategy<Value = OpSpec> {
    (
        0..3u8,
        0..=100u32,
        100..104u16,
        (grid_px(), grid_px(), grid_color()),
    )
}

/// All chains share one start in [0,1] owned by the opening keyframe;
/// the optional second keyframe sits at t >= 2, so chains (max end
/// 1.9s) never change owner. Emission order node-by-node, X before
/// Color, is the encoder's canonical order, which the decoder restores;
/// op lists get sorted to canonical (t, depth) order below for the same
/// reason (Timeline equality is ordered).
fn build(
    specs: Vec<NodeSpec>,
    start: u32,
    second: Option<(u32, Vec<f32>)>,
    ops: Vec<OpSpec>,
) -> Timeline {
    let start_t = start as f32 / 100.0;
    let mut snapshot = Vec::new();
    let mut tracks = Vec::new();
    for (i, (x, y, w, h, c, tx, tc)) in specs.into_iter().enumerate() {
        let id = (i + 1) as u16;
        snapshot.push(rect(id, i as u16, x, y, w, h, c));
        for (prop, segments) in [(Prop::X, tx), (Prop::Color, tc)] {
            if let Some(segments) = segments {
                tracks.push(Track {
                    node_id: id,
                    prop,
                    start_t,
                    segments,
                });
            }
        }
    }
    let mut keyframes = vec![Keyframe {
        t: 0.0,
        snapshot: snapshot.clone(),
    }];
    if let Some((kt, xs)) = second {
        for (node, x) in snapshot.iter_mut().zip(xs) {
            node.props.set(Prop::X, Value::Scalar(x));
        }
        keyframes.push(Keyframe {
            t: kt as f32 / 100.0,
            snapshot,
        });
    }
    let mut places = Vec::new();
    let mut replaces = Vec::new();
    let mut removes = Vec::new();
    for (i, (kind, t_cs, depth, (x, y, c))) in ops.into_iter().enumerate() {
        let t = t_cs as f32 / 100.0;
        let node = rect(200 + i as u16, depth, x, y, 10.0, 10.0, c);
        match kind {
            0 => places.push(PlaceNode { t, node }),
            1 => replaces.push(ReplaceNode { t, depth, node }),
            _ => removes.push(RemoveNode { t, depth }),
        }
    }
    places.sort_by_key(|p| (p.t.to_bits(), p.node.depth));
    replaces.sort_by_key(|r| (r.t.to_bits(), r.depth));
    removes.sort_by_key(|r| (r.t.to_bits(), r.depth));
    Timeline {
        duration_s: 10.0,
        fps_hint: 60,
        keyframes,
        tracks,
        places,
        replaces,
        removes,
    }
}

fn arb_timeline() -> impl Strategy<Value = Timeline> {
    (
        pvec(arb_node_spec(), 1..=4),
        0..=100u32,
        proptest::option::of((200..=900u32, pvec(grid_px(), 4))),
        pvec(arb_op_spec(), 0..=4),
    )
        .prop_map(|(specs, start, second, ops)| build(specs, start, second, ops))
}

fn arb_assets() -> impl Strategy<Value = Vec<Asset>> {
    pvec(
        ((0..3u8), pvec(any::<u8>(), 0..16)).prop_map(|(k, data)| Asset {
            kind: match k {
                0 => AssetKind::TextStyle,
                1 => AssetKind::Image,
                _ => AssetKind::Path,
            },
            data,
        }),
        0..3,
    )
}

proptest! {
    #[test]
    fn round_trip_is_structurally_identical(
        timeline in arb_timeline(),
        assets in arb_assets(),
        desc in proptest::option::of(".{0,12}"),
    ) {
        let descs: Vec<Desc> = desc
            .map(|text| Desc { keyframe: 0, text })
            .into_iter()
            .collect();
        let bytes = encode(&timeline, &assets, &descs).expect("generated timelines are valid");
        let doc = decode(&bytes).expect("encoder output must decode");
        prop_assert_eq!(doc.timeline, timeline);
        prop_assert_eq!(doc.assets, assets);
        prop_assert_eq!(doc.descs, descs);
    }

    #[test]
    fn encoding_is_byte_deterministic(timeline in arb_timeline(), assets in arb_assets()) {
        let a = encode(&timeline, &assets, &[]).expect("valid");
        let b = encode(&timeline, &assets, &[]).expect("valid");
        prop_assert_eq!(a, b);
    }
}
