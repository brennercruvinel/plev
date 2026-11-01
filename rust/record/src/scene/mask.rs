//! the places of a frame whose drawing does not travel (task-79 phase 2):
//! a secret control's bound rect, on the layer it is bound on. the app
//! says which (`Mask`, from its projection and its bindings); the sender
//! writes each masked layer without what the secret draws, before the
//! body, the hash, the patch and the assets are made, so none of them
//! ever holds it.
//!
//! in a mask's place the layer carries a box instead: a dark rounded rect
//! with its edge and [`SECRET_LABEL`], the rect's place and size and
//! nothing else, where the first node the secret draws was (under what
//! the layer draws after it, clipped as that node was). its nodes depend
//! on the rect alone, so a change of the secret changes no byte of the
//! layer: nothing travels when the person types in a secret field, not
//! even the caret.
//!
//! what the secret draws, in the mask's layer: every node inside a clip
//! that lies in the rect (the field's own clip: a line scrolled out of
//! sight in it included), every text, image and path whose extent meets
//! the rect, and every other node that meets it without covering the
//! whole rect (a caret, a selection, a mark). a node that covers the
//! whole rect, out of such a clip, is a ground (the page, a panel, the
//! field's own fill and edge) and travels; a clip travels, it draws
//! nothing. a text's extent is measured by the engine's own measurer, the
//! one its drawing takes, so a text scrolled in a field, its origin out
//! of the rect, is hidden too. what a secret draws on another layer, or
//! out of the rect it is bound to and of its clips, is not seen here: the
//! page marks and binds it.

use std::borrow::Cow;

use engine::drawing::compositor::{SceneNode, TextNodeKey};
use engine::drawing::text::{OverflowWrap, TextMeasurer, TextStyle};

/// a place of a frame whose drawing does not travel: `rect`, `(x, y, w,
/// h)` in logical pixels, on the layer `layer` (the app's `LayerId`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mask {
    pub layer: u32,
    pub rect: [f32; 4],
}

/// what the box in a secret's place says.
pub const SECRET_LABEL: &str = "secret, not mirrored";

/// how far past its measured box a text may ink (a glyph's overhang).
const INK: f32 = 1.0;

/// `nodes`, one layer's, as they travel with every place of `rects`
/// hidden: each node the secret draws left out, the box written where
/// the first of them was.
pub(crate) fn hide<'a>(nodes: &'a [SceneNode], rects: &[[f32; 4]]) -> Vec<Cow<'a, SceneNode>> {
    if rects.is_empty() {
        return nodes.iter().map(Cow::Borrowed).collect();
    }
    let mut boxed = vec![false; rects.len()];
    let mut clips: Vec<[f32; 4]> = Vec::new();
    let mut out = Vec::with_capacity(nodes.len() + 4 * rects.len());
    for node in nodes {
        match node {
            SceneNode::PushClip { x, y, w, h } => clips.push([*x, *y, *w, *h]),
            SceneNode::PopClip => {
                clips.pop();
            }
            _ => {}
        }
        let Some(area) = extent(node) else {
            out.push(Cow::Borrowed(node));
            continue;
        };
        let content = matches!(
            node,
            SceneNode::Text { .. } | SceneNode::Image { .. } | SceneNode::Path { .. }
        );
        let mut drawn_by_secret = false;
        for (at, rect) in rects.iter().enumerate() {
            // inside a clip of the secret's own (one that lies in its
            // place), whatever the node's extent: a line scrolled out of
            // a field is still the field's
            let owned = clips.iter().any(|clip| covers(*rect, *clip));
            let meets = meets(area, *rect) && (content || !covers(area, *rect));
            if !(owned || meets) {
                continue;
            }
            drawn_by_secret = true;
            if !boxed[at] {
                boxed[at] = true;
                out.extend(secret_box(*rect).into_iter().map(Cow::Owned));
            }
        }
        if !drawn_by_secret {
            out.push(Cow::Borrowed(node));
        }
    }
    out
}

