//! Card construction functions for showcase row 3.

use crate::compositor::{Compositor, SceneNode, TextNodeKey};

use super::card_types::*;
use super::helpers::{card, card_label, card_title};

pub(crate) fn card_dispatch(
    compositor: &mut Compositor, lay: CardLayout, accent: [f32; 4],
    green: [f32; 4], red: [f32; 4], text_dim: [f32; 4],
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "DISPATCH", red);
    card_label(compositor, cx + 14.0, cy + 38.0, "Typed actions \u{2022} Any downcast", card_w - 28.0, text_dim);
    let tx = cx + 14.0;
    let mw = card_w - 28.0;
    let lines: &[(&str, [f32; 4])] = &[
        ("queue.emit(id, Action)", green),
        ("queue.drain_typed::<A>()", accent),
        ("  \u{2192} Vec<(u64, A)>", text_dim),
    ];
    for (i, (line, color)) in lines.iter().enumerate() {
        compositor.draw_text(TextNodeKey::new(line, 11.0, 14.0, Some(mw)), tx, cy + 58.0 + i as f32 * 16.0, *color);
    }
    card_label(compositor, cx + 14.0, cy + card_h - 22.0, "Child \u{2192} Parent \u{2022} No bus", card_w - 28.0, text_dim);
}

pub(crate) fn card_overlays(
    compositor: &mut Compositor, lay: CardLayout, yellow: [f32; 4], text_dim: [f32; 4],
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "OVERLAYS", yellow);
    card_label(compositor, cx + 14.0, cy + 38.0, "Z-ordered stack \u{2022} Hit test", card_w - 28.0, text_dim);
    let ow = card_w - 60.0;
    let oh = 50.0;
    let ox = cx + 30.0;
    let oy = cy + 60.0;
    compositor.push(SceneNode::Rect { x: ox, y: oy, w: ow, h: oh, color: [0.20, 0.18, 0.35, 0.7] });
    compositor.draw_text(TextNodeKey::new("Menu z:1000", 9.0, 12.0, None), ox + 4.0, oy + 4.0, text_dim);
    compositor.push(SceneNode::Rect { x: ox + 12.0, y: oy + 14.0, w: ow - 16.0, h: oh - 12.0, color: [0.30, 0.22, 0.50, 0.8] });
    compositor.draw_text(TextNodeKey::new("Modal z:1001", 9.0, 12.0, None), ox + 16.0, oy + 18.0, text_dim);
    compositor.push(SceneNode::Rect { x: ox + 24.0, y: oy + 28.0, w: ow - 32.0, h: oh - 24.0, color: [0.15, 0.40, 0.30, 0.9] });
    compositor.draw_text(TextNodeKey::new("Tooltip z:1002", 9.0, 12.0, None), ox + 28.0, oy + 32.0, text_dim);
    card_label(compositor, cx + 14.0, cy + card_h - 22.0, "ContextMenu \u{2022} Modal \u{2022} Tooltip", card_w - 28.0, text_dim);
}

pub(crate) fn card_animation(
    compositor: &mut Compositor, lay: CardLayout, text_dim: [f32; 4], frame: u64,
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "ANIMATION", PINK);
    card_label(compositor, cx + 14.0, cy + 38.0, "Tween \u{2022} Spring \u{2022} Keyframes", card_w - 28.0, text_dim);
    let aw = card_w - 48.0;
    let bar_h = 6.0;
    let bar_x = cx + 24.0;
    let easings = ["EaseInOut", "Spring", "Bounce", "Step"];
    for (i, name) in easings.iter().enumerate() {
        let by = cy + 60.0 + i as f32 * 22.0;
        compositor.draw_text(TextNodeKey::new(name, 9.0, 12.0, Some(60.0)), bar_x, by, text_dim);
        compositor.push(SceneNode::Rect { x: bar_x + 64.0, y: by + 2.0, w: aw - 64.0, h: bar_h, color: [0.12, 0.12, 0.20, 1.0] });
        let phase = (frame as f32 * 0.02 + i as f32 * 0.7).sin() * 0.5 + 0.5;
        let fill_w = (aw - 64.0) * phase;
        compositor.push(SceneNode::Rect { x: bar_x + 64.0, y: by + 2.0, w: fill_w, h: bar_h, color: PINK });
    }
    card_label(compositor, cx + 14.0, cy + card_h - 22.0, "31 easings \u{2022} Repeat \u{2022} Reverse", card_w - 28.0, text_dim);
}

pub(crate) fn card_vector_paths(
    compositor: &mut Compositor, lay: CardLayout,
    green: [f32; 4], yellow: [f32; 4], cyan: [f32; 4], text_dim: [f32; 4],
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "VECTOR PATHS", cyan);
    card_label(compositor, cx + 14.0, cy + 38.0, "Lyon tessellation \u{2022} Reuses quad", card_w - 28.0, text_dim);
    let px = cx + 20.0;
    let py = cy + 60.0;
    let sw = (card_w - 40.0) / 3.0;
    let shapes = ["Triangle", "Star", "Bezier"];
    let shape_colors = [green, yellow, PURPLE];
    for (i, (name, color)) in shapes.iter().zip(shape_colors.iter()).enumerate() {
        let sx = px + i as f32 * sw;
        compositor.push(SceneNode::Rect { x: sx + 4.0, y: py, w: sw - 8.0, h: sw - 8.0, color: [color[0] * 0.3, color[1] * 0.3, color[2] * 0.3, 0.5] });
        compositor.draw_text(TextNodeKey::new(name, 9.0, 12.0, Some(sw)), sx + 4.0, py + sw - 4.0, *color);
    }
    card_label(compositor, cx + 14.0, cy + card_h - 22.0, "PathBuilder \u{2022} Fill \u{2022} No new shader", card_w - 28.0, text_dim);
}
