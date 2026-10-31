//! The stat family: a tick ruler, the headline value and a dot label,
//! with an optional delta chip.

use super::{Card, Metrics};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::widget::Rect;

impl Card {
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
        let headline = ty.headline();
        let value_y = b.y + m.pad + ruler_h + m.gap;
        self.text(
            c,
            layer,
            value,
            &headline,
            b.x + m.pad,
            value_y,
            text.0,
            None,
        );
        let label_style = ty.base_2r();
        let label_y = value_y + headline.line_height + m.gap_sm;
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
            None,
        );

        // Delta chip: caption-sm in a tooltip-radius surface-hover chip,
        // success or danger colored.
        if let Some((delta, positive)) = delta {
            let style = ty.caption_sm();
            let (tw, _) = TextMeasurer::measure_styled(delta, &style, None);
            let chip_h = style.line_height + m.gap_sm * 2.0;
            let chip_w = tw + m.gap * 2.0;
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
