//! Avatar — HOFF `components/Avatar`: 44x44 circle. Without a photo the
//! fallback is a chip-glass disc (rgba($n2,.05)) with the initial letter
//! in weight 600 at rgba($n2,.76).

use super::hoff;
use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::text::TextStyle;

/// Draw an avatar circle with an initial letter inside.
pub fn draw(compositor: &mut Compositor, theme: &Theme, x: f32, y: f32, size: f32, initial: &str) {
    compositor.push(SceneNode::RoundedRect {
        x,
        y,
        w: size,
        h: size,
        color: theme.chip.to_array(),
        corner_radius: size / 2.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    });

    // Initial letter — weight 600, .76 (active text alpha). One style for
    // measuring and drawing keeps the glyph optically centered.
    let font_size = (size * 0.32).round().max(10.0);
    let style = TextStyle::new(font_size)
        .with_line_height(font_size)
        .with_weight(600);
    let text_w = hoff::measure_text(initial, &style);
    compositor.push(SceneNode::Text {
        key: TextNodeKey::from_style(initial, &style, None),
        x: x + (size - text_w) / 2.0,
        y: y + (size - font_size) / 2.0,
        color: theme.text_active.to_array(),
    });
}
