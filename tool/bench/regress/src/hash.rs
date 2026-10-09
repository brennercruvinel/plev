//! The content hash a baseline keeps for a layer.
//!
//! `Layer::compute_hash` is the dirty tracking's: `FxHasher` over
//! `SceneNode::hash_u64`, crate-private, with no stability promise. The
//! baseline needs a hash that stays put across builds and machines, so
//! this one is FNV-1a 64 over a byte encoding written out below and
//! versioned by [`HASH_VERSION`]; the dirty tracking is not touched.
//!
//! Encoding, in order, every integer little-endian, every `f32` as its
//! `to_bits`, a string as its byte length (`u32`) then its UTF-8 bytes,
//! an `Option` as `0` or `1` then the value, a `bool` as one byte:
//!
//! - the version tag [`HASH_VERSION`];
//! - the layer: `z_order` (`i32`), `visible`, `opacity`, `clip_rect`
//!   (`Option` of four `u32`), the effect count (`u32`) and each effect
//!   (`1` blur: sigma; `2` shadow: sigma, color);
//! - the node count (`u32`), then each node in push order as a tag byte
//!   and its fields in declaration order: `1` rect (x, y, w, h, color),
//!   `2` rounded rect (x, y, w, h, color, corner radius, border width,
//!   border color), `3` gradient rect (x, y, w, h, color, color2, angle,
//!   corner radius, border width, border color), `4` text (text, font
//!   size, line height, max width, weight as `u16`, letter spacing,
//!   family, then x, y, color), `5` path (vertex count, each vertex's
//!   position and color, index count, each index), `6` image (x, y, w, h,
//!   the image's width and height, corner radius), `7` push clip
//!   (x, y, w, h), `8` pop clip, `9` backdrop blur (x, y, w, h, corner
//!   radius, sigma), `10` shadow (x, y, w, h, corner radius, blur radius,
//!   offset, color, inset).
//!
//! An image is hashed by its size, not by its id or atlas placement: the
//! image store numbers images in the order a process loads them, so the
//! same page gets another id after other pages loaded theirs. A change
//! of an image's pixels at the same size does not change the hash.

use engine::compositor::{Layer, LayerEffect, SceneNode};

/// The version of the hash encoding; hashed first, so another encoding
/// changes every hash.
pub const HASH_VERSION: &str = "regress.layer.v1";

/// FNV-1a 64, fed in the encoding the module documents.
struct Fnv(u64);

impl Fnv {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn u8(&mut self, v: u8) {
        self.bytes(&[v]);
    }

    fn bool(&mut self, v: bool) {
        self.u8(u8::from(v));
    }

    fn u16(&mut self, v: u16) {
        self.bytes(&v.to_le_bytes());
    }

    fn u32(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }

    fn i32(&mut self, v: i32) {
        self.bytes(&v.to_le_bytes());
    }

    fn len(&mut self, n: usize) {
        self.u32(u32::try_from(n).unwrap_or(u32::MAX));
    }

    fn f32(&mut self, v: f32) {
        self.u32(v.to_bits());
    }

    fn f32s(&mut self, vs: &[f32]) {
        for v in vs {
            self.f32(*v);
        }
    }

    fn str(&mut self, s: &str) {
        self.len(s.len());
        self.bytes(s.as_bytes());
    }

    fn opt_u32(&mut self, v: Option<u32>) {
        match v {
            None => self.u8(0),
            Some(v) => {
                self.u8(1);
                self.u32(v);
            }
        }
    }

    fn opt_str(&mut self, v: Option<&str>) {
        match v {
            None => self.u8(0),
            Some(v) => {
                self.u8(1);
                self.str(v);
            }
        }
    }
}

/// The layer's content hash for a baseline (see the module doc).
pub fn stable_hash(layer: &Layer) -> u64 {
    let mut h = Fnv::new();
    h.str(HASH_VERSION);
    h.i32(layer.z_order);
    h.bool(layer.visible);
    h.f32(layer.opacity);
    match layer.clip_rect {
        None => h.u8(0),
        Some((x, y, w, hh)) => {
            h.u8(1);
            for v in [x, y, w, hh] {
                h.u32(v);
            }
        }
    }
    h.len(layer.effects().len());
    for effect in layer.effects() {
        match effect {
            LayerEffect::Blur { sigma } => {
                h.u8(1);
                h.f32(*sigma);
            }
            LayerEffect::Shadow { sigma, color } => {
                h.u8(2);
                h.f32(*sigma);
                h.f32s(color);
            }
        }
    }
    h.len(layer.nodes().len());
    for node in layer.nodes() {
        node_into(&mut h, node);
    }
    h.0
}

