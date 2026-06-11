//! Row 2 cards: Animated Pulse, Button Gallery, Color Palette, Bezier Paths.

use plev::compositor::{Compositor, RoundedRectParams, SceneNode, TextNodeKey};
use plev::path::PathBuilder;

use crate::shapes::*;

// One over the limit; the args are the shared row-grid metrics already
// unpacked by the caller (same trade-off as plev's card.rs).
#[allow(clippy::too_many_arguments)]
pub fn draw_row2(
    comp: &mut Compositor,
    margin: f32,
    gap: f32,
    card_w: f32,
    card_h: f32,
    y1: f32,
    p: f32,
    t: f32,
) {
    draw_animated_pulse(comp, margin, y1, card_w, card_h, p);
    draw_button_gallery(comp, margin + card_w + gap, y1, card_w, card_h);
    draw_color_palette(comp, margin + (card_w + gap) * 2.0, y1, card_w, card_h);
    draw_bezier_paths(
        comp,
        margin + (card_w + gap) * 3.0,
        y1,
        card_w,
        card_h,
        p,
        t,
    );
}

// Card 5: Animated Pulse
fn draw_animated_pulse(comp: &mut Compositor, cx: f32, cy: f32, card_w: f32, card_h: f32, p: f32) {
    push_shadow(comp, cx, cy, card_w, card_h, 6);
    comp.push(SceneNode::Rect {
        x: cx,
        y: cy,
        w: card_w,
        h: card_h,
        color: SURFACE,
    });
    comp.push(SceneNode::Rect {
        x: cx,
        y: cy,
        w: card_w,
        h: 2.0,
        color: PINK,
    });
    comp.draw_text(
        TextNodeKey::from_style("ANIMATED PULSE", &card_title_style(12.0, 16.0), None),
        cx + 12.0,
        cy + 10.0,
        PINK,
    );

    let center_x = cx + card_w / 2.0;
    let center_y = cy + card_h / 2.0 + 10.0;

    for i in 0..4 {
        let phase = (p + i as f32 * 0.25) % 1.0;
        let r = 15.0 + phase * 50.0;
        let alpha = (1.0 - phase) * 0.5;
        push_ring(
            comp,
            center_x,
            center_y,
            r + 3.0,
            r,
            [PINK[0], PINK[1], PINK[2], alpha],
        );
    }
    push_circle(comp, center_x, center_y, 12.0, PINK);
}

