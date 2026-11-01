//! The pin: the toggle at the right of the panel's head, beside the
//! logo, that keeps the panel open beside the page. The dock draws it
//! and takes its pointer, so every app that docks a sidebar has the same
//! one; the app gives the head (the strip of its panel the logo sits
//! in). Its box is the recipe's (`button`, `button_radius`,
//! `button_icon`, the navigation link's icon box), its tones the link's
//! surfaces: at rest the bare icon, the hover and the press their fill,
//! and pinned the current link's (a toggle shows its state). In the
//! drawer there is no pin: a window that cannot hold the panel beside the
//! page has nothing to pin it to.

use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::{DockMode, SidebarDock};

impl SidebarDock {
    /// Where the pin is in `head`: at its right, as far in from the edge
    /// as the sidebars' head pads the logo, centred down it.
    pub fn pin_rect(head: Rect, theme: &Theme) -> Rect {
        let b = theme.navigation.dock.button;
        let pad = theme.navigation.sidebars.head_pad;
        let x = (head.right() - pad - b).max(head.x);
        Rect::new(x, head.y + (head.h - b) / 2.0, b, b)
    }

    /// Whether the pin shows in a window `w` wide: not in the drawer.
    pub fn has_pin(&self, theme: &Theme, w: f32) -> bool {
        self.mode(theme, w) != DockMode::Drawer
    }

    /// A pointer event on the pin in `head`: its hover and its press.
    /// `clicked` when a release on it asks the flip, which the app runs
    /// as a reader's and an agent's `toggle` (through its dispatcher, to
    /// [`SidebarDock::toggle_pin`]): the pointer adds no path of its own.
    pub fn pin_event(
        &mut self,
        event: &WidgetEvent,
        head: Rect,
        theme: &Theme,
        w: f32,
    ) -> EventResult {
        let off = !self.has_pin(theme, w);
        self.pin_press
            .handle(event, Self::pin_rect(head, theme), off)
    }

    /// The keyboard focus on the pin, or off it.
    pub fn set_pin_focused(&mut self, on: bool) {
        self.pin_press.focused = on;
    }

    pub fn render_pin(&self, c: &mut Compositor, head: Rect, theme: &Theme, w: f32) {
        if !self.has_pin(theme, w) {
            return;
        }
        let r = theme.navigation.dock;
        let link = &theme.navigation.link;
        let at = Self::pin_rect(head, theme);
        let p = &self.pin_press;
        let surface = if p.pressed {
            Some(&link.pressed)
        } else if self.pinned {
            Some(&link.active)
        } else if p.hovered {
            Some(&link.hover)
        } else {
            None
        };
        if let Some(s) = surface {
            c.push(rounded_rect(
                at.x,
                at.y,
                at.w,
                at.h,
                r.button_radius,
                s.fill.0,
            ));
            if s.border.0[3] > 0.0 && link.border_width > 0.0 {
                c.push(rounded_rect_stroke(
                    at.x,
                    at.y,
                    at.w,
                    at.h,
                    r.button_radius,
                    s.border.0,
                    link.border_width,
                ));
            }
        }
        let tone = surface.unwrap_or(&link.rest).icon.0;
        let (name, size) = (if self.pinned { "pin-off" } else { "pin" }, r.button_icon);
        let (ix, iy) = (at.x + (at.w - size) / 2.0, at.y + (at.h - size) / 2.0);
        if let Some(n) = icons::icon_at(name, size, tone, ix, iy) {
            c.push(n);
        }
        if p.focused {
            c.push(focus_ring(at, r.button_radius, theme));
        }
        if let Some(id) = self.pin_id() {
            c.bind(&id, [at.x, at.y, at.w, at.h]);
        }
    }
}
