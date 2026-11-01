//! The chart family: the value and its legend over grouped bars, the
//! highlighted group in the gradient with its cap, the axis under them.

use super::{Card, Metrics};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::styling::theme::TypeRole;
use crate::widgets::widget::Rect;

impl Card {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_chart(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        value: &str,
        label: &str,
        groups: &[(f32, f32)],
        highlight: usize,
    ) {
        let m = Metrics::of(theme);
        let marks = &theme.disclosure.card;
        let text = theme.colors.text;
        let ty = &theme.typography;

        // Legend: solid dot + value (base-m primary) + label (caption-r).
        let style = ty.run(TypeRole::BodyLg.medium());
        self.dot(
            c,
            layer,
            b.x + m.pad,
            b.y + m.pad + (style.line_height - m.dot) / 2.0,
            true,
            theme,
        );
        let value_x = b.x + m.pad + m.dot + m.gap;
        self.text(c, layer, value, &style, value_x, b.y + m.pad, text.0, None);
        let (vw, _) = TextMeasurer::measure_styled(value, &style, None);
        let label_style = ty.run(TypeRole::Caption.regular());
        self.text(
            c,
            layer,
            label,
            &label_style,
            value_x + vw + m.gap + m.gap_sm,
            b.y + m.pad + (style.line_height - label_style.line_height) / 2.0,
            theme.colors.text_mid.0,
            None,
        );

        // Chart: groups of two bars (an `xs` half apart), group gap a
        // share of the width, bars at the micro radius.
        let chart = Rect::new(
            b.x + m.pad,
            b.y + m.pad + style.line_height + m.gap,
            b.w - m.pad * 2.0,
            Self::chart_h(theme),
        );
        let n = groups.len().max(1) as f32;
        let group_gap = chart.w * 0.045;
        let bar_gap = m.gap_sm / 2.0;
        let group_w = (chart.w - group_gap * (n - 1.0)) / n;
        let bar_w = ((group_w - bar_gap) / 2.0).max(1.0);
        let bar_radius = theme.shape.micro.min(bar_w / 2.0);
        for (i, (a, hb)) in groups.iter().enumerate() {
            let gx = chart.x + i as f32 * (group_w + group_gap);
            let heights = [a.clamp(0.0, 1.0), hb.clamp(0.0, 1.0)];
            for (j, t01) in heights.iter().enumerate() {
                let h = chart.h * t01;
                let x = gx + j as f32 * (bar_w + bar_gap);
                let y = chart.y + chart.h - h;
                if i == highlight && j == 1 {
                    // Highlight: gradient from the wash alpha to nothing,
                    // plus a floating cap above the bar.
                    c.push_to_layer(
                        layer,
                        SceneNode::GradientRect {
                            x,
                            y,
                            w: bar_w,
                            h,
                            color: marks.highlight[0].0,
                            color2: marks.highlight[1].0,
                            angle_deg: 180.0,
                            corner_radius: bar_radius,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                    let cap_h = theme.control.edge_width_strong;
                    c.push_to_layer(
                        layer,
                        SceneNode::RoundedRect {
                            x,
                            y: y - m.gap_sm,
                            w: bar_w,
                            h: cap_h,
                            color: marks.cap.0,
                            corner_radius: cap_h / 2.0,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                } else {
                    let color = marks.series[j].0;
                    c.push_to_layer(
                        layer,
                        SceneNode::RoundedRect {
                            x,
                            y,
                            w: bar_w,
                            h,
                            color,
                            corner_radius: bar_radius,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                }
            }
        }
        // Baseline (surface-active, bleeding to the shell edge) + ticks.
        let line = theme.control.edge_width_strong;
        let base_y = chart.y + chart.h + m.gap_sm / 2.0;
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: b.x + m.gap_sm,
                y: base_y,
                w: b.w - m.gap_sm * 2.0,
                h: line,
                color: marks.axis.0,
            },
        );
        let ticks = ((chart.w / theme.spacing.xxl).floor() as usize).max(2);
        for i in 0..ticks {
            let tx = chart.x + chart.w * i as f32 / (ticks - 1) as f32;
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: tx,
                    y: base_y + line + m.gap_sm,
                    w: line,
                    h: m.gap_sm + m.gap_sm / 2.0,
                    color: marks.tick.0,
                },
            );
        }
    }
}
