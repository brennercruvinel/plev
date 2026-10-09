//! Headless frames: the window's frame path into an offscreen texture.
//!
//! The GPU tests skip (return early) when the machine has no adapter, so
//! the suite still runs on adapterless CI.

use super::*;
use crate::drawing::compositor::{Compositor, LayerId, RoundedRectParams, SceneNode, TextNodeKey};
use crate::styling::theme::hoff::PAGE_BG;

/// Background of the documented pixel sample: the HOFF page tone #303030,
/// which every window measures as 48,48,48.
fn page_bg() -> [f32; 4] {
    PAGE_BG.to_linear_array()
}

fn renderer(viewport: Viewport) -> Option<Renderer> {
    match Renderer::new(viewport, RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

/// A scene with one of each primitive the frame path batches separately:
/// a flat rect, an SDF rounded rect with a border, an analytic shadow, a
/// text run, and a second layer above.
fn scene(c: &mut Compositor, overlay: LayerId) {
    c.begin_frame();
    c.push(SceneNode::Shadow {
        x: 40.0,
        y: 40.0,
        w: 120.0,
        h: 60.0,
        corner_radius: 12.0,
        blur_radius: 16.0,
        offset: [0.0, 4.0],
        color: [0.0, 0.0, 0.0, 0.5],
        inset: false,
    });
    c.draw_rounded_rect(RoundedRectParams {
        x: 40.0,
        y: 40.0,
        w: 120.0,
        h: 60.0,
        color: [0.2, 0.4, 0.9, 1.0],
        corner_radius: 12.0,
        border_width: 1.0,
        border_color: [1.0, 1.0, 1.0, 0.3],
    });
    c.draw_rect(10.0, 10.0, 20.0, 20.0, [1.0, 0.0, 0.0, 1.0]);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(
            "headless",
            &crate::drawing::text::TextStyle::new(16.0).with_line_height(22.4),
            None,
        ),
        x: 50.0,
        y: 60.0,
        color: [1.0, 1.0, 1.0, 1.0],
    });
    c.push_to_layer(
        overlay,
        SceneNode::Rect {
            x: 180.0,
            y: 10.0,
            w: 30.0,
            h: 30.0,
            color: [0.0, 1.0, 0.0, 0.5],
        },
    );
}

fn render_scene(viewport: Viewport) -> Option<Frame> {
    let mut r = renderer(viewport)?;
    let mut c = Compositor::new();
    let overlay = c.create_layer(10);
    scene(&mut c, overlay);
    match r.render(&mut c, page_bg()) {
        Ok(frame) => Some(frame),
        Err(e) => panic!("headless render: {e}"),
    }
}

#[test]
fn physical_size_is_logical_times_scale_rounded_never_zero() {
    assert_eq!(Viewport::new(390.0, 844.0, 3.0).physical(), (1170, 2532));
    assert_eq!(Viewport::new(100.0, 50.0, 1.25).physical(), (125, 63));
    assert_eq!(Viewport::new(0.2, 0.2, 1.0).physical(), (1, 1));
}

#[test]
fn a_degenerate_viewport_is_an_error_before_any_gpu_work() {
    for v in [
        Viewport::new(0.0, 100.0, 1.0),
        Viewport::new(100.0, -1.0, 1.0),
        Viewport::new(100.0, 100.0, 0.0),
        Viewport::new(f32::NAN, 100.0, 1.0),
        Viewport::new(100.0, f32::INFINITY, 1.0),
    ] {
        match Renderer::new(v, RenderConfig::default()) {
            Err(HeadlessError::InvalidViewport(got)) => {
                assert_eq!(got.width.to_bits(), v.width.to_bits())
            }
            Err(e) => panic!("{v:?}: expected InvalidViewport, got {e}"),
            Ok(_) => panic!("{v:?}: a degenerate viewport must not render"),
        }
    }
}

#[test]
fn a_target_past_the_device_limit_is_an_error_not_a_panic() {
    let Some(mut r) = renderer(Viewport::new(64.0, 64.0, 1.0)) else {
        return;
    };
    let huge = Viewport::new(1_000_000.0, 64.0, 1.0);
    match r.resize(huge) {
        Err(HeadlessError::TooLarge { width, max, .. }) => assert!(width > max),
        other => panic!("expected TooLarge, got {:?}", other.map(|_| ())),
    }
    // the renderer is still usable at its old size
    assert_eq!(r.viewport(), Viewport::new(64.0, 64.0, 1.0));
    let mut c = Compositor::new();
    c.begin_frame();
    assert!(r.render(&mut c, page_bg()).is_ok());
}

/// The documented sample: the dark page background measures 48,48,48,
/// the value a window capture converted to sRGB measures. The clear is
/// linear and the sRGB target encodes it back on write.
#[test]
fn the_page_background_measures_48() {
    let Some(mut r) = renderer(Viewport::new(64.0, 48.0, 2.0)) else {
        return;
    };
    let mut c = Compositor::new();
    c.begin_frame();
    let frame = match r.render(&mut c, page_bg()) {
        Ok(f) => f,
        Err(e) => panic!("{e}"),
    };
    assert_eq!((frame.width, frame.height), (128, 96));
    assert_eq!(frame.rgba.len(), 128 * 96 * 4);
    for (i, px) in frame.rgba.chunks_exact(4).enumerate() {
        assert_eq!(px, [48, 48, 48, 255], "pixel {i}");
    }
}

