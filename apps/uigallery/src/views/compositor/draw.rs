//! Drawing helpers the section modules share: a text run, the uppercase
//! group label, the soft panel container.

use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::styling::theme::{Theme, TypeRun};
use engine::widgets::form::label::Label;
use engine::widgets::prelude::{Rect, rounded_rect, rounded_rect_stroke};

/// RGBA with overridden alpha.
pub(crate) fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

/// Push a single-line text node to the default layer at `(x, y)` in a
/// run of the theme's scale (`theme.typography`): the role sets the size,
/// the line height, the weight and the tracking, so a caller centring the
/// run by the role's line height draws it with that same line.
pub(crate) fn text(
    c: &mut Compositor,
    s: &str,
    run: impl Into<TypeRun>,
    (x, y): (f32, f32),
    color: [f32; 4],
    theme: &Theme,
) {
    let style = theme.typography.run(run.into());
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A run of the helper is the theme's style of its role: Inter at
    /// -0.02em under the core kit, the engine's sans under HOFF.
    #[test]
    fn a_text_run_is_the_theme_style_of_its_role() {
        use engine::styling::theme::TypeRole;
        let core = Theme::hoff_core();
        let style = core.typography.role(TypeRole::Label);
        assert_eq!(style.font_family.as_deref(), Some("Inter"));
        let mut c = Compositor::new();
        c.begin_frame();
        text(&mut c, "run", TypeRole::Label, (0.0, 0.0), [1.0; 4], &core);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
        assert!(matches!(&nodes[0], SceneNode::Text { key, .. }
            if *key == TextNodeKey::from_style("run", &style, None)));
        let hoff = Theme::hoff().typography.role(TypeRole::Label);
        assert_eq!(hoff.font_family, None);
    }
}
