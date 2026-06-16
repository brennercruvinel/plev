use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use plev::color::Color;
use crate::theme::Theme;

const MODAL_W: f32 = 360.0;
const MODAL_H: f32 = 162.0;
const PAD: f32 = 20.0;
const TITLE_SIZE: f32 = 15.0;
const BODY_SIZE: f32 = 13.0;
const BTN_H: f32 = 32.0;
const BTN_PAD_X: f32 = 16.0;
const BTN_GAP: f32 = 8.0;

/// Draw a confirmation modal onto `layer_id`.
///
/// `x`, `y` is the top-left of the dialog box (pre-centered by caller).
/// Returns `(confirm_rect, cancel_rect)` — callers use these for hit-testing.
pub fn draw(
    compositor: &mut Compositor,
    layer_id: LayerId,
    theme: &Theme,
    vw: f32,
    vh: f32,
    x: f32,
    y: f32,
    title: &str,
    body: &str,
    confirm_label: &str,
    cancel_label: &str,
    hover_confirm: bool,
    hover_cancel: bool,
) -> ((f32, f32, f32, f32), (f32, f32, f32, f32)) {
    // Semi-transparent backdrop (full screen)
    compositor.push_to_layer(layer_id, SceneNode::Rect {
        x: 0.0,
        y: 0.0,
        w: vw,
        h: vh,
        color: Color::rgba(0.0, 0.0, 0.0, 0.45).to_array(),
    });

    // Dialog box
    compositor.push_to_layer(layer_id, SceneNode::RoundedRect {
        x,
        y,
        w: MODAL_W,
        h: MODAL_H,
        color: theme.bg_2.to_array(),
        corner_radius: theme.radius_ml,
        border_width: 1.0,
        border_color: theme.border.to_array(),
    });

    // Title
    compositor.push_to_layer(layer_id, SceneNode::Text {
        key: TextNodeKey::new(title, TITLE_SIZE, TITLE_SIZE * 1.3, Some(MODAL_W - PAD * 2.0))
            .with_weight(600),
        x: x + PAD,
        y: y + PAD,
        color: theme.text_1.to_array(),
    });

    // Body
    compositor.push_to_layer(layer_id, SceneNode::Text {
        key: TextNodeKey::new(body, BODY_SIZE, BODY_SIZE * 1.5, Some(MODAL_W - PAD * 2.0)),
        x: x + PAD,
        y: y + PAD + TITLE_SIZE * 1.3 + 8.0,
        color: theme.text_2.to_array(),
    });

    // Buttons — right-aligned at bottom
    let btn_area_y = y + MODAL_H - BTN_H - PAD;

    // Cancel button (ghost, on the left of confirm)
    let cancel_w = cancel_label.len() as f32 * BODY_SIZE * 0.6 + BTN_PAD_X * 2.0;
    let confirm_w = confirm_label.len() as f32 * BODY_SIZE * 0.6 + BTN_PAD_X * 2.0;

    let confirm_x = x + MODAL_W - PAD - confirm_w;
    let cancel_x = confirm_x - BTN_GAP - cancel_w;

    // Cancel (ghost)
    let cancel_bg = if hover_cancel {
        theme.hover_bg_3
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    };
    compositor.push_to_layer(layer_id, SceneNode::RoundedRect {
        x: cancel_x,
        y: btn_area_y,
        w: cancel_w,
        h: BTN_H,
        color: cancel_bg.to_array(),
        corner_radius: theme.radius_s,
        border_width: 1.0,
        border_color: theme.border.to_array(),
    });
    compositor.push_to_layer(layer_id, SceneNode::Text {
        key: TextNodeKey::new(cancel_label, BODY_SIZE, BODY_SIZE * 1.3, None).with_weight(500),
        x: cancel_x + BTN_PAD_X,
        y: btn_area_y + (BTN_H - BODY_SIZE * 1.3) / 2.0,
        color: theme.text_1.to_array(),
    });

    // Confirm (danger solid)
    let confirm_bg = if hover_confirm {
        Color::rgba(
            theme.danger.to_array()[0] * 0.85,
            theme.danger.to_array()[1] * 0.85,
            theme.danger.to_array()[2] * 0.85,
            1.0,
        )
    } else {
        theme.danger
    };
    compositor.push_to_layer(layer_id, SceneNode::RoundedRect {
        x: confirm_x,
        y: btn_area_y,
        w: confirm_w,
        h: BTN_H,
        color: confirm_bg.to_array(),
        corner_radius: theme.radius_s,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
    compositor.push_to_layer(layer_id, SceneNode::Text {
        key: TextNodeKey::new(confirm_label, BODY_SIZE, BODY_SIZE * 1.3, None).with_weight(500),
        x: confirm_x + BTN_PAD_X,
        y: btn_area_y + (BTN_H - BODY_SIZE * 1.3) / 2.0,
        color: Color::rgba(1.0, 1.0, 1.0, 1.0).to_array(),
    });

    let confirm_rect = (confirm_x, btn_area_y, confirm_w, BTN_H);
    let cancel_rect = (cancel_x, btn_area_y, cancel_w, BTN_H);
    (confirm_rect, cancel_rect)
}

/// Compute the centered top-left position for a modal given viewport dimensions.
pub fn centered_pos(vw: f32, vh: f32) -> (f32, f32) {
    (
        (vw / 2.0 - MODAL_W / 2.0).max(0.0),
        (vh / 2.0 - MODAL_H / 2.0).max(0.0),
    )
}

pub fn dimensions() -> (f32, f32) {
    (MODAL_W, MODAL_H)
}
