//! The blur and the scale (task-77 phase 1b).
//!
//! The probes (ignored, they print numbers) measure what a backdrop blur
//! costs on the GPU at each scale, on a page with one glass, and how far
//! its profile in logical px is from the one at scale 1:
//!
//! ```text
//! cargo test -p engine --test blur_scale -- --ignored --nocapture --test-threads=1
//! ```
//!
//! The tests hold what the fix of phase 1b promises: at the fractional
//! scales the glass has the reach and the profile of 1x in logical px,
//! and keeps the light of a detail thinner than a logical px wherever it
//! falls on the grid. The test of scale 1 compares a frame of glass and
//! shadows with the frame the engine drew before the blur followed the
//! scale
//! (`tests/fixtures/blur_at_scale_1.png`, recorded at `3a30113e`, the
//! base of phase 1b, with `PLEV_RECORD_BLUR_FIXTURE=1`). Bytes depend on
//! the GPU and its compiler, so the comparison runs on the adapter that
//! recorded it and skips elsewhere, saying so. The adapter's name does
//! not see the system move under it: on 2026-10-27, under macOS 27.0.1,
//! the base commit itself (`7127f65a`) drew 565 text pixels of the
//! fixture differently, the glass and the shadows unchanged, and the head
//! of the tree drew the same bytes as the base. The fixture is that frame
//! of the base, drawn again on the same Apple M4; when the system moves
//! again, record it from the base the same way, never from the head.
//!
//! Needs a GPU adapter; without one every test skips.

use engine::drawing::compositor::{
    Compositor, LayerId, RoundedRectParams, SceneNode, ShadowParams, TextNodeKey,
};
use engine::drawing::gpu::{RenderConfig, load_image_rgba};
use engine::drawing::headless::{Frame, HeadlessError, Renderer, Viewport};
use engine::drawing::path::PathBuilder;
use engine::drawing::text::TextStyle;
use engine::runtime::perf::stages::{self, Stage};

/// The scales of the capture: a plain display, the fractional 125% and
/// 150% of windows and linux, a retina display.
const SCALES: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

/// The theme sigmas: the tests' 4, core's 8, hoff's 50.
const SIGMAS: [f32; 3] = [4.0, 8.0, 50.0];

