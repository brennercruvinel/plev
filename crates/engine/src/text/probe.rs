//! Headless glyph rendering: what the rasterizer actually puts on screen.
//!
//! The project's conventions require pixels for a visual claim, and the
//! window path cannot supply them in CI (or wherever the OS denies screen
//! capture). The probe builds a text scene and renders it through
//! [`crate::headless`], which is the window's own frame path
//! (`window::draw_frame`): compositor layers, the real [`TextSystem`] and
//! glyph atlas, the text pipeline, the composite pass, the sRGB target.
//! It adds no drawing of its own; it only lays out specimens and measures
//! the RGBA8 that comes back.
//!
//! Multi-frame rendering (`render_frames`) exists because atlas-lifecycle
//! behavior only shows under accumulated state: grown atlas, evictions,
//! layers retained across frames.
//!
//! [`TextSystem`]: crate::text::TextSystem

use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::gpu::RenderConfig;
use crate::headless::{HeadlessError, Renderer, Viewport};
use crate::text::TextStyle;

/// Background, linear. The sRGB target encodes it to about 108.
const CLEAR: [f32; 4] = [0.15, 0.15, 0.15, 1.0];

/// Specimen color: white, so coverage reads straight off the red channel.
const INK: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// Above the cleared background (linear 0.15 -> ~108 in sRGB8) and below
/// antialiased glyph edges, so it counts coverage rather than canvas.
const INK_THRESHOLD: u8 = 170;

/// One line of the scene: what to draw, in which style, and where.
#[derive(Clone)]
pub struct Specimen {
    pub text: String,
    pub style: TextStyle,
    pub x: f32,
    pub y: f32,
}

impl Specimen {
    pub fn new(text: impl Into<String>, style: TextStyle, x: f32, y: f32) -> Self {
        Self {
            text: text.into(),
            style,
            x,
            y,
        }
    }

    fn node(&self) -> SceneNode {
        SceneNode::Text {
            key: TextNodeKey::from_style(&self.text, &self.style, None),
            x: self.x,
            y: self.y,
            color: INK,
        }
    }
}

/// An offscreen render: RGBA8, row-major, `width * height * 4` bytes.
pub struct Rendered {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Rendered {
    /// Rows `y..y + h`, for comparing one band between renders whose other
    /// content deliberately differs.
    pub fn band(&self, y: u32, h: u32) -> &[u8] {
        let start = ((y * self.width * 4) as usize).min(self.rgba.len());
        let end = (((y + h) * self.width * 4) as usize).min(self.rgba.len());
        &self.rgba[start..end]
    }

    /// Count of pixels carrying glyph coverage.
    ///
    /// The threshold sits well above the cleared background: the clear color
    /// is linear 0.15, which an sRGB target encodes to about 108, so a naive
    /// `> 60` counts the whole canvas and any comparison built on it is
    /// vacuous.
    pub fn ink(&self) -> usize {
        self.rgba
            .chunks_exact(4)
            .filter(|p| p[0] > INK_THRESHOLD)
            .count()
    }

    /// Horizontal extent of the ink in rows `y..y + h`, as
    /// `(first_x, last_x)` in physical pixels, or `None` if the band is
    /// blank.
    pub fn ink_extent(&self, y: u32, h: u32) -> Option<(u32, u32)> {
        let (mut first, mut last) = (u32::MAX, 0u32);
        for row in y..(y + h).min(self.height) {
            for col in 0..self.width {
                let i = ((row * self.width + col) * 4) as usize;
                if self.rgba.get(i).is_some_and(|&r| r > INK_THRESHOLD) {
                    first = first.min(col);
                    last = last.max(col);
                }
            }
        }
        (first != u32::MAX).then_some((first, last))
    }

