use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};

const TAB_H: f32 = 32.0;
const TAB_PAD_X: f32 = 14.0;
const FONT_SIZE: f32 = 12.0;

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
    // Tab bar background
    compositor.push(SceneNode::Rect {
        x,
        y,
        w,
        h: TAB_H,
        color: theme.bg_2.to_array(),
    });

    let mut hit_rects = Vec::with_capacity(labels.len());
    let mut tx = x;

    for (i, label) in labels.iter().enumerate() {
        let tab_w = label.len() as f32 * FONT_SIZE * 0.6 + TAB_PAD_X * 2.0;
        let is_active = i == active;

        // Active underline
        if is_active {
            compositor.push(SceneNode::Rect {
                x: tx,
                y: y + TAB_H - 2.0,
                w: tab_w,
                h: 2.0,
                color: theme.pop.to_array(),
            });
        }

        // Label
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new(label, FONT_SIZE, 16.0, None).with_weight(if is_active {
                600
            } else {
                400
            }),
            x: tx + TAB_PAD_X,
            y: y + (TAB_H - 16.0) / 2.0,
            color: if is_active {
                theme.text_1
            } else {
                theme.text_3
            }
            .to_array(),
        });

        hit_rects.push((i, tx, y, tab_w, TAB_H));
        tx += tab_w;
    }

    // Bottom border
    compositor.push(SceneNode::Rect {
        x,
        y: y + TAB_H - 1.0,
        w,
        h: 1.0,
        color: theme.border.to_array(),
    });

    hit_rects
}
