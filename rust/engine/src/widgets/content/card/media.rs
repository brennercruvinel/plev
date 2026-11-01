//! The media family: a browser chrome dot, the video box with its play
//! button and badge, the title and the caption.

use super::{Card, Metrics};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::gpu::image::ImageHandle;
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::TypeRole;
use crate::styling::theme::{ControlSize, IconSize, Theme};
use crate::widgets::recipe::glass_pill;
use crate::widgets::widget::Rect;

impl Card {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_media(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        title: &str,
        caption: &str,
        badge: Option<&str>,
        image: Option<ImageHandle>,
    ) {
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;

        // Browser dots: three, one `sm` step each, `xs` apart.
        let dot = Self::browser_dot(theme);
        for i in 0..3 {
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: b.x + m.pad + i as f32 * (dot + m.gap_sm + m.gap_sm / 2.0),
                    y: b.y + m.pad,
                    w: dot,
                    h: dot,
                    color: glass.surface_active.0,
                    corner_radius: dot / 2.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
        }

        // Video box, centered, at the cluster radius.
        let (box_w, box_h) = Self::media_box(theme, b.w);
        let box_radius = theme.shape.cluster;
        let box_r = Rect::new(
            b.x + (b.w - box_w) / 2.0,
            b.y + m.pad + dot + m.gap,
            box_w,
            box_h,
        );
        if let Some(image) = image {
            c.push_to_layer(
                layer,
                SceneNode::Image {
                    x: box_r.x,
                    y: box_r.y,
                    w: box_r.w,
                    h: box_r.h,
                    image,
                    corner_radius: box_radius,
                },
            );
        } else {
            for node in glass_pill(
                box_r,
                box_radius,
                glass.edge.0,
                theme.control.edge_width_strong,
                glass.surface.0,
                theme,
            ) {
                c.push_to_layer(layer, node);
            }
            // Play: a ring one `Xl` control wide (faint text stroke) with
            // a glass core inside the rim.
            let ring_d = theme.control.height(ControlSize::Xl);
            let play = Rect::new(
                box_r.x + (box_r.w - ring_d) / 2.0,
                box_r.y + (box_r.h - ring_d) / 2.0,
                ring_d,
                ring_d,
            );
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: play.x,
                    y: play.y,
                    w: play.w,
                    h: play.h,
                    color: [0.0; 4],
                    corner_radius: ring_d / 2.0,
                    border_width: theme.control.edge_width_strong,
                    border_color: glass.text_faint.0,
                },
            );
            let core = play.inset(theme.control.edge_width_strong * 2.0);
            for node in glass_pill(
                core,
                core.w / 2.0,
                glass.edge.0,
                theme.control.edge_width_strong,
                glass.surface_hover.0,
                theme,
            ) {
                c.push_to_layer(layer, node);
            }
        }
        // Play icon + badge render above the image. The triangle is the
        // vector Lucide icon, not a text glyph: U+25B6 is not covered by
        // the embedded UI faces and would silently rasterize from a
        // system font.
        let play_icon = theme.control.icon(IconSize::Lg);
        if let Some(icon) = crate::widgets::icons::icon_at(
            "play",
            play_icon,
            text.0,
            box_r.x + (box_r.w - play_icon) / 2.0,
            box_r.y + (box_r.h - play_icon) / 2.0,
        ) {
            c.push_to_layer(layer, icon);
        }
        if let Some(badge) = badge {
            let style = ty.run(TypeRole::Caption.strong());
            let (bw, _) = TextMeasurer::measure_styled(badge, &style, None);
            self.text(
                c,
                layer,
                badge,
                &style,
                box_r.x + box_r.w - m.gap - bw,
                box_r.y + m.gap,
                text.0,
                None,
            );
        }

        // Title + caption.
        let title_style = ty.role(TypeRole::Title);
        let ty0 = box_r.y + box_r.h + m.pad;
        self.text(
            c,
            layer,
            title,
            &title_style,
            b.x + m.pad,
            ty0,
            text.0,
            None,
        );
        self.text(
            c,
            layer,
            caption,
            &ty.role(TypeRole::Body),
            b.x + m.pad,
            ty0 + title_style.line_height + m.gap_sm,
            theme.colors.text_mid.0,
            None,
        );
    }
}