    pub fn write_png(&self, path: impl AsRef<std::path::Path>) -> Result<(), HeadlessError> {
        crate::headless::write_png(path.as_ref(), self.width, self.height, &self.rgba)
    }
}

/// Distinct glyph/size/weight combinations, enough of them that the glyph
/// atlas must grow partway through the frame.
///
/// The initial atlas holds a few hundred glyphs; a real screen of widgets
/// exceeds that routinely, so pixel tests that want realistic atlas
/// pressure start from these.
pub fn atlas_filling_specimens() -> Vec<(String, TextStyle)> {
    const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut out = Vec::new();
    for (i, weight) in [400u16, 500, 600, 700].iter().enumerate() {
        for size in [11.0f32, 13.0, 15.0, 17.0, 19.0, 23.0, 29.0, 31.0] {
            out.push((
                format!("{i}{ALPHABET}"),
                TextStyle::new(size)
                    .with_line_height(size * 1.4)
                    .with_weight(*weight),
            ));
        }
    }
    out
}

/// A compositor layer: its text, and whether its scene changes every
/// frame.
///
/// `static_layer` models a sidebar or header whose scene never changes:
/// after frame one its scene hash is unchanged, so the frame path neither
/// re-resolves its text (unless the atlas was disturbed) nor redraws it,
/// and the composite keeps showing the texture frame one drew.
pub struct Layer {
    pub specimens: Vec<Specimen>,
    pub redraws_every_frame: bool,
}

impl Layer {
    /// Changes every frame (animated content, hover states), so the frame
    /// path re-resolves and redraws it every frame.
    pub fn dynamic(specimens: Vec<Specimen>) -> Self {
        Self {
            specimens,
            redraws_every_frame: true,
        }
    }

    /// Unchanged after frame one, like a static panel.
    pub fn static_layer(specimens: Vec<Specimen>) -> Self {
        Self {
            specimens,
            redraws_every_frame: false,
        }
    }
}

/// A node that changes its layer's scene hash each frame and draws
/// nothing: a transparent 1x1 rect left of the viewport, culled before
/// geometry. It is how a dynamic layer's scene changes without changing
/// its pixels.
fn frame_marker(frame: usize) -> SceneNode {
    SceneNode::Rect {
        x: -2.0 - frame as f32,
        y: -2.0,
        w: 1.0,
        h: 1.0,
        color: [0.0; 4],
    }
}

/// Render `frames` frames of `layers` through the frame path, one
/// compositor layer each (z order = slice order), and read back the last.
///
/// This is the shape of the real render loop: one renderer and one
/// compositor across the frames, so retained layers and the shared atlas
/// interact across frames, not within one.
///
/// `None` when no GPU adapter is available, so callers can skip instead of
/// failing on machines without one.
pub fn render_frames(
    layers: &[Layer],
    frames: usize,
    logical_w: u32,
    logical_h: u32,
    raster_scale: f32,
) -> Option<Rendered> {
    let viewport = Viewport::new(logical_w as f32, logical_h as f32, raster_scale);
    let mut renderer = match Renderer::new(viewport, RenderConfig::default()) {
        Ok(r) => r,
        Err(HeadlessError::NoAdapter) => return None,
        Err(e) => {
            log::error!("text probe: {e}");
            return None;
        }
    };
    let mut compositor = Compositor::new();
    let ids: Vec<LayerId> = (1..=layers.len() as i32)
        .map(|z| compositor.create_layer(z))
        .collect();

    for frame in 0..frames.max(1) {
        compositor.begin_frame();
        for (layer, &id) in layers.iter().zip(&ids) {
            if layer.redraws_every_frame {
                compositor.push_to_layer(id, frame_marker(frame));
            }
            for specimen in &layer.specimens {
                compositor.push_to_layer(id, specimen.node());
            }
        }
        renderer.draw(&mut compositor, CLEAR);
    }

    match renderer.read() {
        Ok(frame) => Some(Rendered {
            width: frame.width,
            height: frame.height,
            rgba: frame.rgba,
        }),
        Err(e) => {
            log::error!("text probe: {e}");
            None
        }
    }
}

/// Render `specimens` into a `logical_w x logical_h` scene at `raster_scale`.
///
/// `None` when no GPU adapter is available, so callers can skip instead of
/// failing on machines without one.
pub fn render(
    specimens: &[Specimen],
    logical_w: u32,
    logical_h: u32,
    raster_scale: f32,
) -> Option<Rendered> {
    render_frames(
        &[Layer::dynamic(specimens.to_vec())],
        1,
        logical_w,
        logical_h,
        raster_scale,
    )
}
