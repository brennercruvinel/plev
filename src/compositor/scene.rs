use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub enum SceneNode {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
    },
    RoundedRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        corner_radius: f32,
        border_width: f32,
        border_color: [f32; 4],
    },
    /// Rounded rect filled with a 2-stop linear gradient. Rendered by the
    /// same SDF pipeline as `RoundedRect` (use radius 0 for plain rects).
    GradientRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        color2: [f32; 4],
        /// CSS-style angle in degrees: 0 = first stop at the bottom,
        /// 90 = first stop at the left, clockwise.
        angle_deg: f32,
        corner_radius: f32,
        border_width: f32,
        border_color: [f32; 4],
    },
    Text {
        key: TextNodeKey,
        x: f32,
        y: f32,
        color: [f32; 4],
    },
    Path {
        data: crate::path::TessellatedPath,
    },
    /// An image from the image atlas, optionally with rounded corners.
    Image {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        image: crate::gpu::image::ImageHandle,
        corner_radius: f32,
    },
    /// Push a clip rect onto the clip stack: following nodes are scissored
    /// to the intersection of all pushed rects until the matching `PopClip`.
    PushClip {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    /// Pop the most recent `PushClip`.
    PopClip,
    /// Region backdrop blur (CSS `backdrop-filter: blur(sigma)`): at this
    /// point of the draw sequence, everything already composited below --
    /// lower layers plus what this layer drew so far -- is resolved to a
    /// texture, Gaussian-blurred and drawn back clipped to the rounded
    /// rect. Nodes pushed after draw on top of the frosted region.
    ///
    /// Cost: one backdrop resolve (composite + 2-pass blur over the full
    /// surface) per node; scenes are expected to hold only a few.
    BackdropBlur {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        corner_radius: f32,
        /// Gaussian sigma in pixels.
        sigma: f32,
    },
    /// Analytic shadow of a rounded rect (Evan Wallace approximation, no
    /// blur pass). `x..h` are the bounds of the CASTING rect.
    ///
    /// Drop (`inset: false`): the emitted quad is expanded by the blur and
    /// shifted by `offset`; push it BEFORE the rect that casts it.
    ///
    /// Inset (`inset: true`): the shadow falls INSIDE the rect (CSS
    /// `box-shadow: inset`), clipped to the rounded bounds -- the HOFF
    /// glass key-light (`inset 2px 4px 16px rgba(248,248,248,.06)`). Push
    /// it AFTER the surface fill so it composites on top.
    Shadow {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        corner_radius: f32,
        /// CSS-like blur radius (Gaussian sigma = blur_radius / 2).
        blur_radius: f32,
        offset: [f32; 2],
        color: [f32; 4],
        inset: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextNodeKey {
    pub text: String,
    pub font_size_bits: u32,
    pub line_height_bits: u32,
    pub max_width_bits: Option<u32>,
    pub font_weight: u16,
    pub font_family: Option<String>,
}

impl TextNodeKey {
    pub fn new(text: &str, font_size: f32, line_height: f32, max_width: Option<f32>) -> Self {
        Self {
            text: text.to_string(),
            font_size_bits: font_size.to_bits(),
            line_height_bits: line_height.to_bits(),
            max_width_bits: max_width.map(|w| w.to_bits()),
            font_weight: 400,
            font_family: None,
        }
    }

    pub fn with_weight(mut self, weight: u16) -> Self {
        self.font_weight = weight;
        self
    }

    pub fn with_family(mut self, family: &str) -> Self {
        self.font_family = Some(family.to_string());
        self
    }
}

impl SceneNode {
    pub(crate) fn hash_u64(&self) -> u64 {
        let mut h = FxHasher::default();
        match self {
            SceneNode::Rect {
                x,
                y,
                w,
                h: rh,
                color,
            } => {
                0u8.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                w.to_bits().hash(&mut h);
                rh.to_bits().hash(&mut h);
                for c in color {
                    c.to_bits().hash(&mut h);
                }
            }
            SceneNode::Text { key, x, y, color } => {
                1u8.hash(&mut h);
                key.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                for c in color {
                    c.to_bits().hash(&mut h);
                }
            }
            SceneNode::Path { data } => {
                2u8.hash(&mut h);
                data.hash.hash(&mut h);
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
                3u8.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                w.to_bits().hash(&mut h);
                rh.to_bits().hash(&mut h);
                for c in color {
                    c.to_bits().hash(&mut h);
                }
                corner_radius.to_bits().hash(&mut h);
                border_width.to_bits().hash(&mut h);
                for c in border_color {
                    c.to_bits().hash(&mut h);
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
                8u8.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                w.to_bits().hash(&mut h);
                rh.to_bits().hash(&mut h);
                image.hash(&mut h);
                corner_radius.to_bits().hash(&mut h);
            }
            SceneNode::PushClip { x, y, w, h: rh } => {
                6u8.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                w.to_bits().hash(&mut h);
                rh.to_bits().hash(&mut h);
            }
            SceneNode::PopClip => {
                7u8.hash(&mut h);
            }
            SceneNode::BackdropBlur {
                x,
                y,
                w,
                h: rh,
                corner_radius,
                sigma,
            } => {
                9u8.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                w.to_bits().hash(&mut h);
                rh.to_bits().hash(&mut h);
                corner_radius.to_bits().hash(&mut h);
                sigma.to_bits().hash(&mut h);
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
                5u8.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                w.to_bits().hash(&mut h);
                rh.to_bits().hash(&mut h);
                corner_radius.to_bits().hash(&mut h);
                blur_radius.to_bits().hash(&mut h);
                for o in offset {
                    o.to_bits().hash(&mut h);
                }
                for c in color {
                    c.to_bits().hash(&mut h);
                }
                inset.hash(&mut h);
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
                4u8.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                w.to_bits().hash(&mut h);
                rh.to_bits().hash(&mut h);
                for c in color {
                    c.to_bits().hash(&mut h);
                }
                for c in color2 {
                    c.to_bits().hash(&mut h);
                }
                angle_deg.to_bits().hash(&mut h);
                corner_radius.to_bits().hash(&mut h);
                border_width.to_bits().hash(&mut h);
                for c in border_color {
                    c.to_bits().hash(&mut h);
                }
            }
        }
        h.finish()
    }
}
