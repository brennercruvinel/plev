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

/// Push a single-line text node to the default layer at `(x, y)`, in the
/// theme's family and tracking (`theme.typography.exact`) at `size` and
/// `weight`, one and four tenths of the size tall.
pub(crate) fn text(
    c: &mut Compositor,
    s: &str,
    size: f32,
    weight: u16,
    (x, y): (f32, f32),
    color: [f32; 4],
    theme: &Theme,
) {
    let style = text_style(size, weight, theme);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(s, &style, None),
        x,
        y,
        color,
    });
}

/// The style [`text`] draws with.
fn text_style(size: f32, weight: u16, theme: &Theme) -> TextStyle {
    theme.typography.exact(size, size * TEXT_LINE, weight)
}

/// [`text`]'s line height, in sizes.
const TEXT_LINE: f32 = 1.4;

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

#[cfg(test)]
mod tests {
    use super::*;

    /// A run of the helper is in the theme's face and tracking: Inter at
    /// -0.02em under the core kit, the engine's sans under HOFF.
    #[test]
    fn a_text_run_takes_the_theme_family() {
        let core = Theme::hoff_core();
        let style = text_style(13.0, 500, &core);
        assert_eq!(style.font_family.as_deref(), Some("Inter"));
        assert_eq!(style.letter_spacing, core.typography.tracking_em * 13.0);
        assert_eq!(
            (style.font_weight, style.line_height),
            (500, 13.0 * TEXT_LINE)
        );
        let mut c = Compositor::new();
        c.begin_frame();
        text(&mut c, "run", 13.0, 500, (0.0, 0.0), [1.0; 4], &core);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
        assert!(matches!(&nodes[0], SceneNode::Text { key, .. }
            if *key == TextNodeKey::from_style("run", &style, None)));
        assert_eq!(text_style(13.0, 500, &Theme::hoff()).font_family, None);
    }
}
