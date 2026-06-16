//! Card construction functions for showcase rows 1 and 2.

use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::input::InputState;

use super::card_types::*;
use super::helpers::{card, card_label, card_title};
use super::ShowcaseState;

pub(crate) fn card_quad_rendering(
    compositor: &mut Compositor, lay: CardLayout, colors: &CardColors,
) {
    let CardColors { accent, green, red, yellow, cyan, text_dim, .. } = *colors;
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "QUAD RENDERING", accent);
    card_label(compositor, cx + 14.0, cy + 38.0, "Alpha blending \u{2022} Premultiplied", card_w - 28.0, text_dim);
    let qx = cx + 20.0;
    let qy = cy + 62.0;
    compositor.push(SceneNode::Rect { x: qx, y: qy, w: 105.0, h: 75.0, color: [0.15, 0.35, 0.85, 0.9] });
    compositor.push(SceneNode::Rect { x: qx + 40.0, y: qy + 15.0, w: 55.0, h: 55.0, color: [0.85, 0.20, 0.25, 0.8] });
    compositor.push(SceneNode::Rect { x: qx + 75.0, y: qy + 5.0, w: 55.0, h: 65.0, color: [0.15, 0.75, 0.35, 0.7] });
    compositor.push(SceneNode::Rect { x: qx + 25.0, y: qy + 40.0, w: 85.0, h: 40.0, color: [0.90, 0.75, 0.10, 0.6] });
    let sw = 14.0;
    let sy = cy + card_h - 46.0;
    let colors = [accent, red, green, yellow, PURPLE, PINK, cyan, ORANGE];
    for (i, c) in colors.iter().enumerate() {
        compositor.push(SceneNode::Rect { x: cx + 14.0 + i as f32 * (sw + 3.0), y: sy, w: sw, h: sw, color: *c });
    }
    card_label(compositor, cx + 14.0, sy + 18.0, "8 colors + hex + rgba", card_w - 28.0, text_dim);
}

pub(crate) fn card_text_system(
    compositor: &mut Compositor, lay: CardLayout, colors: &CardColors,
) {
    let CardColors { cyan, text, text_dim, text_mid, .. } = *colors;
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "TEXT SYSTEM", cyan);
    card_label(compositor, cx + 14.0, cy + 38.0, "cosmic-text \u{2022} HarfBuzz \u{2022} Atlas", card_w - 28.0, text_dim);
    let tx = cx + 14.0;
    let mw = card_w - 28.0;
    compositor.draw_text(TextNodeKey::new("36px Title", 36.0, 42.0, Some(mw)), tx, cy + 62.0, text);
    compositor.draw_text(TextNodeKey::new("20px Subtitle", 20.0, 26.0, Some(mw)), tx, cy + 108.0, text_mid);
    compositor.draw_text(TextNodeKey::new("14px Body text with Unicode shaping.", 14.0, 19.0, Some(mw)), tx, cy + 138.0, text_dim);
    compositor.draw_text(TextNodeKey::new("11px Caption \u{2022} 0123456789", 11.0, 15.0, Some(mw)), tx, cy + card_h - 28.0, [0.45, 0.45, 0.55, 1.0]);
}

pub(crate) fn card_layer_system(
    compositor: &mut Compositor, lay: CardLayout, green: [f32; 4], text_dim: [f32; 4],
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "LAYER SYSTEM", green);
    card_label(compositor, cx + 14.0, cy + 38.0, "Per-layer dirty tracking", card_w - 28.0, text_dim);
    let lx = cx + 30.0;
    let ly = cy + 65.0;
    let lw = card_w - 60.0;
    let lh = 80.0;
    compositor.push(SceneNode::Rect { x: lx, y: ly, w: lw, h: lh, color: [0.25, 0.15, 0.50, 0.6] });
    compositor.draw_text(TextNodeKey::new("Layer -1 (bg)", 10.0, 13.0, Some(lw - 12.0)), lx + 6.0, ly + 6.0, [0.7, 0.6, 0.9, 0.8]);
    compositor.push(SceneNode::Rect { x: lx + 15.0, y: ly + 18.0, w: lw - 10.0, h: lh - 20.0, color: [0.15, 0.30, 0.55, 0.7] });
    compositor.draw_text(TextNodeKey::new("Layer 0 (default)", 10.0, 13.0, Some(lw - 30.0)), lx + 21.0, ly + 24.0, [0.6, 0.7, 1.0, 0.9]);
    compositor.push(SceneNode::Rect { x: lx + 30.0, y: ly + 36.0, w: lw - 20.0, h: lh - 40.0, color: [0.20, 0.55, 0.30, 0.8] });
    compositor.draw_text(TextNodeKey::new("Layer 1 (fg, 85%)", 10.0, 13.0, Some(lw - 50.0)), lx + 36.0, ly + 42.0, [0.6, 1.0, 0.7, 1.0]);
    card_label(compositor, cx + 14.0, cy + card_h - 40.0, "Offscreen textures \u{2022} Composite", card_w - 28.0, text_dim);
    card_label(compositor, cx + 14.0, cy + card_h - 24.0, "Hash skip = zero GPU work", card_w - 28.0, text_dim);
}

