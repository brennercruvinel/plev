//! Shared helper functions for card drawing and theme palette extraction.

use crate::compositor::{Compositor, RoundedRectParams, SceneNode, TextNodeKey};
use crate::theme::Theme;

/// Derive clear color from theme background.
pub fn clear_color(theme: &Theme) -> [f64; 4] {
    let bg = theme.colors.bg.to_array();
    [f64::from(bg[0]), f64::from(bg[1]), f64::from(bg[2]), 1.0]
}

/// Resolved color palette from theme tokens, used for scene building.
pub(crate) type Palette = (
    [f32; 4], [f32; 4], [f32; 4], [f32; 4],
    [f32; 4], [f32; 4], [f32; 4], [f32; 4],
    [f32; 4], [f32; 4], [f32; 4], [f32; 4],
);

/// Derive palette arrays from theme tokens for scene building.
pub(crate) fn palette(theme: &Theme) -> Palette {
    (
        theme.colors.bg.to_array(),
        theme.colors.surface.to_array(),
        theme.colors.accent.to_array(),
        theme.colors.accent_dim.to_array(),
        theme.colors.success.to_array(),
        theme.colors.danger.to_array(),
        theme.colors.warning.to_array(),
        theme.colors.info.to_array(),
        theme.colors.text.to_array(),
        theme.colors.text_dim.to_array(),
        theme.colors.text_mid.to_array(),
        theme.colors.divider.to_array(),
    )
}

pub(crate) fn card(
    compositor: &mut Compositor,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    surface: [f32; 4],
    accent_dim: [f32; 4],
) {
    compositor.draw_rounded_rect(RoundedRectParams {
        x, y, w, h, color: surface, corner_radius: 6.0,
        border_width: 0.0, border_color: [0.0; 4],
    });
    compositor.push(SceneNode::Rect {
        x: x + 6.0,
        y,
        w: w - 12.0,
        h: 2.0,
        color: accent_dim,
    });
}

pub(crate) fn card_title(
    compositor: &mut Compositor,
    x: f32,
    y: f32,
    title: &str,
    color: [f32; 4],
) {
    compositor.draw_text(TextNodeKey::new(title, 15.0, 20.0, Some(200.0)), x, y, color);
}

pub(crate) fn card_label(
    compositor: &mut Compositor,
    x: f32,
    y: f32,
    text: &str,
    w: f32,
    text_dim: [f32; 4],
) {
    compositor.draw_text(TextNodeKey::new(text, 11.0, 15.0, Some(w)), x, y, text_dim);
}
