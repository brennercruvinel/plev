//! Checkbox — HOFF field recipe: chip glass at rest (rgba($n2,.05),
//! border transparent), hover rgba($n2,.10); checked = "active" element:
//! bg rgba($n2,.10) + border 1.5px rgba($n2,.40) with the check mark in
//! text-primary. Radius 6 (micro-action).

use crate::theme::Theme;
use engine::compositor::{Compositor, SceneNode, TextNodeKey};

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
    let bg = if checked || hovered {
        theme.surface_active.to_array()
    } else {
        theme.field_bg.to_array()
    };

    let border = if checked {
        theme.border_active.to_array()
    } else {
        [0.0; 4]
    };

    compositor.push(SceneNode::RoundedRect {
        x,
        y,
        w: SIZE,
        h: SIZE,
        color: bg,
        corner_radius: theme.radius_micro,
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
            color: theme.text_primary.to_array(),
        });
    }

    (x, y, SIZE, SIZE)
}
