//! The parts of a header that are its own: an icon action in one of the
//! three looks (with the dot of news), the profile button, and the asset
//! header's strip of thumbnails. Each reads `theme.header`; the header
//! lays them out and routes the pointer to them.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Press, paint_surface};
use crate::widgets::icons;
use crate::widgets::recipe::{
    focus_ring, gradient_fill, rounded_rect, rounded_rect_stroke, shadow_node,
};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::tokens::ActionLook;

/// Which look an icon action takes (`theme.header.{nav,plain,framed}`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionStyle {
    /// Back and forward: grey, pressed in on hover.
    Nav,
    /// The filter, the alerts, the search: a wash on hover.
    Plain,
    /// The asset header's: framed.
    Framed,
}

impl ActionStyle {
    pub fn look(self, theme: &Theme) -> &ActionLook {
        let h = &theme.header;
        match self {
            ActionStyle::Nav => &h.nav,
            ActionStyle::Plain => &h.plain,
            ActionStyle::Framed => &h.framed,
        }
    }
}

/// An icon button of a header, named for a reader by `label`.
#[derive(Clone, Debug)]
pub struct Action {
    pub icon: &'static str,
    pub label: &'static str,
    pub style: ActionStyle,
    /// The dot of news at its corner.
    pub dot: bool,
    pub disabled: bool,
    pub(super) press: Press,
}

impl Action {
    pub fn new(icon: &'static str, label: &'static str, style: ActionStyle) -> Self {
        Self {
            icon,
            label,
            style,
            dot: false,
            disabled: false,
            press: Press::default(),
        }
    }

    pub fn dot(mut self, on: bool) -> Self {
        self.dot = on;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let s = self.style.look(theme).size;
        (s, s)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.press.handle(event, bounds, self.disabled)
    }

    pub fn render(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let look = self.style.look(theme);
        let alpha = if self.disabled {
            look.recipe.disabled_alpha
        } else {
            1.0
        };
        let surface = self.press.surface(&look.recipe);
        let fg = paint_surface(
            c,
            layer,
            bounds,
            look.radius,
            surface,
            alpha,
            self.press.focused,
            theme,
        );
        let i = look.icon;
        let (x, y) = (
            bounds.x + (bounds.w - i) / 2.0,
            bounds.y + (bounds.h - i) / 2.0,
        );
        if let Some(node) = icons::icon_at(self.icon, i, fg, x, y) {
            c.push_to_layer(layer, node);
        }
        if self.dot {
            let d = &theme.header.dot;
            let mut color = d.color.0;
            color[3] *= alpha;
            c.push_to_layer(
                layer,
                rounded_rect(
                    bounds.right() - d.inset - d.size,
                    bounds.y + d.inset,
                    d.size,
                    d.size,
                    d.size / 2.0,
                    color,
                ),
            );
        }
    }
}

/// The profile button: the person's picture, ringed on hover.
#[derive(Clone, Debug, Default)]
pub struct Avatar {
    pub(super) press: Press,
}

impl Avatar {
    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let s = theme.header.avatar.size;
        (s, s)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.press.handle(event, bounds, false)
    }

    pub fn render(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let a = &theme.header.avatar;
        let round = bounds.w / 2.0;
        if self.press.hovered {
            c.push_to_layer(
                layer,
                rounded_rect_stroke(
                    bounds.x,
                    bounds.y,
                    bounds.w,
                    bounds.h,
                    round,
                    a.ring_hover.0,
                    a.ring_w,
                ),
            );
        }
        let pic = Rect::new(
            bounds.x + a.pad,
            bounds.y + a.pad,
            bounds.w - a.pad * 2.0,
            bounds.h - a.pad * 2.0,
        );
        c.push_to_layer(
            layer,
            gradient_fill(pic, pic.w / 2.0, a.picture[0].0, a.picture[1].0),
        );
        if self.press.focused {
            c.push_to_layer(layer, focus_ring(bounds, round, theme));
        }
    }
}

/// The asset header's thumbnails: one is the current asset, drawn larger
/// and lifted; the fades at the ends go while the pointer is on them.
#[derive(Clone, Debug)]
pub struct Thumbs {
    pub count: usize,
    pub current: usize,
    /// The thumbnail the keyboard is on.
    pub focus: Option<usize>,
    pub(super) hovered: bool,
}

impl Thumbs {
    pub fn new(count: usize, current: usize) -> Self {
        Self {
            count,
            current: current.min(count.saturating_sub(1)),
            focus: None,
            hovered: false,
        }
    }

    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.header.thumbs;
        let n = self.count as f32;
        let w = t.size * (n - 1.0).max(0.0) + t.current + t.gap * (n - 1.0).max(0.0);
        (w, t.current)
    }

    /// Each thumbnail's box in `bounds`, centered on the strip's middle.
    pub fn rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let t = &theme.header.thumbs;
        let mut x = bounds.x;
        let mid = bounds.y + bounds.h / 2.0;
        (0..self.count)
            .map(|i| {
                let s = if i == self.current { t.current } else { t.size };
                let r = Rect::new(x, mid - s / 2.0, s, s);
                x += s + t.gap;
                r
            })
            .collect()
    }

    /// The pointer: on the strip the fades go; a click on a thumbnail
    /// makes it the current one (`clicked`).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let (x, y) = event.pos();
        let inside = bounds.contains(x, y);
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseMove { .. }) && inside != self.hovered {
            self.hovered = inside;
            result = EventResult::changed();
        }
        if let WidgetEvent::MouseDown { .. } = event {
            let hit = self
                .rects(bounds, theme)
                .iter()
                .position(|r| r.contains(x, y));
            if let Some(i) = hit {
                self.current = i;
                return EventResult::clicked();
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let t = &theme.header.thumbs;
        for (i, r) in self.rects(bounds, theme).into_iter().enumerate() {
            if i == self.current {
                for spec in &t.shadow {
                    if let Some(node) = shadow_node(r, t.radius, spec) {
                        c.push_to_layer(layer, node);
                    }
                }
            }
            if let Some(pic) = t.pictures.get(i % t.pictures.len().max(1)) {
                c.push_to_layer(layer, gradient_fill(r, t.radius, pic[0].0, pic[1].0));
            }
            c.push_to_layer(
                layer,
                rounded_rect_stroke(r.x, r.y, r.w, r.h, t.radius, t.border.0, t.border_w),
            );
            if self.focus == Some(i) {
                c.push_to_layer(layer, focus_ring(r, t.radius, theme));
            }
        }
        if self.hovered {
            return;
        }
        // The fades: the bar's fill to clear, inward from each end.
        let fill = theme.header.fill.0;
        let clear = [fill[0], fill[1], fill[2], 0.0];
        let w = t.fade.min(bounds.w / 2.0);
        for (x, from, to) in [(bounds.x, fill, clear), (bounds.right() - w, clear, fill)] {
            // css `bg-gradient-to-r`: 90 degrees, left to right.
            c.push_to_layer(
                layer,
                SceneNode::GradientRect {
                    x,
                    y: bounds.y,
                    w,
                    h: bounds.h,
                    color: from,
                    color2: to,
                    angle_deg: 90.0,
                    corner_radius: 0.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
        }
    }
}
