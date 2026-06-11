//! lowering anm IR scenes to plev `SceneNode`s, the player half of the
//! spec's node model (doc/anm-format-v0.md): the codec's IR mirrors the
//! animatable surface, the player maps it onto the real compositor enum
//! at render time, so the frozen format never chases `SceneNode`.
//!
//! asset-backed kinds (text, image, path) reference runtime resources
//! the file cannot carry: a [`LoweredAsset`] bank, indexed by `AssetId`
//! exactly like the container's asset table, supplies them. a node
//! whose asset is missing or of the wrong kind lowers to nothing,
//! deterministically; playback never panics on an unresolved asset.
//!
//! the scene is a flat map depth -> node (spec decision 1), so lowering
//! emits nodes in ascending depth: the compositor paints in push order.

use crate::ir::{Node, NodeKind, Prop, Props, Value};
use plev::compositor::{SceneNode, TextNodeKey};
use plev::gpu::ImageHandle;
use plev::path::TessellatedPath;

/// Runtime resource behind an `AssetId`. The bank is positional: index
/// in the slice == asset id, mirroring the file's asset table, so a
/// decoded `Document` maps 1:1 after the app resolves each payload.
#[derive(Clone, Debug, PartialEq)]
pub enum LoweredAsset {
    /// Shared measurement-and-drawing key for text nodes (the engine's
    /// one-TextStyle-per-run rule).
    TextStyle(TextNodeKey),
    /// Atlas placement of a decoded image; plain data, GPU free.
    Image(ImageHandle),
    /// Pre-tessellated geometry (morph = cpu re-tessellation in v0).
    Path(TessellatedPath),
}

/// Scalar prop value with the v0 default: a prop absent from the
/// snapshot and untouched by tracks reads as 0.
fn scalar(props: &Props, prop: Prop) -> f32 {
    match props.get(prop) {
        Some(Value::Scalar(v)) => v,
        _ => 0.0,
    }
}

/// Color prop value; absent colors read as transparent black, so an
/// unauthored fill draws nothing instead of flashing a default.
fn color(props: &Props, prop: Prop) -> [f32; 4] {
    match props.get(prop) {
        Some(Value::Color(c)) => c,
        _ => [0.0; 4],
    }
}

/// Lower one IR scene (a keyframe snapshot with track values applied)
/// to compositor nodes in ascending depth order.
pub fn lower_scene(nodes: &[Node], assets: &[LoweredAsset]) -> Vec<SceneNode> {
    let mut by_depth: Vec<&Node> = nodes.iter().collect();
    by_depth.sort_by_key(|n| n.depth);
    by_depth
        .into_iter()
        .filter_map(|n| lower_node(n, assets))
        .collect()
}

/// Map one IR node onto the spec's `SceneNode` for its kind; `None`
/// when an asset reference does not resolve.
pub fn lower_node(node: &Node, assets: &[LoweredAsset]) -> Option<SceneNode> {
    let p = &node.props;
    Some(match node.kind {
        NodeKind::Rect => SceneNode::Rect {
            x: scalar(p, Prop::X),
            y: scalar(p, Prop::Y),
            w: scalar(p, Prop::W),
            h: scalar(p, Prop::H),
            color: color(p, Prop::Color),
        },
        NodeKind::RoundedRect => SceneNode::RoundedRect {
            x: scalar(p, Prop::X),
            y: scalar(p, Prop::Y),
            w: scalar(p, Prop::W),
            h: scalar(p, Prop::H),
            color: color(p, Prop::Color),
            corner_radius: scalar(p, Prop::CornerRadius),
            border_width: scalar(p, Prop::BorderWidth),
            border_color: color(p, Prop::BorderColor),
        },
        NodeKind::GradientRect => SceneNode::GradientRect {
            x: scalar(p, Prop::X),
            y: scalar(p, Prop::Y),
            w: scalar(p, Prop::W),
            h: scalar(p, Prop::H),
            color: color(p, Prop::Color),
            color2: color(p, Prop::Color2),
            angle_deg: scalar(p, Prop::AngleDeg),
            corner_radius: scalar(p, Prop::CornerRadius),
            border_width: scalar(p, Prop::BorderWidth),
            border_color: color(p, Prop::BorderColor),
        },
        NodeKind::Text { style } => match assets.get(usize::from(style))? {
            LoweredAsset::TextStyle(key) => SceneNode::Text {
                key: key.clone(),
                x: scalar(p, Prop::X),
                y: scalar(p, Prop::Y),
                color: color(p, Prop::Color),
            },
            _ => return None,
        },
        NodeKind::Image { image } => match assets.get(usize::from(image))? {
            LoweredAsset::Image(handle) => SceneNode::Image {
                x: scalar(p, Prop::X),
                y: scalar(p, Prop::Y),
                w: scalar(p, Prop::W),
                h: scalar(p, Prop::H),
                image: *handle,
                corner_radius: scalar(p, Prop::CornerRadius),
            },
            _ => return None,
        },
        NodeKind::Path { path } => match assets.get(usize::from(path))? {
            LoweredAsset::Path(data) => SceneNode::Path { data: data.clone() },
            _ => return None,
        },
    })
}
