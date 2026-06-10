use super::{Compositor, LayerId, SceneNode};
use crate::compositor::scene::TextNodeKey;

/// Parameters for drawing a rounded rectangle with optional border.
pub struct RoundedRectParams {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
}

/// Parameters for an analytic shadow cast by a rounded rect.
pub struct ShadowParams {
    /// Bounds of the rect casting the shadow (not the expanded quad).
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub corner_radius: f32,
    /// CSS-like blur radius (Gaussian sigma = blur_radius / 2).
    pub blur_radius: f32,
    pub offset: [f32; 2],
    pub color: [f32; 4],
    /// CSS `box-shadow: inset`: the shadow falls INSIDE the rect, clipped
    /// to its rounded bounds.
    pub inset: bool,
}

/// Parameters for drawing a rounded rect filled with a 2-stop linear gradient.
pub struct GradientRectParams {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
    pub color2: [f32; 4],
    /// CSS-style angle in degrees (0 = first stop at the bottom, clockwise).
    pub angle_deg: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
}

impl Compositor {
    pub fn push(&mut self, node: SceneNode) {
        self.push_to_layer(LayerId::DEFAULT, node);
    }

    pub fn push_to_layer(&mut self, layer_id: LayerId, node: SceneNode) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) {
            layer.nodes.push(node);
        } else {
            log::warn!("push_to_layer: layer {:?} not found", layer_id);
        }
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        self.push(SceneNode::Rect { x, y, w, h, color });
    }

    /// Scope following nodes to `rect` (intersected with any active clip)
    /// until the matching `pop_clip`.
    pub fn push_clip(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.push(SceneNode::PushClip { x, y, w, h });
    }

    pub fn pop_clip(&mut self) {
        self.push(SceneNode::PopClip);
    }

    pub fn draw_text(&mut self, key: TextNodeKey, x: f32, y: f32, color: [f32; 4]) {
        self.push(SceneNode::Text { key, x, y, color });
    }

    pub fn draw_path(&mut self, data: crate::path::TessellatedPath) {
        self.push(SceneNode::Path { data });
    }

    pub fn draw_rounded_rect(&mut self, p: RoundedRectParams) {
        self.push(SceneNode::RoundedRect {
            x: p.x,
            y: p.y,
            w: p.w,
            h: p.h,
            color: p.color,
            corner_radius: p.corner_radius,
            border_width: p.border_width,
            border_color: p.border_color,
        });
    }

    /// Draw an image from the image atlas (see `gpu::image::load_image_bytes`).
    pub fn draw_image(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        image: crate::gpu::image::ImageHandle,
        corner_radius: f32,
    ) {
        self.push(SceneNode::Image {
            x,
            y,
            w,
            h,
            image,
            corner_radius,
        });
    }

    /// Draw an analytic shadow. Push drop shadows BEFORE the rect that
    /// casts them so the rect paints on top; push inset shadows AFTER the
    /// surface fill so they composite over it (draw order follows push
    /// order). Multi-shadow stacks (CSS comma lists) are just several
    /// pushes.
    pub fn draw_shadow(&mut self, p: ShadowParams) {
        self.push(SceneNode::Shadow {
            x: p.x,
            y: p.y,
            w: p.w,
            h: p.h,
            corner_radius: p.corner_radius,
            blur_radius: p.blur_radius,
            offset: p.offset,
            color: p.color,
            inset: p.inset,
        });
    }

    pub fn draw_gradient_rect(&mut self, p: GradientRectParams) {
        self.push(SceneNode::GradientRect {
            x: p.x,
            y: p.y,
            w: p.w,
            h: p.h,
            color: p.color,
            color2: p.color2,
            angle_deg: p.angle_deg,
            corner_radius: p.corner_radius,
            border_width: p.border_width,
            border_color: p.border_color,
        });
    }
}
