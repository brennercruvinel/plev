use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;
use super::badge;

const HEADER_H: f32 = 44.0;
const PAD_X: f32 = 12.0;

/// Draw a panel header with title and optional badge. Returns the header height.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    w: f32,
    title: &str,
    badge_text: Option<&str>,
) -> f32 {
    compositor.push(SceneNode::Rect {
        x, y, w, h: HEADER_H,
        color: theme.bg_2.to_array(),
    });

    compositor.push(SceneNode::Text {
        key: TextNodeKey::new(title, 12.0, 16.0, None).with_weight(600),
        x: x + PAD_X,
        y: y + 14.0,
        color: theme.text_2.to_array(),
    });

    if let Some(text) = badge_text {
        badge::draw(
            compositor, theme,
            x + w - PAD_X - 28.0, y + 13.0,
            text,
            badge::BadgeKind::Neutral,
        );
    }

    // Divider
    compositor.push(SceneNode::Rect {
        x, y: y + HEADER_H, w, h: 1.0,
        color: theme.border.to_array(),
    });

    HEADER_H + 1.0
}
