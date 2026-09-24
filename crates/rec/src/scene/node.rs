//! what travels, written and read: a node, a layer's properties, a
//! layer's body and patch, the frame's head. the layout is in the module
//! doc of `scene`.

use engine::compositor::{Layer, LayerEffect, SceneNode, TextNodeKey};

use super::SceneError;
use super::asset::{image_key, path_key};
use super::bytes::{Put, Take};

/// a node as it travels: the compositor's own, save that an image and a
/// path are named by the key of their asset, which travels apart, once.
#[derive(Clone, Debug, PartialEq)]
pub enum Drawn {
    /// any node but an image or a path, as it is.
    Node(SceneNode),
    Image {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        corner_radius: f32,
        key: u64,
    },
    Path {
        key: u64,
    },
}

/// a layer's properties, as the compositor sets them.
#[derive(Clone, Debug)]
pub struct Props {
    pub z: i32,
    pub visible: bool,
    pub opacity: f32,
    pub clip: Option<(u32, u32, u32, u32)>,
    pub effects: Vec<LayerEffect>,
}

impl Props {
    pub(crate) fn of(layer: &Layer) -> Props {
        Props {
            z: layer.z_order,
            visible: layer.visible,
            opacity: layer.opacity,
            clip: layer.clip_rect,
            effects: layer.effects().to_vec(),
        }
    }
}

/// the frame's head: the logical size the scene is laid out in, the
/// scale to physical pixels, and the clear, in linear space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Head {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub clear: [f32; 4],
}

impl Head {
    pub(crate) fn write(&self) -> Vec<u8> {
        let mut b = Put::default();
        b.f32s(&[self.width, self.height, self.scale]);
        b.f32s(&self.clear);
        b.0
    }

    pub(crate) fn read(bytes: &[u8]) -> Result<Head, SceneError> {
        let mut t = Take::new(bytes);
        let [width, height, scale] = t.f32s("the head")?;
        let clear = t.f32s("the head")?;
        t.done("the head")?;
        Ok(Head {
            width,
            height,
            scale,
            clear,
        })
    }
}

fn put_props(b: &mut Put, p: &Props) {
    b.u32(p.z as u32);
    b.u8(u8::from(p.visible));
    b.f32(p.opacity);
    match p.clip {
        Some((x, y, w, h)) => {
            b.u8(1);
            for v in [x, y, w, h] {
                b.u32(v);
            }
        }
        None => b.u8(0),
    }
    b.len(p.effects.len());
    for e in &p.effects {
        match e {
            LayerEffect::Blur { sigma } => {
                b.u8(1);
                b.f32(*sigma);
            }
            LayerEffect::Shadow { sigma, color } => {
                b.u8(2);
                b.f32(*sigma);
                b.f32s(color);
            }
        }
    }
}

fn take_props(t: &mut Take<'_>) -> Result<Props, SceneError> {
    let z = t.u32("a layer's z-order")? as i32;
    let visible = t.bool("a layer's visibility")?;
    let opacity = t.f32("a layer's opacity")?;
    let clip = if t.bool("a layer's clip")? {
        Some((
            t.u32("a layer's clip")?,
            t.u32("a layer's clip")?,
            t.u32("a layer's clip")?,
            t.u32("a layer's clip")?,
        ))
    } else {
        None
    };
    let n = t.count(5, "a layer's effects")?;
    let mut effects = Vec::with_capacity(n);
    for _ in 0..n {
        effects.push(match t.u8("an effect")? {
            1 => LayerEffect::Blur {
                sigma: t.f32("a blur")?,
            },
            2 => LayerEffect::Shadow {
                sigma: t.f32("a shadow")?,
                color: t.f32s("a shadow")?,
            },
            tag => return Err(SceneError::Tag(tag)),
        });
    }
    Ok(Props {
        z,
        visible,
        opacity,
        clip,
        effects,
    })
}

fn put_key(b: &mut Put, k: &TextNodeKey) {
    b.str(&k.text);
    b.u32(k.font_size_bits);
    b.u32(k.line_height_bits);
    b.opt_u32(k.max_width_bits);
    b.u16(k.font_weight);
    b.u32(k.letter_spacing_bits);
    b.opt_str(k.font_family.as_deref());
}

