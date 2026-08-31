use cosmic_text::SwashContent;
use etagere::size2;

use super::cache::{GlyphCacheKey, GlyphEntry};
use super::system::{MAX_ATLAS_SIZE, TextSystem};
use super::vertex::TextVertex;

use cosmic_text::Buffer;

/// Empty texels reserved around every glyph in the atlas. One texel on each
/// side is what a bilinear tap can reach past the UV rect.
pub(super) const GLYPH_PADDING: u32 = 1;

pub(super) fn create_atlas_texture(
    device: &wgpu::Device,
    size: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glyph_atlas"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// GPU resources needed during glyph emission.
pub(super) struct GlyphGpuResources<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub text_bind_group_layout: &'a wgpu::BindGroupLayout,
}

pub(super) fn emit_glyphs(
    sys: &mut TextSystem,
    gpu: &GlyphGpuResources<'_>,
    buffer: &Buffer,
    x: f32,
    y: f32,
    color: [f32; 4],
) {
    // Glyphs are rasterized at physical resolution (`raster_scale`); quad
    // geometry is mapped back to logical coordinates, which the projection
    // matrix scales 1:1 onto the surface. Rasterizing at scale 1.0 here
    // would stretch small bitmaps over HiDPI pixels — visibly blurry text.
    let scale = sys.raster_scale;
    for run in buffer.layout_runs() {
        let line_y = y + run.line_y;
        for glyph in run.glyphs.iter() {
            // Silent font leak detector: a glyph shaped into a face outside
            // the embedded set came from the system fallback chain — on
            // screen that reads as "some text is a different font". Warn
            // once per face, with the offending cluster for debugging.
            if !sys.embedded_fonts.is_empty()
                && !sys.embedded_fonts.contains(&glyph.font_id)
                && sys.warned_fallback_fonts.insert(glyph.font_id)
            {
                let family = sys
                    .font_system
                    .db()
                    .face(glyph.font_id)
                    .and_then(|f| f.families.first().map(|(name, _)| name.clone()))
                    .unwrap_or_else(|| "<unknown>".to_string());
                let cluster = run.text.get(glyph.start..glyph.end).unwrap_or("?");
                log::warn!(
                    "text fallback: cluster {cluster:?} rasterized from non-embedded \
                     face '{family}' — check family/weight/glyph coverage"
                );
            }
            let physical = glyph.physical((x * scale, line_y * scale), scale);
            let cache_key = physical.cache_key;
            sys.glyphs_in_use.insert(cache_key);

            let entry = if let Some(entry) = sys.glyph_cache.get(&cache_key) {
                entry.clone()
            } else {
                match rasterize_and_upload(
                    sys,
                    gpu.device,
                    gpu.queue,
                    gpu.text_bind_group_layout,
                    cache_key,
                ) {
                    Some(entry) => entry,
                    None => continue,
                }
            };

            if entry.width == 0 || entry.height == 0 {
                continue;
            }

            let gx = (physical.x as f32 + entry.left) / scale;
            let gy = (physical.y as f32 - entry.top) / scale;
            let gw = entry.width as f32 / scale;
            let gh = entry.height as f32 / scale;

            let [u0, v0, u1, v1] = glyph_uv_rect(&entry);

            let base = sys.staging_vertices.len() as u32;
            sys.staging_vertices.extend_from_slice(&[
                TextVertex {
                    position: [gx, gy],
                    uv: [u0, v0],
                    color,
                },
                TextVertex {
                    position: [gx + gw, gy],
                    uv: [u1, v0],
                    color,
                },
                TextVertex {
                    position: [gx + gw, gy + gh],
                    uv: [u1, v1],
                    color,
                },
                TextVertex {
                    position: [gx, gy + gh],
                    uv: [u0, v1],
                    color,
                },
            ]);
            sys.staging_indices.extend_from_slice(&[
                base,
                base + 1,
                base + 2,
                base + 2,
                base + 3,
                base,
            ]);
        }
    }
}

/// Atlas rect of a glyph as `[u0, v0, u1, v1]` in **texels**.
///
/// Deliberately not normalized: the shader divides by the size of the atlas
/// bound at draw time (see `text.wgsl`), so quads emitted before a mid-frame
/// atlas grow stay valid.
pub(super) fn glyph_uv_rect(entry: &GlyphEntry) -> [f32; 4] {
    [
        entry.atlas_x as f32,
        entry.atlas_y as f32,
        (entry.atlas_x + entry.width) as f32,
        (entry.atlas_y + entry.height) as f32,
    ]
}

