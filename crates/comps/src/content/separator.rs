//! Hairline separators in the divider tone, one rim width thick.

use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::theme::Theme;

use crate::core::Rect;

/// Separator direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Separator {
    /// A horizontal line across the width of its bounds.
    #[default]
    Horizontal,
    /// A vertical line down the height of its bounds.
    Vertical,
}

impl Separator {
    /// Line thickness: the rim width.
    pub fn thickness(theme: &Theme) -> f32 {
        theme.control.edge_width
    }

    /// The line rect inside `bounds`: full width (or height) of the
    /// bounds, centered on the other axis.
    pub fn line_rect(self, bounds: Rect, theme: &Theme) -> Rect {
        let t = Self::thickness(theme);
        match self {
            Separator::Horizontal => {
                Rect::new(bounds.x, bounds.y + (bounds.h - t) / 2.0, bounds.w, t)
            }
            Separator::Vertical => {
                Rect::new(bounds.x + (bounds.w - t) / 2.0, bounds.y, t, bounds.h)
            }
        }
    }

    pub fn render(self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let r = self.line_rect(bounds, theme);
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
                color: theme.colors.divider.0,
            },
        );
    }
}