/// a text key read back field by field, never through
/// `TextNodeKey::new`, which would take the string the source has now
/// under hot reload.
fn take_key(t: &mut Take<'_>) -> Result<TextNodeKey, SceneError> {
    Ok(TextNodeKey {
        text: t.str("a text")?,
        font_size_bits: t.u32("a text's size")?,
        line_height_bits: t.u32("a text's line height")?,
        max_width_bits: t.opt_u32("a text's width")?,
        font_weight: t.u16("a text's weight")?,
        letter_spacing_bits: t.u32("a text's letter spacing")?,
        font_family: t.opt_str("a text's family")?,
    })
}

pub(crate) fn put_node(b: &mut Put, node: &Drawn) {
    let node = match node {
        Drawn::Image {
            x,
            y,
            w,
            h,
            corner_radius,
            key,
        } => {
            b.u8(6);
            b.f32s(&[*x, *y, *w, *h, *corner_radius]);
            b.u64(*key);
            return;
        }
        Drawn::Path { key } => {
            b.u8(5);
            b.u64(*key);
            return;
        }
        Drawn::Node(node) => node,
    };
    match node {
        SceneNode::Rect { x, y, w, h, color } => {
            b.u8(1);
            b.f32s(&[*x, *y, *w, *h]);
            b.f32s(color);
        }
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h,
            color,
            corner_radius,
            border_width,
            border_color,
        } => {
            b.u8(2);
            b.f32s(&[*x, *y, *w, *h]);
            b.f32s(color);
            b.f32s(&[*corner_radius, *border_width]);
            b.f32s(border_color);
        }
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
        } => {
            b.u8(3);
            b.f32s(&[*x, *y, *w, *h]);
            b.f32s(color);
            b.f32s(color2);
            b.f32s(&[*angle_deg, *corner_radius, *border_width]);
            b.f32s(border_color);
        }
        SceneNode::Text { key, x, y, color } => {
            b.u8(4);
            put_key(b, key);
            b.f32s(&[*x, *y]);
            b.f32s(color);
        }
        SceneNode::PushClip { x, y, w, h } => {
            b.u8(7);
            b.f32s(&[*x, *y, *w, *h]);
        }
        SceneNode::PopClip => b.u8(8),
        SceneNode::BackdropBlur {
            x,
            y,
            w,
            h,
            corner_radius,
            sigma,
        } => {
            b.u8(9);
            b.f32s(&[*x, *y, *w, *h, *corner_radius, *sigma]);
        }
        SceneNode::Shadow {
            x,
            y,
            w,
            h,
            corner_radius,
            blur_radius,
            offset,
            color,
            inset,
        } => {
            b.u8(10);
            b.f32s(&[*x, *y, *w, *h, *corner_radius, *blur_radius]);
            b.f32s(offset);
            b.f32s(color);
            b.u8(u8::from(*inset));
        }
        // a `Drawn::Node` never holds these (see `Drawn`); written as the
        // keyed forms they would have been, so the bytes stay readable.
        SceneNode::Image {
            x,
            y,
            w,
            h,
            image,
            corner_radius,
        } => put_node(
            b,
            &Drawn::Image {
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                corner_radius: *corner_radius,
                key: image_key(image, false),
            },
        ),
        SceneNode::Path { data } => put_node(
            b,
            &Drawn::Path {
                key: path_key(data),
            },
        ),
    }
}

