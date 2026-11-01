//! The CTA family: the title, the body and the call to action's pill.

use super::{Card, Metrics};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::TypeRole;
use crate::styling::theme::{ControlSize, Theme};
use crate::widgets::recipe::glass_pill;
use crate::widgets::widget::{Rect, with_alpha};

impl Card {
    // The parameters mirror `CardVariant::Cta`'s fields, like the other
    // render_* dispatch helpers; a bag struct would only duplicate the enum.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_cta(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        title: &str,
        body: &str,
        button: &str,
    ) {
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;
        let inset = Self::cta_inset(theme);
        let x = b.x + inset;
        let w = b.w - inset * 2.0;

        // Title, then the body paragraph, then the CTA pill.
        let title_style = ty.role(TypeRole::Title);
        self.text(c, layer, title, &title_style, x, b.y + m.pad, text.0, None);
        let style = ty.role(TypeRole::BodyLg);
        let (_, body_h) = TextMeasurer::measure_styled(body, &style, Some(w));
        let body_y = b.y + m.pad + title_style.line_height + m.gap;
        self.text(
            c,
            layer,
            body,
            &style,
            x,
            body_y,
            theme.colors.text_mid.0,
            Some(w),
        );

        // CTA pill: an `Lg` control with the `Xl` padding, base-m label,
        // faint edge ring + frozen glow.
        let bstyle = ty.run(TypeRole::BodyLg.medium());
        let (tw, _) = TextMeasurer::measure_styled(button, &bstyle, None);
        let btn = Rect::new(
            x,
            body_y + body_h.max(style.line_height) + theme.spacing.xl,
            tw + theme.control.pad_x(ControlSize::Xl) * 2.0,
            theme.control.height(ControlSize::Lg),
        );
        let ring = with_alpha(glass.field_focus_border, glass.edge_soft.0[3]);
        for node in glass_pill(
            btn,
            btn.h / 2.0,
            ring,
            theme.control.edge_width,
            glass.button.0,
            theme,
        ) {
            c.push_to_layer(layer, node);
        }
        // The orbiting conic glow, frozen as a faint top wash.
        let glow = btn.inset(theme.control.edge_width);
        c.push_to_layer(
            layer,
            SceneNode::GradientRect {
                x: glow.x,
                y: glow.y,
                w: glow.w,
                h: glow.h,
                color: glass.surface_hover.0,
                color2: with_alpha(text, 0.0),
                angle_deg: 180.0,
                corner_radius: glow.h / 2.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            },
        );
        self.text(
            c,
            layer,
            button,
            &bstyle,
            btn.x + (btn.w - tw) / 2.0,
            btn.y + TextMeasurer::vertical_center(&bstyle, btn.h),
            theme.colors.text_mid.0,
            None,
        );
    }
}
