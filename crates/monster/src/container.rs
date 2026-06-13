//! monster v0 container layout (kdb/adr/monster-format-v0.md "container layout"):
//! little-endian, byte-aligned, values quantized per `crate::quant`.
//! no zstd in v0; header bit 0 of `flags` is reserved for the envelope.
//!
//! file := header section_payload*
//! header :=
//!   magic        4   "MON0"
//!   version      u16 0
//!   flags        u16 0 (bit 0 reserved: zstd envelope)
//!   duration_s   f32
//!   fps_hint     u16
//!   asset_count  u16
//!   asset*           kind u8 (0 text_style | 1 image | 2 path),
//!                    data_len u16, data bytes
//!   easing_count u16
//!   curve*       4   custom cubic bezier x1 y1 x2 y2, quantized u8,
//!                    deduped first-seen; segments reference by index
//!   desc_offset  u32 absolute offset of the T payload, 0 when absent
//!   sec_count    u16
//!   sec_entry*   41  tag u8 ('K'|'D'|'X'|'T'), offset u32, len u32,
//!                    sha256 of the payload (32 bytes)
//!
//! K payload := t f32, node_count u16, node*
//! node := id u16, depth u16, kind u8 (0 rect | 1 rounded_rect |
//!   2 gradient_rect | 3 text | 4 image | 5 path), asset_ref u16 (only
//!   kinds 3..=5), presence u16 (bit = prop wire id), values ascending
//! value := i32 twips (x y w h corner_radius border_width)
//!        | rgba8 (color border_color color2) | u16 fixed (angle_deg)
//!
//! D payload := op_count u16, op*. every op starts op_code u8
//!   (0 place | 1 modify | 2 replace | 3 remove), at_s f32 (offset from
//!   the owning keyframe's t). place|replace carry a full node (its
//!   depth names the slot); remove carries depth u16 (the scene is a
//!   flat map depth -> node, so structural ops address slots); modify
//!   carries node_id u16, presence u16, and per present prop ascending:
//!   seg_count u16, seg*. an unchanged field has no presence bit and
//!   costs zero bytes. the remove operand was first decodable in this
//!   revision (earlier decoders rejected every structural op), so no
//!   wire ever carried the node_id reading container.rs once described.
//! seg := easing u8, curve_idx u16 (only when easing == 0xFF),
//!   dur_s f32, target value
//!
//! T payload := desc_count u16, (keyframe u16, len u16, utf8)*

use crate::easing::{Easing, EasingTable, quantize_curve};
use crate::ir::{Depth, Node, NodeId, NodeKind, Prop, Segment, Value};
use crate::quant;
use sha2::{Digest, Sha256};

pub const MAGIC: [u8; 4] = *b"MON0";
pub const VERSION: u16 = 0;
/// Reserved flag: payloads wrapped in a zstd envelope. Never set in v0.
pub const FLAG_ZSTD: u16 = 0x0001;

pub const SEC_KEYFRAME: u8 = b'K';
pub const SEC_DELTA: u8 = b'D';
/// Script sidecar tag, reserved; v0 encoder never emits it.
pub const SEC_SCRIPT: u8 = b'X';
pub const SEC_DESC: u8 = b'T';

pub const OP_PLACE: u8 = 0;
pub const OP_MODIFY: u8 = 1;
pub const OP_REPLACE: u8 = 2;
pub const OP_REMOVE: u8 = 3;

/// Bytes of one section index entry: tag + offset + len + sha256.
pub const SEC_ENTRY_LEN: usize = 1 + 4 + 4 + 32;
/// Bytes of the fixed header head: magic, version, flags, duration, fps.
pub const HEAD_LEN: usize = 4 + 2 + 2 + 4 + 2;

/// Definition declared once and instanced by reference (decision 6).
/// v0 treats payloads as opaque bytes; the asset id is the table index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub kind: AssetKind,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    TextStyle,
    Image,
    Path,
}

impl AssetKind {
    pub fn byte(self) -> u8 {
        match self {
            AssetKind::TextStyle => 0,
            AssetKind::Image => 1,
            AssetKind::Path => 2,
        }
    }
}

/// Optional utf-8 description for one keyframe (spec decision 7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Desc {
    pub keyframe: u16,
    pub text: String,
}

