//! raise, the mirror of lower: every compositor kind the IR carries
//! round-trips raise -> lower to the same node, the kinds it does not
//! carry are reported with their position, each distinct asset enters the
//! bank once, and raised frames feed discover and play back.

use engine::drawing::compositor::{SceneNode, TextNodeKey};
use engine::drawing::gpu::ImageHandle;
use engine::drawing::path::{PathBuilder, TessellatedPath};
use engine::drawing::text::TextStyle;
use motion::mappings::lower::lower_scene;
use motion::{
    AssetBank, DiscoverConfig, MAX_RAISED, MonsterPlayer, NodeKind, Prop, RaiseError, Skipped,
    Unsupported, Value, decode, discover, encode, quantize_value, raise_scene,
};

fn text(s: &str) -> TextNodeKey {
    TextNodeKey::from_style(
        s,
        &TextStyle::new(14.0).with_line_height(20.0).with_weight(600),
        Some(240.0),
    )
}

fn handle(id: u32) -> ImageHandle {
    ImageHandle {
        id,
        atlas_x: 16 * id,
        atlas_y: 0,
        width: 16,
        height: 16,
    }
}

fn triangle(x: f32) -> TessellatedPath {
    PathBuilder::new()
        .move_to(x, 0.0)
        .line_to(x + 10.0, 0.0)
        .line_to(x + 5.0, 8.0)
        .close()
        .fill([0.2, 0.4, 0.6, 1.0])
}

/// one node of every kind the IR carries.
fn supported() -> Vec<SceneNode> {
    vec![
        SceneNode::Rect {
            x: 1.5,
            y: -2.0,
            w: 30.0,
            h: 12.25,
            color: [0.1, 0.2, 0.3, 0.4],
        },
        SceneNode::RoundedRect {
            x: 4.0,
            y: 8.0,
            w: 120.0,
            h: 36.0,
            color: [1.0, 1.0, 1.0, 0.9],
            corner_radius: 8.0,
            border_width: 1.0,
            border_color: [0.0, 0.0, 0.0, 0.25],
        },
        SceneNode::GradientRect {
            x: 0.0,
            y: 44.0,
            w: 200.0,
            h: 20.0,
            color: [0.9, 0.1, 0.1, 1.0],
            color2: [0.1, 0.1, 0.9, 1.0],
            angle_deg: 135.0,
            corner_radius: 4.0,
            border_width: 0.5,
            border_color: [0.5, 0.5, 0.5, 1.0],
        },
        SceneNode::Text {
            key: text("Absorb the plev demos"),
            x: 12.0,
            y: 70.0,
            color: [0.95, 0.95, 0.95, 1.0],
        },
        SceneNode::Image {
            x: 10.0,
            y: 100.0,
            w: 32.0,
            h: 32.0,
            image: handle(3),
            corner_radius: 16.0,
        },
        SceneNode::Path {
            data: triangle(0.0),
        },
    ]
}

fn unsupported_nodes() -> Vec<SceneNode> {
    vec![
        SceneNode::PushClip {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
        },
        SceneNode::PopClip,
        SceneNode::BackdropBlur {
            x: 0.0,
            y: 0.0,
            w: 50.0,
            h: 50.0,
            corner_radius: 12.0,
            sigma: 8.0,
        },
        SceneNode::Shadow {
            x: 0.0,
            y: 0.0,
            w: 50.0,
            h: 50.0,
            corner_radius: 12.0,
            blur_radius: 16.0,
            offset: [0.0, 4.0],
            color: [0.0, 0.0, 0.0, 0.3],
            inset: false,
        },
    ]
}

#[test]
fn every_kind_the_ir_carries_round_trips_raise_then_lower() {
    for node in supported() {
        let mut bank = AssetBank::new();
        let Ok(raised) = raise_scene([&node], &mut bank) else {
            panic!("{node:?} does not raise");
        };
        assert!(raised.skipped.is_empty());
        assert_eq!(raised.nodes.len(), 1);
        assert_eq!(
            lower_scene(&raised.nodes, bank.assets()),
            vec![node.clone()]
        );
    }
}

