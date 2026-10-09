//! The desktop against itself (task-77 probe): one scene holding every
//! primitive the frame path draws, rendered by two GPU contexts in one
//! process, gives the same bytes at scale 1, 2 and the fractional 1.25
//! and 1.5; and a renderer resized across those scales, in another
//! order, gives the bytes a fresh one gives. Two processes are the
//! showcase's `regress capture --against` (every gallery page).
//!
//! The primitives sit on fractional logical coordinates on purpose: at a
//! fractional scale their edges fall inside pixels, where the SDF
//! feather, the MSAA resolve and the blend decide the bytes.
//!
//! One test pins the divergence the probe found across scales (a blur's
//! sigma in physical texels): ignored until the engine's fix (task-77
//! phase 1b), it runs with the rest now; `blur_scale.rs` has the others.
//!
//! Needs a GPU adapter; without one every test skips.

use engine::compositor::{
    Compositor, GradientRectParams, LayerEffect, RoundedRectParams, SceneNode, ShadowParams,
    TextNodeKey,
};
use engine::gpu::{RenderConfig, load_image_rgba};
use engine::headless::{Frame, HeadlessError, Renderer, Viewport};
use engine::path::PathBuilder;
use engine::text::TextStyle;
use engine::theme::hoff::PAGE_BG;

const WIDTH: f32 = 240.0;
const HEIGHT: f32 = 160.0;
const SCALES: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

