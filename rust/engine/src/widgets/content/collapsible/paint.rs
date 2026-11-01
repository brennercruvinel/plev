//! The collapsible's pixels: the divider, the trigger in its state (fill,
//! border, focus ring, icon tile, label, counter, the chevron turned as
//! far as it has turned) and the content, clipped to what is revealed.

use std::hash::{DefaultHasher, Hash, Hasher};

use super::super::tokens::{DisclosureRecipe, DisclosureSurface};
use super::Collapsible;
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{
    focus_ring_inside, rounded_rect, rounded_rect_stroke, shadow_drops, shadow_insets, shadow_node,
};
use crate::widgets::widget::Rect;

impl Collapsible {
    /// The trigger's surface: a disabled one shows no state (an open or
    /// current fill under faded text reads as the one in use).
    pub(super) fn surface<'r>(&self, r: &'r DisclosureRecipe) -> &'r DisclosureSurface {
        if self.disabled {
            &r.rest
        } else if self.current {
            &r.current
        } else if self.open {
            &r.open
        } else if self.press.hovered {
            &r.hover
        } else {
            &r.rest
        }
    }

    /// Draws the divider, the trigger and, clipped to what is revealed,
    /// the content through `content` at its full-size rect (`content_h`
    /// tall, the height given to [`Collapsible::height`]).
    pub fn render(
        &self,
        c: &mut Compositor,
        bounds: Rect,
        content_h: f32,
        theme: &Theme,
        content: impl FnOnce(&mut Compositor, Rect),
    ) {
        let r = self.recipe(theme);
        let alpha = if self.disabled {
            theme.glass.disabled_alpha
        } else {
            1.0
        };
        let divider = self.divider_h(r);
        if divider > 0.0 && r.divider.0[3] > 0.0 {
            c.push(SceneNode::Rect {
                x: bounds.x,
                y: bounds.y,
                w: bounds.w,
                h: divider,
                color: faded(r.divider, alpha),
            });
        }
        let trigger = self.trigger_rect(bounds, theme);
        if let Some(id) = &self.sem {
            let at = self.press_rect(bounds, theme);
            c.bind(id, [at.x, at.y, at.w, at.h]);
        }
        self.paint_trigger(c, trigger, r, alpha, theme);

        let shown = self.shown_rect(bounds, theme);
        if shown.h > 0.0 {
            let full = self.content_rect(bounds, content_h, theme);
            c.push_clip(shown.x, shown.y, shown.w, shown.h);
            if self.open
                && let Some(panel) = self.sem.as_ref().and_then(|id| id.child("panel").ok())
            {
                c.bind(&panel, [full.x, full.y, full.w, full.h]);
            }
            content(c, full);
            c.pop_clip();
        }
    }

    fn paint_trigger(
        &self,
        c: &mut Compositor,
        trigger: Rect,
        r: &DisclosureRecipe,
        alpha: f32,
        theme: &Theme,
    ) {
        let radius = r.radius.min(trigger.h / 2.0);
        let surface = self.surface(r);
        if surface.fill.0[3] > 0.0 {
            c.push(rounded_rect(
                trigger.x,
                trigger.y,
                trigger.w,
                trigger.h,
                radius,
                faded(surface.fill, alpha),
            ));
        }
        if r.border_width > 0.0 && surface.border.0[3] > 0.0 {
            c.push(rounded_rect_stroke(
                trigger.x,
                trigger.y,
                trigger.w,
                trigger.h,
                radius,
                faded(surface.border, alpha),
                r.border_width,
            ));
        }
        if self.press.focused && !self.disabled {
            c.push(focus_ring_inside(trigger, radius, theme));
        }

        let inner = self.inner(trigger, theme);
        let mut x = inner.x;
        if let (Some(tile), Some(name)) = (r.tile.as_ref(), self.icon) {
            let at = Rect::new(
                x,
                inner.y + (inner.h - tile.size) / 2.0,
                tile.size,
                tile.size,
            );
            let color = if self.current {
                if let Some(node) = shadow_node(at, tile.radius, &tile.shadow_current) {
                    c.push(node);
                }
                c.push(rounded_rect(
                    at.x,
                    at.y,
                    at.w,
                    at.h,
                    tile.radius,
                    faded(tile.fill_current, alpha),
                ));
                tile.icon_current
            } else {
                tile.icon_rest
            };
            let inset = (tile.size - tile.icon) / 2.0;
            if let Some(node) = icons::icon_at(
                name,
                tile.icon,
                faded(color, alpha),
                at.x + inset,
                at.y + inset,
            ) {
                c.push(node);
            }
            x += tile.size + tile.gap;
        }

        let style = Self::label_style(r, theme);
        let label_y = inner.y + (inner.h - inner.label_h) / 2.0;
        c.push_clip(x, label_y, inner.label_w, inner.label_h);
        let key = if self.truncate {
            let shown = TextMeasurer::truncate_to_width(&self.title, &style, inner.label_w);
            TextNodeKey::from_style(&shown, &style, None)
        } else {
            TextNodeKey::from_style(&self.title, &style, Some(inner.label_w))
        };
        c.push(SceneNode::Text {
            key,
            x,
            y: label_y,
            color: faded(r.label, alpha),
        });
        c.pop_clip();

        let box_x = inner.x + inner.w - r.chevron_box;
        self.paint_counter(c, r, box_x - r.chevron_gap, &inner, alpha, theme);
        let box_y = inner.y + (inner.h - r.chevron_box) / 2.0;
        let inset = (r.chevron_box - r.chevron) / 2.0;
        if let Some(node) = turned(
            "chevron-down",
            r.chevron,
            faded(r.chevron_color, alpha),
            box_x + inset,
            box_y + inset,
            self.turn.get() * std::f32::consts::PI,
        ) {
            c.push(node);
        }
    }

    /// The counter's pill, its right edge at `right`, centered on the
    /// trigger's row: drop shadows, fill, inset shadows, the count.
    fn paint_counter(
        &self,
        c: &mut Compositor,
        r: &DisclosureRecipe,
        right: f32,
        inner: &super::Inner,
        alpha: f32,
        theme: &Theme,
    ) {
        let (Some(text), Some(counter), Some(style), Some((w, h))) = (
            self.counter.as_deref(),
            r.counter.as_ref(),
            Self::counter_style(r, theme),
            self.counter_span(r, theme),
        ) else {
            return;
        };
        let at = Rect::new(right - w, inner.y + (inner.h - h) / 2.0, w, h);
        let shadow: Vec<_> = counter
            .shadow
            .iter()
            .map(|s| {
                let mut s = *s;
                s.color = Color(faded(s.color, alpha));
                s
            })
            .collect();
        shadow_drops(c, c.target(), at, counter.radius, &shadow);
        c.push(rounded_rect(
            at.x,
            at.y,
            at.w,
            at.h,
            counter.radius,
            faded(counter.fill, alpha),
        ));
        shadow_insets(c, c.target(), at, counter.radius, &shadow);
        let [pad_x, pad_y] = counter.pad;
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(text, &style, None),
            x: at.x + pad_x,
            y: at.y + pad_y,
            color: faded(counter.text, alpha),
        });
    }
}

fn faded(c: Color, alpha: f32) -> [f32; 4] {
    [c.0[0], c.0[1], c.0[2], c.0[3] * alpha]
}

/// An icon turned by `angle` radians about its center, its hash moved
/// with the angle so the compositor's diff sees the turn.
fn turned(name: &str, size: f32, color: [f32; 4], x: f32, y: f32, angle: f32) -> Option<SceneNode> {
    let mut data = icons::icon_path(name, size, color, x, y)?;
    if angle != 0.0 {
        let (cx, cy) = (x + size / 2.0, y + size / 2.0);
        let (sin, cos) = angle.sin_cos();
        for v in &mut data.vertices {
            let (dx, dy) = (v.position[0] - cx, v.position[1] - cy);
            v.position = [cx + dx * cos - dy * sin, cy + dx * sin + dy * cos];
        }
        let mut h = DefaultHasher::new();
        data.hash.hash(&mut h);
        angle.to_bits().hash(&mut h);
        data.hash = h.finish();
    }
    Some(SceneNode::Path { data })
}