// Card 6: Button Gallery
fn draw_button_gallery(comp: &mut Compositor, cx: f32, cy: f32, card_w: f32, card_h: f32) {
    push_shadow(comp, cx, cy, card_w, card_h, 6);
    comp.push(SceneNode::Rect {
        x: cx,
        y: cy,
        w: card_w,
        h: card_h,
        color: SURFACE,
    });
    comp.push(SceneNode::Rect {
        x: cx,
        y: cy,
        w: card_w,
        h: 2.0,
        color: YELLOW,
    });
    comp.draw_text(
        TextNodeKey::from_style("BUTTON GALLERY", &card_title_style(12.0, 16.0), None),
        cx + 12.0,
        cy + 10.0,
        YELLOW,
    );

    let bw = card_w - 24.0;
    let bh = 30.0;
    // (label, fill, corner radius, border width, border color).
    type ButtonSpec = (&'static str, [f32; 4], f32, f32, [f32; 4]);
    let btns: &[ButtonSpec] = &[
        ("Primary", ACCENT, 6.0, 0.0, [0.0; 4]),
        ("Success", GREEN, 6.0, 0.0, [0.0; 4]),
        ("Danger", RED, 6.0, 0.0, [0.0; 4]),
        ("Outlined", [0.0, 0.0, 0.0, 0.0], 6.0, 1.5, ACCENT),
        (
            "Ghost",
            [ACCENT[0] * 0.15, ACCENT[1] * 0.15, ACCENT[2] * 0.15, 0.3],
            6.0,
            0.0,
            [0.0; 4],
        ),
        ("Pill", PURPLE, 15.0, 0.0, [0.0; 4]),
    ];
    for (i, (label, bg, radius, border_w, border_c)) in btns.iter().enumerate() {
        let by = cy + 32.0 + i as f32 * (bh + 6.0);
        comp.draw_rounded_rect(RoundedRectParams {
            x: cx + 12.0,
            y: by,
            w: bw,
            h: bh,
            color: *bg,
            corner_radius: *radius,
            border_width: *border_w,
            border_color: *border_c,
        });
        let text_color = if bg[3] < 0.1 { ACCENT } else { TEXT };
        comp.draw_text(
            TextNodeKey::from_style(label, &label_style(11.0, 15.0), Some(bw - 16.0)),
            cx + 20.0,
            by + 8.0,
            text_color,
        );
    }
}

// Card 7: Color Palette
fn draw_color_palette(comp: &mut Compositor, cx: f32, cy: f32, card_w: f32, card_h: f32) {
    push_shadow(comp, cx, cy, card_w, card_h, 6);
    comp.push(SceneNode::Rect {
        x: cx,
        y: cy,
        w: card_w,
        h: card_h,
        color: SURFACE,
    });
    comp.push(SceneNode::Rect {
        x: cx,
        y: cy,
        w: card_w,
        h: 2.0,
        color: ORANGE,
    });
    comp.draw_text(
        TextNodeKey::from_style("COLOR PALETTE", &card_title_style(12.0, 16.0), None),
        cx + 12.0,
        cy + 10.0,
        ORANGE,
    );

    let colors: &[(&str, [f32; 4])] = &[
        ("Accent", ACCENT),
        ("Green", GREEN),
        ("Red", RED),
        ("Yellow", YELLOW),
        ("Purple", PURPLE),
        ("Cyan", CYAN),
        ("Orange", ORANGE),
        ("Pink", PINK),
    ];
    let sw = (card_w - 24.0) / 4.0;
    let sh = 40.0;
    for (i, (name, color)) in colors.iter().enumerate() {
        let col = i % 4;
        let row = i / 4;
        let sx = cx + 12.0 + col as f32 * sw;
        let sy = cy + 34.0 + row as f32 * (sh + 20.0);
        comp.draw_rounded_rect(RoundedRectParams {
            x: sx + 2.0,
            y: sy,
            w: sw - 4.0,
            h: sh,
            color: *color,
            corner_radius: 6.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        comp.draw_text(
            TextNodeKey::from_style(name, &label_style(9.0, 12.0), Some(sw)),
            sx + 2.0,
            sy + sh + 4.0,
            TEXT_DIM,
        );

        for a in 0..3 {
            let ay = sy + sh + 18.0 + a as f32 * 6.0;
            let alpha = 1.0 - a as f32 * 0.3;
            comp.push(SceneNode::Rect {
                x: sx + 2.0,
                y: ay,
                w: sw - 4.0,
                h: 4.0,
                color: [color[0], color[1], color[2], alpha],
            });
        }
    }
}

// Card 8: Bezier Paths
fn draw_bezier_paths(
    comp: &mut Compositor,
    cx: f32,
    cy: f32,
    card_w: f32,
    card_h: f32,
    p: f32,
    t: f32,
) {
    push_shadow(comp, cx, cy, card_w, card_h, 6);
    comp.push(SceneNode::Rect {
        x: cx,
        y: cy,
        w: card_w,
        h: card_h,
        color: SURFACE,
    });
    comp.push(SceneNode::Rect {
        x: cx,
        y: cy,
        w: card_w,
        h: 2.0,
        color: RED,
    });
    comp.draw_text(
        TextNodeKey::from_style("BEZIER PATHS", &card_title_style(12.0, 16.0), None),
        cx + 12.0,
        cy + 10.0,
        RED,
    );

    let wave_y = cy + 80.0;
    let wave_w = card_w - 24.0;
    let wave_x = cx + 12.0;
    let segments = 8;
    let seg_w = wave_w / segments as f32;

    // Wave 1
    let mut b = PathBuilder::new();
    b = b.move_to(wave_x, wave_y + 40.0);
    for i in 0..segments {
        let sx = wave_x + i as f32 * seg_w;
        let ex = sx + seg_w;
        let amplitude = 20.0 + (t + i as f32 * 0.5).sin() * 10.0 * p;
        let cp1y = wave_y + 40.0 - amplitude;
        let cp2y = wave_y + 40.0 + amplitude;
        b = b.cubic_bezier_to(
            [sx + seg_w * 0.33, cp1y],
            [sx + seg_w * 0.66, cp2y],
            [ex, wave_y + 40.0],
        );
    }
    b = b.line_to(wave_x + wave_w, wave_y + 80.0);
    b = b.line_to(wave_x, wave_y + 80.0);
    comp.draw_path(b.close().fill([RED[0], RED[1], RED[2], 0.3]));

    // Wave 2
    let mut b2 = PathBuilder::new();
    b2 = b2.move_to(wave_x, wave_y + 50.0);
    for i in 0..segments {
        let sx = wave_x + i as f32 * seg_w;
        let ex = sx + seg_w;
        let amplitude = 15.0 + (t * 0.7 + i as f32 * 0.3 + 1.0).sin() * 8.0 * p;
        b2 = b2.cubic_bezier_to(
            [sx + seg_w * 0.33, wave_y + 50.0 - amplitude],
            [sx + seg_w * 0.66, wave_y + 50.0 + amplitude],
            [ex, wave_y + 50.0],
        );
    }
    b2 = b2.line_to(wave_x + wave_w, wave_y + 80.0);
    b2 = b2.line_to(wave_x, wave_y + 80.0);
    comp.draw_path(b2.close().fill([ORANGE[0], ORANGE[1], ORANGE[2], 0.25]));

    comp.draw_text(
        TextNodeKey::from_style("cubic_to + animated", &code_style(9.0, 12.0), None),
        cx + 12.0,
        cy + card_h - 18.0,
        TEXT_DIM,
    );
}
