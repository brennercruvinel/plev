//! lowering tests, all GPU free: exact IR -> SceneNode field mapping,
//! depth ordering, the asset bank (resolved, missing, mistyped), and
//! the compositor seam: lowered scenes feed Compositor::resolve_scene
//! headless, where the dirty hash makes unchanged pushes free.

use crate::ir::{Keyframe, Node, NodeKind, Prop, Props, Timeline, Value};
use crate::lower::{LoweredAsset, lower_node, lower_scene};
use crate::play::MonsterPlayer;
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::gpu::ImageHandle;
use engine::path::TessellatedPath;

fn node(id: u16, depth: u16, kind: NodeKind, props: Props) -> Node {
    Node {
        id,
        depth,
        kind,
        props,
    }
}

fn geo_props() -> Props {
    Props::new()
        .with(Prop::X, Value::Scalar(10.0))
        .with(Prop::Y, Value::Scalar(20.0))
        .with(Prop::W, Value::Scalar(100.0))
        .with(Prop::H, Value::Scalar(50.0))
        .with(Prop::Color, Value::Color([0.5, 0.25, 0.0, 1.0]))
}

fn handle() -> ImageHandle {
    ImageHandle {
        id: 7,
        atlas_x: 100,
        atlas_y: 200,
        width: 32,
        height: 16,
    }
}

fn path_data() -> TessellatedPath {
    TessellatedPath {
        vertices: Vec::new(),
        indices: Vec::new(),
        hash: 42,
    }
}

fn bank() -> Vec<LoweredAsset> {
    vec![
        LoweredAsset::TextStyle(TextNodeKey::new("monster", 16.0, 20.0, None)),
        LoweredAsset::Image(handle()),
        LoweredAsset::Path(path_data()),
    ]
}

#[test]
fn rect_maps_field_for_field() {
    let n = node(1, 0, NodeKind::Rect, geo_props());
    assert_eq!(
        lower_node(&n, &[]),
        Some(SceneNode::Rect {
            x: 10.0,
            y: 20.0,
            w: 100.0,
            h: 50.0,
            color: [0.5, 0.25, 0.0, 1.0],
        })
    );
}

#[test]
fn rounded_and_gradient_map_their_extra_props() {
    let rounded = geo_props()
        .with(Prop::CornerRadius, Value::Scalar(8.0))
        .with(Prop::BorderWidth, Value::Scalar(2.0))
        .with(Prop::BorderColor, Value::Color([0.0, 0.0, 0.0, 1.0]));
    let n = node(1, 0, NodeKind::RoundedRect, rounded.clone());
    match lower_node(&n, &[]) {
        Some(SceneNode::RoundedRect {
            corner_radius,
            border_width,
            border_color,
            ..
        }) => {
            assert_eq!(corner_radius, 8.0);
            assert_eq!(border_width, 2.0);
            assert_eq!(border_color, [0.0, 0.0, 0.0, 1.0]);
        }
        other => panic!("expected rounded rect, got {other:?}"),
    }
    let gradient = rounded
        .with(Prop::Color2, Value::Color([0.0, 1.0, 0.0, 1.0]))
        .with(Prop::AngleDeg, Value::Scalar(90.0));
    let n = node(1, 0, NodeKind::GradientRect, gradient);
    match lower_node(&n, &[]) {
        Some(SceneNode::GradientRect {
            color2, angle_deg, ..
        }) => {
            assert_eq!(color2, [0.0, 1.0, 0.0, 1.0]);
            assert_eq!(angle_deg, 90.0);
        }
        other => panic!("expected gradient rect, got {other:?}"),
    }
}

#[test]
fn missing_props_default_to_zero_and_transparent() {
    let n = node(1, 0, NodeKind::Rect, Props::new());
    assert_eq!(
        lower_node(&n, &[]),
        Some(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            color: [0.0; 4],
        })
    );
}

#[test]
fn asset_backed_kinds_resolve_through_the_bank() {
    let assets = bank();
    let text = node(
        1,
        0,
        NodeKind::Text { style: 0 },
        Props::new()
            .with(Prop::X, Value::Scalar(5.0))
            .with(Prop::Color, Value::Color([1.0; 4])),
    );
    assert_eq!(
        lower_node(&text, &assets),
        Some(SceneNode::Text {
            key: TextNodeKey::new("monster", 16.0, 20.0, None),
            x: 5.0,
            y: 0.0,
            color: [1.0; 4],
        })
    );
    let image = node(2, 1, NodeKind::Image { image: 1 }, geo_props());
    match lower_node(&image, &assets) {
        Some(SceneNode::Image { image, .. }) => assert_eq!(image, handle()),
        other => panic!("expected image, got {other:?}"),
    }
    let path = node(3, 2, NodeKind::Path { path: 2 }, Props::new());
    assert_eq!(
        lower_node(&path, &assets),
        Some(SceneNode::Path { data: path_data() })
    );
}

