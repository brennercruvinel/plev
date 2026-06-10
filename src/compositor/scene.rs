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
    Text {
        key: TextNodeKey,
        x: f32,
        y: f32,
        color: [f32; 4],
    },
    Path {
        data: crate::path::TessellatedPath,
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
        }
        h.finish()
    }
}