pub(crate) fn card_effects(
    compositor: &mut Compositor, lay: CardLayout, text_dim: [f32; 4], text_mid: [f32; 4],
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "EFFECTS", PURPLE);
    card_label(compositor, cx + 14.0, cy + 38.0, "Blur \u{2022} Shadow \u{2022} Opacity", card_w - 28.0, text_dim);
    let ex = cx + 24.0;
    let ey = cy + 65.0;
    let ew = card_w - 48.0;
    let eh = 50.0;
    for i in 0..5 {
        let spread = (5 - i) as f32 * 2.0;
        let alpha = 0.03 + i as f32 * 0.02;
        compositor.push(SceneNode::Rect { x: ex + 4.0 - spread, y: ey + 4.0 - spread, w: ew + spread * 2.0, h: eh + spread * 2.0, color: [0.0, 0.0, 0.0, alpha] });
    }
    compositor.push(SceneNode::Rect { x: ex, y: ey, w: ew, h: eh, color: [0.18, 0.18, 0.30, 1.0] });
    compositor.draw_text(TextNodeKey::new("13-tap Gaussian", 12.0, 16.0, Some(ew - 16.0)), ex + 8.0, ey + 8.0, text_mid);
    compositor.draw_text(TextNodeKey::new("Separable H+V", 12.0, 16.0, Some(ew - 16.0)), ex + 8.0, ey + 28.0, text_dim);
    let by = cy + 130.0;
    for i in 0..6 {
        let alpha = 1.0 - i as f32 * 0.15;
        let spread = i as f32 * 1.5;
        compositor.push(SceneNode::Rect { x: ex + spread, y: by + i as f32 * 7.0, w: ew - spread * 2.0, h: 5.0, color: [0.50, 0.25, 0.85, alpha] });
    }
    card_label(compositor, cx + 14.0, cy + card_h - 24.0, "Fragment-only \u{2022} TexturePool", card_w - 28.0, text_dim);
}

pub(crate) fn card_builder_api(
    compositor: &mut Compositor, lay: CardLayout, colors: &CardColors,
) {
    let CardColors { accent, green, red, yellow, text, text_dim, text_mid, .. } = *colors;
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "BUILDER API", yellow);
    card_label(compositor, cx + 14.0, cy + 38.0, "Declarative \u{2022} #[component]", card_w - 28.0, text_dim);
    let tx = cx + 14.0;
    let mw = card_w - 28.0;
    let lines: &[(&str, [f32; 4])] = &[
        ("div()", accent), ("  .col().gap(16.0)", text_mid),
        ("  .bg(hex(0x1a1a2e))", text_mid), ("  .child(", text_dim),
        ("    text(\"Hello\")", green), ("  )", text_dim),
    ];
    for (i, (line, color)) in lines.iter().enumerate() {
        compositor.draw_text(TextNodeKey::new(line, 11.0, 14.0, Some(mw)), tx, cy + 58.0 + i as f32 * 15.0, *color);
    }
    let by = cy + card_h - 46.0;
    let bw = 58.0;
    let btns = [(accent, "Primary"), (red, "Danger"), (green, "Success")];
    for (i, (color, label)) in btns.iter().enumerate() {
        let bx = cx + 14.0 + i as f32 * (bw + 5.0);
        compositor.push(SceneNode::Rect { x: bx, y: by, w: bw, h: 20.0, color: *color });
        compositor.draw_text(TextNodeKey::new(label, 10.0, 13.0, Some(bw - 8.0)), bx + 4.0, by + 4.0, text);
    }
    card_label(compositor, cx + 14.0, cy + card_h - 22.0, "div() \u{2022} text() \u{2022} button()", card_w - 28.0, text_dim);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn card_input_system(
    showcase: &mut ShowcaseState, compositor: &mut Compositor, input_state: &mut InputState,
    lay: CardLayout, accent: [f32; 4], green: [f32; 4],
    text: [f32; 4], text_dim: [f32; 4], text_mid: [f32; 4],
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "INPUT SYSTEM", PINK);
    card_label(compositor, cx + 14.0, cy + 38.0, "Mouse \u{2022} Touch \u{2022} Gestures", card_w - 28.0, text_dim);
    let btn_x = cx + 24.0;
    let btn_y = cy + 64.0;
    let btn_w = card_w - 48.0;
    let btn_h = 40.0;
    let btn_color = if showcase.btn_hovered { HOVER } else { accent };
    compositor.push(SceneNode::Rect { x: btn_x, y: btn_y, w: btn_w, h: btn_h, color: btn_color });
    compositor.draw_text(TextNodeKey::new("Click me!", 16.0, 21.0, Some(btn_w - 20.0)), btn_x + 10.0, btn_y + 10.0, text);
    let counter_text = format!("Clicks: {}", showcase.click_count);
    compositor.push(SceneNode::Rect { x: cx + 14.0, y: btn_y + btn_h + 8.0, w: card_w - 28.0, h: 26.0, color: [0.12, 0.12, 0.20, 1.0] });
    compositor.draw_text(
        TextNodeKey::new(&counter_text, 14.0, 18.0, Some(card_w - 48.0)),
        cx + 24.0, btn_y + btn_h + 12.0,
        if showcase.click_count > 0 { green } else { text_mid },
    );
    let gy = btn_y + btn_h + 44.0;
    let gestures = ["Tap", "Double-tap", "Long-press", "Swipe", "Drag", "Pinch"];
    for (i, g) in gestures.iter().enumerate() {
        compositor.draw_text(
            TextNodeKey::new(g, 10.0, 13.0, Some(64.0)),
            cx + 14.0 + (i % 3) as f32 * 62.0, gy + (i / 3) as f32 * 14.0, text_dim,
        );
    }
    let btn_id = input_state.next_view_id();
    input_state.register_hit_region(btn_id, btn_x, btn_y, btn_w, btn_h, true);
    showcase.btn_view_id = Some(btn_id);
}

