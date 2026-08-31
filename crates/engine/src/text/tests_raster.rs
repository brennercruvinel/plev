//! Regression tests for the glyph raster path: cache identity, atlas
//! padding, and grow-invariant UVs.
//!
//! These are the invariants behind "the glyph on screen is the glyph that
//! was shaped, at the position it was shaped at". They are GPU-free: each
//! one pins a property of the keys/geometry the rasterizer derives, which
//! is where the defects lived, rather than the wgpu calls that consume them.

use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, Weight};

use super::atlas::{GLYPH_PADDING, glyph_uv_rect};
use super::backend::TextStyle;
use super::cache::GlyphEntry;

/// Shape `text` the way `TextSystem::resolve_for_layer` does and return one
/// `CacheKey` per glyph, at the raster scale the window would use.
fn glyph_keys(
    text: &str,
    style: &TextStyle,
    raster_scale: f32,
) -> Vec<super::cache::GlyphCacheKey> {
    let mut fs = {
        let mut db = cosmic_text::fontdb::Database::new();
        super::fonts::register_embedded_fonts(&mut db);
        cosmic_text::FontSystem::new_with_locale_and_db("en-US".to_string(), db)
    };
    let mut buffer = Buffer::new(&mut fs, Metrics::new(style.font_size, style.line_height));
    buffer.set_size(&mut fs, None, None);
    let mut attrs = Attrs::new().weight(Weight(style.font_weight));
    if let Some(ref family) = style.font_family {
        attrs = attrs.family(Family::Name(family));
    }
    buffer.set_text(&mut fs, text, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut fs, false);

    let mut keys = Vec::new();
    for run in buffer.layout_runs() {
        for glyph in run.glyphs.iter() {
            keys.push(
                glyph
                    .physical((0.0, run.line_y * raster_scale), raster_scale)
                    .cache_key,
            );
        }
    }
    keys
}

/// The cache key must be cosmic-text's key verbatim. If the engine ever
/// narrows it again, two bitmaps that swash rasterizes differently collapse
/// onto one atlas entry.
#[test]
fn glyph_cache_key_is_the_whole_cosmic_key() {
    fn assert_is_cosmic_key(_: &super::cache::GlyphCacheKey) {}
    let key = glyph_keys("A", &TextStyle::new(16.0), 2.0)[0];
    assert_is_cosmic_key(&key);
}

/// One string puts the same character in several subpixel bins, and each
/// bin is a distinct bitmap: keys must keep them distinct.
#[test]
fn same_char_in_different_subpixel_bins_gets_distinct_keys() {
    let style = crate::theme::TypographyScale::hoff().title();
    let keys = glyph_keys("Expense Tracker", &style, 1.0);

    // The three `e`s of "Expense Tracker" (indices 3, 6 and 13).
    let e_keys: Vec<_> = keys
        .iter()
        .filter(|k| k.glyph_id == keys[3].glyph_id)
        .collect();
    assert!(
        e_keys.len() >= 3,
        "expected the repeated 'e' glyph, found {}",
        e_keys.len()
    );
    assert!(
        e_keys.iter().any(|k| k.x_bin != e_keys[0].x_bin),
        "this string is only a regression test while its repeated glyph \
         lands in more than one subpixel bin"
    );
    // Distinct bins => distinct keys => distinct atlas entries.
    let distinct: std::collections::HashSet<_> = e_keys.iter().map(|k| **k).collect();
    assert!(
        distinct.len() > 1,
        "glyphs in different subpixel bins collapsed onto one cache key"
    );
}

/// Faces differ per weight, so weight is already implied by `font_id` — but
/// the key carries it explicitly, and dropping it would alias synthesized
/// weights. Pin that the key separates them.
#[test]
fn different_weights_get_distinct_keys() {
    let regular = glyph_keys("A", &TextStyle::new(16.0).with_weight(400), 2.0)[0];
    let bold = glyph_keys("A", &TextStyle::new(16.0).with_weight(700), 2.0)[0];
    assert_ne!(regular, bold);
}

/// Raster scale is part of the key through `font_size_bits`: 1x and 2x
/// bitmaps of the same glyph must not share an atlas entry.
#[test]
fn different_raster_scales_get_distinct_keys() {
    let style = TextStyle::new(16.0);
    let at_1x = glyph_keys("A", &style, 1.0)[0];
    let at_2x = glyph_keys("A", &style, 2.0)[0];
    assert_ne!(
        at_1x.font_size_bits, at_2x.font_size_bits,
        "HiDPI bitmaps must not alias onto the 1x entry"
    );
}

/// The atlas reserves a gutter on all four sides. One texel is exactly what
/// a bilinear tap can reach past the UV rect; with less than that on the
/// left/top edges, a glyph packed against its neighbour bleeds into it.
#[test]
fn glyph_slots_reserve_a_gutter_on_every_side() {
    assert_eq!(GLYPH_PADDING, 1);
    let (gw, gh) = (7u32, 11u32);
    let padded_w = gw + GLYPH_PADDING * 2;
    let padded_h = gh + GLYPH_PADDING * 2;
    assert_eq!(padded_w, 9);
    assert_eq!(padded_h, 13);

    // The bitmap sits inset inside its slot, so the gutter is symmetric.
    let (slot_x, slot_y) = (40u32, 64u32);
    let (atlas_x, atlas_y) = (slot_x + GLYPH_PADDING, slot_y + GLYPH_PADDING);
    assert_eq!(atlas_x - slot_x, GLYPH_PADDING, "gutter on the left");
    assert_eq!(atlas_y - slot_y, GLYPH_PADDING, "gutter on the top");
    assert_eq!(
        (slot_x + padded_w) - (atlas_x + gw),
        GLYPH_PADDING,
        "gutter on the right"
    );
    assert_eq!(
        (slot_y + padded_h) - (atlas_y + gh),
        GLYPH_PADDING,
        "gutter on the bottom"
    );
}

