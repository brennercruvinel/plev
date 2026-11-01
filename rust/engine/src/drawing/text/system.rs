#[cfg(not(target_arch = "wasm32"))]
use cosmic_text::CacheKey;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight};
use etagere::{BucketedAtlasAllocator, size2};
use lru::LruCache;
use rustc_hash::{FxHashMap, FxHashSet};
use std::num::NonZeroUsize;

use crate::drawing::compositor::{SceneNode, TextNodeKey};

use super::atlas::{
    self, create_atlas_bind_group, create_atlas_sampler, create_atlas_texture, emit_glyphs,
};
use super::cache::{GlyphCacheKey, GlyphEntry, ShapedEntry};
use super::vertex::TextVertex;

/// One glyph of a shaped run as the frame places it (see
/// [`TextSystem::shaped_glyphs`]): which face, glyph and size (`key`, at
/// the glyph's own size, no subpixel bin), where its origin sits on the
/// baseline in logical pixels relative to the text node's `(x, y)`, and
/// how far the pen advances after it.
/// Native only, like its one reader, the headless svg export.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShapedGlyph {
    pub(crate) key: CacheKey,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) advance: f32,
    /// The laid-out line the glyph sits on; wrapped text has several.
    pub(crate) line: usize,
}

// ---------------------------------------------------------------------------
// TextSystem
// ---------------------------------------------------------------------------

pub struct TextSystem {
    pub font_system: FontSystem,
    // Glyph atlas
    pub(super) allocator: BucketedAtlasAllocator,
    pub(super) glyph_cache: LruCache<GlyphCacheKey, GlyphEntry>,
    pub(super) glyphs_in_use: FxHashSet<GlyphCacheKey>,
    pub(super) atlas_texture: wgpu::Texture,
    pub(super) atlas_view: wgpu::TextureView,
    pub(super) atlas_sampler: wgpu::Sampler,
    pub atlas_bind_group: wgpu::BindGroup,
    pub(super) atlas_size: u32,
    // Shaping cache
    pub(super) shaping_cache: FxHashMap<TextNodeKey, ShapedEntry>,
    // Keys referenced this frame (for eviction)
    pub(super) keys_this_frame: FxHashSet<TextNodeKey>,
    // Staging buffers used during resolve_for_layer
    pub(super) staging_vertices: Vec<TextVertex>,
    pub(super) staging_indices: Vec<u32>,
    /// Physical pixels per logical pixel. Glyph bitmaps are rasterized at
    /// `font_size * raster_scale` so text stays crisp on HiDPI surfaces;
    /// quad geometry stays in logical coordinates (the projection matrix
    /// scales it back up 1:1). Set via [`Self::set_raster_scale`].
    pub(super) raster_scale: f32,
    /// Faces registered by [`super::fonts::register_embedded_fonts`]. Any
    /// glyph rasterized from a face outside this set came from a system
    /// fallback — a silent font leak we want to hear about.
    pub(super) embedded_fonts: FxHashSet<cosmic_text::fontdb::ID>,
    /// font_ids already warned about (warn once per face).
    pub(super) warned_fallback_fonts: FxHashSet<cosmic_text::fontdb::ID>,
    /// Set when an atlas slot was reused for a different glyph (eviction, a
    /// raster-scale change, a purge). Any layer that was skipped this frame
    /// is still pointing at the old slot, so the caller has to re-resolve
    /// everything. Read and cleared with [`Self::take_atlas_disturbed`].
    pub(super) atlas_disturbed: bool,
    /// The late faces this font system holds (`super::on_demand`) and the
    /// generation they are of.
    late: Vec<cosmic_text::fontdb::ID>,
    late_generation: u64,
}

pub(super) const INITIAL_ATLAS_SIZE: u32 = 512;
pub(super) const MAX_ATLAS_SIZE: u32 = 4096;
/// Sized to exceed the glyph count a full atlas can hold: entries are per
/// (face, glyph, size, weight, subpixel bin) and cost ~64 bytes each.
const GLYPH_CACHE_CAPACITY: usize = 32768;

