use plev::color::Color;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;

/// Badge kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeKind {
    Pop,
    Danger,
    Safe,
    Warn,
    Purple,
    Neutral,
}

impl BadgeKind {
    fn colors(self, theme: &Theme) -> (Color, Color) {
        match self {
            BadgeKind::Pop     => (Color::rgba(theme.pop.0[0], theme.pop.0[1], theme.pop.0[2], 0.15),     theme.pop),
            BadgeKind::Danger  => (Color::rgba(theme.danger.0[0], theme.danger.0[1], theme.danger.0[2], 0.15), theme.danger),
            BadgeKind::Safe    => (Color::rgba(theme.safe.0[0], theme.safe.0[1], theme.safe.0[2], 0.15),  theme.safe),
            BadgeKind::Warn    => (Color::rgba(theme.warn.0[0], theme.warn.0[1], theme.warn.0[2], 0.15),  theme.warn),
            BadgeKind::Purple  => (Color::rgba(theme.purple.0[0], theme.purple.0[1], theme.purple.0[2], 0.15), theme.purple),
            BadgeKind::Neutral => (theme.bg_3, theme.text_2),
        }
    }
}

/// Draw a small inline badge: rounded rect + centered label.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    label: &str,
    kind: BadgeKind,
) -> f32 {
    let font_size = 11.0;
    let pad_x = 6.0;
    let pad_y = 2.0;
    let text_w = label.len() as f32 * font_size * 0.55;
    let badge_w = text_w + pad_x * 2.0;
    let badge_h = font_size + pad_y * 2.0;
    let (bg, text_color) = kind.colors(theme);

    compositor.push(SceneNode::Rect { x, y, w: badge_w, h: badge_h, color: bg.to_array() });
    compositor.push(SceneNode::Text {
        key: TextNodeKey::new(label, font_size, font_size * 1.2, None).with_weight(500),
        x: x + pad_x,
        y: y + pad_y,
        color: text_color.to_array(),
    });
    badge_w
}