fn renderer(scale: f32) -> Option<Renderer> {
    match Renderer::new(Viewport::new(WIDTH, HEIGHT, scale), RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

/// A 16 x 16 checker, two colors, opaque.
fn checker() -> Vec<u8> {
    let mut px = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16u32 {
        for x in 0..16u32 {
            let on = (x / 4 + y / 4) % 2 == 0;
            px.extend_from_slice(if on {
                &[230, 90, 40, 255]
            } else {
                &[30, 60, 200, 255]
            });
        }
    }
    px
}

/// Every primitive of the frame path on the base layer, a glass layer
/// above it (backdrop blur under a translucent fill, at an opacity) and
/// a layer with a blur effect.
fn scene() -> Compositor {
    let mut c = Compositor::new();
    let glass = c.create_layer(10);
    let blurred = c.create_layer(20);
    c.begin_frame();
    c.draw_rect(4.3, 4.3, 30.4, 18.6, [0.9, 0.2, 0.2, 1.0]);
    c.draw_shadow(ShadowParams {
        x: 40.2,
        y: 10.7,
        w: 70.5,
        h: 40.3,
        corner_radius: 9.5,
        blur_radius: 14.0,
        offset: [1.5, 3.5],
        color: [0.0, 0.0, 0.0, 0.6],
        inset: false,
    });
    c.draw_rounded_rect(RoundedRectParams {
        x: 40.2,
        y: 10.7,
        w: 70.5,
        h: 40.3,
        color: [0.2, 0.45, 0.9, 1.0],
        corner_radius: 9.5,
        border_width: 1.3,
        border_color: [1.0, 1.0, 1.0, 0.35],
    });
    c.draw_shadow(ShadowParams {
        x: 40.2,
        y: 10.7,
        w: 70.5,
        h: 40.3,
        corner_radius: 9.5,
        blur_radius: 6.0,
        offset: [0.0, 2.0],
        color: [0.0, 0.0, 0.0, 0.5],
        inset: true,
    });
    c.draw_gradient_rect(GradientRectParams {
        x: 120.6,
        y: 8.1,
        w: 110.2,
        h: 44.9,
        color: [0.95, 0.75, 0.1, 1.0],
        color2: [0.1, 0.8, 0.7, 0.0],
        angle_deg: 30.0,
        corner_radius: 12.25,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
    c.draw_path(PathBuilder::circle(30.5, 90.5, 18.3).fill([0.3, 0.8, 0.4, 1.0]));
    c.draw_path(PathBuilder::circle(30.5, 90.5, 24.1).stroke([0.9, 0.9, 0.9, 0.8], 1.5));
    match load_image_rgba(16, 16, checker()) {
        Ok(image) => c.draw_image(64.4, 66.6, 33.3, 33.3, image, 6.0),
        Err(e) => panic!("image: {e}"),
    }
    c.push(SceneNode::PushClip {
        x: 105.5,
        y: 60.5,
        w: 120.0,
        h: 40.0,
    });
    for (i, size) in [11.0, 13.5, 16.0].into_iter().enumerate() {
        let style = TextStyle::new(size).with_line_height(size * 1.3);
        c.draw_text(
            TextNodeKey::from_style("Determinism, 1.25x", &style, None),
            108.3,
            62.7 + i as f32 * 15.3,
            [0.95, 0.95, 0.95, 1.0],
        );
    }
    c.push(SceneNode::PopClip);
    c.push_to_layer(
        glass,
        SceneNode::BackdropBlur {
            x: 20.4,
            y: 110.2,
            w: 120.3,
            h: 40.1,
            corner_radius: 10.0,
            sigma: 6.0,
        },
    );
    c.push_to_layer(
        glass,
        SceneNode::RoundedRect {
            x: 20.4,
            y: 110.2,
            w: 120.3,
            h: 40.1,
            color: [1.0, 1.0, 1.0, 0.12],
            corner_radius: 10.0,
            border_width: 1.0,
            border_color: [1.0, 1.0, 1.0, 0.25],
        },
    );
    c.set_layer_opacity(glass, 0.8);
    c.push_to_layer(
        blurred,
        SceneNode::Rect {
            x: 160.7,
            y: 105.3,
            w: 50.2,
            h: 30.9,
            color: [0.8, 0.3, 0.9, 1.0],
        },
    );
    c.set_layer_effects(blurred, vec![LayerEffect::Blur { sigma: 3.0 }]);
    c
}

fn draw(r: &mut Renderer) -> Frame {
    let mut c = scene();
    match r.render(&mut c, PAGE_BG.to_linear_array()) {
        Ok(f) => f,
        Err(e) => panic!("render: {e}"),
    }
}

/// How many pixels differ, and the largest channel difference.
fn differ(a: &Frame, b: &Frame) -> (usize, u8) {
    let mut n = 0;
    let mut max = 0;
    for (p, q) in a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4)) {
        if p != q {
            n += 1;
            let d = p.iter().zip(q).map(|(x, y)| x.abs_diff(*y)).max();
            max = max.max(d.unwrap_or(0));
        }
    }
    (n, max)
}

#[test]
fn two_gpu_contexts_draw_every_primitive_to_the_same_bytes_at_every_scale() {
    for scale in SCALES {
        let (Some(mut a), Some(mut b)) = (renderer(scale), renderer(scale)) else {
            return;
        };
        let (fa, fb) = (draw(&mut a), draw(&mut b));
        let drawn = fa.rgba.chunks_exact(4).filter(|p| p[..3] != [48, 48, 48]);
        assert!(
            drawn.count() > fa.rgba.len() / 16,
            "{scale}x drew too little"
        );
        assert_eq!(
            (fa.width, fa.height),
            Viewport::new(WIDTH, HEIGHT, scale).physical()
        );
        let (n, max) = differ(&fa, &fb);
        assert!(
            n == 0,
            "{scale}x: {n} pixels differ between two contexts, delta max {max}"
        );
    }
}

#[test]
fn a_renderer_resized_across_scales_draws_what_a_fresh_one_draws() {
    let Some(mut kept) = renderer(2.0) else {
        return;
    };
    // down, then back up: every resize is one a window can make
    for scale in [2.0, 1.5, 1.25, 1.0, 1.5, 2.0] {
        if let Err(e) = kept.resize(Viewport::new(WIDTH, HEIGHT, scale)) {
            panic!("{e}");
        }
        let Some(mut fresh) = renderer(scale) else {
            return;
        };
        let (n, max) = differ(&draw(&mut kept), &draw(&mut fresh));
        assert!(
            n == 0,
            "{scale}x: {n} pixels differ after resizes, delta max {max}"
        );
    }
}

/// How far a backdrop blur of sigma 4 spreads a white-to-black edge, in
/// logical px: the columns of the middle row between 10% and 90% of the
/// ramp, divided by the scale.
fn blur_reach(scale: f32) -> Option<f32> {
    let mut r = match Renderer::new(Viewport::new(100.0, 40.0, scale), RenderConfig::default()) {
        Ok(r) => r,
        Err(HeadlessError::NoAdapter) => return None,
        Err(e) => panic!("headless renderer: {e}"),
    };
    let mut c = Compositor::new();
    let glass = c.create_layer(10);
    c.begin_frame();
    c.draw_rect(0.0, 0.0, 50.0, 40.0, [1.0, 1.0, 1.0, 1.0]);
    c.push_to_layer(
        glass,
        SceneNode::BackdropBlur {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 40.0,
            corner_radius: 0.0,
            sigma: 4.0,
        },
    );
    let f = match r.render(&mut c, [0.0, 0.0, 0.0, 1.0]) {
        Ok(f) => f,
        Err(e) => panic!("render: {e}"),
    };
    let y = f.height / 2;
    let ramp = (0..f.width)
        .filter_map(|x| f.pixel(x, y))
        .filter(|p| (26..=229).contains(&p[0]))
        .count();
    Some(ramp as f32 / scale)
}

/// The divergence the probe found (task-77): the backdrop blur, and the
/// layer blur and shadow effects, took their sigma in physical texels
/// (`effects/apply.rs`, `apply_blur`), so the same glass spread 9
/// logical px at 1x, 6 at 1.5x and 4.5 at 2x. Since phase 1b the kernel
/// of 1x is laid over the physical texels at any other scale
/// (`effects::scaled_weights`).
#[test]
fn a_backdrop_blur_reaches_as_far_in_logical_px_at_every_scale() {
    let Some(one) = blur_reach(1.0) else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    for scale in [1.5, 2.0] {
        let Some(reach) = blur_reach(scale) else {
            return;
        };
        assert!(
            (reach - one).abs() <= 1.0,
            "{scale}x: the blur spreads {reach} logical px, {one} at 1x"
        );
    }
}
