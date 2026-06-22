use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;

const SIZE: f32 = 16.0;
const BORDER_W: f32 = 1.5;

/// Draw a checkbox. Returns (x, y, w, h) hit rect.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    checked: bool,
    hovered: bool,
) -> (f32, f32, f32, f32) {
    let bg = if checked {
        theme.pop.to_array()
    } else if hovered {
        theme.hover_bg_2.to_array()
    } else {
        theme.bg_2.to_array()
    };

    let border = if checked {
        theme.pop.to_array()
    } else {
        theme.border.to_array()
    };

    compositor.push(SceneNode::RoundedRect {
        x, y, w: SIZE, h: SIZE,
        color: bg,
        corner_radius: 3.0,
        border_width: BORDER_W,
        border_color: border,
    });

    // Checkmark (codicon)
    if checked {
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new("\u{EAB2}", 12.0, 12.0, None)
                .with_weight(400)
                .with_family("codicon"),
            x: x + 2.0,
            y: y + 2.0,
            color: [1.0, 1.0, 1.0, 1.0],
        });
    }

    (x, y, SIZE, SIZE)
}
