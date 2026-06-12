//! monster v0 section payload parsing: the bounds-checked cursor and the
//! K/D/T payload readers behind [`crate::read::decode`]. every reader
//! consumes its payload exactly; leftover bytes are an error, so a
//! malformed length can never hide trailing garbage.

use crate::container::{self, Asset, AssetKind, Desc, OP_MODIFY, OP_PLACE, OP_REMOVE, OP_REPLACE};
use crate::easing::{CUSTOM_BEZIER_BYTE, Easing};
use crate::ir::{
    Keyframe, Node, NodeKind, PlaceNode, Prop, Props, RemoveNode, ReplaceNode, Segment, Track,
    Value,
};
use crate::quant;
use crate::read::ReadError;

/// Everything D blocks decode into, in wire order per list: modify ops
/// become tracks, the structural ops their timeline lists.
#[derive(Default)]
pub(crate) struct DeltaOut {
    pub(crate) tracks: Vec<Track>,
    pub(crate) places: Vec<PlaceNode>,
    pub(crate) replaces: Vec<ReplaceNode>,
    pub(crate) removes: Vec<RemoveNode>,
}

/// Bounds-checked little-endian cursor; every read names the field it
/// was after, so truncation errors point at it.
pub(crate) struct Cur<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Cur<'a> {
    pub(crate) fn new(b: &'a [u8]) -> Self {
        Self { b, at: 0 }
    }

    pub(crate) fn take(&mut self, n: usize, what: &'static str) -> Result<&'a [u8], ReadError> {
        let end = self
            .at
            .checked_add(n)
            .filter(|end| *end <= self.b.len())
            .ok_or(ReadError::Truncated { what })?;
        let slice = &self.b[self.at..end];
        self.at = end;
        Ok(slice)
    }

    pub(crate) fn arr<const N: usize>(&mut self, what: &'static str) -> Result<[u8; N], ReadError> {
        let mut a = [0u8; N];
        a.copy_from_slice(self.take(N, what)?);
        Ok(a)
    }

    pub(crate) fn u8(&mut self, what: &'static str) -> Result<u8, ReadError> {
        Ok(self.take(1, what)?[0])
    }

    pub(crate) fn u16(&mut self, what: &'static str) -> Result<u16, ReadError> {
        Ok(u16::from_le_bytes(self.arr(what)?))
    }

    pub(crate) fn u32(&mut self, what: &'static str) -> Result<u32, ReadError> {
        Ok(u32::from_le_bytes(self.arr(what)?))
    }

    pub(crate) fn i32(&mut self, what: &'static str) -> Result<i32, ReadError> {
        Ok(i32::from_le_bytes(self.arr(what)?))
    }

    /// All f32 fields in the format are times or durations; NaN and
    /// infinity would slip through `Timeline::validate`'s comparisons,
    /// so the decoder rejects them at the read.
    pub(crate) fn f32(&mut self, what: &'static str) -> Result<f32, ReadError> {
        let v = f32::from_le_bytes(self.arr(what)?);
        if v.is_finite() {
            Ok(v)
        } else {
            Err(ReadError::BadValue { what })
        }
    }

    pub(crate) fn done(&self) -> bool {
        self.at == self.b.len()
    }

    pub(crate) fn at(&self) -> usize {
        self.at
    }
}

pub(crate) fn parse_asset(c: &mut Cur) -> Result<Asset, ReadError> {
    let kind = match c.u8("asset kind")? {
        0 => AssetKind::TextStyle,
        1 => AssetKind::Image,
        2 => AssetKind::Path,
        byte => return Err(ReadError::UnknownAssetKind(byte)),
    };
    let len = usize::from(c.u16("asset data_len")?);
    let data = c.take(len, "asset data")?.to_vec();
    Ok(Asset { kind, data })
}

pub(crate) fn parse_keyframe(payload: &[u8]) -> Result<Keyframe, ReadError> {
    let mut c = Cur::new(payload);
    let t = c.f32("keyframe t")?;
    let count = c.u16("node_count")?;
    let mut snapshot = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        snapshot.push(parse_node(&mut c)?);
    }
    if !c.done() {
        return Err(ReadError::TrailingPayload('K'));
    }
    Ok(Keyframe { t, snapshot })
}

fn parse_node(c: &mut Cur) -> Result<Node, ReadError> {
    let id = c.u16("node id")?;
    let depth = c.u16("node depth")?;
    let kind = match c.u8("node kind")? {
        0 => NodeKind::Rect,
        1 => NodeKind::RoundedRect,
        2 => NodeKind::GradientRect,
        3 => NodeKind::Text {
            style: c.u16("text style ref")?,
        },
        4 => NodeKind::Image {
            image: c.u16("image ref")?,
        },
        5 => NodeKind::Path {
            path: c.u16("path ref")?,
        },
        byte => return Err(ReadError::UnknownNodeKind(byte)),
    };
    let mask = c.u16("presence mask")?;
    let mut props = Props::new();
    for prop in mask_props(mask)? {
        props.set(prop, read_value(c, prop)?);
    }
    Ok(Node {
        id,
        depth,
        kind,
        props,
    })
}