/// where a node can put ink, `(x, y, w, h)`; `None` for a clip, which
/// draws nothing.
fn extent(node: &SceneNode) -> Option<[f32; 4]> {
    match node {
        SceneNode::Rect { x, y, w, h, .. }
        | SceneNode::RoundedRect { x, y, w, h, .. }
        | SceneNode::GradientRect { x, y, w, h, .. }
        | SceneNode::Image { x, y, w, h, .. }
        | SceneNode::BackdropBlur { x, y, w, h, .. } => Some([*x, *y, *w, *h]),
        SceneNode::Shadow {
            x,
            y,
            w,
            h,
            blur_radius,
            offset,
            ..
        } => {
            let spread = blur_radius.abs();
            let [dx, dy] = *offset;
            Some([
                x + dx.min(0.0) - spread,
                y + dy.min(0.0) - spread,
                w + dx.abs() + 2.0 * spread,
                h + dy.abs() + 2.0 * spread,
            ])
        }
        SceneNode::Text { key, x, y, .. } => Some(text_extent(key, *x, *y)),
        SceneNode::Path { data } => {
            let mut points = data.vertices.iter().map(|v| v.position);
            let [x0, y0] = points.next()?;
            let (mut lo, mut hi) = ([x0, y0], [x0, y0]);
            for [px, py] in points {
                lo = [lo[0].min(px), lo[1].min(py)];
                hi = [hi[0].max(px), hi[1].max(py)];
            }
            Some([lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]])
        }
        SceneNode::PushClip { .. } | SceneNode::PopClip => None,
    }
}

/// a text's box as the engine lays it out from `(x, y)`, measured with
/// the typography its key carries, a pixel of ink around it.
fn text_extent(key: &TextNodeKey, x: f32, y: f32) -> [f32; 4] {
    let style = TextStyle {
        font_size: f32::from_bits(key.font_size_bits),
        line_height: f32::from_bits(key.line_height_bits),
        font_weight: key.font_weight,
        letter_spacing: f32::from_bits(key.letter_spacing_bits),
        font_family: key.font_family.clone(),
        features: key.features,
        overflow_wrap: key.overflow_wrap,
    };
    let wrap = key.max_width_bits.map(f32::from_bits);
    let (w, h) = TextMeasurer::measure_styled(&key.text, &style, wrap);
    [
        x - INK,
        y - INK,
        w + 2.0 * INK,
        h.max(style.line_height) + 2.0 * INK,
    ]
}

/// the two boxes share some area (touching edges do not).
fn meets(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}

/// `a` holds the whole of `b`.
fn covers(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] <= b[0] && a[1] <= b[1] && a[0] + a[2] >= b[0] + b[2] && a[1] + a[3] >= b[1] + b[3]
}

/// the box in a secret's place: a dark rounded rect with its edge, and
/// [`SECRET_LABEL`] in it, clipped to it. it depends on the rect alone.
fn secret_box([x, y, w, h]: [f32; 4]) -> [SceneNode; 4] {
    let line: f32 = 16.0;
    [
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h,
            color: [0.06, 0.06, 0.07, 1.0],
            corner_radius: 4.0f32.min(h / 2.0).max(0.0),
            border_width: 1.0,
            border_color: [0.42, 0.42, 0.46, 1.0],
        },
        SceneNode::PushClip { x, y, w, h },
        SceneNode::Text {
            key: TextNodeKey {
                text: SECRET_LABEL.to_owned(),
                font_size_bits: 12.0f32.to_bits(),
                line_height_bits: line.to_bits(),
                max_width_bits: None,
                font_weight: 500,
                letter_spacing_bits: 0.0f32.to_bits(),
                font_family: None,
                features: engine::drawing::text::TextFeatures::NONE,
                overflow_wrap: OverflowWrap::Normal,
            },
            x: x + 8.0,
            y: y + ((h - line) / 2.0).max(0.0),
            color: [0.78, 0.78, 0.8, 1.0],
        },
        SceneNode::PopClip,
    ]
}