pub(crate) fn card_signals(
    compositor: &mut Compositor, lay: CardLayout, orange: [f32; 4], text_dim: [f32; 4],
    counter_value: u64,
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "SIGNALS", orange);
    card_label(compositor, cx + 14.0, cy + 38.0, "Push-pull reactive \u{2022} SlotMap", card_w - 28.0, text_dim);
    let count_text = format!("{}", counter_value);
    compositor.push(SceneNode::Rect { x: cx + 14.0, y: cy + 58.0, w: card_w - 28.0, h: 55.0, color: [0.12, 0.10, 0.08, 1.0] });
    compositor.draw_text(TextNodeKey::new(&count_text, 38.0, 46.0, Some(card_w - 48.0)), cx + 24.0, cy + 64.0, orange);
    compositor.draw_text(TextNodeKey::new("frames", 11.0, 14.0, Some(80.0)), cx + 24.0, cy + 103.0, text_dim);
    let dy = cy + 124.0;
    compositor.draw_text(TextNodeKey::new("Signal<u64> \u{2192} ReadSignal \u{2192} View", 10.0, 13.0, Some(card_w - 28.0)), cx + 14.0, dy, text_dim);
    compositor.draw_text(TextNodeKey::new("Component<T> + Lifecycle trait", 10.0, 13.0, Some(card_w - 28.0)), cx + 14.0, dy + 16.0, text_dim);
    card_label(compositor, cx + 14.0, cy + card_h - 22.0, "create_signal() \u{2022} get/set", card_w - 28.0, text_dim);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn card_platforms(
    compositor: &mut Compositor, lay: CardLayout, accent: [f32; 4],
    green: [f32; 4], yellow: [f32; 4], red: [f32; 4], cyan: [f32; 4], text_dim: [f32; 4],
) {
    let CardLayout { cx, cy, card_w, card_h, surface, accent_dim } = lay;
    card(compositor, cx, cy, card_w, card_h, surface, accent_dim);
    card_title(compositor, cx + 14.0, cy + 14.0, "6 PLATFORMS", cyan);
    card_label(compositor, cx + 14.0, cy + 38.0, "One codebase \u{2022} Zero branches", card_w - 28.0, text_dim);
    let platforms = [
        ("\u{25CF} macOS", "Metal", accent), ("\u{25CF} iOS", "Metal", green),
        ("\u{25CF} Linux", "Vulkan", PURPLE), ("\u{25CF} Android", "Vulkan", yellow),
        ("\u{25CF} Windows", "DX12", red), ("\u{25CF} Browser", "WebGPU", cyan),
    ];
    for (i, (name, backend, color)) in platforms.iter().enumerate() {
        let py = cy + 60.0 + i as f32 * 20.0;
        compositor.draw_text(TextNodeKey::new(name, 12.0, 16.0, Some(100.0)), cx + 14.0, py, *color);
        compositor.draw_text(TextNodeKey::new(backend, 10.0, 13.0, Some(80.0)), cx + card_w - 62.0, py + 1.0, text_dim);
    }
    card_label(compositor, cx + 14.0, cy + card_h - 22.0, "Safe areas \u{2022} IME \u{2022} Lifecycle", card_w - 28.0, text_dim);
}

