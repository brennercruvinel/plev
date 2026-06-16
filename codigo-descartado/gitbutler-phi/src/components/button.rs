use phi::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;

/// Button visual style variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonKind {
    Solid,
    Ghost,
    Danger,
}

/// Button size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

impl ButtonSize {
    fn padding(self) -> (f32, f32) {
        match self {
            ButtonSize::Sm => (6.0, 10.0),
            ButtonSize::Md => (8.0, 14.0),
            ButtonSize::Lg => (10.0, 18.0),
        }
    }
    fn font_size(self) -> f32 {
        match self {
            ButtonSize::Sm => 12.0,
            ButtonSize::Md => 13.0,
            ButtonSize::Lg => 14.0,
        }
    }
}

/// Draw a button and return its bounding box (x, y, w, h).
///
/// `hovered` should be driven by the app's hit-test result.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    label: &str,
    kind: ButtonKind,
    size: ButtonSize,
    hovered: bool,
    disabled: bool,
) -> (f32, f32, f32, f32) {
    let (pad_y, pad_x) = size.padding();
    let font_size = size.font_size();
    let text_w = label.len() as f32 * font_size * 0.58;
    let btn_w = text_w + pad_x * 2.0;
    let btn_h = font_size + pad_y * 2.0;

    let alpha = if disabled { 0.45 } else { 1.0 };

    let (bg, text_col) = match kind {
        ButtonKind::Solid => {
            let c = theme.pop;
            let bg = if hovered && !disabled {
                phi::color::Color::rgba(c.0[0] * 0.9, c.0[1] * 0.9, c.0[2] * 1.0, alpha)
            } else {
                phi::color::Color::rgba(c.0[0], c.0[1], c.0[2], alpha)
            };
            (bg, theme.bg_1)
        }
        ButtonKind::Ghost => {
            let bg = if hovered && !disabled {
                theme.hover_bg_2
            } else {
                phi::color::Color::rgba(0.0, 0.0, 0.0, 0.0)
            };
            (bg, theme.text_1)
        }
        ButtonKind::Danger => {
            let c = theme.danger;
            let bg = if hovered && !disabled {
                phi::color::Color::rgba(c.0[0] * 0.9, c.0[1] * 0.85, c.0[2] * 0.85, alpha)
            } else {
                phi::color::Color::rgba(c.0[0], c.0[1], c.0[2], alpha * 0.15)
            };
            (bg, theme.danger)
        }
    };

    compositor.push(SceneNode::Rect { x, y, w: btn_w, h: btn_h, color: bg.to_array() });
    compositor.push(SceneNode::Text {
        key: TextNodeKey::new(label, font_size, font_size * 1.2, None).with_weight(500),
        x: x + pad_x,
        y: y + pad_y,
        color: text_col.to_array(),
    });

    (x, y, btn_w, btn_h)
}
