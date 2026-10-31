//! Drawing helpers the section modules share: a text run, the uppercase
//! group label, the soft panel container.

use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::drawing::text::TextStyle;
use engine::styling::theme::Theme;
use engine::widgets::form::label::Label;
use engine::widgets::prelude::{Rect, rounded_rect, rounded_rect_stroke};

/// RGBA with overridden alpha.
pub(crate) fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

/// Push a single-line text node to the default layer.
pub(crate) fn text(
    c: &mut Compositor,
    s: &str,
    size: f32,
    weight: u16,
    x: f32,
    y: f32,
    color: [f32; 4],
) {
    let style = TextStyle::new(size)
        .with_line_height(size * 1.4)
        .with_weight(weight);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(s, &style, None),
        x,
        y,
        color,
    });
}

/// Uppercase group label — the HOFF accordion head: the engine's
/// caption [`Label`] (`theme.form.label.caption`).
pub(crate) fn group_label(c: &mut Compositor, s: &str, x: f32, y: f32, theme: &Theme) {
    Label::new(s)
        .caption()
        .draw(c, LayerId::DEFAULT, x, y, None, theme);
}

/// Soft panel container — HOFF list card: radius 20, .02 white glass,
/// soft edge. Icon paths pushed after it stack on top (push order is
/// preserved across primitive types).
pub(crate) fn panel(c: &mut Compositor, rect: Rect, theme: &Theme) {
    c.push(rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        theme.glass.surface.0,
    ));
    c.push(rounded_rect_stroke(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        theme.glass.edge_soft.0,
        1.0,
    ));
}
