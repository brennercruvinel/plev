// ---------------------------------------------------------------------------
// Glyph atlas internals
// ---------------------------------------------------------------------------

use cosmic_text::CacheKeyFlags;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct GlyphCacheKey {
    pub(crate) font_id: cosmic_text::fontdb::ID,
    pub(crate) glyph_id: u16,
    pub(crate) font_size_bits: u32,
    pub(crate) flags: CacheKeyFlags,
}

impl GlyphCacheKey {
    pub(crate) fn from_cosmic(key: &cosmic_text::CacheKey) -> Self {
        Self {
            font_id: key.font_id,
            glyph_id: key.glyph_id,
            font_size_bits: key.font_size_bits,
            flags: key.flags,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct GlyphEntry {
    pub(crate) alloc_id: etagere::AllocId,
    pub(crate) atlas_x: u32,
    pub(crate) atlas_y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) left: f32,
    pub(crate) top: f32,
}

// ---------------------------------------------------------------------------
// Shaping cache — keyed by TextNodeKey, stores a shaped Buffer
// ---------------------------------------------------------------------------

use cosmic_text::Buffer;

pub(crate) struct ShapedEntry {
    pub(crate) buffer: Buffer,
}