/// Props named by a presence mask, in ascending wire id order: the
/// canonical order values follow on the wire.
fn mask_props(mask: u16) -> Result<Vec<Prop>, ReadError> {
    let mut props = Vec::with_capacity(mask.count_ones() as usize);
    for bit in 0..16u8 {
        if mask & (1 << bit) != 0 {
            props.push(
                container::prop_from_wire_id(bit).ok_or(ReadError::UnknownPresenceBits(mask))?,
            );
        }
    }
    Ok(props)
}

fn read_value(c: &mut Cur, prop: Prop) -> Result<Value, ReadError> {
    Ok(match prop {
        Prop::AngleDeg => Value::Scalar(quant::u16_to_angle_deg(c.u16("angle value")?)),
        p if p.is_color() => Value::Color(quant::bytes_to_rgba(c.arr("color value")?)),
        _ => Value::Scalar(quant::twips_to_px(c.i32("twips value")?)),
    })
}

/// One D block: every op anchors at `kf_t + at_s` and lands in `out` in
/// wire order, which is the encoder's canonical order per list.
pub(crate) fn parse_delta(
    payload: &[u8],
    kf_t: f32,
    curves: &[[u8; 4]],
    out: &mut DeltaOut,
) -> Result<(), ReadError> {
    let mut c = Cur::new(payload);
    for _ in 0..c.u16("op_count")? {
        let code = c.u8("op code")?;
        if ![OP_MODIFY, OP_PLACE, OP_REPLACE, OP_REMOVE].contains(&code) {
            return Err(ReadError::UnknownOpCode(code));
        }
        let at_s = c.f32("op at_s")?;
        if at_s < 0.0 {
            return Err(ReadError::BadValue { what: "op at_s" });
        }
        let t = kf_t + at_s;
        match code {
            OP_MODIFY => {
                let node_id = c.u16("op node id")?;
                for prop in mask_props(c.u16("modify presence")?)? {
                    let seg_count = c.u16("seg_count")?;
                    let mut segments = Vec::with_capacity(usize::from(seg_count));
                    for _ in 0..seg_count {
                        segments.push(parse_segment(&mut c, prop, curves)?);
                    }
                    out.tracks.push(Track {
                        node_id,
                        prop,
                        start_t: t,
                        segments,
                    });
                }
            }
            OP_PLACE => {
                let node = parse_node(&mut c)?;
                out.places.push(PlaceNode { t, node });
            }
            OP_REPLACE => {
                let node = parse_node(&mut c)?;
                out.replaces.push(ReplaceNode {
                    t,
                    depth: node.depth,
                    node,
                });
            }
            OP_REMOVE => {
                let depth = c.u16("op depth")?;
                out.removes.push(RemoveNode { t, depth });
            }
            // validated above; kept as an error so a decoder bug can
            // never panic on attacker-controlled input
            code => return Err(ReadError::UnknownOpCode(code)),
        }
    }
    if !c.done() {
        return Err(ReadError::TrailingPayload('D'));
    }
    Ok(())
}

fn parse_segment(c: &mut Cur, prop: Prop, curves: &[[u8; 4]]) -> Result<Segment, ReadError> {
    let byte = c.u8("easing byte")?;
    let easing = if byte == CUSTOM_BEZIER_BYTE {
        let index = c.u16("curve index")?;
        let [x1, y1, x2, y2] =
            *curves
                .get(usize::from(index))
                .ok_or(ReadError::CurveIndexOutOfRange {
                    index,
                    len: curves.len(),
                })?;
        Easing::CustomBezier {
            x1: quant::u8_to_bezier_x(x1),
            y1: quant::u8_to_bezier_y(y1),
            x2: quant::u8_to_bezier_x(x2),
            y2: quant::u8_to_bezier_y(y2),
        }
    } else {
        Easing::from_preset_byte(byte).ok_or(ReadError::UnknownEasing(byte))?
    };
    let dur_s = c.f32("segment dur_s")?;
    let target = read_value(c, prop)?;
    Ok(Segment {
        target,
        easing,
        dur_s,
    })
}

pub(crate) fn parse_descs(payload: &[u8]) -> Result<Vec<Desc>, ReadError> {
    let mut c = Cur::new(payload);
    let count = c.u16("desc_count")?;
    let mut descs = Vec::with_capacity(usize::from(count));
    let mut prev: Option<u16> = None;
    for _ in 0..count {
        let keyframe = c.u16("desc keyframe")?;
        if prev.is_some_and(|p| keyframe <= p) {
            return Err(ReadError::DescOutOfOrder(keyframe));
        }
        prev = Some(keyframe);
        let len = usize::from(c.u16("desc len")?);
        let text = std::str::from_utf8(c.take(len, "desc text")?)?.to_string();
        descs.push(Desc { keyframe, text });
    }
    if !c.done() {
        return Err(ReadError::TrailingPayload('T'));
    }
    Ok(descs)
}
