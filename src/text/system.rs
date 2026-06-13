use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Weight};
use etagere::{BucketedAtlasAllocator, size2};
use lru::LruCache;
use rustc_hash::{FxHashMap, FxHashSet};
use std::num::NonZeroUsize;

use crate::compositor::{SceneNode, TextNodeKey};

use super::atlas::{self, create_atlas_texture, emit_glyphs};
use super::cache::{GlyphCacheKey, GlyphEntry, ShapedEntry};
use super::vertex::TextVertex;

// ---------------------------------------------------------------------------
// TextSystem
// ---------------------------------------------------------------------------

pub struct TextSystem {
    pub font_system: FontSystem,
    pub(super) swash_cache: SwashCache,
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
}

pub(super) const INITIAL_ATLAS_SIZE: u32 = 512;
pub(super) const MAX_ATLAS_SIZE: u32 = 4096;
const GLYPH_CACHE_CAPACITY: usize = 4096;

impl TextSystem {
    pub fn new(device: &wgpu::Device, text_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        // System fonts (desktop) are fallback only; the embedded faces below
        // are the source of truth so rasterization matches `TextMeasurer`.
        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        let mut font_system = FontSystem::new();

        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
        let mut font_system = FontSystem::new_with_locale_and_db(
            "en-US".to_string(),
            cosmic_text::fontdb::Database::new(),
        );

        super::fonts::register_embedded_fonts(font_system.db_mut());

        let swash_cache = SwashCache::new();
        let allocator = BucketedAtlasAllocator::new(size2(
            INITIAL_ATLAS_SIZE as i32,
            INITIAL_ATLAS_SIZE as i32,
        ));
        let glyph_cache = LruCache::new(NonZeroUsize::new(GLYPH_CACHE_CAPACITY).unwrap());

        let (atlas_texture, atlas_view) = create_atlas_texture(device, INITIAL_ATLAS_SIZE);

        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atlas_bg"),
            layout: text_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        Self {
            font_system,
            swash_cache,
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
        }
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

            if !self.shaping_cache.contains_key(key) {
                let font_size = f32::from_bits(key.font_size_bits);
                let line_height = f32::from_bits(key.line_height_bits);
                let letter_spacing = f32::from_bits(key.letter_spacing_bits);
                let max_width = key.max_width_bits.map(f32::from_bits);

                let mut buffer =
                    Buffer::new(&mut self.font_system, Metrics::new(font_size, line_height));
                buffer.set_size(&mut self.font_system, max_width, None);
                let mut attrs = Attrs::new().weight(Weight(key.font_weight));
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
                    &key.text.get(..40.min(key.text.len()))
                );
            }
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
                    &key.text.get(..40.min(key.text.len()))
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