pub(super) fn rasterize_and_upload(
    sys: &mut TextSystem,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    text_bind_group_layout: &wgpu::BindGroupLayout,
    cache_key: GlyphCacheKey,
) -> Option<GlyphEntry> {
    // Fresh swash context per rasterization: the shared `ScaleContext`
    // keeps per-font state across calls, and interleaving sizes of one face
    // can make a CacheKey rasterize at a stale size. Rasterization only
    // happens on an atlas cache miss (once per glyph), so this costs
    // nothing per frame.
    let mut swash = cosmic_text::SwashCache::new();
    let image = swash.get_image_uncached(&mut sys.font_system, cache_key)?;

    if image.content != SwashContent::Mask {
        return None;
    }

    let gw = image.placement.width;
    let gh = image.placement.height;

    if gw == 0 || gh == 0 {
        let entry = GlyphEntry {
            alloc_id: None,
            atlas_x: 0,
            atlas_y: 0,
            width: 0,
            height: 0,
            left: image.placement.left as f32,
            top: image.placement.top as f32,
        };
        cache_insert(sys, cache_key, entry.clone());
        return Some(entry);
    }

    // Transparent border on every side: the atlas samples with
    // `FilterMode::Linear`, and a bilinear tap can reach one texel past the
    // UV rect — the gutter makes it bleed transparency, not a neighbour.
    let padded_w = gw as i32 + GLYPH_PADDING as i32 * 2;
    let padded_h = gh as i32 + GLYPH_PADDING as i32 * 2;

    // Grow before evicting: eviction is only safe for glyphs nothing still
    // draws, and `glyphs_in_use` only covers layers resolved this frame — a
    // skipped layer's retained vertices still reference their slots. The
    // atlas grows to MAX_ATLAS_SIZE first; when it must evict anyway, the
    // disturbance is recorded so every layer re-resolves next frame.
    let alloc = loop {
        if let Some(alloc) = sys.allocator.allocate(size2(padded_w, padded_h)) {
            break alloc;
        }

        if sys.atlas_size < MAX_ATLAS_SIZE {
            let new_size = (sys.atlas_size * 2).min(MAX_ATLAS_SIZE);
            grow_atlas(sys, device, queue, text_bind_group_layout, new_size);
            continue;
        }

        let mut freed = None;
        let mut evicted = false;
        while let Some((evict_key, evict_entry)) = sys.glyph_cache.peek_lru() {
            if sys.glyphs_in_use.contains(evict_key) {
                break;
            }
            let evict_alloc_id = evict_entry.alloc_id;
            let evict_key_copy = *evict_key;
            // Only glyphs that actually reserved a rectangle give one back
            // (empty glyphs carry `None`).
            if let Some(id) = evict_alloc_id {
                sys.allocator.deallocate(id);
            }
            sys.glyph_cache.pop(&evict_key_copy);
            sys.atlas_disturbed = true;
            evicted = true;

            // Keep the allocation this eviction made room for — a probe
            // allocation whose result is dropped would leak the rectangle.
            if let Some(alloc) = sys.allocator.allocate(size2(padded_w, padded_h)) {
                freed = Some(alloc);
                break;
            }
        }
        if let Some(alloc) = freed {
            break alloc;
        }

        if !evicted {
            log::warn!(
                "Glyph atlas full at maximum size {}x{}: cannot allocate glyph_id={} \
                 ({}x{} px); glyph will not be rendered",
                sys.atlas_size,
                sys.atlas_size,
                cache_key.glyph_id,
                padded_w,
                padded_h
            );
            return None;
        }
    };

    // The allocation covers glyph + gutter; the bitmap goes at the inset
    // origin so the gutter stays empty on all four sides.
    let slot_x = u32::try_from(alloc.rectangle.min.x).expect("atlas x coordinate negative");
    let slot_y = u32::try_from(alloc.rectangle.min.y).expect("atlas y coordinate negative");
    let atlas_x = slot_x + GLYPH_PADDING;
    let atlas_y = slot_y + GLYPH_PADDING;

    // Upload glyph and gutter as one zeroed block: a reused slot's border
    // must be transparent, not whatever its previous occupant left there.
    let pw = padded_w as u32;
    let ph = padded_h as u32;
    let mut padded = vec![0u8; (pw * ph) as usize];
    for row in 0..gh {
        let src = (row * gw) as usize;
        let dst = ((row + GLYPH_PADDING) * pw + GLYPH_PADDING) as usize;
        padded[dst..dst + gw as usize].copy_from_slice(&image.data[src..src + gw as usize]);
    }

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &sys.atlas_texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: slot_x,
                y: slot_y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &padded,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(pw),
            rows_per_image: Some(ph),
        },
        wgpu::Extent3d {
            width: pw,
            height: ph,
            depth_or_array_layers: 1,
        },
    );

    let entry = GlyphEntry {
        alloc_id: Some(alloc.id),
        atlas_x,
        atlas_y,
        width: gw,
        height: gh,
        left: image.placement.left as f32,
        top: image.placement.top as f32,
    };
    cache_insert(sys, cache_key, entry.clone());
    log::debug!(
        "Atlas alloc: glyph_id={} at ({}, {}), size={}x{}",
        cache_key.glyph_id,
        atlas_x,
        atlas_y,
        gw,
        gh
    );
    Some(entry)
}

/// Insert into the glyph cache, giving back the atlas rectangle of whatever
/// the LRU capacity pushed out — a dropped entry's rectangle must return to
/// the allocator, or it stays reserved with nothing able to free it.
fn cache_insert(sys: &mut TextSystem, key: GlyphCacheKey, entry: GlyphEntry) {
    if let Some((dropped_key, dropped)) = sys.glyph_cache.push(key, entry) {
        // `push` returns the replaced value for the *same* key, or the
        // capacity-evicted LRU pair for a different key. Either way its
        // rectangle is now unreachable and must go back to the allocator.
        if let Some(id) = dropped.alloc_id {
            sys.allocator.deallocate(id);
            sys.atlas_disturbed = true;
        }
        let _ = dropped_key;
    }
}

pub(super) fn grow_atlas(
    sys: &mut TextSystem,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    text_bind_group_layout: &wgpu::BindGroupLayout,
    new_size: u32,
) {
    log::info!(
        "Growing atlas from {}x{} to {}x{}",
        sys.atlas_size,
        sys.atlas_size,
        new_size,
        new_size
    );

    let (new_texture, new_view) = create_atlas_texture(device, new_size);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("atlas_grow_encoder"),
    });
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &sys.atlas_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: &new_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: sys.atlas_size,
            height: sys.atlas_size,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    sys.allocator.grow(size2(new_size as i32, new_size as i32));
    sys.atlas_texture = new_texture;
    sys.atlas_view = new_view;
    sys.atlas_size = new_size;

    sys.atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("atlas_bg"),
        layout: text_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&sys.atlas_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sys.atlas_sampler),
            },
        ],
    });
}