fn renderer(v: Viewport, config: RenderConfig) -> Option<Renderer> {
    match Renderer::new(v, config) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

fn render(v: Viewport, c: &mut Compositor, clear: [f32; 4]) -> Option<Frame> {
    let mut r = renderer(v, RenderConfig::default())?;
    match r.render(c, clear) {
        Ok(f) => Some(f),
        Err(e) => panic!("render: {e}"),
    }
}

/// An sRGB byte in linear light.
fn linear(byte: u8) -> f64 {
    let c = f64::from(byte) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear light back to an sRGB value on the 0..255 scale, unrounded.
fn srgb(l: f64) -> f64 {
    let l = l.clamp(0.0, 1.0);
    let c = if l <= 0.003_130_8 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    c * 255.0
}

// -- the edge ----------------------------------------------------------------

/// A white-to-black edge at x = 48 logical under a backdrop blur of
/// `sigma` covering the whole 100 x 40 frame. 48 falls on a physical
/// pixel's edge at every scale of `SCALES`: no pixel is half covered, and
/// what differs from 1x is the blur alone.
fn edge(sigma: f32, scale: f32) -> Option<Frame> {
    let mut c = Compositor::new();
    let glass = c.create_layer(10);
    c.begin_frame();
    c.draw_rect(0.0, 0.0, 48.0, 40.0, [1.0, 1.0, 1.0, 1.0]);
    c.push_to_layer(
        glass,
        SceneNode::BackdropBlur {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 40.0,
            corner_radius: 0.0,
            sigma,
        },
    );
    render(
        Viewport::new(100.0, 40.0, scale),
        &mut c,
        [0.0, 0.0, 0.0, 1.0],
    )
}

/// The red channel of the middle row.
fn middle_row(f: &Frame) -> Vec<u8> {
    let y = f.height / 2;
    (0..f.width)
        .filter_map(|x| f.pixel(x, y))
        .map(|p| p[0])
        .collect()
}

/// The ramp between 10% and 90% of the edge, in logical px (the metric
/// of `headless_determinism`'s reach test).
fn reach(row: &[u8], scale: f32) -> f32 {
    row.iter().filter(|v| (26..=229).contains(*v)).count() as f32 / scale
}

/// The row at `scale` against the row at 1x resampled onto its pixels:
/// each physical pixel's center, in logical px, reads the 1x row by
/// linear interpolation in linear light between the two logical pixel
/// centers around it. The largest and the mean difference, 0 to 255.
fn against_1x(one: &[u8], row: &[u8], scale: f32) -> (f64, f64) {
    let at = |u: f64| {
        let last = one.len() - 1;
        let u = u.clamp(0.0, last as f64);
        let i = (u.floor() as usize).min(last - 1);
        let t = u - i as f64;
        srgb(linear(one[i]) * (1.0 - t) + linear(one[i + 1]) * t)
    };
    let mut max: f64 = 0.0;
    let mut sum = 0.0;
    for (p, &v) in row.iter().enumerate() {
        let u = (p as f64 + 0.5) / f64::from(scale) - 0.5;
        let d = (f64::from(v) - at(u)).abs();
        max = max.max(d);
        sum += d;
    }
    (max, sum / row.len() as f64)
}

/// A white line one physical px wide, `phase` physical px right of x = 48
/// logical, on black, under a backdrop blur of `sigma` covering the
/// whole 100 x 40 frame.
fn line(sigma: f32, scale: f32, phase: u32) -> Option<Frame> {
    let mut c = Compositor::new();
    let glass = c.create_layer(10);
    c.begin_frame();
    c.draw_rect(
        48.0 + phase as f32 / scale,
        0.0,
        1.0 / scale,
        40.0,
        [1.0; 4],
    );
    c.push_to_layer(
        glass,
        SceneNode::BackdropBlur {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 40.0,
            corner_radius: 0.0,
            sigma,
        },
    );
    render(
        Viewport::new(100.0, 40.0, scale),
        &mut c,
        [0.0, 0.0, 0.0, 1.0],
    )
}

/// The row's light, in linear, summed over its physical px: a blur that
/// keeps the line's light gives 1 (the line is one physical px of white)
/// whatever the line's phase on the grid.
fn light(row: &[u8]) -> f64 {
    row.iter().map(|v| linear(*v)).sum()
}

// -- a scene under glass -----------------------------------------------------

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

/// Detail under the glass: an edge on a fractional x, stripes one
/// logical px wide, a checker image, a circle, text.
fn detail(c: &mut Compositor) {
    c.draw_rect(0.0, 0.0, 40.5, 96.0, [0.95, 0.95, 0.95, 1.0]);
    for i in 0..8 {
        let x = 44.25 + i as f32 * 2.0;
        c.draw_rect(x, 6.0, 1.0, 84.0, [0.1, 0.7, 0.3, 1.0]);
    }
    match load_image_rgba(16, 16, checker()) {
        Ok(image) => c.draw_image(64.4, 8.6, 40.0, 40.0, image, 0.0),
        Err(e) => panic!("image: {e}"),
    }
    c.draw_path(PathBuilder::circle(130.5, 30.5, 16.3).fill([0.9, 0.8, 0.1, 1.0]));
    for (i, size) in [11.0, 14.0].into_iter().enumerate() {
        let style = TextStyle::new(size).with_line_height(size * 1.3);
        c.draw_text(
            TextNodeKey::from_style("glass 1.25x", &style, None),
            64.3,
            56.7 + i as f32 * 17.0,
            [0.9, 0.9, 0.9, 1.0],
        );
    }
}

/// `detail` at 160 x 96, under a backdrop blur of `sigma` over
/// (16, 12, 128, 72), or none.
fn under_glass(sigma: Option<f32>, scale: f32) -> Option<Frame> {
    let mut c = Compositor::new();
    let glass = c.create_layer(10);
    c.begin_frame();
    detail(&mut c);
    if let Some(sigma) = sigma {
        c.push_to_layer(
            glass,
            SceneNode::BackdropBlur {
                x: 16.0,
                y: 12.0,
                w: 128.0,
                h: 72.0,
                corner_radius: 0.0,
                sigma,
            },
        );
    }
    render(
        Viewport::new(160.0, 96.0, scale),
        &mut c,
        [0.02, 0.02, 0.03, 1.0],
    )
}

/// The frame at `scale` brought to the 1x grid by area: each logical
/// pixel is the mean, in linear light, of the physical pixels it covers,
/// weighted by the covered area. Compared with the 1x frame inside the
/// glass (inset by 4 logical px from its edge): the largest and the mean
/// channel difference (RGB), 0 to 255.
fn area_against_1x(one: &Frame, other: &Frame, scale: f32) -> (f64, f64) {
    let s = f64::from(scale);
    let cover = |m: u32, p: u32| {
        let (a, b) = (f64::from(m) * s, f64::from(m + 1) * s);
        (f64::from(p + 1).min(b) - f64::from(p).max(a)).max(0.0)
    };
    let (mut max, mut sum, mut n): (f64, f64, f64) = (0.0, 0.0, 0.0);
    for my in 16..80u32 {
        for mx in 20..140u32 {
            let py = (f64::from(my) * s).floor() as u32..=(f64::from(my + 1) * s).ceil() as u32;
            let mut acc = [0.0f64; 3];
            let mut area = 0.0;
            for y in py {
                let wy = cover(my, y);
                if wy == 0.0 {
                    continue;
                }
                let px = (f64::from(mx) * s).floor() as u32..=(f64::from(mx + 1) * s).ceil() as u32;
                for x in px {
                    let w = wy * cover(mx, x);
                    let Some(p) = other.pixel(x, y).filter(|_| w > 0.0) else {
                        continue;
                    };
                    for (a, &v) in acc.iter_mut().zip(&p[..3]) {
                        *a += w * linear(v);
                    }
                    area += w;
                }
            }
            let Some(q) = one.pixel(mx, my) else {
                continue;
            };
            for (a, &v) in acc.iter().zip(&q[..3]) {
                let d = (srgb(a / area) - f64::from(v)).abs();
                max = max.max(d);
                sum += d;
                n += 1.0;
            }
        }
    }
    (max, sum / n)
}

#[test]
#[ignore = "probe: prints the blur's profile against 1x at each scale"]
fn probe_the_blur_profile_against_1x() {
    println!("edge under a backdrop blur (reach: the 10%-90% ramp in logical px)");
    println!("| sigma | scale | reach | edge vs 1x, max | edge vs 1x, mean |");
    println!("|---|---|---|---|---|");
    for sigma in SIGMAS {
        let Some(one) = edge(sigma, 1.0).map(|f| middle_row(&f)) else {
            return;
        };
        for scale in SCALES {
            let Some(row) = edge(sigma, scale).map(|f| middle_row(&f)) else {
                return;
            };
            let (max, mean) = against_1x(&one, &row, scale);
            println!(
                "| {sigma} | {scale} | {:.2} | {max:.1} | {mean:.2} |",
                reach(&row, scale)
            );
        }
    }
    println!();
    println!("a line one physical px wide at each phase of the grid, sigma 8: its light");
    println!("after the blur (1 keeps it) and the row's peak, per phase");
    println!("| scale | light | peak |");
    println!("|---|---|---|");
    for scale in [1.0f32, 1.25, 1.5, 2.0, 3.0] {
        let (mut lights, mut peaks) = (Vec::new(), Vec::new());
        for phase in 0..scale.ceil() as u32 {
            let Some(row) = line(8.0, scale, phase).map(|f| middle_row(&f)) else {
                return;
            };
            lights.push(format!("{:.2}", light(&row)));
            peaks.push(row.iter().max().copied().unwrap_or(0).to_string());
        }
        println!(
            "| {scale} | {} | {} |",
            lights.join(" / "),
            peaks.join(" / ")
        );
    }
    println!();
    println!("detail under glass, the frame at the scale by area onto the 1x grid");
    println!("| sigma | scale | max | mean |");
    println!("|---|---|---|---|");
    for sigma in [None, Some(4.0), Some(8.0), Some(50.0)] {
        let Some(one) = under_glass(sigma, 1.0) else {
            return;
        };
        for scale in SCALES.into_iter().skip(1) {
            let Some(other) = under_glass(sigma, scale) else {
                return;
            };
            let (max, mean) = area_against_1x(&one, &other, scale);
            let label = sigma.map_or("no glass".to_string(), |s| s.to_string());
            println!("| {label} | {scale} | {max:.1} | {mean:.2} |");
        }
    }
}

#[test]
fn a_backdrop_blur_has_the_profile_of_1x_at_the_fractional_scales() {
    for sigma in SIGMAS {
        let Some(one) = edge(sigma, 1.0).map(|f| middle_row(&f)) else {
            return;
        };
        for scale in [1.25, 1.5] {
            let Some(row) = edge(sigma, scale).map(|f| middle_row(&f)) else {
                return;
            };
            let (r1, r) = (reach(&one, 1.0), reach(&row, scale));
            assert!(
                (r - r1).abs() <= 1.0,
                "sigma {sigma} at {scale}x spreads {r} logical px, {r1} at 1x"
            );
            let (max, mean) = against_1x(&one, &row, scale);
            assert!(
                max <= 2.0,
                "sigma {sigma} at {scale}x: {max} from 1x resampled (mean {mean})"
            );
        }
    }
}

#[test]
fn a_line_thinner_than_a_logical_px_keeps_its_light_under_the_glass() {
    for scale in [1.25f32, 1.5, 3.0] {
        let mut peaks = Vec::new();
        for phase in 0..scale.ceil() as u32 {
            let Some(row) = line(8.0, scale, phase).map(|f| middle_row(&f)) else {
                return;
            };
            let light = light(&row);
            assert!(
                (light - 1.0).abs() <= 0.02,
                "{scale}x, phase {phase}: the line's light is {light} after the blur"
            );
            peaks.push(row.iter().max().copied().unwrap_or(0));
        }
        let (lo, hi) = (peaks.iter().min(), peaks.iter().max());
        assert!(
            hi.zip(lo).is_some_and(|(h, l)| h - l <= 1),
            "{scale}x: the peak moves with the phase, {peaks:?}"
        );
    }
}

// -- the cost ----------------------------------------------------------------

/// A page of 1280 x 800 logical: cards with shadows and borders, text,
/// an image; above it, on its own layer, a dialog-sized glass (a
/// backdrop blur of `sigma` under a translucent fill), or the fill alone.
fn page(c: &mut Compositor, glass_layer: LayerId, sigma: Option<f32>) {
    c.draw_rect(0.0, 0.0, 1280.0, 800.0, [0.12, 0.12, 0.14, 1.0]);
    let style = TextStyle::new(13.0).with_line_height(17.0);
    for row in 0..4 {
        for col in 0..6 {
            let (x, y) = (24.0 + col as f32 * 208.0, 24.0 + row as f32 * 190.0);
            c.draw_shadow(ShadowParams {
                x,
                y,
                w: 184.0,
                h: 166.0,
                corner_radius: 12.0,
                blur_radius: 16.0,
                offset: [0.0, 4.0],
                color: [0.0, 0.0, 0.0, 0.5],
                inset: false,
            });
            c.draw_rounded_rect(RoundedRectParams {
                x,
                y,
                w: 184.0,
                h: 166.0,
                color: [0.2 + 0.1 * col as f32, 0.3, 0.5 - 0.08 * row as f32, 1.0],
                corner_radius: 12.0,
                border_width: 1.0,
                border_color: [1.0, 1.0, 1.0, 0.3],
            });
            c.draw_text(
                TextNodeKey::from_style("a card under the glass", &style, None),
                x + 12.0,
                y + 12.0,
                [0.95, 0.95, 0.95, 1.0],
            );
        }
    }
    match load_image_rgba(16, 16, checker()) {
        Ok(image) => c.draw_image(560.0, 300.0, 160.0, 160.0, image, 8.0),
        Err(e) => panic!("image: {e}"),
    }
    let (x, y, w, h) = (400.0, 250.0, 480.0, 300.0);
    if let Some(sigma) = sigma {
        c.push_to_layer(
            glass_layer,
            SceneNode::BackdropBlur {
                x,
                y,
                w,
                h,
                corner_radius: 16.0,
                sigma,
            },
        );
    }
    c.push_to_layer(
        glass_layer,
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h,
            color: [1.0, 1.0, 1.0, 0.12],
            corner_radius: 16.0,
            border_width: 1.0,
            border_color: [1.0, 1.0, 1.0, 0.25],
        },
    );
}

/// The GPU span (first pass start to composite end, from the timestamps)
/// of `frames` frames of `page` redrawn whole, after a warm-up: its p50
/// in microseconds. `None` without an adapter or timestamps.
///
/// No GPU wait between frames: with the wait the GPU idles after every
/// frame and the span reads its clock coming back up (p50s of one case
/// 400 us apart run to run), without it the frames queue back to back as
/// in a window that animates.
fn gpu_span(scale: f32, sigma: Option<f32>, frames: usize) -> Option<f64> {
    let config = RenderConfig {
        gpu_timestamps: true,
        ..RenderConfig::default()
    };
    let mut r = renderer(Viewport::new(1280.0, 800.0, scale), config)?;
    let mut c = Compositor::new();
    let glass = c.create_layer(10);
    stages::enable(true);
    let mut spans = Vec::with_capacity(frames);
    for i in 0..frames + WARM_UP {
        c.begin_frame();
        page(&mut c, glass, sigma);
        c.mark_all_dirty();
        r.draw(&mut c, [0.02, 0.02, 0.03, 1.0]);
        let cost = stages::take().get(Stage::GpuSpan);
        if i >= WARM_UP && cost.calls > 0 {
            spans.push(cost.micros() / cost.calls as f64);
        }
    }
    stages::enable(false);
    if spans.is_empty() {
        eprintln!("no gpu timestamps read; skipping");
        return None;
    }
    Some(median(spans))
}

/// Frames drawn before the measured ones.
const WARM_UP: usize = 30;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

#[test]
#[ignore = "probe: prints what a page with glass costs the GPU at each scale"]
fn probe_the_cost_of_a_page_with_glass() {
    const RUNS: usize = 5;
    const FRAMES: usize = 240;
    println!(
        "1280x800 logical, every layer redrawn each frame, {FRAMES} frames after \
         {WARM_UP} of warm-up, {RUNS} runs; gpu span p50 in us, the median of the runs \
         [their range]"
    );
    println!("| scale | no glass | glass, sigma 8 | the glass |");
    println!("|---|---|---|---|");
    let range = |v: &[f64]| {
        v.iter()
            .fold((f64::MAX, 0.0f64), |(l, h), x| (l.min(*x), h.max(*x)))
    };
    for scale in SCALES {
        // the two cases alternate, run by run, so a change of the GPU's
        // clock between runs falls on both
        let (mut bare, mut glass) = (Vec::new(), Vec::new());
        for _ in 0..RUNS {
            let (Some(b), Some(g)) = (
                gpu_span(scale, None, FRAMES),
                gpu_span(scale, Some(8.0), FRAMES),
            ) else {
                return;
            };
            bare.push(b);
            glass.push(g);
        }
        let ((bl, bh), (gl, gh)) = (range(&bare), range(&glass));
        let (b, g) = (median(bare), median(glass));
        println!(
            "| {scale} | {b:.0} [{bl:.0}-{bh:.0}] | {g:.0} [{gl:.0}-{gh:.0}] | {:.0} |",
            g - b
        );
    }
}

// -- scale 1, byte for byte --------------------------------------------------

/// Where the frame of before lives, and the adapter that drew it.
const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/blur_at_scale_1.png"
);
const RECORDED_ON: (&str, wgpu::Backend) = ("Apple M4", wgpu::Backend::Metal);

