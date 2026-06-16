use phi::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;

/// Draw an avatar circle with an initial letter inside.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    size: f32,
    initial: &str,
) {
    // Circle background (using rounded rect with radius = size/2)
    compositor.push(SceneNode::RoundedRect {
        x, y, w: size, h: size,
        color: theme.bg_3.to_array(),
        corner_radius: size / 2.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    });

    // Initial letter
    let font_size = (size * 0.45).round();
    compositor.push(SceneNode::Text {
        key: TextNodeKey::new(initial, font_size, font_size, None).with_weight(600),
        x: x + (size - font_size * 0.6) / 2.0,
        y: y + (size - font_size) / 2.0,
        color: theme.text_2.to_array(),
    });
}
