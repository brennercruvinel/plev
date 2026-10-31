//! raising plev `SceneNode`s to monster IR nodes, the mirror of
//! [`crate::lower`]: a recorder samples the frame the compositor holds,
//! raises it and feeds the frames to [`crate::discover`]. the IR stays
//! the codec's own model (docs/referencia/arquitetura/monster-format-v0.md, "node model"):
//! raising maps the compositor enum onto it and never widens it.
//!
//! the IR v0 carries rect, rounded_rect, gradient_rect, text, image and
//! path. the compositor also emits clip pushes and pops, backdrop blurs
//! and shadows, which the IR has no node for: raising reports each one
//! it meets ([`Skipped`], with its position and [`Unsupported`] kind),
//! the way the importers report what they cannot represent, and never
//! drops one silently.
//!
//! a scene has no identity but its push order, so [`raise_scene`] puts
//! the n-th raised node at depth n with id n; lowering emits ascending
//! depth, so raise then lower gives the supported nodes back in push
//! order. asset-backed kinds reference an [`AssetBank`], the positional
//! `&[LoweredAsset]` lowering reads, filled once per distinct content: a
//! text by its key (typography and string), an image by its handle (the
//! engine's image store already hands one handle per distinct content),
//! a path by its content hash. a prop equal to what lowering reads for an
//! absent one (positive zero, transparent black) is left out, so the
//! node costs fewer bytes and lowers to the same bits.

use std::collections::HashMap;

use engine::drawing::compositor::{SceneNode, TextNodeKey};
use engine::drawing::gpu::ImageHandle;
use engine::drawing::path::TessellatedPath;

use crate::mappings::lower::LoweredAsset;
use crate::timeline::{AssetId, Depth, Node, NodeId, NodeKind, Prop, Props, Value};

/// The most assets one bank holds and the most nodes one raised scene
/// holds: the container counts both with a u16 (asset table, keyframe
/// node count), so a raised scene always fits a v0 file.
pub const MAX_RAISED: usize = u16::MAX as usize;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum RaiseError {
    #[error("more than 65535 distinct assets in one bank; the asset table counts them with a u16")]
    TooManyAssets,
    #[error("more than 65535 nodes in one raised scene; a keyframe counts them with a u16")]
    TooManyNodes,
}

/// A compositor node kind the IR v0 has no node for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Unsupported {
    PushClip,
    PopClip,
    BackdropBlur,
    Shadow,
}

impl Unsupported {
    pub const ALL: [Unsupported; 4] = [
        Unsupported::PushClip,
        Unsupported::PopClip,
        Unsupported::BackdropBlur,
        Unsupported::Shadow,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Unsupported::PushClip => "push_clip",
            Unsupported::PopClip => "pop_clip",
            Unsupported::BackdropBlur => "backdrop_blur",
            Unsupported::Shadow => "shadow",
        }
    }
}

/// A scene node that was not raised: its position in the input and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skipped {
    pub index: usize,
    pub kind: Unsupported,
}

/// A raised scene: the IR nodes in push order and the report of what
/// the IR could not carry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Raised {
    pub nodes: Vec<Node>,
    pub skipped: Vec<Skipped>,
}

/// The assets raised nodes reference, positional like the container's
/// asset table and the bank [`crate::lower::lower_scene`] reads: index ==
/// `AssetId`. Each distinct content enters once, in first-seen order.
#[derive(Clone, Debug, Default)]
pub struct AssetBank {
    assets: Vec<LoweredAsset>,
    texts: HashMap<TextNodeKey, AssetId>,
    images: HashMap<ImageHandle, AssetId>,
    paths: HashMap<u64, AssetId>,
}

impl AssetBank {
    pub fn new() -> Self {
        Self::default()
    }

    /// The id of a text style; the key is cloned only the first time.
    pub fn text(&mut self, key: &TextNodeKey) -> Result<AssetId, RaiseError> {
        if let Some(&id) = self.texts.get(key) {
            return Ok(id);
        }
        let id = self.push(LoweredAsset::TextStyle(key.clone()))?;
        self.texts.insert(key.clone(), id);
        Ok(id)
    }

    /// The id of an image placement.
    pub fn image(&mut self, handle: ImageHandle) -> Result<AssetId, RaiseError> {
        if let Some(&id) = self.images.get(&handle) {
            return Ok(id);
        }
        let id = self.push(LoweredAsset::Image(handle))?;
        self.images.insert(handle, id);
        Ok(id)
    }

    /// The id of a tessellated path, by its content hash; the geometry
    /// is cloned only the first time.
    pub fn path(&mut self, path: &TessellatedPath) -> Result<AssetId, RaiseError> {
        if let Some(&id) = self.paths.get(&path.hash) {
            return Ok(id);
        }
        let id = self.push(LoweredAsset::Path(path.clone()))?;
        self.paths.insert(path.hash, id);
        Ok(id)
    }

    fn push(&mut self, asset: LoweredAsset) -> Result<AssetId, RaiseError> {
        if self.assets.len() >= MAX_RAISED {
            return Err(RaiseError::TooManyAssets);
        }
        let id = AssetId::try_from(self.assets.len()).map_err(|_| RaiseError::TooManyAssets)?;
        self.assets.push(asset);
        Ok(id)
    }