/// Convention over the description track: importers open the first
/// entry with `stage WxH` so a player learns the composition bounds
/// from the format itself, no importer linked at playback.
pub fn stage_size(descs: &[Desc]) -> Option<(f32, f32)> {
    let text = &descs.iter().find(|d| d.keyframe == 0)?.text;
    let spec = text.strip_prefix("stage ")?.split(' ').next()?;
    let (w, h) = spec.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

pub fn put_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub fn put_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub fn put_i32(buf: &mut Vec<u8>, v: i32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

pub fn put_f32(buf: &mut Vec<u8>, v: f32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

/// Wire id of a prop: the presence bit position and the canonical
/// serialization order inside nodes and modify ops.
pub fn prop_wire_id(prop: Prop) -> u8 {
    match prop {
        Prop::X => 0,
        Prop::Y => 1,
        Prop::W => 2,
        Prop::H => 3,
        Prop::Color => 4,
        Prop::CornerRadius => 5,
        Prop::BorderWidth => 6,
        Prop::BorderColor => 7,
        Prop::Color2 => 8,
        Prop::AngleDeg => 9,
    }
}

/// Inverse of [`prop_wire_id`]; `None` for the unassigned ids 10..=15.
pub fn prop_from_wire_id(id: u8) -> Option<Prop> {
    Some(match id {
        0 => Prop::X,
        1 => Prop::Y,
        2 => Prop::W,
        3 => Prop::H,
        4 => Prop::Color,
        5 => Prop::CornerRadius,
        6 => Prop::BorderWidth,
        7 => Prop::BorderColor,
        8 => Prop::Color2,
        9 => Prop::AngleDeg,
        _ => return None,
    })
}

/// Quantize and append one prop value. Coordinates and widths are i32
/// twips, colors rgba8, angles u16 fixed (decision 5). The caller has
/// validated value kind against the prop (`Timeline::validate`).
pub fn put_value(buf: &mut Vec<u8>, prop: Prop, value: Value) {
    match (prop, value) {
        (Prop::AngleDeg, Value::Scalar(deg)) => put_u16(buf, quant::angle_deg_to_u16(deg)),
        (_, Value::Scalar(px)) => put_i32(buf, quant::px_to_twips(px)),
        (_, Value::Color(rgba)) => buf.extend_from_slice(&quant::rgba_to_bytes(rgba)),
    }
}

fn kind_code(kind: NodeKind) -> (u8, Option<u16>) {
    match kind {
        NodeKind::Rect => (0, None),
        NodeKind::RoundedRect => (1, None),
        NodeKind::GradientRect => (2, None),
        NodeKind::Text { style } => (3, Some(style)),
        NodeKind::Image { image } => (4, Some(image)),
        NodeKind::Path { path } => (5, Some(path)),
    }
}

/// Append one full node entry: identity, kind, asset reference when the
/// kind has one, presence mask, values in ascending wire id order.
pub fn put_node(buf: &mut Vec<u8>, node: &Node) {
    put_u16(buf, node.id);
    put_u16(buf, node.depth);
    let (code, asset_ref) = kind_code(node.kind);
    buf.push(code);
    if let Some(id) = asset_ref {
        put_u16(buf, id);
    }
    let mut entries: Vec<(u8, Prop, Value)> = node
        .props
        .iter()
        .map(|(p, v)| (prop_wire_id(*p), *p, *v))
        .collect();
    entries.sort_by_key(|(id, ..)| *id);
    let mask = entries.iter().fold(0u16, |m, (id, ..)| m | 1 << id);
    put_u16(buf, mask);
    for (_, prop, value) in entries {
        put_value(buf, prop, value);
    }
}

/// Append one segment: easing byte, curve index for custom curves
/// (already interned in `table`), duration, quantized target.
pub fn put_segment(buf: &mut Vec<u8>, prop: Prop, seg: &Segment, table: &EasingTable) {
    buf.push(seg.easing.byte());
    if let Easing::CustomBezier { x1, y1, x2, y2 } = seg.easing {
        let idx = table
            .index_of(quantize_curve(x1, y1, x2, y2))
            .expect("encoder interns every custom curve before serialization");
        put_u16(buf, idx);
    }
    put_f32(buf, seg.dur_s);
    put_value(buf, prop, seg.target);
}

/// One delta op. `at_s` is the start offset from the owning keyframe's
/// t. Mode A lowering emits modify from tracks and place|replace|remove
/// from the timeline's structural op lists.
#[derive(Clone, Debug, PartialEq)]
pub enum DeltaOp {
    Place {
        at_s: f32,
        node: Node,
    },
    Modify {
        at_s: f32,
        node_id: NodeId,
        props: Vec<(Prop, Vec<Segment>)>,
    },
    Replace {
        at_s: f32,
        node: Node,
    },
    Remove {
        at_s: f32,
        depth: Depth,
    },
}

/// Append one delta op. Modify `props` must be sorted by wire id and
/// unique; the encoder guarantees it (`write::encode` rejects duplicate
/// tracks) so a violation here is a bug, not an input error.
pub fn put_op(buf: &mut Vec<u8>, op: &DeltaOp, table: &EasingTable) {
    match op {
        DeltaOp::Place { at_s, node } => {
            buf.push(OP_PLACE);
            put_f32(buf, *at_s);
            put_node(buf, node);
        }
        DeltaOp::Modify {
            at_s,
            node_id,
            props,
        } => {
            buf.push(OP_MODIFY);
            put_f32(buf, *at_s);
            put_u16(buf, *node_id);
            debug_assert!(
                props
                    .windows(2)
                    .all(|w| prop_wire_id(w[0].0) < prop_wire_id(w[1].0)),
                "modify props must be sorted by wire id and unique"
            );
            let mask = props
                .iter()
                .fold(0u16, |m, (p, _)| m | 1 << prop_wire_id(*p));
            put_u16(buf, mask);
            for (prop, segments) in props {
                put_u16(buf, segments.len() as u16);
                for seg in segments {
                    put_segment(buf, *prop, seg, table);
                }
            }
        }
        DeltaOp::Replace { at_s, node } => {
            buf.push(OP_REPLACE);
            put_f32(buf, *at_s);
            put_node(buf, node);
        }
        DeltaOp::Remove { at_s, depth } => {
            buf.push(OP_REMOVE);
            put_f32(buf, *at_s);
            put_u16(buf, *depth);
        }
    }
}

/// One serialized section payload, ready for the index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub tag: u8,
    pub payload: Vec<u8>,
}

/// Assemble the final file: header with asset and easing tables, the
/// section index with absolute offsets and per-section sha256 (nest
/// lesson), then the payloads in the given order.
pub fn assemble(
    duration_s: f32,
    fps_hint: u16,
    assets: &[Asset],
    easings: &EasingTable,
    sections: &[Section],
) -> Vec<u8> {
    let asset_bytes: usize = assets.iter().map(|a| 3 + a.data.len()).sum();
    let header_len =
        HEAD_LEN + 2 + asset_bytes + 2 + 4 * easings.len() + 4 + 2 + SEC_ENTRY_LEN * sections.len();
    let total = header_len + sections.iter().map(|s| s.payload.len()).sum::<usize>();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&MAGIC);
    put_u16(&mut out, VERSION);
    put_u16(&mut out, 0); // flags: zstd bit reserved, always clear in v0
    put_f32(&mut out, duration_s);
    put_u16(&mut out, fps_hint);
    put_u16(&mut out, assets.len() as u16);
    for asset in assets {
        out.push(asset.kind.byte());
        put_u16(&mut out, asset.data.len() as u16);
        out.extend_from_slice(&asset.data);
    }
    put_u16(&mut out, easings.len() as u16);
    for curve in easings.curves() {
        out.extend_from_slice(curve);
    }
    let mut offset = header_len;
    let mut desc_offset = 0u32;
    let mut index: Vec<(u8, u32, u32, [u8; 32])> = Vec::with_capacity(sections.len());
    for section in sections {
        if section.tag == SEC_DESC && desc_offset == 0 {
            desc_offset = offset as u32;
        }
        let digest: [u8; 32] = Sha256::digest(&section.payload).into();
        index.push((
            section.tag,
            offset as u32,
            section.payload.len() as u32,
            digest,
        ));
        offset += section.payload.len();
    }
    put_u32(&mut out, desc_offset);
    put_u16(&mut out, sections.len() as u16);
    for (tag, off, len, digest) in index {
        out.push(tag);
        put_u32(&mut out, off);
        put_u32(&mut out, len);
        out.extend_from_slice(&digest);
    }
    for section in sections {
        out.extend_from_slice(&section.payload);
    }
    debug_assert_eq!(out.len(), total);
    out
}