#[test]
fn a_mixed_scene_reports_what_the_ir_cannot_carry() {
    let kept = supported();
    let dropped = unsupported_nodes();
    let scene = vec![
        dropped[0].clone(),
        kept[0].clone(),
        dropped[3].clone(),
        kept[3].clone(),
        dropped[1].clone(),
        dropped[2].clone(),
        kept[4].clone(),
    ];
    let mut bank = AssetBank::new();
    let Ok(raised) = raise_scene(&scene, &mut bank) else {
        panic!("the scene does not raise");
    };
    assert_eq!(
        raised.skipped,
        [
            Skipped {
                index: 0,
                kind: Unsupported::PushClip
            },
            Skipped {
                index: 2,
                kind: Unsupported::Shadow
            },
            Skipped {
                index: 4,
                kind: Unsupported::PopClip
            },
            Skipped {
                index: 5,
                kind: Unsupported::BackdropBlur
            },
        ]
    );
    let depths: Vec<u16> = raised.nodes.iter().map(|n| n.depth).collect();
    assert_eq!(depths, [0, 1, 2]);
    assert!(raised.nodes.iter().all(|n| n.id == n.depth));
    assert_eq!(
        lower_scene(&raised.nodes, bank.assets()),
        vec![kept[0].clone(), kept[3].clone(), kept[4].clone()]
    );
    let names: Vec<&str> = Unsupported::ALL.iter().map(|k| k.name()).collect();
    assert_eq!(names, ["push_clip", "pop_clip", "backdrop_blur", "shadow"]);
}

#[test]
fn each_distinct_asset_enters_the_bank_once() {
    let at = |key: TextNodeKey, y: f32| SceneNode::Text {
        key,
        x: 0.0,
        y,
        color: [1.0; 4],
    };
    let image = |id: u32, x: f32| SceneNode::Image {
        x,
        y: 0.0,
        w: 16.0,
        h: 16.0,
        image: handle(id),
        corner_radius: 0.0,
    };
    let scene = vec![
        at(text("same"), 0.0),
        at(text("same"), 20.0),
        at(text("same").with_weight(400), 40.0),
        image(1, 0.0),
        image(1, 20.0),
        SceneNode::Path {
            data: triangle(0.0),
        },
        SceneNode::Path {
            data: triangle(0.0),
        },
        SceneNode::Path {
            data: triangle(3.0),
        },
    ];
    let mut bank = AssetBank::new();
    let Ok(raised) = raise_scene(&scene, &mut bank) else {
        panic!("the scene does not raise");
    };
    assert_eq!(bank.len(), 5, "two texts, one image, two paths");
    let kinds: Vec<_> = raised.nodes.iter().map(|n| n.kind).collect();
    assert_eq!(kinds[0], kinds[1]);
    assert_ne!(kinds[0], kinds[2]);
    assert_eq!(kinds[3], kinds[4]);
    assert_eq!(kinds[5], kinds[6]);
    assert_ne!(kinds[5], kinds[7]);
    assert_eq!(lower_scene(&raised.nodes, bank.assets()), scene);
}

#[test]
fn a_prop_equal_to_the_lowering_default_is_left_out() {
    let node = SceneNode::RoundedRect {
        x: 0.0,
        y: -0.0,
        w: 10.0,
        h: 0.0,
        color: [0.0; 4],
        corner_radius: 0.0,
        border_width: 0.0,
        border_color: [0.0, 0.0, 0.0, 0.5],
    };
    let mut bank = AssetBank::new();
    let Ok(raised) = raise_scene([&node], &mut bank) else {
        panic!("the node does not raise");
    };
    let props: Vec<(Prop, Value)> = raised.nodes[0].props.iter().copied().collect();
    assert_eq!(
        props,
        [
            (Prop::Y, Value::Scalar(-0.0)),
            (Prop::W, Value::Scalar(10.0)),
            (Prop::BorderColor, Value::Color([0.0, 0.0, 0.0, 0.5])),
        ],
        "negative zero is not the default and stays"
    );
    let back = lower_scene(&raised.nodes, bank.assets());
    let Some(SceneNode::RoundedRect { y, .. }) = back.first() else {
        panic!("expected a rounded rect, got {back:?}");
    };
    assert_eq!(y.to_bits(), (-0.0f32).to_bits());
    assert_eq!(back, vec![node]);
}