    /// The bank, indexed by `AssetId`, ready for lowering.
    pub fn assets(&self) -> &[LoweredAsset] {
        &self.assets
    }

    pub fn into_assets(self) -> Vec<LoweredAsset> {
        self.assets
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

/// The kind the IR v0 has no node for, or `None` when `node` raises.
pub fn unsupported(node: &SceneNode) -> Option<Unsupported> {
    match node {
        SceneNode::PushClip { .. } => Some(Unsupported::PushClip),
        SceneNode::PopClip => Some(Unsupported::PopClip),
        SceneNode::BackdropBlur { .. } => Some(Unsupported::BackdropBlur),
        SceneNode::Shadow { .. } => Some(Unsupported::Shadow),
        SceneNode::Rect { .. }
        | SceneNode::RoundedRect { .. }
        | SceneNode::GradientRect { .. }
        | SceneNode::Text { .. }
        | SceneNode::Image { .. }
        | SceneNode::Path { .. } => None,
    }
}

/// Props under construction; values lowering reads for an absent prop
/// are left out.
struct Draft(Props);

impl Draft {
    fn scalar(mut self, prop: Prop, v: f32) -> Self {
        if v.to_bits() != 0 {
            self.0.set(prop, Value::Scalar(v));
        }
        self
    }

    fn color(mut self, prop: Prop, c: [f32; 4]) -> Self {
        if c.iter().any(|ch| ch.to_bits() != 0) {
            self.0.set(prop, Value::Color(c));
        }
        self
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Self {
        Draft(Props::new())
            .scalar(Prop::X, x)
            .scalar(Prop::Y, y)
            .scalar(Prop::W, w)
            .scalar(Prop::H, h)
    }
}

/// Raise one node at `depth` with identity `id`. `Ok(None)` exactly when
/// [`unsupported`] names its kind; an asset-backed node enters the bank.
pub fn raise_node(
    node: &SceneNode,
    id: NodeId,
    depth: Depth,
    bank: &mut AssetBank,
) -> Result<Option<Node>, RaiseError> {
    let (kind, draft) = match node {
        SceneNode::Rect { x, y, w, h, color } => (
            NodeKind::Rect,
            Draft::rect(*x, *y, *w, *h).color(Prop::Color, *color),
        ),
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h,
            color,
            corner_radius,
            border_width,
            border_color,
        } => (
            NodeKind::RoundedRect,
            Draft::rect(*x, *y, *w, *h)
                .color(Prop::Color, *color)
                .scalar(Prop::CornerRadius, *corner_radius)
                .scalar(Prop::BorderWidth, *border_width)
                .color(Prop::BorderColor, *border_color),
        ),
        SceneNode::GradientRect {
            x,
            y,
            w,
            h,
            color,
            color2,
            angle_deg,
            corner_radius,
            border_width,
            border_color,
        } => (
            NodeKind::GradientRect,
            Draft::rect(*x, *y, *w, *h)
                .color(Prop::Color, *color)
                .color(Prop::Color2, *color2)
                .scalar(Prop::AngleDeg, *angle_deg)
                .scalar(Prop::CornerRadius, *corner_radius)
                .scalar(Prop::BorderWidth, *border_width)
                .color(Prop::BorderColor, *border_color),
        ),
        SceneNode::Text { key, x, y, color } => (
            NodeKind::Text {
                style: bank.text(key)?,
            },
            Draft(Props::new())
                .scalar(Prop::X, *x)
                .scalar(Prop::Y, *y)
                .color(Prop::Color, *color),
        ),
        SceneNode::Image {
            x,
            y,
            w,
            h,
            image,
            corner_radius,
        } => (
            NodeKind::Image {
                image: bank.image(*image)?,
            },
            Draft::rect(*x, *y, *w, *h).scalar(Prop::CornerRadius, *corner_radius),
        ),
        SceneNode::Path { data } => (
            NodeKind::Path {
                path: bank.path(data)?,
            },
            Draft(Props::new()),
        ),
        SceneNode::PushClip { .. }
        | SceneNode::PopClip
        | SceneNode::BackdropBlur { .. }
        | SceneNode::Shadow { .. } => return Ok(None),
    };
    Ok(Some(Node {
        id,
        depth,
        kind,
        props: draft.0,
    }))
}

/// Raise a scene in push order: the n-th raised node sits at depth n
/// with id n, and every node the IR cannot carry is reported in
/// `skipped` with its position in `scene`.
pub fn raise_scene<'a>(
    scene: impl IntoIterator<Item = &'a SceneNode>,
    bank: &mut AssetBank,
) -> Result<Raised, RaiseError> {
    let mut out = Raised::default();
    for (index, node) in scene.into_iter().enumerate() {
        if let Some(kind) = unsupported(node) {
            out.skipped.push(Skipped { index, kind });
            continue;
        }
        if out.nodes.len() >= MAX_RAISED {
            return Err(RaiseError::TooManyNodes);
        }
        let slot = NodeId::try_from(out.nodes.len()).map_err(|_| RaiseError::TooManyNodes)?;
        if let Some(raised) = raise_node(node, slot, slot, bank)? {
            out.nodes.push(raised);
        }
    }
    Ok(out)
}
