//! Per-layer draw sequence: the ordered list of draw commands a layer's
//! geometry build produces. Unlike the per-type [`DrawRange`] lists (which
//! group all primitives of one pipeline together), the sequence preserves
//! the *push order* across primitive types, so the encoder can interleave
//! pipelines and reproduce exactly what the scene author stacked: a path
//! icon over an SDF pill over a card, text behind a later rect, etc.
//! Pipeline switches inside a UI render pass are cheap; consecutive
//! commands of the same kind and clip are merged so steady-state scenes
//! still issue few draws.

use crate::compositor::clip::{ClipRect, DrawRange};

/// Which geometry buffers / pipeline a draw command targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawKind {
    /// Plain rects and tessellated paths (quad pipeline).
    Quad,
    /// Analytic shadows (drop and inset).
    Shadow,
    /// Rounded and gradient rects (SDF pipeline).
    SdfRect,
    /// Image-atlas sprites.
    Image,
    /// Glyph quads. Index ranges are patched in after the text system
    /// resolves glyphs (see [`Layer::set_text_data_with_ranges`]).
    ///
    /// [`Layer::set_text_data_with_ranges`]: crate::compositor::Layer::set_text_data_with_ranges
    Text,
}

/// One step of a layer's draw sequence, in scene push order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawCommand {
    /// Draw `range` from the layer's `kind` geometry buffers.
    Geometry { kind: DrawKind, range: DrawRange },
    /// Backdrop blur: the encoder suspends the layer pass here, resolves
    /// everything composited below (lower layers + this layer so far) to a
    /// texture, blurs it, then resumes the pass drawing the blurred quad
    /// (`first_index..first_index+6` in the backdrop buffers) clipped to
    /// its rounded rect. One resolve per command -- see
    /// [`SceneNode::BackdropBlur`].
    ///
    /// [`SceneNode::BackdropBlur`]: crate::compositor::SceneNode::BackdropBlur
    BackdropBlur {
        first_index: u32,
        /// Gaussian sigma in pixels.
        sigma: f32,
        clip: Option<ClipRect>,
    },
}

/// Append a geometry command, merging into the previous one when it has
/// the same kind and clip and the index range is contiguous (text commands
/// never merge: their ranges are placeholders patched after glyph
/// resolution, see [`DrawKind::Text`]).
pub(crate) fn push_geometry(
    sequence: &mut Vec<DrawCommand>,
    kind: DrawKind,
    first_index: u32,
    index_count: u32,
    clip: Option<ClipRect>,
) {
    if index_count == 0 && kind != DrawKind::Text {
        return;
    }
    if kind != DrawKind::Text
        && let Some(DrawCommand::Geometry {
            kind: last_kind,
            range,
        }) = sequence.last_mut()
        && *last_kind == kind
        && range.clip == clip
        && range.first_index + range.index_count == first_index
    {
        range.index_count += index_count;
        return;
    }
    sequence.push(DrawCommand::Geometry {
        kind,
        range: DrawRange {
            first_index,
            index_count,
            clip,
        },
    });
}
