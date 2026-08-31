//! Scene content for the visual demo: analytic shadows, gradients, atlas
//! images and the clipped panel. Pure compositor calls — no window or GPU.

use engine::compositor::{
    Compositor, GradientRectParams, RoundedRectParams, ShadowParams, TextNodeKey,
};
use engine::text::TextStyle;

const TEXT: [f32; 4] = [0.92, 0.93, 0.95, 1.0];
const MUTED: [f32; 4] = [0.55, 0.58, 0.64, 1.0];
const CARD: [f32; 4] = [0.16, 0.17, 0.21, 1.0];

// One TextStyle per run, shared by measurement and drawing
// (docs/adr/one-text-style-for-measurement-and-drawing.md). Weights are
// semantic: 700 page title, 500 section label, 400 body; numeric readouts
// render in the same embedded Inclusive Sans (the only UI family).

fn title_style(size: f32) -> TextStyle {
    TextStyle::new(size).with_weight(700)
}

fn section_style(size: f32) -> TextStyle {
    TextStyle::new(size).with_weight(500)
}

fn body_style(size: f32) -> TextStyle {
    TextStyle::new(size)
}

fn code_style(size: f32) -> TextStyle {
    TextStyle::new(size).with_family("Inclusive Sans")
}

fn label(c: &mut Compositor, text: &str, style: &TextStyle, x: f32, y: f32, color: [f32; 4]) {
    c.draw_text(TextNodeKey::from_style(text, style, None), x, y, color);
}

/// Build the whole demo scene into `c`. `elapsed` drives the animated
/// scroll of the clipped panel.
pub fn build_scene(
    c: &mut Compositor,
    logo: Option<engine::gpu::ImageHandle>,
    pattern: Option<engine::gpu::ImageHandle>,
    elapsed: f32,
) {
    label(
        c,
        "plev renderer -- visual capabilities",
        &title_style(22.0),
        32.0,
        24.0,
        TEXT,
    );

    // -- Analytic shadows at several blur radii --------------------------
    label(
        c,
        "analytic shadows (blur 4 / 12 / 24 / 48)",
        &section_style(14.0),
        32.0,
        68.0,
        MUTED,
    );
    for (i, blur) in [4.0f32, 12.0, 24.0, 48.0].into_iter().enumerate() {
        let x = 32.0 + i as f32 * 180.0;
        let y = 100.0;
        c.draw_shadow(ShadowParams {
            x,
            y,
            w: 150.0,
            h: 90.0,
            corner_radius: 12.0,
            blur_radius: blur,
            offset: [0.0, 6.0],
            color: [0.0, 0.0, 0.0, 0.55],
            inset: false,
        });
        c.draw_rounded_rect(RoundedRectParams {
            x,
            y,
            w: 150.0,
            h: 90.0,
            color: CARD,
            corner_radius: 12.0,
            border_width: 1.0,
            border_color: [1.0, 1.0, 1.0, 0.08],
        });
        label(
            c,
            &format!("blur {blur}"),
            &code_style(13.0),
            x + 16.0,
            y + 36.0,
            TEXT,
        );
    }

    // -- Linear gradients -------------------------------------------------
    label(
        c,
        "linear gradients (0 / 45 / 90 / 135 deg)",
        &section_style(14.0),
        32.0,
        230.0,
        MUTED,
    );
    let stops = [
        ([0.98, 0.45, 0.25, 1.0], [0.85, 0.20, 0.55, 1.0], 0.0),
        ([0.20, 0.55, 0.95, 1.0], [0.25, 0.90, 0.75, 1.0], 45.0),
        ([0.55, 0.30, 0.95, 1.0], [0.95, 0.40, 0.45, 1.0], 90.0),
        ([0.95, 0.75, 0.25, 1.0], [0.90, 0.30, 0.30, 1.0], 135.0),
    ];
    for (i, (from, to, angle)) in stops.into_iter().enumerate() {
        let x = 32.0 + i as f32 * 180.0;
        c.draw_gradient_rect(GradientRectParams {
            x,
            y: 262.0,
            w: 150.0,
            h: 70.0,
            color: from,
            color2: to,
            angle_deg: angle,
            corner_radius: 10.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
    }

    // -- Images from the atlas -------------------------------------------
    label(
        c,
        "image atlas (png decode + procedural rgba)",
        &section_style(14.0),
        32.0,
        356.0,
        MUTED,
    );
    if let Some(logo) = logo {
        // Keep the aspect ratio at a fixed display height.
        let h = 96.0;
        let w = h * logo.width as f32 / logo.height as f32;
        c.draw_image(32.0, 388.0, w, h, logo, 8.0);
    }
    if let Some(pattern) = pattern {
        c.draw_image(160.0, 388.0, 96.0, 96.0, pattern, 48.0);
        c.draw_image(272.0, 388.0, 96.0, 96.0, pattern, 12.0);
    }

    // -- Clipped panel with oversized content ------------------------------
    let (px, py, pw, ph) = (420.0, 388.0, 300.0, 180.0);
    label(
        c,
        "clip stack (content larger than panel)",
        &section_style(14.0),
        px,
        356.0,
        MUTED,
    );
    c.draw_rounded_rect(RoundedRectParams {
        x: px,
        y: py,
        w: pw,
        h: ph,
        color: [0.13, 0.14, 0.17, 1.0],
        corner_radius: 8.0,
        border_width: 1.0,
        border_color: [1.0, 1.0, 1.0, 0.10],
    });

    // Oscillating scroll offset shows rows being cut at the edges.
    let scroll = ((elapsed * 0.7).sin() * 0.5 + 0.5) * 220.0;
    c.push_clip(px, py, pw, ph);
    for row in 0..14 {
        let y = py + 8.0 + row as f32 * 28.0 - scroll;
        // Wider than the panel on purpose: the right edge is clipped too.
        c.draw_rounded_rect(RoundedRectParams {
            x: px + 8.0,
            y,
            w: pw + 60.0,
            h: 22.0,
            color: if row % 2 == 0 {
                [0.20, 0.22, 0.27, 1.0]
            } else {
                [0.17, 0.18, 0.22, 1.0]
            },
            corner_radius: 4.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        label(
            c,
            &format!("row {row} -- clipped to the panel bounds"),
            &body_style(12.0),
            px + 16.0,
            y + 4.0,
            TEXT,
        );
    }
    c.pop_clip();
}

/// Procedural test image: radial color wheel with an alpha falloff.
pub fn procedural_pattern(size: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    let center = size as f32 / 2.0;
    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f32 - center, y as f32 - center);
            let dist = (dx * dx + dy * dy).sqrt() / center;
            let angle = dy.atan2(dx);
            let r = (angle.sin() * 0.5 + 0.5) * 255.0;
            let g = ((angle + 2.1).sin() * 0.5 + 0.5) * 255.0;
            let b = ((angle + 4.2).sin() * 0.5 + 0.5) * 255.0;
            let a = ((1.0 - dist).clamp(0.0, 1.0) * 255.0 * 1.5).min(255.0);
            pixels.extend_from_slice(&[r as u8, g as u8, b as u8, a as u8]);
        }
    }
    pixels
}
