//! The stat family: a tick ruler, the headline value and a dot label,
//! with an optional delta chip.

use super::{Card, Metrics};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::styling::theme::TypeRole;
use crate::widgets::widget::Rect;

/// The stat card's texts in a card `w` wide: the delta as its chip
/// shows it, the chip's width, and the widths the value (beside the
/// chip) and the label (after its dot) wrap at.
struct StatText {
    delta: Option<String>,
    chip_w: f32,
    value_w: f32,
    label_w: f32,
}

impl StatText {
    fn of(w: f32, theme: &Theme, delta: Option<&(String, bool)>) -> Self {
        let m = Metrics::of(theme);
        let inner = (w - m.pad * 2.0).max(0.0);
        let style = theme.typography.run(TypeRole::Caption.strong());
        let delta = delta.map(|(d, _)| Card::fit(d, &style, inner - m.gap * 2.0));
        let chip_w = delta.as_ref().map_or(0.0, |d| {
            TextMeasurer::measure_styled(d, &style, None).0 + m.gap * 2.0
        });
        let beside = if delta.is_some() { chip_w + m.gap } else { 0.0 };
        Self {
            delta,
            chip_w,
            value_w: (inner - beside).max(0.0),
            label_w: (inner - m.dot - m.gap).max(0.0),
        }
    }
}

impl Card {
    /// The stat card's height at width `w`: the ruler, the value and the
    /// label as they wrap there.
    pub(super) fn stat_h(
        w: f32,
        theme: &Theme,
        value: &str,
        label: &str,
        delta: Option<&(String, bool)>,
    ) -> f32 {
        let m = Metrics::of(theme);
        let ty = &theme.typography;
        let t = StatText::of(w, theme, delta);
        m.pad
            + Self::ruler_h(theme)
            + m.gap
            + Self::wrapped_h(value, &ty.role(TypeRole::H3), t.value_w)
            + m.gap_sm
            + Self::wrapped_h(label, &ty.role(TypeRole::Body), t.label_w)
            + m.pad
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_stat(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        value: &str,
        label: &str,
        delta: Option<&(String, bool)>,
    ) {
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;

        // Tick ruler (card 2): hairline spans at the surface-active tone,
        // one per `xl` step across the inner width.
        let ruler_h = Self::ruler_h(theme);
        let inner_w = b.w - m.pad * 2.0;
        let ticks = ((inner_w / theme.spacing.xl).floor() as usize).max(2);
        for i in 0..ticks {
            let tx = b.x + m.pad + inner_w * i as f32 / (ticks - 1) as f32;
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: tx,
                    y: b.y + m.pad,
                    w: theme.control.edge_width_strong,
                    h: ruler_h,
                    color: glass.surface_active.0,
                },
            );
        }

        // Headline value, then dot + label (base-2r, secondary).
        let t = StatText::of(b.w, theme, delta);
        let headline = ty.role(TypeRole::H3);
        let value_y = b.y + m.pad + ruler_h + m.gap;
        self.text(
            c,
            layer,
            value,
            &headline,
            b.x + m.pad,
            value_y,
            text.0,
            Some(t.value_w),
        );
        let label_style = ty.role(TypeRole::Body);
        let label_y = value_y + Self::wrapped_h(value, &headline, t.value_w) + m.gap_sm;
        self.dot(
            c,
            layer,
            b.x + m.pad,
            label_y + (label_style.line_height - m.dot) / 2.0,
            false,
            theme,
        );
        self.text(
            c,
            layer,
            label,
            &label_style,
            b.x + m.pad + m.dot + m.gap,
            label_y,
            theme.colors.text_mid.0,
            Some(t.label_w),
        );

        // Delta chip: caption-sm in a tooltip-radius surface-hover chip,
        // success or danger colored.
        if let (Some((_, positive)), Some(delta)) = (delta, &t.delta) {
            let style = ty.run(TypeRole::Caption.strong());
            let chip_h = style.line_height + m.gap_sm * 2.0;
            let chip_w = t.chip_w;
            let chip = Rect::new(b.x + b.w - m.pad - chip_w, value_y, chip_w, chip_h);
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: chip.x,
                    y: chip.y,
                    w: chip.w,
                    h: chip.h,
                    color: glass.surface_hover.0,
                    corner_radius: theme.shape.tooltip,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
            let accent = if *positive {
                theme.colors.success.0
            } else {
                theme.colors.danger.0
            };
            self.text(
                c,
                layer,
                delta,
                &style,
                chip.x + m.gap,
                chip.y + m.gap_sm,
                accent,
                None,
            );
        }
    }
}
