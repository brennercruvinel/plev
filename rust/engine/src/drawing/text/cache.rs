// ---------------------------------------------------------------------------
// Glyph atlas internals
// ---------------------------------------------------------------------------

/// Identity of one rasterized glyph bitmap.
///
/// This is cosmic-text's own [`cosmic_text::CacheKey`] verbatim, and it must
/// stay that way: the key has to name *every* input `SwashCache` rasterizes
/// with (face, glyph, size, weight, subpixel bins, flags), or two different
/// bitmaps alias onto one atlas entry.
pub(crate) type GlyphCacheKey = cosmic_text::CacheKey;

#[derive(Clone, Debug)]
pub(crate) struct GlyphEntry {
    /// The atlas rectangle backing this glyph, or `None` for a glyph with no
    /// bitmap at all (a space, and anything else swash rasterizes to a
    /// zero-size mask). Empty glyphs reserve nothing, so eviction must not
    /// deallocate anything for them.
    pub(crate) alloc_id: Option<etagere::AllocId>,
    pub(crate) atlas_x: u32,
    pub(crate) atlas_y: u32,
    /// Size in pixels of the bitmap. A mask glyph takes one texel per
    /// pixel; a color glyph takes [`super::atlas::COLOR_TEXELS`] per pixel
    /// on a row, from an `atlas_x` that is a multiple of it.
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) left: f32,
    pub(crate) top: f32,
    /// A color glyph (an emoji bitmap): straight-alpha sRGB RGBA, drawn
    /// with its own color. `false` is a coverage mask tinted by the text.
    pub(crate) color: bool,
}

// ---------------------------------------------------------------------------
// Shaping cache — keyed by TextNodeKey, stores a shaped Buffer
// ---------------------------------------------------------------------------

use cosmic_text::Buffer;

pub(crate) struct ShapedEntry {
    pub(crate) buffer: Buffer,
}
