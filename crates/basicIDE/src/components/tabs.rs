//! Tabs — HOFF `components/Tabs`: container radius 22, padding 4,
//! bg rgba($n3,.6); equal-width 36px buttons in base-2sm (14/600)
//! $text-secondary -> active $text-primary; the active block is a
//! radius-18 glass slab (bg rgba($n2,.05)) with the spec shadow
//! `0 8px 16px -4px rgba(18,18,18,.20)` and an edge-light rim.

use super::hoff;
use crate::theme::{SHADOW_TABS_BLOCK, Theme};
use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use plev::text::TextStyle;

const PAD: f32 = 4.0;
const BTN_H: f32 = 36.0;
pub const TABS_H: f32 = BTN_H + PAD * 2.0;
const FONT_SIZE: f32 = 14.0;
const LINE_H: f32 = 14.0 * 1.4;
/// Minimum horizontal breathing room between a label and its segment edge.
const BTN_PAD_X_MIN: f32 = 12.0;

/// The base-2sm (14/600) style tab labels are measured and drawn with.
fn label_style() -> TextStyle {
    TextStyle::new(FONT_SIZE)
        .with_line_height(LINE_H)
        .with_weight(600)
}

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
    let style = label_style();
    // Equal-width segments, but never narrower than the widest REAL shaped
    // label + minimum padding: a tight container must not let labels leak
    // out of their segment (min-content clamp).
    let equal_w = (w - PAD * 2.0) / labels.len() as f32;
    let min_content = labels
        .iter()
        .map(|l| hoff::measure_text(l, &style) + BTN_PAD_X_MIN * 2.0)
        .fold(0.0_f32, f32::max);
    let btn_w = equal_w.max(min_content);

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
        let text_w = hoff::measure_text(label, &style);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(label, &style, None),
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

#[cfg(test)]
mod tests {
    use super::*;
    use plev::compositor::Compositor;

    /// Segments are clamped to min-content: in a container too narrow for
    /// its labels, each segment still fits its widest REAL shaped label
    /// plus the minimum padding (text must never leak out of a segment).
    #[test]
    fn segment_width_clamps_to_widest_measured_label() {
        let mut c = Compositor::new();
        c.begin_frame();
        let labels = ["Unstaged changes", "Staged"];
        // 120px is far too narrow for two segments of these labels.
        let rects = draw(&mut c, &crate::theme::DARK, 0.0, 0.0, 120.0, &labels, 0);
        let widest = labels
            .iter()
            .map(|l| hoff::measure_text(l, &label_style()))
            .fold(0.0_f32, f32::max);
        for (_, _, _, w, _) in &rects {
            assert!(
                *w >= widest + 2.0 * BTN_PAD_X_MIN - 1e-3,
                "segment {w} must fit widest label {widest} + 2*{BTN_PAD_X_MIN}"
            );
        }
    }

    /// With enough room the segments stay equal-width (spec behavior).
    #[test]
    fn segments_share_width_equally_when_space_allows() {
        let mut c = Compositor::new();
        c.begin_frame();
        let rects = draw(
            &mut c,
            &crate::theme::DARK,
            0.0,
            0.0,
            600.0,
            &["One", "Two", "Three"],
            1,
        );
        let expect = (600.0 - PAD * 2.0) / 3.0;
        for (_, _, _, w, _) in &rects {
            assert!((w - expect).abs() < 1e-3);
        }
    }
}
