//! anm IR v0 (doc/anm-format-v0.md, "node model"): the codec works on
//! its own model mirroring plev's animatable surface, so the frozen
//! format never chases `SceneNode`; the player lowers `Node` to
//! `SceneNode` at render time.
//!
//! scene shape: flat map depth -> node (decision 1). keyframe = full
//! snapshot = O(1) random access (decision 2). interframe = per-node,
//! per-property segment chains the renderer interpolates (decision 3).

use crate::easing::Easing;

/// Instance identity of a node across keyframes and tracks.
pub type NodeId = u16;
/// Stacking position; the scene is a flat map depth -> node.
pub type Depth = u16;
/// Index into the asset table (text styles, image handles, paths),
/// definitions separated from instances (decision 6).
pub type AssetId = u16;

/// v0 node kinds. Typography, image bits and path geometry are assets;
/// the kind carries only the reference. Per-node opacity is the color
/// alpha; transform (rotation/scale/skew) is not in v0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Rect,
    RoundedRect,
    GradientRect,
    Text {
        style: AssetId,
    },
    Image {
        image: AssetId,
    },
    /// Tessellated geometry asset; morph = cpu re-tessellation in v0,
    /// so the path instance has no animatable props yet.
    Path {
        path: AssetId,
    },
}

/// Animatable properties, the spec's exact per-kind surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prop {
    X,
    Y,
    W,
    H,
    Color,
    CornerRadius,
    BorderWidth,
    BorderColor,
    Color2,
    AngleDeg,
}

impl Prop {
    /// Color props carry [`Value::Color`]; everything else is scalar.
    pub fn is_color(self) -> bool {
        matches!(self, Prop::Color | Prop::BorderColor | Prop::Color2)
    }
}

/// Property payload: f32 in memory, quantized on the wire (twips for
/// coordinates, rgba8 for colors, u16 fixed for angles and ratios; see
/// `crate::quant`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Scalar(f32),
    Color([f32; 4]),
}

const RECT_PROPS: &[Prop] = &[Prop::X, Prop::Y, Prop::W, Prop::H, Prop::Color];
const ROUNDED_RECT_PROPS: &[Prop] = &[
    Prop::X,
    Prop::Y,
    Prop::W,
    Prop::H,
    Prop::Color,
    Prop::CornerRadius,
    Prop::BorderWidth,
    Prop::BorderColor,
];
const GRADIENT_RECT_PROPS: &[Prop] = &[
    Prop::X,
    Prop::Y,
    Prop::W,
    Prop::H,
    Prop::Color,
    Prop::CornerRadius,
    Prop::BorderWidth,
    Prop::BorderColor,
    Prop::Color2,
    Prop::AngleDeg,
];
const TEXT_PROPS: &[Prop] = &[Prop::X, Prop::Y, Prop::Color];
const IMAGE_PROPS: &[Prop] = &[Prop::X, Prop::Y, Prop::W, Prop::H, Prop::CornerRadius];
const PATH_PROPS: &[Prop] = &[];

impl NodeKind {
    /// The spec's exact animatable surface for this kind.
    pub fn animatable_props(self) -> &'static [Prop] {
        match self {
            NodeKind::Rect => RECT_PROPS,
            NodeKind::RoundedRect => ROUNDED_RECT_PROPS,
            NodeKind::GradientRect => GRADIENT_RECT_PROPS,
            NodeKind::Text { .. } => TEXT_PROPS,
            NodeKind::Image { .. } => IMAGE_PROPS,
            NodeKind::Path { .. } => PATH_PROPS,
        }
    }

    pub fn allows(self, prop: Prop) -> bool {
        self.animatable_props().contains(&prop)
    }

    pub fn name(self) -> &'static str {
        match self {
            NodeKind::Rect => "rect",
            NodeKind::RoundedRect => "rounded_rect",
            NodeKind::GradientRect => "gradient_rect",
            NodeKind::Text { .. } => "text",
            NodeKind::Image { .. } => "image",
            NodeKind::Path { .. } => "path",
        }
    }
}

/// Current values of a node's animatable props: a small ordered map,
/// at most 10 entries (gradient_rect).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Props(Vec<(Prop, Value)>);

impl Props {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set `prop` to `value`, replacing an existing entry in place.
    pub fn set(&mut self, prop: Prop, value: Value) {
        match self.0.iter_mut().find(|(p, _)| *p == prop) {
            Some(entry) => entry.1 = value,
            None => self.0.push((prop, value)),
        }
    }

    /// Builder-style [`Props::set`].
    pub fn with(mut self, prop: Prop, value: Value) -> Self {
        self.set(prop, value);
        self
    }

    pub fn get(&self, prop: Prop) -> Option<Value> {
        self.0.iter().find(|(p, _)| *p == prop).map(|(_, v)| *v)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Prop, Value)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// One placed instance: identity, stacking depth, kind, prop values.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub depth: Depth,
    pub kind: NodeKind,
    pub props: Props,
}

/// I-frame: full scene snapshot at time `t`; seek is O(1).
#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    pub t: f32,
    pub snapshot: Vec<Node>,
}