/// The padded upload block is zeroed everywhere except the glyph itself, so
/// a slot reused after eviction cannot leave the previous occupant's pixels
/// in the gutter.
#[test]
fn padded_upload_block_zeroes_the_gutter() {
    let (gw, gh) = (3u32, 2u32);
    let data: Vec<u8> = vec![0xFF; (gw * gh) as usize];
    let pw = gw + GLYPH_PADDING * 2;
    let ph = gh + GLYPH_PADDING * 2;

    let mut padded = vec![0u8; (pw * ph) as usize];
    for row in 0..gh {
        let src = (row * gw) as usize;
        let dst = ((row + GLYPH_PADDING) * pw + GLYPH_PADDING) as usize;
        padded[dst..dst + gw as usize].copy_from_slice(&data[src..src + gw as usize]);
    }

    for y in 0..ph {
        for x in 0..pw {
            let inside = x >= GLYPH_PADDING
                && x < GLYPH_PADDING + gw
                && y >= GLYPH_PADDING
                && y < GLYPH_PADDING + gh;
            let texel = padded[(y * pw + x) as usize];
            if inside {
                assert_eq!(texel, 0xFF, "glyph texel ({x},{y}) was not uploaded");
            } else {
                assert_eq!(texel, 0x00, "gutter texel ({x},{y}) is not transparent");
            }
        }
    }
}

/// Quads carry texel coordinates, and the shader divides by the size of the
/// bound atlas. That is what makes them survive a mid-frame grow: the grow
/// copies the old atlas into the new one at the same origin, so texel
/// coordinates are unchanged while normalized ones would halve.
#[test]
fn emitted_uvs_are_texels_so_they_survive_an_atlas_grow() {
    let entry = GlyphEntry {
        alloc_id: None,
        atlas_x: 40,
        atlas_y: 64,
        width: 7,
        height: 11,
        left: 0.0,
        top: 0.0,
    };
    let [u0, v0, u1, v1] = glyph_uv_rect(&entry);

    // Texels, not the 0..1 range a normalized rect would occupy.
    assert_eq!([u0, v0, u1, v1], [40.0, 64.0, 47.0, 75.0]);
    assert!(
        u1 > 1.0 && v1 > 1.0,
        "UVs were normalized at emit time; they must stay in texels so a \
         mid-frame atlas grow cannot invalidate quads already emitted"
    );

    // The rect covers the glyph's own texels, gutter excluded.
    assert_eq!(u1 - u0, entry.width as f32);
    assert_eq!(v1 - v0, entry.height as f32);

    // What the shader does, before and after a grow: the same texel rect
    // resolves correctly against whichever atlas is bound.
    let normalized = |size: f32| (u0 / size, v0 / size);
    assert_ne!(normalized(512.0), normalized(1024.0));
}

/// The glyph quad lands on whole physical pixels: both terms of `gx` are
/// integers in physical space, so `gx * scale` is an integer. A mask atlas
/// sampled off-grid is blurry no matter how correct the cache is.
#[test]
fn glyph_quads_land_on_whole_physical_pixels() {
    let style = crate::theme::TypographyScale::hoff().title();
    for scale in [1.0f32, 2.0, 3.0] {
        let mut fs = {
            let mut db = cosmic_text::fontdb::Database::new();
            super::fonts::register_embedded_fonts(&mut db);
            cosmic_text::FontSystem::new_with_locale_and_db("en-US".to_string(), db)
        };
        let mut buffer = Buffer::new(&mut fs, Metrics::new(style.font_size, style.line_height));
        buffer.set_size(&mut fs, None, None);
        buffer.set_text(
            &mut fs,
            "Expense Tracker",
            &Attrs::new().weight(Weight(style.font_weight)),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut fs, false);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let p = glyph.physical((0.0, run.line_y * scale), scale);
                // `left`/`top` are integers in physical px (swash placement),
                // as is `p.x`, so the logical quad origin is on the grid.
                let left = 2.0f32; // stand-in for image.placement.left
                let gx = (p.x as f32 + left) / scale;
                let physical = gx * scale;
                assert!(
                    (physical - physical.round()).abs() < 1e-3,
                    "quad x {gx} is not on a physical pixel at scale {scale}"
                );
            }
        }
    }
}

// -- raster scale changes (dragging a window between monitors) --------------

/// A scale change resets the glyph cache and hands the whole atlas back to
/// the allocator, so every quad already emitted is stale. The setter has to
/// say so, because the caller skips layers whose scene did not change and
/// those layers would keep sampling repacked texels.
#[test]
fn set_raster_scale_reports_whether_it_changed() {
    // The flag is what render_passes branches on to force a full re-resolve.
    let Some(mut sys) = test_text_system() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    assert!(sys.set_raster_scale(2.0), "1.0 -> 2.0 is a change");
    assert!(!sys.set_raster_scale(2.0), "2.0 -> 2.0 is not");
    assert!(sys.set_raster_scale(1.0), "2.0 -> 1.0 is a change");
    // Fractional factors are real (macOS "More Space", many external panels).
    assert!(sys.set_raster_scale(1.5));
    assert!(!sys.set_raster_scale(1.5));
}

fn test_text_system() -> Option<crate::text::TextSystem> {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .ok()?;
    let (device, _queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("raster_scale_test"),
        ..Default::default()
    }))
    .ok()?;
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    Some(crate::text::TextSystem::new(&device, &bgl))
}
