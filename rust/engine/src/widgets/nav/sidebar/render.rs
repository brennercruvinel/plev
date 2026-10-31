//! Sidebar drawing: the scrim and surface, the brand block, the band of
//! group captions and links (clipped, with the overflow fades), the
//! pinned footer links and the footer hints.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::{SidebarMode, Theme};

use crate::widgets::icons;
use crate::widgets::widget::Rect;

use super::{NavGroup, Sidebar};

impl Sidebar {
    pub fn render(&mut self, c: &mut Compositor, layer: LayerId, theme: &Theme, vw: f32, vh: f32) {
        let mode = self.mode(theme, vw);
        let Some(rect) = self.rect(theme, vw, vh) else {
            return;
        };
        self.sync_scroll(rect, theme, mode);
        let glass = &theme.glass;

        // Drawer: scrim behind the sheet.
        if mode == SidebarMode::Drawer {
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: 0.0,
                    y: 0.0,
                    w: vw,
                    h: vh,
                    color: glass.scrim.0,
                },
            );
        }
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                color: theme.colors.surface.0,
            },
        );

        // Brand block.
        if !self.brand.is_empty() && mode != SidebarMode::Rail {
            let pad = theme.spacing.xl;
            let title = theme.typography.title();
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&self.brand, &title, Some(rect.w - pad * 2.0)),
                    x: rect.x + pad,
                    y: rect.y + pad,
                    color: theme.colors.text.0,
                },
            );
            if !self.tagline.is_empty() {
                let small = theme.typography.small_sm();
                c.push_to_layer(
                    layer,
                    SceneNode::Text {
                        key: TextNodeKey::from_style(
                            &self.tagline,
                            &small,
                            Some(rect.w - pad * 2.0),
                        ),
                        x: rect.x + pad,
                        y: rect.y + pad + title.line_height,
                        color: glass.text_placeholder.0,
                    },
                );
            }
        }

        // Links, clipped to their band.
        let band = self.band(rect, theme, mode);
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: band.x,
                y: band.y,
                w: band.w,
                h: band.h,
            },
        );
        let (group_rects, link_rects) = self.rows(band, theme);
        for (group, r) in self.groups.iter().zip(group_rects) {
            if r.y + r.h < band.y || r.y > band.y + band.h {
                continue;
            }
            self.render_group(c, layer, group, r, theme, mode);
        }
        for (link, r) in self.links.iter().zip(link_rects) {
            if r.h == 0.0 || r.y + r.h < band.y || r.y > band.y + band.h {
                continue;
            }
            link.render_with(c, layer, r, theme, &theme.navigation.sidebar_link);
        }
        c.push_to_layer(layer, SceneNode::PopClip);

        // Links past the band's edge: the edge fades into the surface so
        // the cut-off row reads as "more below", not as the end.
        if self.scroll.is_scrollable() {
            let fade_h = theme.spacing.xxl.min(band.h / 2.0);
            let surface = theme.colors.surface.0;
            let clear = [surface[0], surface[1], surface[2], 0.0];
            let fade = |y: f32, top: [f32; 4], bottom: [f32; 4]| SceneNode::GradientRect {
                x: band.x,
                y,
                w: band.w,
                h: fade_h,
                color: top,
                color2: bottom,
                angle_deg: 180.0,
                corner_radius: 0.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            };
            if self.scroll.offset() > 0.0 {
                c.push_to_layer(layer, fade(band.y, surface, clear));
            }
            if self.scroll.offset() < self.scroll.max_offset() {
                c.push_to_layer(layer, fade(band.y + band.h - fade_h, clear, surface));
            }
        }

        // Pinned footer links.
        for (link, r) in self
            .footer_links
            .iter()
            .zip(self.footer_link_rects(rect, theme))
        {
            link.render_with(c, layer, r, theme, &theme.navigation.sidebar_link);
        }

        // Footer hints.
        if !self.footer.is_empty() && mode != SidebarMode::Rail {
            let style = theme.typography.caption_r();
            let pad = theme.spacing.xl;
            let mut y = rect.y + rect.h - self.footer_h(theme, mode) + theme.spacing.lg;
            for line in &self.footer {
                let shown = TextMeasurer::truncate_to_width(line, &style, rect.w - pad * 2.0);
                c.push_to_layer(
                    layer,
                    SceneNode::Text {
                        key: TextNodeKey::from_style(&shown, &style, None),
                        x: rect.x + pad,
                        y,
                        color: glass.text_placeholder.0,
                    },
                );
                y += style.line_height;
            }
        }
    }

    /// A group caption: the fold chevron and, outside the rail, the
    /// label in the placeholder tone.
    fn render_group(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        group: &NavGroup,
        r: Rect,
        theme: &Theme,
        mode: SidebarMode,
    ) {
        let style = theme.typography.caption_sm();
        let color = theme.glass.text_placeholder.0;
        let icon = if group.expanded {
            "chevron-down"
        } else {
            "chevron-right"
        };
        let size = theme.control.icon(crate::styling::theme::IconSize::Sm);
        let icon_y = r.y + (r.h - size) / 2.0;
        let icon_x = if mode == SidebarMode::Rail {
            r.x + (r.w - size) / 2.0
        } else {
            r.x + theme.spacing.sm
        };
        if let Some(node) = icons::icon_at(icon, size, color, icon_x, icon_y) {
            c.push_to_layer(layer, node);
        }
        if mode == SidebarMode::Rail {
            return;
        }
        let x = icon_x + size + theme.spacing.xs;
        let shown = TextMeasurer::truncate_to_width(&group.label, &style, r.right() - x);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &style, None),
                x,
                y: r.y + (r.h - style.line_height) / 2.0,
                color,
            },
        );
    }
}