/// Glass at three sigmas (a layer blur's 3, core's 8, hoff's 50) over
/// `detail`, with a drop and an inset analytic shadow, at 160 x 96.
fn glass_and_shadows() -> Compositor {
    let mut c = Compositor::new();
    let glass = c.create_layer(10);
    c.begin_frame();
    detail(&mut c);
    c.draw_shadow(ShadowParams {
        x: 110.2,
        y: 58.7,
        w: 36.5,
        h: 24.3,
        corner_radius: 6.5,
        blur_radius: 10.0,
        offset: [1.5, 2.5],
        color: [0.0, 0.0, 0.0, 0.6],
        inset: false,
    });
    c.draw_shadow(ShadowParams {
        x: 110.2,
        y: 58.7,
        w: 36.5,
        h: 24.3,
        corner_radius: 6.5,
        blur_radius: 6.0,
        offset: [0.0, 2.0],
        color: [0.0, 0.0, 0.0, 0.5],
        inset: true,
    });
    for (i, sigma) in [3.0, 8.0, 50.0].into_iter().enumerate() {
        let x = 4.5 + i as f32 * 51.3;
        c.push_to_layer(
            glass,
            SceneNode::BackdropBlur {
                x,
                y: 20.4,
                w: 46.2,
                h: 30.1,
                corner_radius: 8.0,
                sigma,
            },
        );
        c.push_to_layer(
            glass,
            SceneNode::RoundedRect {
                x,
                y: 20.4,
                w: 46.2,
                h: 30.1,
                color: [1.0, 1.0, 1.0, 0.12],
                corner_radius: 8.0,
                border_width: 1.0,
                border_color: [1.0, 1.0, 1.0, 0.25],
            },
        );
    }
    c.set_layer_opacity(glass, 0.9);
    c
}

