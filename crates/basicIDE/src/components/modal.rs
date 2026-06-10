//! Confirmation modal — HOFF Modal recipe (`components/Modal`):
//! overlay rgba(35,34,34,.9); container max-width 400, radius 32,
//! padding 32; surface rgba(40,40,40,.7) + deep shadow stack +
//! edge-light 1.5px rgba(255,255,255,.05) (mask 175deg -> 60%).

use super::button::{ButtonKind, ButtonSize, draw_to_layer as draw_button};
use super::hoff;
use crate::theme::{SHADOW_MODAL, Theme};
use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};

const MODAL_W: f32 = 400.0;
const MODAL_H: f32 = 216.0;
const PAD: f32 = 32.0;
/// Title: title mixin (20px / 1.2 / 500).
const TITLE_SIZE: f32 = 20.0;
const TITLE_LINE_H: f32 = 20.0 * 1.2;
/// Body: body-2r (14px / 1.7 / 400).
const BODY_SIZE: f32 = 14.0;
const BODY_LINE_H: f32 = 14.0 * 1.7;
const BTN_GAP: f32 = 8.0;

/// Draw a confirmation modal onto `layer_id`.
///
/// `x`, `y` is the top-left of the dialog box (pre-centered by caller).
/// Returns `(confirm_rect, cancel_rect)` — callers use these for hit-testing.
#[allow(clippy::too_many_arguments)]
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
    // Full-screen overlay: rgba(35,34,34,.9).
    compositor.push_to_layer(
        layer_id,
        SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h: vh,
            color: theme.bg_overlay.to_array(),
        },
    );

    // Deep shadow stack, then glass container with top-lit rim.
    hoff::shadow_stack(
        compositor,
        layer_id,
        x,
        y,
        MODAL_W,
        MODAL_H,
        theme.radius_pill,
        &SHADOW_MODAL,
    );
    hoff::glass(
        compositor,
        layer_id,
        x,
        y,
        MODAL_W,
        MODAL_H,
        theme.radius_pill,
        theme.bg_panel,
        Some((1.5, theme.edge)),
    );

    // Title — 20/500, text-primary (.95).
    compositor.push_to_layer(
        layer_id,
        SceneNode::Text {
            key: TextNodeKey::new(title, TITLE_SIZE, TITLE_LINE_H, Some(MODAL_W - PAD * 2.0))
                .with_weight(500),
            x: x + PAD,
            y: y + PAD,
            color: theme.text_primary.to_array(),
        },
    );

    // Body — 14/1.7, text-secondary (.70).
    compositor.push_to_layer(
        layer_id,
        SceneNode::Text {
            key: TextNodeKey::new(body, BODY_SIZE, BODY_LINE_H, Some(MODAL_W - PAD * 2.0)),
            x: x + PAD,
            y: y + PAD + TITLE_LINE_H + 12.0,
            color: theme.text_secondary.to_array(),
        },
    );

    // Buttons — 44px pills, right-aligned at the bottom.
    let btn_h = 44.0;
    let btn_area_y = y + MODAL_H - btn_h - PAD;
    let confirm_w = hoff::text_width(confirm_label, 14.0) + 24.0 * 2.0;
    let cancel_w = hoff::text_width(cancel_label, 14.0) + 24.0 * 2.0;
    let confirm_x = x + MODAL_W - PAD - confirm_w;
    let cancel_x = confirm_x - BTN_GAP - cancel_w;

    let cancel_rect = draw_button(
        compositor,
        layer_id,
        theme,
        cancel_x,
        btn_area_y,
        cancel_label,
        ButtonKind::Glass,
        ButtonSize::Md,
        hover_cancel,
        false,
    );
    let confirm_rect = draw_button(
        compositor,
        layer_id,
        theme,
        confirm_x,
        btn_area_y,
        confirm_label,
        ButtonKind::Danger,
        ButtonSize::Md,
        hover_confirm,
        false,
    );

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