/// Logical coordinates land at logical * scale physical pixels: the red
/// 20x20 rect at (10, 10) covers physical (20..60, 20..60) at scale 2.
#[test]
fn a_rect_lands_at_its_logical_position_times_scale() {
    let Some(frame) = render_scene(Viewport::new(240.0, 140.0, 2.0)) else {
        return;
    };
    assert_eq!(frame.pixel(21, 21), Some([255, 0, 0, 255]));
    assert_eq!(frame.pixel(58, 58), Some([255, 0, 0, 255]));
    assert_eq!(frame.pixel(19, 30), Some([48, 48, 48, 255]));
    assert_eq!(frame.pixel(61, 30), Some([48, 48, 48, 255]));
    // the half-transparent green on the upper layer blends over the page
    let [r, g, b, a] = frame.pixel(390, 50).unwrap_or_default();
    assert!(
        g > 150 && r < g && b < g && a == 255,
        "blend {r},{g},{b},{a}"
    );
    assert_eq!(frame.pixel(480, 0), None);
}

/// The acceptance criterion for CI and agents: the same frame twice gives
/// the same bytes, pixels and PNG.
#[test]
fn two_renders_of_one_scene_are_byte_identical() {
    let v = Viewport::new(240.0, 140.0, 2.0);
    let Some(a) = render_scene(v) else {
        return;
    };
    let Some(b) = render_scene(v) else {
        return;
    };
    assert!(a.rgba.iter().any(|&x| x != 48), "the scene drew nothing");
    assert_eq!(a.rgba, b.rgba);
    let (pa, pb) = match (a.encode_png(), b.encode_png()) {
        (Ok(pa), Ok(pb)) => (pa, pb),
        (Err(e), _) | (_, Err(e)) => panic!("{e}"),
    };
    assert_eq!(pa, pb);
}

/// A renderer kept across frames (the running-app model) draws a clean
/// frame again identically: retained layers are not redrawn, and what the
/// target shows does not drift.
#[test]
fn a_second_frame_of_an_unchanged_scene_is_identical_to_the_first() {
    let Some(mut r) = renderer(Viewport::new(240.0, 140.0, 2.0)) else {
        return;
    };
    let mut c = Compositor::new();
    let overlay = c.create_layer(10);
    scene(&mut c, overlay);
    let first = r.render(&mut c, page_bg()).map_err(|e| e.to_string());
    scene(&mut c, overlay);
    let second = r.render(&mut c, page_bg()).map_err(|e| e.to_string());
    assert_eq!(first, second);
}

/// A resize keeps the renderer usable and changes the frame size.
#[test]
fn resize_changes_the_frame_size() {
    let Some(mut r) = renderer(Viewport::new(100.0, 80.0, 1.0)) else {
        return;
    };
    if let Err(e) = r.resize(Viewport::new(50.0, 40.0, 3.0)) {
        panic!("{e}");
    }
    let mut c = Compositor::new();
    c.begin_frame();
    match r.render(&mut c, page_bg()) {
        Ok(f) => assert_eq!((f.width, f.height), (150, 120)),
        Err(e) => panic!("{e}"),
    }
}

/// The PNG holds exactly the frame's pixels.
#[test]
fn png_round_trips_the_pixels() {
    let frame = Frame {
        width: 3,
        height: 2,
        rgba: (0u8..24).collect(),
    };
    let bytes = match frame.encode_png() {
        Ok(b) => b,
        Err(e) => panic!("{e}"),
    };
    let decoded = match image::load_from_memory_with_format(&bytes, image::ImageFormat::Png) {
        Ok(img) => img.to_rgba8(),
        Err(e) => panic!("{e}"),
    };
    assert_eq!(decoded.dimensions(), (3, 2));
    assert_eq!(decoded.into_raw(), frame.rgba);
}

#[test]
fn a_frame_with_the_wrong_byte_count_does_not_encode() {
    let frame = Frame {
        width: 4,
        height: 4,
        rgba: vec![0; 10],
    };
    assert!(matches!(frame.encode_png(), Err(HeadlessError::Png(_))));
    assert_eq!(frame.pixel(3, 3), None);
}

/// A window's surface is BGRA on macOS; the readback returns RGBA for it
/// too, so a window capture and a headless frame compare byte for byte.
#[test]
fn a_bgra_texture_reads_back_as_rgba() {
    let Some(r) = renderer(Viewport::new(8.0, 8.0, 1.0)) else {
        return;
    };
    let gpu = &r.gpu;
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("bgra"),
        size: wgpu::Extent3d {
            width: 8,
            height: 3,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    // texel memory order is B, G, R, A
    let texels: Vec<u8> = (0..24).flat_map(|_| [10u8, 20, 30, 40]).collect();
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(8 * 4),
            rows_per_image: Some(3),
        },
        wgpu::Extent3d {
            width: 8,
            height: 3,
            depth_or_array_layers: 1,
        },
    );
    let frame = match read_texture(gpu, &texture) {
        Ok(f) => f,
        Err(e) => panic!("{e}"),
    };
    assert_eq!((frame.width, frame.height), (8, 3));
    assert!(frame.rgba.chunks_exact(4).all(|p| p == [30, 20, 10, 40]));
}

#[test]
fn a_format_the_readback_cannot_convert_is_an_error() {
    let Some(r) = renderer(Viewport::new(8.0, 8.0, 1.0)) else {
        return;
    };
    let texture = r.gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("r8"),
        size: wgpu::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    assert!(matches!(
        read_texture(&r.gpu, &texture),
        Err(HeadlessError::UnsupportedFormat(
            wgpu::TextureFormat::R8Unorm
        ))
    ));
}
