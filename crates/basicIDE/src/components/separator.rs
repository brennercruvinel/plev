use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode};

/// Emit a 1px horizontal divider at (x, y) of width w.
pub fn horizontal(compositor: &mut Compositor, theme: &Theme, x: f32, y: f32, w: f32) {
    compositor.push(SceneNode::Rect {
        x,
        y,
        w,
        h: 1.0,
        color: theme.border.to_array(),
    });
}

/// Emit a 1px vertical divider at (x, y) of height h.
pub fn vertical(compositor: &mut Compositor, theme: &Theme, x: f32, y: f32, h: f32) {
    compositor.push(SceneNode::Rect {
        x,
        y,
        w: 1.0,
        h,
        color: theme.border.to_array(),
    });
}