impl TextSystem {
    pub fn new(device: &wgpu::Device, text_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        // The same constructor as `TextMeasurer`, so rasterization matches
        // measurement; the text probe can ask for the embedded faces alone.
        let (font_system, embedded_fonts) =
            super::fonts::font_system(super::fonts::text_system_faces());

        let allocator = BucketedAtlasAllocator::new(size2(
            INITIAL_ATLAS_SIZE as i32,
            INITIAL_ATLAS_SIZE as i32,
        ));
        let glyph_cache = LruCache::new(NonZeroUsize::new(GLYPH_CACHE_CAPACITY).unwrap());

        let (atlas_texture, atlas_view) = create_atlas_texture(device, INITIAL_ATLAS_SIZE);
        let atlas_sampler = create_atlas_sampler(device);
        let atlas_bind_group =
            create_atlas_bind_group(device, text_bind_group_layout, &atlas_view, &atlas_sampler);

        Self {
            font_system,

            allocator,
            glyph_cache,
            glyphs_in_use: FxHashSet::default(),
            atlas_texture,
            atlas_view,
            atlas_sampler,
            atlas_bind_group,
            atlas_size: INITIAL_ATLAS_SIZE,
            shaping_cache: FxHashMap::default(),
            keys_this_frame: FxHashSet::default(),
            staging_vertices: Vec::new(),
            staging_indices: Vec::new(),
            raster_scale: 1.0,
            embedded_fonts: embedded_fonts.into_iter().collect(),
            warned_fallback_fonts: FxHashSet::default(),
            atlas_disturbed: false,
            late: Vec::new(),
            late_generation: 0,
        }
    }

    /// Loads the late faces again when more arrived (`super::on_demand`),
    /// and drops every run shaped and every glyph rasterized without them.
    /// `true` when they changed: every layer's text has to be drawn again,
    /// the layers whose scene did not change included, since their pixels
    /// hold the text as it was shaped before.
    pub(crate) fn sync_late_faces(&mut self) -> bool {
        let generation = super::on_demand::generation();
        if generation == self.late_generation {
            return false;
        }
        for id in &self.late {
            self.embedded_fonts.remove(id);
        }
        self.late = super::on_demand::load(self.font_system.db_mut(), &self.late);
        self.embedded_fonts.extend(self.late.iter().copied());
        self.late_generation = generation;
        self.shaping_cache.clear();
        self.glyph_cache.clear();
        self.allocator =
            BucketedAtlasAllocator::new(size2(self.atlas_size as i32, self.atlas_size as i32));
        self.atlas_disturbed = true;
        true
    }

    /// Physical pixels per logical pixel (HiDPI raster scale). Glyph bitmaps
    /// are rasterized at `font_size * scale`; when the scale changes the
    /// cached bitmaps are invalid and the glyph cache is reset (shaping is
    /// scale-independent and survives).
    ///
    /// Returns `true` when the scale actually changed. Callers must then
    /// re-resolve every layer's text, dirty or not: resetting the cache and
    /// the allocator repacks the atlas, so vertices retained by unchanged
    /// layers point at slots that will hold different glyphs.
    #[must_use = "a scale change invalidates every layer's text vertices"]
    pub fn set_raster_scale(&mut self, scale: f32) -> bool {
        let scale = scale.max(0.01);
        if (self.raster_scale - scale).abs() <= 1e-6 {
            return false;
        }
        log::info!("Text raster scale: {} -> {scale}", self.raster_scale);
        self.raster_scale = scale;
        self.glyph_cache.clear();
        self.allocator =
            BucketedAtlasAllocator::new(size2(self.atlas_size as i32, self.atlas_size as i32));
        self.atlas_disturbed = true;
        true
    }

