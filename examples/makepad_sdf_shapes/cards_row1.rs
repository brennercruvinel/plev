//! Row 1 cards: Rounded Rects, Circles & Rings, Polygons, Depth & Shadow.

use plev::compositor::{Compositor, RoundedRectParams, SceneNode, TextNodeKey};

use crate::shapes::*;

pub fn draw_row1(
    comp: &mut Compositor,
    margin: f32,
    gap: f32,
    card_w: f32,
    card_h: f32,
    y0: f32,
    t: f32,
) {
    draw_rounded_rects(comp, margin, y0, card_w, card_h);
    draw_circles_rings(comp, margin + card_w + gap, y0, card_w, card_h, t);
    draw_polygons(comp, margin + (card_w + gap) * 2.0, y0, card_w, card_h);
    draw_depth_shadow(comp, margin + (card_w + gap) * 3.0, y0, card_w, card_h);
}

// Card 1: Rounded Rectangles
fn draw_rounded_rects(comp: &mut Compositor, cx: f32, cy: f32, card_w: f32, card_h: f32) {
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
        color: ACCENT,
    });
    comp.draw_text(
        TextNodeKey::new("ROUNDED RECTS", 12.0, 16.0, None),
        cx + 12.0,
        cy + 10.0,
        ACCENT,
    );

    let radii = [4.0, 8.0, 16.0, 32.0];
    let rw = (card_w - 24.0 - 8.0 * 3.0) / 4.0;
    for (i, r) in radii.iter().enumerate() {
        let rx = cx + 12.0 + i as f32 * (rw + 8.0);
        let ry = cy + 34.0;
        comp.draw_rounded_rect(RoundedRectParams {
            x: rx,
            y: ry,
            w: rw,
            h: 60.0,
            color: SURFACE_2,
            corner_radius: *r,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        let label = format!("r={}", *r as u32);
        comp.draw_text(
            TextNodeKey::new(&label, 9.0, 12.0, Some(rw)),
            rx + 4.0,
            ry + 64.0,
            TEXT_DIM,
        );
    }

    let bw = (card_w - 24.0 - 8.0 * 2.0) / 3.0;
    let by = cy + 110.0;
    let borders = [(ACCENT, "1px"), (GREEN, "2px"), (PURPLE, "3px")];
    for (i, (bc, label)) in borders.iter().enumerate() {
        let bx = cx + 12.0 + i as f32 * (bw + 8.0);
        comp.draw_rounded_rect(RoundedRectParams {
            x: bx,
            y: by,
            w: bw,
            h: 50.0,
            color: SURFACE_2,
            corner_radius: 8.0,
            border_width: (i + 1) as f32,
            border_color: *bc,
        });
        comp.draw_text(
            TextNodeKey::new(label, 9.0, 12.0, Some(bw)),
            bx + 4.0,
            by + 54.0,
            TEXT_DIM,
        );
    }
}

// Card 2: Circles & Rings
fn draw_circles_rings(comp: &mut Compositor, cx: f32, cy: f32, card_w: f32, card_h: f32, t: f32) {
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
        color: CYAN,
    });
    comp.draw_text(
        TextNodeKey::new("CIRCLES & RINGS", 12.0, 16.0, None),
        cx + 12.0,
        cy + 10.0,
        CYAN,
    );

    let center_x = cx + card_w / 2.0;
    let center_y = cy + 100.0;

    push_ring(
        comp,
        center_x,
        center_y,
        55.0,
        48.0,
        [CYAN[0], CYAN[1], CYAN[2], 0.3],
    );
    push_ring(
        comp,
        center_x,
        center_y,
        45.0,
        38.0,
        [CYAN[0], CYAN[1], CYAN[2], 0.5],
    );
    push_ring(
        comp,
        center_x,
        center_y,
        35.0,
        28.0,
        [CYAN[0], CYAN[1], CYAN[2], 0.7],
    );
    push_circle(comp, center_x, center_y, 25.0, CYAN);

    let colors = [RED, GREEN, YELLOW, PURPLE, ORANGE, PINK];
    for (i, c) in colors.iter().enumerate() {
        let a = std::f32::consts::TAU * (i as f32 / colors.len() as f32) + t * 0.5;
        let dx = center_x + 70.0 * a.cos();
        let dy = center_y + 70.0 * a.sin();
        push_circle(comp, dx, dy, 6.0, *c);
    }

    comp.draw_text(
        TextNodeKey::new("PathBuilder::circle", 9.0, 12.0, None),
        cx + 12.0,
        cy + card_h - 18.0,
        TEXT_DIM,
    );
}

