//! the recorded frame, fit into a rect of the view. the session keeps a
//! frame as the nodes the app drew, in logical window pixels, and where
//! each control showed in the same space (`rec::Bound`). it keeps neither
//! the window's size nor its scale: the size is the extent of the nodes
//! and the bindings, from the origin (every node the recorded apps draw
//! starts at the window background), and the scale does not enter the
//! drawing (the nodes are logical; the view draws them at its own).
//!
//! a fit is one scale and one offset. every node goes through it: a rect
//! by its corners, a text by its typography (size, line height, width,
//! letter spacing), a path by its vertices. an image never comes through:
//! a session carries no pixel, and its replay lowers an image to nothing.

use engine::drawing::compositor::{SceneNode, TextNodeKey};
use engine::drawing::path::TessellatedPath;
use engine::widgets::Rect;
use record::Bound;

/// how a frame of `extent` sits in the view: `view = frame * scale +
/// offset`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub scale: f32,
    pub dx: f32,
    pub dy: f32,
    /// the frame's size in its own logical pixels.
    pub extent: (f32, f32),
}

/// the size a frame was drawn at: the farthest right and bottom edge of
/// its nodes and its bindings, from the origin. `None` when nothing in it
/// has a size.
pub fn extent(scene: &[SceneNode], bound: &[Bound]) -> Option<(f32, f32)> {
    let mut w: f32 = 0.0;
    let mut h: f32 = 0.0;
    let mut grow = |x: f32, y: f32| {
        if x.is_finite() && y.is_finite() {
            w = w.max(x);
            h = h.max(y);
        }
    };
    for node in scene {
        match node {
            SceneNode::Rect { x, y, w, h, .. }
            | SceneNode::RoundedRect { x, y, w, h, .. }
            | SceneNode::GradientRect { x, y, w, h, .. }
            | SceneNode::Image { x, y, w, h, .. }
            | SceneNode::BackdropBlur { x, y, w, h, .. }
            | SceneNode::Shadow { x, y, w, h, .. } => grow(x + w, y + h),
            SceneNode::Path { data } => {
                for v in &data.vertices {
                    grow(v.position[0], v.position[1]);
                }
            }
            SceneNode::Text { .. } | SceneNode::PushClip { .. } | SceneNode::PopClip => {}
        }
    }
    for b in bound {
        let [x, y, bw, bh] = b.rect;
        grow(x + bw, y + bh);
    }
    (w > 0.0 && h > 0.0).then_some((w, h))
}

impl Fit {
    /// the largest scale that shows all of `extent` inside `into`,
    /// centered in it.
    pub fn into(into: Rect, extent: (f32, f32)) -> Fit {
        let (w, h) = extent;
        let scale = if w > 0.0 && h > 0.0 {
            (into.w / w).min(into.h / h).max(0.0)
        } else {
            0.0
        };
        Fit {
            scale,
            dx: into.x + (into.w - w * scale) / 2.0,
            dy: into.y + (into.h - h * scale) / 2.0,
            extent,
        }
    }

    /// where the frame shows in the view.
    pub fn frame(&self) -> Rect {
        self.rect([0.0, 0.0, self.extent.0, self.extent.1])
    }

    /// a rect of the frame, `(x, y, w, h)`, as it shows in the view.
    pub fn rect(&self, r: [f32; 4]) -> Rect {
        let [x, y, w, h] = r;
        Rect::new(
            x * self.scale + self.dx,
            y * self.scale + self.dy,
            w * self.scale,
            h * self.scale,
        )
    }

    /// a point of the view, in the frame's own pixels; `None` outside the
    /// frame.
    pub fn to_frame(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.scale <= 0.0 || !self.frame().contains(x, y) {
            return None;
        }
        Some(((x - self.dx) / self.scale, (y - self.dy) / self.scale))
    }

    fn px(&self, x: f32) -> f32 {
        x * self.scale + self.dx
    }

    fn py(&self, y: f32) -> f32 {
        y * self.scale + self.dy
    }

    fn len(&self, v: f32) -> f32 {
        v * self.scale
    }

    /// a node of the frame as it shows in the view. an image draws
    /// nothing: the frame the view shows never carries a pixel.
    pub fn node(&self, node: &SceneNode) -> Option<SceneNode> {
        let s = |v: f32| self.len(v);
        Some(match node.clone() {
            SceneNode::Rect { x, y, w, h, color } => SceneNode::Rect {
                x: self.px(x),
                y: self.py(y),
                w: s(w),
                h: s(h),
                color,
            },
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                color,
                corner_radius,
                border_width,
                border_color,
            } => SceneNode::RoundedRect {
                x: self.px(x),
                y: self.py(y),
                w: s(w),
                h: s(h),
                color,
                corner_radius: s(corner_radius),
                border_width: s(border_width),
                border_color,
            },
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
            } => SceneNode::GradientRect {
                x: self.px(x),
                y: self.py(y),
                w: s(w),
                h: s(h),
                color,
                color2,
                angle_deg,
                corner_radius: s(corner_radius),
                border_width: s(border_width),
                border_color,
            },
            SceneNode::Text { key, x, y, color } => SceneNode::Text {
                key: self.key(key),
                x: self.px(x),
                y: self.py(y),
                color,
            },
            SceneNode::Path { data } => SceneNode::Path {
                data: self.path(data),
            },
            SceneNode::Image { .. } => return None,
            SceneNode::PushClip { x, y, w, h } => SceneNode::PushClip {
                x: self.px(x),
                y: self.py(y),
                w: s(w),
                h: s(h),
            },
            SceneNode::PopClip => SceneNode::PopClip,
            SceneNode::BackdropBlur {
                x,
                y,
                w,
                h,
                corner_radius,
                sigma,
            } => SceneNode::BackdropBlur {
                x: self.px(x),
                y: self.py(y),
                w: s(w),
                h: s(h),
                corner_radius: s(corner_radius),
                sigma: s(sigma),
            },
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
            } => SceneNode::Shadow {
                x: self.px(x),
                y: self.py(y),
                w: s(w),
                h: s(h),
                corner_radius: s(corner_radius),
                blur_radius: s(blur_radius),
                offset: [s(offset[0]), s(offset[1])],
                color,
                inset,
            },
        })
    }

    /// a text key at this scale: the same string and face, every length
    /// of its typography scaled.
    fn key(&self, key: TextNodeKey) -> TextNodeKey {
        let s = |bits: u32| self.len(f32::from_bits(bits)).to_bits();
        TextNodeKey {
            font_size_bits: s(key.font_size_bits),
            line_height_bits: s(key.line_height_bits),
            max_width_bits: key.max_width_bits.map(s),
            letter_spacing_bits: s(key.letter_spacing_bits),
            ..key
        }
    }

    /// a path at this fit: every vertex moved, its hash mixed with the fit
    /// so two fits of one path stay two cache entries.
    fn path(&self, mut path: TessellatedPath) -> TessellatedPath {
        for v in &mut path.vertices {
            v.position = [self.px(v.position[0]), self.py(v.position[1])];
        }
        let mix = [self.scale, self.dx, self.dy]
            .iter()
            .fold(0u64, |h, v| h.rotate_left(21) ^ u64::from(v.to_bits()));
        path.hash ^= mix;
        path
    }
}
