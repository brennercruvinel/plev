use cosmic_text::SwashContent;
use etagere::size2;

use super::cache::{GlyphCacheKey, GlyphEntry};
use super::system::{MAX_ATLAS_SIZE, TextSystem};
use super::vertex::TextVertex;

use cosmic_text::Buffer;
use etagere::AllocId;

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
    for run in buffer.layout_runs() {
        let line_y = y + run.line_y;
        for glyph in run.glyphs.iter() {
            let physical = glyph.physical((x, line_y), 1.0);
            let cache_key = GlyphCacheKey::from_cosmic(&physical.cache_key);
            sys.glyphs_in_use.insert(cache_key);

            let entry = if let Some(entry) = sys.glyph_cache.get(&cache_key) {
                entry.clone()
            } else {
                match rasterize_and_upload(
                    sys,
                    gpu.device,
                    gpu.queue,
                    gpu.text_bind_group_layout,
                    &physical.cache_key,
                    cache_key,
                ) {
                    Some(entry) => entry,
                    None => continue,
                }
            };

            if entry.width == 0 || entry.height == 0 {
                continue;
            }

            let gx = physical.x as f32 + entry.left;
            let gy = physical.y as f32 - entry.top;
            let gw = entry.width as f32;
            let gh = entry.height as f32;

            let atlas = sys.atlas_size as f32;
            let u0 = entry.atlas_x as f32 / atlas;
            let v0 = entry.atlas_y as f32 / atlas;
            let u1 = (entry.atlas_x + entry.width) as f32 / atlas;
            let v1 = (entry.atlas_y + entry.height) as f32 / atlas;

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

pub(super) fn rasterize_and_upload(
    sys: &mut TextSystem,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    text_bind_group_layout: &wgpu::BindGroupLayout,
    cosmic_key: &cosmic_text::CacheKey,
    cache_key: GlyphCacheKey,
) -> Option<GlyphEntry> {
    let image = sys
        .swash_cache
        .get_image_uncached(&mut sys.font_system, *cosmic_key)?;

    if image.content != SwashContent::Mask {
        return None;
    }

    let gw = image.placement.width;
    let gh = image.placement.height;

    if gw == 0 || gh == 0 {
        let entry = GlyphEntry {
            alloc_id: AllocId::deserialize(0),
            atlas_x: 0,
            atlas_y: 0,
            width: 0,
            height: 0,
            left: image.placement.left as f32,
            top: image.placement.top as f32,
        };
        sys.glyph_cache.put(cache_key, entry.clone());
        return Some(entry);
    }

    let padded_w = gw as i32 + 1;
    let padded_h = gh as i32 + 1;

    let alloc = loop {
        if let Some(alloc) = sys.allocator.allocate(size2(padded_w, padded_h)) {
            break alloc;
        }

        let mut evicted = false;
        while let Some((evict_key, evict_entry)) = sys.glyph_cache.peek_lru() {
            if sys.glyphs_in_use.contains(evict_key) {
                break;
            }
            let evict_alloc_id = evict_entry.alloc_id;
            let evict_key_copy = *evict_key;
            sys.allocator.deallocate(evict_alloc_id);
            sys.glyph_cache.pop(&evict_key_copy);
            evicted = true;

            if sys.allocator.allocate(size2(padded_w, padded_h)).is_some() {
                break;
            }
        }

        if !evicted {
            let new_size = (sys.atlas_size * 2).min(MAX_ATLAS_SIZE);
            if new_size == sys.atlas_size {
                log::error!("Atlas at maximum size and cannot allocate glyph");
                return None;
            }
            grow_atlas(sys, device, queue, text_bind_group_layout, new_size);
        }
    };

    let atlas_x = u32::try_from(alloc.rectangle.min.x).expect("atlas x coordinate negative");
    let atlas_y = u32::try_from(alloc.rectangle.min.y).expect("atlas y coordinate negative");

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &sys.atlas_texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: atlas_x,
                y: atlas_y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &image.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(gw),
            rows_per_image: Some(gh),
        },
        wgpu::Extent3d {
            width: gw,
            height: gh,
            depth_or_array_layers: 1,
        },
    );

    let entry = GlyphEntry {
        alloc_id: alloc.id,
        atlas_x,
        atlas_y,
        width: gw,
        height: gh,
        left: image.placement.left as f32,
        top: image.placement.top as f32,
    };
    sys.glyph_cache.put(cache_key, entry.clone());
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