#[test]
fn missing_or_mistyped_asset_skips_only_that_node() {
    // style id 1 points at an Image asset; id 9 points past the table
    let nodes = vec![
        node(1, 0, NodeKind::Rect, geo_props()),
        node(2, 1, NodeKind::Text { style: 1 }, Props::new()),
        node(3, 2, NodeKind::Text { style: 9 }, Props::new()),
    ];
    let scene = lower_scene(&nodes, &bank());
    assert_eq!(scene.len(), 1);
    assert!(matches!(scene[0], SceneNode::Rect { .. }));
}

#[test]
fn lowering_orders_by_depth() {
    let nodes = vec![
        node(
            1,
            2,
            NodeKind::Rect,
            geo_props().with(Prop::X, Value::Scalar(2.0)),
        ),
        node(
            2,
            0,
            NodeKind::Rect,
            geo_props().with(Prop::X, Value::Scalar(0.0)),
        ),
        node(
            3,
            1,
            NodeKind::Rect,
            geo_props().with(Prop::X, Value::Scalar(1.0)),
        ),
    ];
    let xs: Vec<f32> = lower_scene(&nodes, &[])
        .iter()
        .map(|n| match n {
            SceneNode::Rect { x, .. } => *x,
            other => panic!("expected rect, got {other:?}"),
        })
        .collect();
    assert_eq!(xs, vec![0.0, 1.0, 2.0]);
}

fn asset_timeline() -> Timeline {
    Timeline {
        duration_s: 1.0,
        fps_hint: 60,
        keyframes: vec![Keyframe {
            t: 0.0,
            snapshot: vec![
                node(1, 0, NodeKind::Rect, geo_props()),
                node(
                    2,
                    1,
                    NodeKind::Text { style: 0 },
                    Props::new().with(Prop::Color, Value::Color([1.0; 4])),
                ),
                node(
                    3,
                    2,
                    NodeKind::Image { image: 1 },
                    Props::new()
                        .with(Prop::X, Value::Scalar(4.0))
                        .with(Prop::Y, Value::Scalar(8.0))
                        .with(Prop::W, Value::Scalar(32.0))
                        .with(Prop::H, Value::Scalar(16.0)),
                ),
            ],
        }],
        tracks: vec![crate::ir::Track {
            node_id: 1,
            prop: Prop::X,
            start_t: 0.0,
            segments: vec![crate::ir::Segment {
                target: Value::Scalar(300.0),
                easing: crate::easing::Easing::Linear,
                dur_s: 1.0,
            }],
        }],
        ..Timeline::default()
    }
}

/// The contract the showcase motion tab relies on: the player's scene
/// feeds Compositor::resolve_scene headless, an unchanged push costs
/// zero redraws (dirty hash), a moved playhead redraws exactly once.
#[test]
fn lowered_scene_drives_compositor_resolve_scene() {
    let mut player = MonsterPlayer::new(asset_timeline()).unwrap();
    player.set_assets(bank());
    let mut comp = Compositor::new();

    let push = |comp: &mut Compositor, scene: &[SceneNode]| {
        comp.begin_frame();
        for n in scene {
            comp.push(n.clone());
        }
        comp.resolve_scene((800.0, 600.0));
        // the render loop marks layers clean after uploading; headless
        // tests emulate that step to exercise the dirty hash
        comp.mark_layer_clean(LayerId::DEFAULT);
    };

    let at_zero = player.scene_at(0.0);
    push(&mut comp, &at_zero);
    assert_eq!(comp.stats().layers_redrawn, 1);
    assert_eq!(comp.layer(LayerId::DEFAULT).unwrap().nodes(), &at_zero[..]);

    // same playhead, same scene: the push is free
    push(&mut comp, &player.scene_at(0.0));
    assert_eq!(comp.stats().layers_redrawn, 0);

    // moved playhead: exactly one redraw, scene reflects the tween
    let at_half = player.scene_at(0.5);
    assert_ne!(at_half, at_zero);
    push(&mut comp, &at_half);
    assert_eq!(comp.stats().layers_redrawn, 1);
    assert_eq!(comp.layer(LayerId::DEFAULT).unwrap().nodes(), &at_half[..]);
}