/// One eased step of a track: interpolate from the previous value
/// (keyframe snapshot or prior segment target) to `target` over
/// `dur_s` seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub target: Value,
    pub easing: Easing,
    pub dur_s: f32,
}

/// Per-node, per-property segment chain. In the container the time
/// anchor is implicit in the D-block position after its keyframe; the
/// flat IR carries it explicitly as `start_t`.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub node_id: NodeId,
    pub prop: Prop,
    pub start_t: f32,
    pub segments: Vec<Segment>,
}

impl Track {
    /// Time where the last segment lands.
    pub fn end_t(&self) -> f32 {
        self.start_t + self.segments.iter().map(|s| s.dur_s).sum::<f32>()
    }
}

/// The authored animation: keyframes plus discovered deltas.
#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    pub duration_s: f32,
    pub fps_hint: u16,
    pub keyframes: Vec<Keyframe>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum IrError {
    #[error("timeline needs an opening keyframe at t=0 for O(1) random access")]
    MissingOpeningKeyframe,
    #[error("keyframes must be strictly increasing within [0, duration]; offender at t={t}")]
    KeyframeOutOfOrder { t: f32 },
    #[error("duplicate depth {depth} in keyframe at t={t}; the scene is a flat map depth -> node")]
    DuplicateDepth { t: f32, depth: Depth },
    #[error("duplicate node id {id} in keyframe at t={t}")]
    DuplicateNodeId { t: f32, id: NodeId },
    #[error("prop {prop:?} is not animatable on {kind}")]
    PropNotAnimatable { kind: &'static str, prop: Prop },
    #[error("prop {prop:?} expects a {expected} value")]
    ValueKindMismatch { prop: Prop, expected: &'static str },
    #[error("track references unknown node id {node_id}")]
    UnknownNode { node_id: NodeId },
    #[error("track on node {node_id} has a segment with non-positive duration")]
    NonPositiveDuration { node_id: NodeId },
    #[error("track on node {node_id} ends at t={end_t}, past duration {duration_s}")]
    TrackPastEnd {
        node_id: NodeId,
        end_t: f32,
        duration_s: f32,
    },
}

fn check_value(prop: Prop, value: Value) -> Result<(), IrError> {
    let ok = match value {
        Value::Scalar(_) => !prop.is_color(),
        Value::Color(_) => prop.is_color(),
    };
    if ok {
        Ok(())
    } else {
        let expected = if prop.is_color() { "color" } else { "scalar" };
        Err(IrError::ValueKindMismatch { prop, expected })
    }
}

impl Timeline {
    /// Structural validation: ordered keyframes, flat-map snapshots,
    /// props within each kind's animatable surface, tracks anchored to
    /// known nodes and inside the duration.
    pub fn validate(&self) -> Result<(), IrError> {
        match self.keyframes.first() {
            Some(first) if first.t == 0.0 => {}
            _ => return Err(IrError::MissingOpeningKeyframe),
        }
        let mut prev_t = f32::NEG_INFINITY;
        for kf in &self.keyframes {
            if kf.t <= prev_t || kf.t > self.duration_s {
                return Err(IrError::KeyframeOutOfOrder { t: kf.t });
            }
            prev_t = kf.t;
            let mut seen: Vec<(Depth, NodeId)> = Vec::with_capacity(kf.snapshot.len());
            for node in &kf.snapshot {
                if seen.iter().any(|(d, _)| *d == node.depth) {
                    return Err(IrError::DuplicateDepth {
                        t: kf.t,
                        depth: node.depth,
                    });
                }
                if seen.iter().any(|(_, i)| *i == node.id) {
                    return Err(IrError::DuplicateNodeId {
                        t: kf.t,
                        id: node.id,
                    });
                }
                seen.push((node.depth, node.id));
                for (prop, value) in node.props.iter() {
                    if !node.kind.allows(*prop) {
                        return Err(IrError::PropNotAnimatable {
                            kind: node.kind.name(),
                            prop: *prop,
                        });
                    }
                    check_value(*prop, *value)?;
                }
            }
        }
        for track in &self.tracks {
            let kind = self
                .keyframes
                .iter()
                .flat_map(|kf| kf.snapshot.iter())
                .find(|n| n.id == track.node_id)
                .map(|n| n.kind)
                .ok_or(IrError::UnknownNode {
                    node_id: track.node_id,
                })?;
            if !kind.allows(track.prop) {
                return Err(IrError::PropNotAnimatable {
                    kind: kind.name(),
                    prop: track.prop,
                });
            }
            for seg in &track.segments {
                if seg.dur_s <= 0.0 {
                    return Err(IrError::NonPositiveDuration {
                        node_id: track.node_id,
                    });
                }
                check_value(track.prop, seg.target)?;
            }
            // half a frame of f32 slack at the default 60fps hint
            let end_t = track.end_t();
            if track.start_t < 0.0 || end_t > self.duration_s + 1e-4 {
                return Err(IrError::TrackPastEnd {
                    node_id: track.node_id,
                    end_t,
                    duration_s: self.duration_s,
                });
            }
        }
        Ok(())
    }
}