fn node_into(h: &mut Fnv, node: &SceneNode) {
    match node {
        SceneNode::Rect {
            x,
            y,
            w,
            h: rh,
            color,
        } => {
            h.u8(1);
            h.f32s(&[*x, *y, *w, *rh]);
            h.f32s(color);
        }
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h: rh,
            color,
            corner_radius,
            border_width,
            border_color,
        } => {
            h.u8(2);
            h.f32s(&[*x, *y, *w, *rh]);
            h.f32s(color);
            h.f32s(&[*corner_radius, *border_width]);
            h.f32s(border_color);
        }
        SceneNode::GradientRect {
            x,
            y,
            w,
            h: rh,
            color,
            color2,
            angle_deg,
            corner_radius,
            border_width,
            border_color,
        } => {
            h.u8(3);
            h.f32s(&[*x, *y, *w, *rh]);
            h.f32s(color);
            h.f32s(color2);
            h.f32s(&[*angle_deg, *corner_radius, *border_width]);
            h.f32s(border_color);
        }
        SceneNode::Text { key, x, y, color } => {
            h.u8(4);
            h.str(&key.text);
            h.u32(key.font_size_bits);
            h.u32(key.line_height_bits);
            h.opt_u32(key.max_width_bits);
            h.u16(key.font_weight);
            h.u32(key.letter_spacing_bits);
            h.opt_str(key.font_family.as_deref());
            h.f32s(&[*x, *y]);
            h.f32s(color);
        }
        SceneNode::Path { data } => {
            h.u8(5);
            h.len(data.vertices.len());
            for v in &data.vertices {
                h.f32s(&v.position);
                h.f32s(&v.color);
            }
            h.len(data.indices.len());
            for i in &data.indices {
                h.u32(*i);
            }
        }
        SceneNode::Image {
            x,
            y,
            w,
            h: rh,
            image,
            corner_radius,
        } => {
            h.u8(6);
            h.f32s(&[*x, *y, *w, *rh]);
            h.u32(image.width);
            h.u32(image.height);
            h.f32(*corner_radius);
        }
        SceneNode::PushClip { x, y, w, h: rh } => {
            h.u8(7);
            h.f32s(&[*x, *y, *w, *rh]);
        }
        SceneNode::PopClip => h.u8(8),
        SceneNode::BackdropBlur {
            x,
            y,
            w,
            h: rh,
            corner_radius,
            sigma,
        } => {
            h.u8(9);
            h.f32s(&[*x, *y, *w, *rh, *corner_radius, *sigma]);
        }
        SceneNode::Shadow {
            x,
            y,
            w,
            h: rh,
            corner_radius,
            blur_radius,
            offset,
            color,
            inset,
        } => {
            h.u8(10);
            h.f32s(&[*x, *y, *w, *rh, *corner_radius, *blur_radius]);
            h.f32s(offset);
            h.f32s(color);
            h.bool(*inset);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::compositor::{Compositor, LayerId};

    use crate::scene::capture;
    use crate::scene::tests::{BLUE, RED, VIEW, page, rect};

    #[test]
    fn the_hash_is_pinned_by_its_encoding() {
        // an empty default layer: the version tag, z 0, visible, opacity
        // 1, no clip, no effects, no nodes. the value was computed by a
        // script of its own from the encoding the module documents
        // (python: struct.pack + FNV-1a 64). another value means the
        // encoding moved, which needs a new HASH_VERSION and a bless of
        // every baseline
        let c = Compositor::new();
        let Some(layer) = c.layer(LayerId::DEFAULT) else {
            panic!("no default layer");
        };
        assert_eq!(stable_hash(layer), 0x8090_10ca_4ca6_3ee2);
    }

    #[test]
    fn the_hash_moves_with_a_value_and_not_with_a_binding() {
        let hash = |c: &Compositor| capture(c, VIEW, false).layers[0].hash;
        let base = hash(&page(&["a"]));
        assert_eq!(base, hash(&page(&["a"])));
        let mut recolored = page(&["a"]);
        recolored.begin_frame();
        recolored.push(rect(100.0, 20.0, 280.0, 260.0, BLUE));
        recolored.push(rect(110.0, 30.0, 260.0, 24.0, [1.0, 0.0, 0.0, 0.5]));
        assert_ne!(base, hash(&recolored));
        // the same nodes without a single bind: the hash is the scene's
        let mut unbound = page(&["a"]);
        unbound.begin_frame();
        unbound.push(rect(100.0, 20.0, 280.0, 260.0, BLUE));
        unbound.push(rect(110.0, 30.0, 260.0, 24.0, RED));
        assert_eq!(base, capture(&unbound, VIEW, false).layers[0].hash);
        // a layer property is part of what is drawn
        let mut faded = page(&["a"]);
        faded.set_layer_opacity(LayerId::DEFAULT, 0.5);
        assert_ne!(base, hash(&faded));
    }
}