// Card 3: Polygons
fn draw_polygons(comp: &mut Compositor, cx: f32, cy: f32, card_w: f32, card_h: f32) {
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
        color: GREEN,
    });
    comp.draw_text(
        TextNodeKey::new("POLYGONS", 12.0, 16.0, None),
        cx + 12.0,
        cy + 10.0,
        GREEN,
    );

    let polys = [
        (3, RED, "Tri"),
        (5, YELLOW, "Pent"),
        (6, PURPLE, "Hex"),
        (8, ORANGE, "Oct"),
    ];
    let pw = (card_w - 24.0) / 4.0;
    for (i, (sides, color, label)) in polys.iter().enumerate() {
        let px = cx + 12.0 + i as f32 * pw + pw / 2.0;
        let py = cy + 70.0;
        let r = (pw / 2.0 - 6.0).min(28.0);
        push_polygon(comp, px, py, r, *sides, *color);
        comp.draw_text(
            TextNodeKey::new(label, 9.0, 12.0, Some(pw)),
            cx + 12.0 + i as f32 * pw,
            cy + 105.0,
            TEXT_DIM,
        );
    }

    let stars = [(5, YELLOW, "5-pt"), (6, CYAN, "6-pt"), (8, PINK, "8-pt")];
    let sw = (card_w - 24.0) / 3.0;
    for (i, (pts, color, label)) in stars.iter().enumerate() {
        let sx = cx + 12.0 + i as f32 * sw + sw / 2.0;
        let sy = cy + 160.0;
        let r = (sw / 2.0 - 8.0).min(24.0);
        push_star(comp, sx, sy, r, r * 0.45, *pts, *color);
        comp.draw_text(
            TextNodeKey::new(label, 9.0, 12.0, Some(sw)),
            cx + 12.0 + i as f32 * sw,
            cy + 190.0,
            TEXT_DIM,
        );
    }
}

// Card 4: Shadows & Depth
fn draw_depth_shadow(comp: &mut Compositor, cx: f32, cy: f32, card_w: f32, card_h: f32) {
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
        color: PURPLE,
    });
    comp.draw_text(
        TextNodeKey::new("DEPTH & SHADOW", 12.0, 16.0, None),
        cx + 12.0,
        cy + 10.0,
        PURPLE,
    );

    let levels = ["Flat", "Raised", "Floating", "Modal"];
    let shadow_layers = [0, 3, 6, 10];
    let lw = (card_w - 24.0 - 8.0 * 3.0) / 4.0;
    for (i, (label, layers)) in levels.iter().zip(shadow_layers.iter()).enumerate() {
        let lx = cx + 12.0 + i as f32 * (lw + 8.0);
        let ly = cy + 38.0;
        push_shadow(comp, lx, ly, lw, 60.0, *layers);
        comp.draw_rounded_rect(RoundedRectParams {
            x: lx,
            y: ly,
            w: lw,
            h: 60.0,
            color: SURFACE_2,
            corner_radius: 6.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        comp.draw_text(
            TextNodeKey::new(label, 9.0, 12.0, Some(lw)),
            lx + 2.0,
            ly + 64.0,
            TEXT_DIM,
        );
    }

    let sx = cx + 20.0;
    let sy = cy + 130.0;
    let sw = card_w - 40.0;
    for i in 0..4 {
        let offset = i as f32 * 6.0;
        let alpha = 1.0 - i as f32 * 0.15;
        push_shadow(comp, sx + offset, sy + offset, sw - offset * 2.0, 40.0, 4);
        comp.draw_rounded_rect(RoundedRectParams {
            x: sx + offset,
            y: sy + offset,
            w: sw - offset * 2.0,
            h: 40.0,
            color: [SURFACE_2[0], SURFACE_2[1], SURFACE_2[2], alpha],
            corner_radius: 4.0,
            border_width: 1.0,
            border_color: [PURPLE[0], PURPLE[1], PURPLE[2], alpha * 0.3],
        });
    }
}
