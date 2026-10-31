//! The profile family: an avatar with its online dot, the name and the
//! handle, the bio and the action.

use super::{Card, Metrics};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::gpu::image::ImageHandle;
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::{ControlSize, Theme};
use crate::widgets::recipe::glass_pill;
use crate::widgets::widget::Rect;

impl Card {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_profile(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        name: &str,
        username: &str,
        bio: &str,
        action: &str,
        online: bool,
        avatar: Option<ImageHandle>,
    ) {
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;
        let pad = Self::profile_pad(theme);
        let size = Self::avatar_size(theme);
        let av = Rect::new(b.x + pad, b.y + pad, size, size);

        // Avatar: image, or a monochrome glass disc with an initial.
        if let Some(image) = avatar {
            c.push_to_layer(
                layer,
                SceneNode::Image {
                    x: av.x,
                    y: av.y,
                    w: av.w,
                    h: av.h,
                    image,
                    corner_radius: av.w / 2.0,
                },
            );
        } else {
            for node in glass_pill(
                av,
                av.w / 2.0,
                glass.edge.0,
                theme.control.edge_width_strong,
                glass.surface_active.0,
                theme,
            ) {
                c.push_to_layer(layer, node);
            }
            if let Some(initial) = name.chars().next() {
                let s: String = initial.to_uppercase().collect();
                // Avatar initial: the title style, centered.
                let style = ty.title();
                let (tw, _) = TextMeasurer::measure_styled(&s, &style, None);
                self.text(
                    c,
                    layer,
                    &s,
                    &style,
                    av.x + (av.w - tw) / 2.0,
                    av.y + TextMeasurer::vertical_center(&style, av.h),
                    text.0,
                    None,
                );
            }
        }
        // Online dot at the avatar's top-left corner (knob gradient).
        if online {
            let off = theme.control.edge_width * 2.0;
            self.dot(c, layer, av.x - off, av.y - off, true, theme);
        }

        // Name (base-2sm, primary) and username (caption-r, the meta
        // register text-tertiary, not the fainter placeholder tone, which
        // read gray-on-gray over the lifted card).
        let name_style = ty.base_2sm();
        let text_x = av.x + av.w + m.gap + m.gap_sm;
        self.text(c, layer, name, &name_style, text_x, b.y + pad, text.0, None);
        self.text(
            c,
            layer,
            username,
            &ty.caption_r(),
            text_x,
            b.y + pad + name_style.line_height + m.gap_sm / 2.0,
            theme.colors.text_dim.0,
            None,
        );

        // Action pill (FollowButton): a `Sm` glass button, caption-sm.
        let style = ty.caption_sm();
        let (tw, _) = TextMeasurer::measure_styled(action, &style, None);
        let pad_x = theme.control.pad_x(ControlSize::Sm);
        let bh = theme.control.height(ControlSize::Sm);
        let bw = (tw + pad_x * 2.0).max(bh * 2.0);
        let btn = Rect::new(b.x + b.w - pad - bw, b.y + pad, bw, bh);
        for node in glass_pill(
            btn,
            btn.h / 2.0,
            glass.edge.0,
            theme.control.edge_width_strong,
            glass.button.0,
            theme,
        ) {
            c.push_to_layer(layer, node);
        }
        self.text(
            c,
            layer,
            action,
            &style,
            btn.x + (btn.w - tw) / 2.0,
            btn.y + TextMeasurer::vertical_center(&style, btn.h),
            theme.colors.text_mid.0,
            None,
        );

        // Bio: body-2r at the secondary body register, indented past the
        // avatar column.
        self.text(
            c,
            layer,
            bio,
            &ty.body_2r(),
            text_x,
            b.y + pad + size + m.gap,
            theme.colors.text_mid.0,
            Some(b.w - pad * 2.0 - size - m.gap * 2.0),
        );
    }
}