    /// A new, empty atlas on `device` after the old device was lost: the
    /// same size (no growth churn on the next frame), a new sampler and
    /// bind group. Every cached glyph pointed into the dead texture, so
    /// the cache and the allocator reset and `atlas_disturbed` makes every
    /// layer re-resolve; the glyphs come back on the next frame through
    /// the raster path (a cache miss), keys, gutter and all. The shaping
    /// cache and the raster scale are CPU state and stay.
    pub fn recreate_atlas(
        &mut self,
        device: &wgpu::Device,
        text_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        let (texture, view) = create_atlas_texture(device, self.atlas_size);
        self.atlas_texture = texture;
        self.atlas_view = view;
        self.atlas_sampler = create_atlas_sampler(device);
        self.atlas_bind_group = create_atlas_bind_group(
            device,
            text_bind_group_layout,
            &self.atlas_view,
            &self.atlas_sampler,
        );
        self.glyph_cache.clear();
        self.allocator =
            BucketedAtlasAllocator::new(size2(self.atlas_size as i32, self.atlas_size as i32));
        self.atlas_disturbed = true;
        log::info!(
            "glyph atlas recreated at {}x{} after a lost device",
            self.atlas_size,
            self.atlas_size
        );
    }

    /// Whether the atlas was repacked since the last check, clearing the
    /// flag. When it returns `true` every layer's text vertices are stale,
    /// including layers whose scene did not change.
    pub fn take_atlas_disturbed(&mut self) -> bool {
        std::mem::take(&mut self.atlas_disturbed)
    }

    /// Call at the start of each frame before resolve_for_layer calls.
    pub fn begin_frame(&mut self) {
        self.glyphs_in_use.clear();
        self.keys_this_frame.clear();
    }