/// The name and backend of the adapter the headless renderer asks for.
fn adapter() -> Option<(String, wgpu::Backend)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let info = adapter.get_info();
    Some((info.name, info.backend))
}

#[test]
fn the_glass_at_scale_1_is_the_glass_of_before_byte_for_byte() {
    let Some(frame) = render(
        Viewport::new(160.0, 96.0, 1.0),
        &mut glass_and_shadows(),
        [0.02, 0.02, 0.03, 1.0],
    ) else {
        return;
    };
    if std::env::var_os("PLEV_RECORD_BLUR_FIXTURE").is_some() {
        if let Err(e) = frame.write_png(std::path::Path::new(FIXTURE)) {
            panic!("{FIXTURE}: {e}");
        }
        return;
    }
    let here = adapter();
    if here.as_ref().map(|(n, b)| (n.as_str(), *b)) != Some(RECORDED_ON) {
        eprintln!("the fixture was drawn on {RECORDED_ON:?}, this is {here:?}; skipping");
        return;
    }
    let before = match image::open(FIXTURE) {
        Ok(img) => img.to_rgba8(),
        Err(e) => panic!("{FIXTURE}: {e}"),
    };
    assert_eq!((frame.width, frame.height), before.dimensions());
    let differ = frame
        .rgba
        .chunks_exact(4)
        .zip(before.as_raw().chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differ == 0,
        "{differ} pixels differ from the frame of before"
    );
}
