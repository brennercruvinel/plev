//! mode B end to end, measured (verification workstream): a synthetic
//! rich animation (20 nodes: 12 eased movers, 4 color faders, 2 size
//! pulsers with a hard reset, 2 intermittent nodes that enter and
//! leave twice) is sampled at 60 fps for 4 s, then pushed through
//! discover -> optimize -> encode -> decode -> play. the played scene
//! at every sample time is compared against the quantized input; the
//! pass bound is one wire grid step per prop (0.05 px on coordinates,
//! 1/255 per color channel), twice what quantization alone commits,
//! because discovery fits and RDP each spend strictly less than half a
//! step. byte and B/s numbers print with --nocapture.

use monster::{
    DiscoverConfig, Easing, MonsterPlayer, Node, NodeKind, OptimizeCfg, Prop, Props, Value, decode,
    discover, encode, optimize, quantize_value,
};
use plev::compositor::SceneNode;

const FPS: usize = 60;
const DUR_S: f32 = 4.0;

fn rect(id: u16, depth: u16, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Node {
    Node {
        id,
        depth,
        kind: NodeKind::Rect,
        props: Props::new()
            .with(Prop::X, Value::Scalar(x))
            .with(Prop::Y, Value::Scalar(y))
            .with(Prop::W, Value::Scalar(w))
            .with(Prop::H, Value::Scalar(h))
            .with(Prop::Color, Value::Color(color)),
    }
}

/// The authored scene at `t`: nothing here knows about the codec; the
/// codec has to rediscover all of it from samples.
fn scene_at(t: f32) -> Vec<Node> {
    let mut nodes = Vec::new();
    // 12 movers: triangle sweeps through four different easings.
    let eases = [
        Easing::EaseInOutCubic,
        Easing::EaseOutBack,
        Easing::EaseInOutSine,
        Easing::EaseOutQuart,
    ];
    for i in 0..12u16 {
        let phase = (t / DUR_S + i as f32 / 12.0) % 1.0;
        let lin = if phase < 0.5 {
            phase * 2.0
        } else {
            2.0 - phase * 2.0
        };
        let k = eases[i as usize % 4].sample(lin);
        let x = 40.0 + 30.0 * i as f32 + 220.0 * k;
        let y = 60.0 + 24.0 * i as f32;
        nodes.push(rect(1 + i, i, x, y, 24.0, 24.0, [0.2, 0.4, 0.8, 1.0]));
    }
    // 4 color faders.
    for i in 0..4u16 {
        let k = Easing::EaseInOutSine.sample((t * (0.25 + 0.125 * i as f32)) % 1.0);
        let color = [k, 0.5, 1.0 - k, 1.0];
        nodes.push(rect(
            13 + i,
            12 + i,
            500.0,
            40.0 * i as f32,
            60.0,
            20.0,
            color,
        ));
    }
    // 2 size pulsers with a hard reset at t = 2 s (a discontinuity the
    // discoverer must turn into a snapshot keyframe, never a segment).
    for i in 0..2u16 {
        let k = Easing::EaseInOut.sample((t / 2.0) % 1.0);
        let w = 40.0 + 160.0 * k;
        let y = 100.0 + 120.0 * i as f32;
        nodes.push(rect(
            17 + i,
            16 + i,
            620.0,
            y,
            w,
            30.0,
            [0.9, 0.6, 0.1, 1.0],
        ));
    }
    // 2 intermittent nodes: place/remove ops territory.
    if (0.5..1.5).contains(&t) || (2.2..3.4).contains(&t) {
        let x = 100.0 + 50.0 * (t - 0.5);
        nodes.push(rect(19, 18, x, 400.0, 30.0, 30.0, [0.1, 0.8, 0.3, 1.0]));
    }
    if (1.0..2.0).contains(&t) || (3.0..3.8).contains(&t) {
        let x = 300.0 - 40.0 * (t - 1.0);
        nodes.push(rect(20, 19, x, 440.0, 30.0, 30.0, [0.8, 0.2, 0.6, 1.0]));
    }
    nodes
}

/// (x, y, w, h, color) of the only kind this scene uses.
fn rect_fields(node: &SceneNode) -> (f32, f32, f32, f32, [f32; 4]) {
    match node {
        SceneNode::Rect { x, y, w, h, color } => (*x, *y, *w, *h, *color),
        other => panic!("mode B scene lowers to rects only, got {other:?}"),
    }
}

/// Quantized input node lowered by hand to the same tuple.
fn expected_fields(node: &Node) -> (f32, f32, f32, f32, [f32; 4]) {
    let sc = |p: Prop| match quantize_value(p, node.props.get(p).expect("prop set")) {
        Value::Scalar(v) => v,
        Value::Color(_) => unreachable!("scalar prop"),
    };
    let Value::Color(c) = quantize_value(Prop::Color, node.props.get(Prop::Color).expect("color"))
    else {
        unreachable!("color prop")
    };
    (sc(Prop::X), sc(Prop::Y), sc(Prop::W), sc(Prop::H), c)
}

#[test]
fn mode_b_discover_optimize_encode_decode_play_round_trip() {
    let frames: Vec<(f32, Vec<Node>)> = (0..=(FPS as f32 * DUR_S) as usize)
        .map(|i| {
            let t = i as f32 / FPS as f32;
            (t, scene_at(t))
        })
        .collect();
    let discovered = discover(&frames, &DiscoverConfig::default()).expect("discovers");
    let raw = encode(&discovered, &[], &[]).expect("discovered timeline encodes");
    let optimized = optimize(&discovered, &OptimizeCfg::default()).expect("optimizes");
    let bytes = encode(&optimized, &[], &[]).expect("optimized timeline encodes");
    let doc = decode(&bytes).expect("decodes back");
    assert_eq!(
        doc.timeline, optimized,
        "decode(encode(optimized)) is structurally identical"
    );
    assert!(
        !discovered.places.is_empty() && !discovered.removes.is_empty(),
        "intermittent nodes must discover as structural ops"
    );

    let mut player = MonsterPlayer::new(doc.timeline).expect("decoded timeline plays");
    let (mut max_px, mut max_ch) = (0.0f32, 0.0f32);
    for (t, nodes) in &frames {
        let played = player.scene_at(*t);
        assert_eq!(played.len(), nodes.len(), "node count at t={t}");
        let mut expected: Vec<&Node> = nodes.iter().collect();
        expected.sort_by_key(|n| n.depth);
        for (have, want) in played.iter().zip(&expected) {
            let (x0, y0, w0, h0, c0) = rect_fields(have);
            let (x1, y1, w1, h1, c1) = expected_fields(want);
            for d in [x0 - x1, y0 - y1, w0 - w1, h0 - h1] {
                max_px = max_px.max(d.abs());
            }
            for (a, b) in c0.iter().zip(c1.iter()) {
                max_ch = max_ch.max((a - b).abs());
            }
        }
    }

    let segs = |tl: &monster::Timeline| tl.tracks.iter().map(|t| t.segments.len()).sum::<usize>();
    println!("== mode B end to end (20 nodes, 60 fps x 4 s = 241 samples) ==");
    println!(
        "  discovered: keyframes {}  tracks {}  segments {}  place/replace/remove {}/{}/{}",
        discovered.keyframes.len(),
        discovered.tracks.len(),
        segs(&discovered),
        discovered.places.len(),
        discovered.replaces.len(),
        discovered.removes.len()
    );
    println!(
        "  optimized:  tracks {} -> {}, segments {} -> {}",
        discovered.tracks.len(),
        optimized.tracks.len(),
        segs(&discovered),
        segs(&optimized)
    );
    println!(
        "  bytes: raw {} B -> optimized {} B ({:+} B); {:.1} B/s over {DUR_S} s",
        raw.len(),
        bytes.len(),
        bytes.len() as i64 - raw.len() as i64,
        bytes.len() as f32 / DUR_S
    );
    println!("  playback deviation: scalars {max_px:.4} px, colors {max_ch:.5} per channel");
    let (px_step, ch_step) = (0.05, 1.0 / 255.0);
    assert!(
        max_px <= px_step + 1e-4,
        "played scalars must stay within one grid step of the input, got {max_px}"
    );
    assert!(
        max_ch <= ch_step + 1e-5,
        "played colors must stay within one grid step of the input, got {max_ch}"
    );
}
