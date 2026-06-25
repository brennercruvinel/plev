//! Column header — HOFF social column head: 68px tall, 12px padding,
//! title in the `title` mixin (20px / 1.2 / 500) at rgba($n2,.56),
//! optional count chip on the right.

use super::badge;
use crate::theme::Theme;
use engine::compositor::{Compositor, SceneNode, TextNodeKey};

pub const HEADER_H: f32 = 68.0;
const PAD_X: f32 = 12.0;
const TITLE_SIZE: f32 = 20.0;
const TITLE_LINE_H: f32 = 20.0 * 1.2;

/// Draw a column header with title and optional count chip.
/// Returns the header height.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    w: f32,
    title: &str,
    badge_text: Option<&str>,
) -> f32 {
    compositor.push(SceneNode::Text {
        key: TextNodeKey::new(title, TITLE_SIZE, TITLE_LINE_H, None).with_weight(500),
        x: x + PAD_X,
        y: y + (HEADER_H - TITLE_LINE_H) / 2.0,
        color: theme.text_default.to_array(),
    });

    if let Some(text) = badge_text {
        // Real chip footprint (same measurement badge::draw uses), so the
        // right-aligned chip never overruns the panel edge.
        let badge_w = badge::tag_width(text);
        badge::draw(
            compositor,
            theme,
            x + w - PAD_X - badge_w,
            y + (HEADER_H - 22.0) / 2.0,
            text,
            badge::BadgeKind::Tag,
        );
    }

    HEADER_H
}
