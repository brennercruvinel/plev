//! The list family: a title over rows of a radio, a label, a trailing
//! value and an optional progress track.

use super::{Card, CardListRow, Metrics};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::styling::theme::TypeRole;
use crate::widgets::selection::paint_radio;
use crate::widgets::widget::{Rect, with_alpha};

impl Card {
    pub(super) fn render_list(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        title: &str,
        rows: &[CardListRow],
    ) {
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;
        let title_style = ty.role(TypeRole::Title);

        let inner = b.w - m.pad * 2.0;
        self.text(
            c,
            layer,
            title,
            &title_style,
            b.x + m.pad,
            b.y + m.pad,
            text.0,
            Some(inner),
        );

        let row_h = Self::list_row_h(theme);
        let radio_d = theme.control.box_size;
        let mut y = b.y + m.pad + Self::wrapped_h(title, &title_style, inner) + m.gap;
        for row in rows {
            let r = Rect::new(b.x + m.pad, y, b.w - m.pad * 2.0, row_h);
            // Row: nav radius, strong rim (active = the active border),
            // a whisper of surface.
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                    color: with_alpha(glass.surface, glass.surface.0[3] / 2.0),
                    corner_radius: theme.shape.nav,
                    border_width: theme.control.edge_width_strong,
                    border_color: if row.active {
                        theme.colors.border_active.0
                    } else {
                        glass.edge_soft.0
                    },
                },
            );

            // Radio: the selection radio's disc (`selection::paint_radio`)
            // centered in the checkbox box's slot.
            let radio = Rect::new(
                r.x + m.pad - theme.control.edge_width_strong,
                r.y + (r.h - radio_d) / 2.0,
                radio_d,
                radio_d,
            );
            paint_radio(c, layer, radio, row.active, 1.0, theme);

            let content_x = radio.x + radio.w + m.gap + m.gap_sm;
            // Trailing value: caption-sm, primary, right aligned.
            // never past the row's middle: the label keeps the rest
            let trailing_style = ty.run(TypeRole::Caption.strong());
            let half = (r.right() - m.pad - content_x) / 2.0;
            let trailing = Self::fit(&row.trailing, &trailing_style, half);
            let (tw, _) = TextMeasurer::measure_styled(&trailing, &trailing_style, None);
            let trailing_x = r.x + r.w - m.pad - tw;
            let content_w = (trailing_x - m.gap * 2.0 - content_x).max(0.0);
            match row.progress {
                Some(p) => {
                    // Download row: label up, gradient progress under.
                    let label_style = ty.run(TypeRole::Caption.strong());
                    let block_h = label_style.line_height + m.gap_sm + theme.control.progress_track;
                    let top = r.y + (r.h - block_h) / 2.0;
                    self.text(
                        c,
                        layer,
                        &Self::fit(&row.label, &label_style, content_w),
                        &label_style,
                        content_x,
                        top,
                        theme.colors.text_mid.0,
                        None,
                    );
                    let track = Rect::new(
                        content_x,
                        top + label_style.line_height + m.gap_sm,
                        content_w,
                        theme.control.progress_track,
                    );
                    self.progress(c, layer, track, p, theme);
                }
                None => {
                    if row.label.is_empty() {
                        // Skeleton row (card 19's fake text): the active
                        // row reads as primary text, the others as the
                        // surface-active wash.
                        let (w, color) = if row.active {
                            (content_w * 0.4, text.0)
                        } else {
                            (content_w * 0.7, glass.surface_active.0)
                        };
                        self.skeleton(
                            c,
                            layer,
                            content_x,
                            r.y + (r.h - theme.control.progress_fill) / 2.0,
                            w,
                            color,
                            theme,
                        );
                    } else {
                        let style = ty.run(TypeRole::Body.medium());
                        self.text(
                            c,
                            layer,
                            &Self::fit(&row.label, &style, content_w),
                            &style,
                            content_x,
                            r.y + TextMeasurer::vertical_center(&style, r.h),
                            theme.colors.text_mid.0,
                            None,
                        );
                    }
                }
            }

            self.text(
                c,
                layer,
                &trailing,
                &trailing_style,
                trailing_x,
                r.y + TextMeasurer::vertical_center(&trailing_style, r.h),
                text.0,
                None,
            );

            y += row_h + m.gap;
        }
    }
}