    /// Resolve text nodes for a specific layer. Returns (vertices, indices) to be
    /// stored in the layer's text buffers.
    pub fn resolve_for_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        text_bind_group_layout: &wgpu::BindGroupLayout,
        text_nodes: &[SceneNode],
    ) -> (Vec<TextVertex>, Vec<u32>) {
        self.staging_vertices.clear();
        self.staging_indices.clear();

        if text_nodes.is_empty() {
            return (Vec::new(), Vec::new());
        }

        // Phase 1: ensure all text nodes are shaped (populates shaping_cache)
        for node in text_nodes.iter() {
            let SceneNode::Text { key, .. } = node else {
                continue;
            };
            self.keys_this_frame.insert(key.clone());
            self.ensure_shaped(key);
        }

        // Phase 2: emit glyph vertices (needs &mut self for glyph atlas, but
        // we temporarily take buffers out of the shaping cache to avoid borrow conflict)
        for node in text_nodes.iter() {
            let SceneNode::Text { key, x, y, color } = node else {
                continue;
            };
            // Temporarily remove the shaped entry to split the borrow
            let Some(shaped) = self.shaping_cache.remove(key) else {
                log::warn!(
                    "Shaping cache missing entry for text node {:?}; skipping",
                    key.text.get(..40.min(key.text.len()))
                );
                continue;
            };
            emit_glyphs(
                self,
                &atlas::GlyphGpuResources {
                    device,
                    queue,
                    text_bind_group_layout,
                },
                &shaped.buffer,
                *x,
                *y,
                *color,
            );
            self.shaping_cache.insert(key.clone(), shaped);
        }

        let vertices = std::mem::take(&mut self.staging_vertices);
        let indices = std::mem::take(&mut self.staging_indices);
        (vertices, indices)
    }

    /// Shape `key` into the cache when it is not there yet. The one shaping
    /// path: the frame (`resolve_for_layer`) and the headless svg export
    /// (`shaped_glyphs`) both come here, so a run is shaped once and every
    /// consumer sees the same glyphs.
    fn ensure_shaped(&mut self, key: &TextNodeKey) {
        if self.shaping_cache.contains_key(key) {
            return;
        }
        super::on_demand::want(&key.text);
        let font_size = f32::from_bits(key.font_size_bits);
        let line_height = f32::from_bits(key.line_height_bits);
        let letter_spacing = f32::from_bits(key.letter_spacing_bits);
        let max_width = key.max_width_bits.map(f32::from_bits);

        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(font_size, line_height));
        buffer.set_wrap(&mut self.font_system, key.overflow_wrap.cosmic(&key.text));
        buffer.set_size(&mut self.font_system, max_width, None);
        let weight = super::fonts::matched_weight(
            self.font_system.db(),
            key.font_family.as_deref(),
            key.font_weight,
        );
        let mut attrs = Attrs::new().weight(Weight(weight));
        if letter_spacing != 0.0 && font_size > 0.0 {
            // cosmic-text tracking is in EM; the key stores px.
            attrs = attrs.letter_spacing(letter_spacing / font_size);
        }
        if let Some(ref family) = key.font_family {
            attrs = attrs.family(Family::Name(family));
        }
        buffer.set_text(
            &mut self.font_system,
            &key.text,
            &attrs,
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        self.shaping_cache
            .insert(key.clone(), ShapedEntry { buffer });
        log::debug!(
            "Shaping cache miss: {:?}",
            key.text.get(..40.min(key.text.len()))
        );
    }

    /// The glyphs of `key` as the frame places them, in logical pixels
    /// relative to the node's `(x, y)`: the same shaping `resolve_for_layer`
    /// draws from, so a run drawn this frame comes back exactly as drawn,
    /// and a run not drawn yet comes back as it will be. The headless svg
    /// export asks here for the glyphs whose outlines it writes.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn shaped_glyphs(&mut self, key: &TextNodeKey) -> Vec<ShapedGlyph> {
        self.ensure_shaped(key);
        let Some(shaped) = self.shaping_cache.get(key) else {
            return Vec::new();
        };
        let mut glyphs = Vec::new();
        for (line, run) in shaped.buffer.layout_runs().enumerate() {
            for glyph in run.glyphs.iter() {
                // The origin the raster path derives in `atlas::emit_glyphs`
                // through `LayoutGlyph::physical`, before it snaps to the
                // physical pixel grid: the offsets are in em.
                let x = glyph.x + glyph.font_size * glyph.x_offset;
                let y = run.line_y + glyph.y - glyph.font_size * glyph.y_offset;
                let (key, _, _) = CacheKey::new(
                    glyph.font_id,
                    glyph.glyph_id,
                    glyph.font_size,
                    (0.0, 0.0),
                    glyph.font_weight,
                    glyph.cache_key_flags,
                );
                glyphs.push(ShapedGlyph {
                    key,
                    x,
                    y,
                    advance: glyph.w,
                    line,
                });
            }
        }
        glyphs
    }

    /// Resident bytes of the glyph atlas texture (R8Unorm, 1 byte per
    /// pixel). Feeds the perf monitor's memory stats.
    pub fn atlas_memory_bytes(&self) -> u64 {
        u64::from(self.atlas_size) * u64::from(self.atlas_size)
    }

    /// Purge all caches in response to memory pressure.
    pub fn purge_caches(&mut self) {
        let count = self.shaping_cache.len();
        self.shaping_cache.clear();
        self.glyph_cache.clear();
        self.allocator =
            BucketedAtlasAllocator::new(size2(self.atlas_size as i32, self.atlas_size as i32));
        self.atlas_disturbed = true;
        log::info!("Purged caches: {count} shaping entries, glyph atlas reset");
    }

    /// Call after all resolve_for_layer calls. Evicts unused shaping entries.
    pub fn finish_frame(&mut self) {
        // Evict shaping entries not used this frame
        if !self.keys_this_frame.is_empty() {
            self.shaping_cache
                .retain(|k, _| self.keys_this_frame.contains(k));
        }
    }
}