pub(crate) fn take_node(t: &mut Take<'_>) -> Result<Drawn, SceneError> {
    let node = match t.u8("a node's kind")? {
        1 => {
            let [x, y, w, h] = t.f32s("a rect")?;
            SceneNode::Rect {
                x,
                y,
                w,
                h,
                color: t.f32s("a rect")?,
            }
        }
        2 => {
            let [x, y, w, h] = t.f32s("a rounded rect")?;
            let color = t.f32s("a rounded rect")?;
            let [corner_radius, border_width] = t.f32s("a rounded rect")?;
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                color,
                corner_radius,
                border_width,
                border_color: t.f32s("a rounded rect")?,
            }
        }
        3 => {
            let [x, y, w, h] = t.f32s("a gradient rect")?;
            let color = t.f32s("a gradient rect")?;
            let color2 = t.f32s("a gradient rect")?;
            let [angle_deg, corner_radius, border_width] = t.f32s("a gradient rect")?;
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
                border_color: t.f32s("a gradient rect")?,
            }
        }
        4 => {
            let key = take_key(t)?;
            let [x, y] = t.f32s("a text's place")?;
            SceneNode::Text {
                key,
                x,
                y,
                color: t.f32s("a text's color")?,
            }
        }
        5 => {
            return Ok(Drawn::Path {
                key: t.u64("a path's key")?,
            });
        }
        6 => {
            let [x, y, w, h, corner_radius] = t.f32s("an image")?;
            return Ok(Drawn::Image {
                x,
                y,
                w,
                h,
                corner_radius,
                key: t.u64("an image's key")?,
            });
        }
        7 => {
            let [x, y, w, h] = t.f32s("a clip")?;
            SceneNode::PushClip { x, y, w, h }
        }
        8 => SceneNode::PopClip,
        9 => {
            let [x, y, w, h, corner_radius, sigma] = t.f32s("a backdrop blur")?;
            SceneNode::BackdropBlur {
                x,
                y,
                w,
                h,
                corner_radius,
                sigma,
            }
        }
        10 => {
            let [x, y, w, h, corner_radius, blur_radius] = t.f32s("a shadow")?;
            SceneNode::Shadow {
                x,
                y,
                w,
                h,
                corner_radius,
                blur_radius,
                offset: t.f32s("a shadow")?,
                color: t.f32s("a shadow")?,
                inset: t.bool("a shadow's inset")?,
            }
        }
        tag => return Err(SceneError::Tag(tag)),
    };
    Ok(Drawn::Node(node))
}

/// a whole layer: its key, its properties, its nodes.
pub(crate) fn body(key: u32, props: &Props, nodes: &[Drawn]) -> Vec<u8> {
    let mut b = Put::default();
    b.u32(key);
    put_props(&mut b, props);
    b.len(nodes.len());
    for n in nodes {
        put_node(&mut b, n);
    }
    b.0
}

pub(crate) fn read_body(bytes: &[u8]) -> Result<(u32, Props, Vec<Drawn>), SceneError> {
    let mut t = Take::new(bytes);
    let key = t.u32("a layer's key")?;
    let props = take_props(&mut t)?;
    let nodes = take_nodes(&mut t)?;
    t.done("a layer")?;
    Ok((key, props, nodes))
}

fn take_nodes(t: &mut Take<'_>) -> Result<Vec<Drawn>, SceneError> {
    let n = t.count(1, "a layer's nodes")?;
    let mut nodes = Vec::with_capacity(n);
    for _ in 0..n {
        nodes.push(take_node(t)?);
    }
    Ok(nodes)
}

/// a layer written against the version a receiver holds, `before`: its
/// properties, how many nodes of the start and of the end stay, and the
/// nodes between, written out.
pub(crate) fn patch(props: &Props, before: &[Drawn], after: &[Drawn]) -> Vec<u8> {
    let head = before
        .iter()
        .zip(after)
        .take_while(|(a, b)| same(a, b))
        .count();
    let room = before.len().min(after.len()) - head;
    let tail = before
        .iter()
        .rev()
        .zip(after.iter().rev())
        .take(room)
        .take_while(|(a, b)| same(a, b))
        .count();
    let mut b = Put::default();
    put_props(&mut b, props);
    b.len(head);
    b.len(tail);
    let middle = &after[head..after.len() - tail];
    b.len(middle.len());
    for n in middle {
        put_node(&mut b, n);
    }
    b.0
}

/// two nodes that write the same bytes (a NaN is the same as itself,
/// a negative zero is not a zero, as the layer's hash sees them).
fn same(a: &Drawn, b: &Drawn) -> bool {
    let (mut x, mut y) = (Put::default(), Put::default());
    put_node(&mut x, a);
    put_node(&mut y, b);
    x.0 == y.0
}

/// a patch applied to the nodes a receiver holds.
pub(crate) fn apply_patch(
    bytes: &[u8],
    before: &[Drawn],
) -> Result<(Props, Vec<Drawn>), SceneError> {
    let mut t = Take::new(bytes);
    let props = take_props(&mut t)?;
    let head = t.u32("a patch")? as usize;
    let tail = t.u32("a patch")? as usize;
    if head.saturating_add(tail) > before.len() {
        return Err(SceneError::Invalid(
            "a patch keeps more nodes than its base has",
        ));
    }
    let middle = take_nodes(&mut t)?;
    t.done("a patch")?;
    let mut nodes = Vec::with_capacity(head + middle.len() + tail);
    nodes.extend_from_slice(&before[..head]);
    nodes.extend(middle);
    nodes.extend_from_slice(&before[before.len() - tail..]);
    Ok((props, nodes))
}
