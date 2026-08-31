// ---------------------------------------------------------------------------
// Glyph atlas internals
// ---------------------------------------------------------------------------

/// Identity of one rasterized glyph bitmap.
///
/// This is cosmic-text's own [`cosmic_text::CacheKey`] verbatim, and it must
/// stay that way: the key has to name *every* input `SwashCache` rasterizes
/// with, or two different bitmaps alias onto one atlas entry.
///
/// It previously kept only `font_id`, `glyph_id`, `font_size_bits` and
/// `flags`, dropping `x_bin`, `y_bin` and `font_weight`. The subpixel bins
/// are the damaging omission: `CacheKey::new` splits a glyph's fractional
/// x into quarter-pixel bins, and swash rasterizes each bin as a *different*
/// bitmap with its own `placement.left`/`top`. Shaping one string routinely
/// hits several bins for the same character ("Expense Tracker" at 20px/500
/// puts its three `e`s in bins Zero, Zero and One), so the first `e` to be
/// rasterized was reused — bitmap *and* placement — for every later phase.
/// Every glyph after the first therefore drew up to 0.75 physical px off its
/// shaped position, with a mask rasterized for the wrong phase.
pub(crate) type GlyphCacheKey = cosmic_text::CacheKey;

#[derive(Clone, Debug)]
pub(crate) struct GlyphEntry {
    /// The atlas rectangle backing this glyph, or `None` for a glyph with no
    /// bitmap at all (a space, and anything else swash rasterizes to a
    /// zero-size mask).
    ///
    /// Empty glyphs reserve nothing, so there is nothing to hand back on
    /// eviction. This used to be a plain `AllocId` with `AllocId::deserialize(0)`
    /// standing in for "none" — but that is a *valid* id (bucket 0,
    /// generation 0), so evicting a space called `deallocate` on another
    /// glyph's live bucket and drove its refcount below zero:
    /// `assertion failed: bucket.refcount > 0` inside etagere.
    pub(crate) alloc_id: Option<etagere::AllocId>,
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
