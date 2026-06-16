use phi::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use phi::overlay::MenuItem;
use crate::theme::Theme;

const MENU_W: f32 = 160.0;
const ITEM_H: f32 = 32.0;
const PAD_Y: f32 = 4.0;  // top/bottom padding inside the menu
const PAD_X: f32 = 10.0;
const FONT_SIZE: f32 = 13.0;

/// Draw a context menu onto `layer_id` and return per-item hit rects.
///
/// `x`, `y` is the top-left of the menu (from [`phi::overlay::Overlay`]).
/// Returns `(menu_w, menu_h, item_rects)` so the caller can call
/// [`phi::overlay::OverlayManager::set_bounds`] on the first frame.
pub fn draw(
    compositor: &mut Compositor,
    layer_id: LayerId,
    theme: &Theme,
    x: f32,
    y: f32,
    items: &[MenuItem],
    hover_item: Option<usize>,
) -> (f32, f32, Vec<(f32, f32, f32, f32)>) {
    let menu_h = PAD_Y * 2.0 + items.len() as f32 * ITEM_H;

    // Background + border
    compositor.push_to_layer(layer_id, SceneNode::RoundedRect {
        x,
        y,
        w: MENU_W,
        h: menu_h,
        color: theme.bg_2.to_array(),
        corner_radius: theme.radius_s,
        border_width: 1.0,
        border_color: theme.border.to_array(),
    });

    let mut item_rects = Vec::with_capacity(items.len());

    for (i, item) in items.iter().enumerate() {
        let iy = y + PAD_Y + i as f32 * ITEM_H;

        // Hover highlight
        if hover_item == Some(i) {
            compositor.push_to_layer(layer_id, SceneNode::Rect {
                x: x + 2.0,
                y: iy + 1.0,
                w: MENU_W - 4.0,
                h: ITEM_H - 2.0,
                color: theme.hover_bg_2.to_array(),
            });
        }

        let text_color = if hover_item == Some(i) { theme.text_1 } else { theme.text_2 };

        compositor.push_to_layer(layer_id, SceneNode::Text {
            key: TextNodeKey::new(&item.label, FONT_SIZE, FONT_SIZE * 1.4, Some(MENU_W - PAD_X * 2.0)),
            x: x + PAD_X,
            y: iy + (ITEM_H - FONT_SIZE * 1.4) / 2.0,
            color: text_color.to_array(),
        });

        item_rects.push((x, iy, MENU_W, ITEM_H));
    }

    (MENU_W, menu_h, item_rects)
}
