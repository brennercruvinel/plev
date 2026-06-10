//! Tabs — HOFF `components/Tabs`: container radius 22, padding 4,
//! bg rgba($n3,.6); equal-width 36px buttons in base-2sm (14/600)
//! $text-secondary -> active $text-primary; the active block is a
//! radius-18 glass slab (bg rgba($n2,.05)) with the spec shadow
//! `0 8px 16px -4px rgba(18,18,18,.20)` and an edge-light rim.

use super::hoff;
use crate::theme::{SHADOW_TABS_BLOCK, Theme};
use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};

const PAD: f32 = 4.0;
const BTN_H: f32 = 36.0;
pub const TABS_H: f32 = BTN_H + PAD * 2.0;
const FONT_SIZE: f32 = 14.0;
const LINE_H: f32 = 14.0 * 1.4;

/// Draw a horizontal tab bar. Returns hit rects (index, x, y, w, h) for each tab.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    w: f32,
    labels: &[&str],
    active: usize,
) -> Vec<(usize, f32, f32, f32, f32)> {
    // Container pill.
    compositor.push(SceneNode::RoundedRect {
        x,
        y,
        w,
        h: TABS_H,
        color: theme.bg_tabs.to_array(),
        corner_radius: theme.radius_tabs,
        border_width: 0.0,
        border_color: [0.0; 4],
    });

    if labels.is_empty() {
        return Vec::new();
    }
    let btn_w = (w - PAD * 2.0) / labels.len() as f32;

    // Active sliding block: glass slab + spec shadow + edge-light rim.
    let active_x = x + PAD + active.min(labels.len() - 1) as f32 * btn_w;
    hoff::shadow(
        compositor,
        LayerId::DEFAULT,
        active_x,
        y + PAD,
        btn_w,
        BTN_H,
        theme.radius_block,
        &SHADOW_TABS_BLOCK,
    );
    hoff::glass(
        compositor,
        LayerId::DEFAULT,
        active_x,
        y + PAD,
        btn_w,
        BTN_H,
        theme.radius_block,
        theme.surface_hover,
        Some((1.5, theme.edge_strong)),
    );

    let mut hit_rects = Vec::with_capacity(labels.len());
    for (i, label) in labels.iter().enumerate() {
        let tx = x + PAD + i as f32 * btn_w;
        let is_active = i == active;
        let text_w = hoff::text_width(label, FONT_SIZE);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new(label, FONT_SIZE, LINE_H, None).with_weight(600),
            x: tx + (btn_w - text_w) / 2.0,
            y: y + PAD + (BTN_H - LINE_H) / 2.0,
            color: if is_active {
                theme.text_primary
            } else {
                theme.text_secondary
            }
            .to_array(),
        });
        hit_rects.push((i, tx, y + PAD, btn_w, BTN_H));
    }

    hit_rects
}