#[test]
fn the_bank_and_the_scene_stop_at_the_u16_limits() {
    let mut bank = AssetBank::new();
    for i in 0..MAX_RAISED {
        assert!(bank.text(&text(&i.to_string())).is_ok(), "asset {i}");
    }
    assert_eq!(bank.text(&text("0")), Ok(0), "a known asset still resolves");
    assert_eq!(
        bank.text(&text("one too many")),
        Err(RaiseError::TooManyAssets)
    );

    let rect = SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
        color: [1.0; 4],
    };
    let mut bank = AssetBank::new();
    let fits = vec![rect.clone(); MAX_RAISED];
    assert!(raise_scene(&fits, &mut bank).is_ok());
    let over = vec![rect; MAX_RAISED + 1];
    assert_eq!(raise_scene(&over, &mut bank), Err(RaiseError::TooManyNodes));
}

#[test]
fn raised_frames_feed_discover_and_play_back() {
    let frame = |i: u16| {
        let x = f32::from(i) * 4.0;
        let mut scene = vec![SceneNode::Rect {
            x,
            y: 10.0,
            w: 40.0,
            h: 20.0,
            color: [0.2, 0.6, 0.2, 1.0],
        }];
        if i >= 5 {
            scene.push(SceneNode::Text {
                key: text("done"),
                x: 0.0,
                y: 40.0,
                color: [1.0; 4],
            });
        }
        scene
    };
    let mut bank = AssetBank::new();
    let mut frames = Vec::new();
    for i in 0..10u16 {
        let Ok(raised) = raise_scene(&frame(i), &mut bank) else {
            panic!("frame {i} does not raise");
        };
        frames.push((f32::from(i) / 60.0, raised.nodes));
    }
    let Ok(timeline) = discover(&frames, &DiscoverConfig::default()) else {
        panic!("discover refused the raised frames");
    };
    let Ok(bytes) = encode(&timeline, &[], &[]) else {
        panic!("the discovered timeline does not encode");
    };
    let Ok(doc) = decode(&bytes) else {
        panic!("the chunk does not decode");
    };
    let Ok(mut player) = MonsterPlayer::new(doc.timeline) else {
        panic!("the decoded timeline does not play");
    };
    player.set_assets(bank.assets().to_vec());
    for i in 0..10u16 {
        let played = player.scene_at(f32::from(i) / 60.0);
        assert_eq!(
            on_grid(&played, &mut bank),
            on_grid(&frame(i), &mut bank),
            "frame {i} plays back as sampled, on the wire grid"
        );
    }
}

/// one raised node on the wire grid: depth, kind, quantized props.
type GridNode = (u16, NodeKind, Vec<(Prop, Value)>);

/// a scene as the wire carries it: raised, every prop quantized. playback
/// interpolates in f32, so a sample comes back within half a
/// quantization step, which is equality here.
fn on_grid(scene: &[SceneNode], bank: &mut AssetBank) -> Vec<GridNode> {
    let Ok(raised) = raise_scene(scene, bank) else {
        panic!("the scene does not raise");
    };
    raised
        .nodes
        .into_iter()
        .map(|n| {
            let props = n
                .props
                .iter()
                .map(|(p, v)| (*p, quantize_value(*p, *v)))
                .collect();
            (n.depth, n.kind, props)
        })
        .collect()
}
