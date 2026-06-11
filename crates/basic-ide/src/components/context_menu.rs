//! Context menu — the HOFF Actions dropdown (`components/Actions`):
//! 240px body, radius 24, padding 8, solid #3b3b3b, floating-menu shadow
//! `0 24px 32px -12px rgba(18,18,18,.10)` + edge-light rim; items 44px
//! radius 16 base-2sm rgba($n2,.56) -> hover bg rgba($n2,.1) text .76.

use super::hoff;
use crate::theme::{SHADOW_MENU, Theme};
use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use plev::overlay::MenuItem;

const MENU_W: f32 = 240.0;
const ITEM_H: f32 = 44.0;
const PAD: f32 = 8.0; // padding around the item column
const ITEM_PAD_X: f32 = 8.0;
const FONT_SIZE: f32 = 14.0;
const LINE_H: f32 = 14.0 * 1.4;

/// Draw a context menu onto `layer_id` and return per-item hit rects.
///
/// `x`, `y` is the top-left of the menu (from [`plev::overlay::Overlay`]).
/// Returns `(menu_w, menu_h, item_rects)` so the caller can call
/// [`plev::overlay::OverlayManager::set_bounds`] on the first frame.
pub fn draw(
    compositor: &mut Compositor,
    layer_id: LayerId,
    theme: &Theme,
    x: f32,
    y: f32,
    items: &[MenuItem],
    hover_item: Option<usize>,
) -> (f32, f32, Vec<(f32, f32, f32, f32)>) {
    let menu_h = PAD * 2.0 + items.len() as f32 * ITEM_H;

    // Floating-menu drop shadow (analytic), then solid body + edge-light.
    hoff::shadow(
        compositor,
        layer_id,
        x,
        y,
        MENU_W,
        menu_h,
        theme.radius_dropdown,
        &SHADOW_MENU,
    );
    hoff::glass(
        compositor,
        layer_id,
        x,
        y,
        MENU_W,
        menu_h,
        theme.radius_dropdown,
        theme.bg_popover,
        Some((1.0, theme.edge_strong)),
    );

    let mut item_rects = Vec::with_capacity(items.len());

    for (i, item) in items.iter().enumerate() {
        let iy = y + PAD + i as f32 * ITEM_H;
        let hovered = hover_item == Some(i);

        if hovered {
            compositor.push_to_layer(
                layer_id,
                SceneNode::RoundedRect {
                    x: x + PAD,
                    y: iy,
                    w: MENU_W - PAD * 2.0,
                    h: ITEM_H,
                    color: theme.surface_active.to_array(),
                    corner_radius: theme.radius_item,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
        }

        let text_color = if hovered {
            theme.text_active
        } else {
            theme.text_default
        };

        compositor.push_to_layer(
            layer_id,
            SceneNode::Text {
                key: TextNodeKey::new(
                    &item.label,
                    FONT_SIZE,
                    LINE_H,
                    Some(MENU_W - (PAD + ITEM_PAD_X) * 2.0),
                )
                .with_weight(600),
                x: x + PAD + ITEM_PAD_X,
                y: iy + (ITEM_H - LINE_H) / 2.0,
                color: text_color.to_array(),
            },
        );

        item_rects.push((x + PAD, iy, MENU_W - PAD * 2.0, ITEM_H));
    }

    (MENU_W, menu_h, item_rects)
}
