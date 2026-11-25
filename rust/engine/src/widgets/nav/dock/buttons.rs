//! The dock's buttons, drawn and hit by the dock, so every app that
//! docks a sidebar has the same ones; the app gives the head (the strip
//! of its panel the logo sits in) and the shell the menu button's rect.
//!
//! - **The pin**, at the right of the panel's head, beside the logo: the
//!   panel-side control (`panel-left-open` while the panel peeks,
//!   `panel-left-close` while it is pinned) that keeps the panel open
//!   beside the page. Pinned it wears the current link's surface too (a
//!   toggle shows its state).
//! - **The close**, in the same place in the open drawer, the kit's X
//!   (GuidelineSidebar): a window that cannot hold the panel beside the
//!   page has nothing to pin it to, and the drawer closes from there.
//! - **The menu button**, in the drawer, at the top of the page: the
//!   kit's three bars (Layout/Header) that open the drawer, in the
//!   current link's ink, as the kit draws them.
//!
//! Their box is the recipe's (`button`, `button_radius`, `button_icon`,
//! the navigation link's icon box), their tones the link's surfaces: at
//! rest the bare icon, the hover and the press their fill. Each is
//! pressed and bound in [`hit_rect`] around that box (44 px with a
//! finger), which is what draws.

use accord::SemId;

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::hit::hit_rect;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::{DockMode, SidebarDock};

/// The pin's icon while the panel peeks and while it is pinned.
const PIN_OPEN: &str = "panel-left-open";
const PIN_CLOSE: &str = "panel-left-close";
/// The drawer's close and its menu button.
const CLOSE: &str = "kit-close";
const MENU: &str = "kit-hamburger";

impl SidebarDock {
    /// Where the pin (and the drawer's close) is in `head`: at its
    /// right, as far in from the edge as the sidebars' head pads the
    /// logo, centred down it.
    pub fn pin_rect(head: Rect, theme: &Theme) -> Rect {
        let b = theme.navigation.dock.button;
        let pad = theme.navigation.sidebars.head_pad;
        let x = (head.right() - pad - b).max(head.x);
        Rect::new(x, head.y + (head.h - b) / 2.0, b, b)
    }

    /// The menu button's edge: the recipe's button box.
    pub fn menu_size(theme: &Theme) -> f32 {
        theme.navigation.dock.button
    }

    /// Whether the pin shows in a window `w` wide: not in the drawer.
    pub fn has_pin(&self, theme: &Theme, w: f32) -> bool {
        self.mode(theme, w) != DockMode::Drawer
    }

    /// Whether the close shows: in the open drawer.
    pub fn has_close(&self, theme: &Theme, w: f32) -> bool {
        self.mode(theme, w) == DockMode::Drawer && self.drawer_open
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
            .handle(event, hit_rect(Self::pin_rect(head, theme), theme), off)
    }

    /// A pointer event on the open drawer's close in `head`. `clicked`
    /// asks it closed, which the app runs as an `activate` on
    /// `<id>.close` (to [`SidebarDock::toggle_expanded`]).
    pub fn close_event(
        &mut self,
        event: &WidgetEvent,
        head: Rect,
        theme: &Theme,
        w: f32,
    ) -> EventResult {
        let off = !self.has_close(theme, w);
        self.close_press
            .handle(event, hit_rect(Self::pin_rect(head, theme), theme), off)
    }

    /// A pointer event on the drawer's menu button at `at` (the shell's
    /// [`menu_button`](crate::widgets::shell::ShellLayout::menu_button)).
    /// `clicked` when a release on it asks the drawer open.
    pub fn menu_event(
        &mut self,
        event: &WidgetEvent,
        at: Rect,
        theme: &Theme,
        w: f32,
    ) -> EventResult {
        let off = self.mode(theme, w) != DockMode::Drawer;
        self.menu_press.handle(event, hit_rect(at, theme), off)
    }

    /// The keyboard focus on the pin, or off it.
    pub fn set_pin_focused(&mut self, on: bool) {
        self.pin_press.focused = on;
    }

    /// The keyboard focus on the drawer's close, or off it.
    pub fn set_close_focused(&mut self, on: bool) {
        self.close_press.focused = on;
    }

    /// The keyboard focus on the drawer's menu button, or off it.
    pub fn set_menu_focused(&mut self, on: bool) {
        self.menu_press.focused = on;
    }

    /// The pin in `head`, or in the open drawer the close.
    pub fn render_pin(&self, c: &mut Compositor, head: Rect, theme: &Theme, w: f32) {
        let at = Self::pin_rect(head, theme);
        if self.has_close(theme, w) {
            self.paint(c, at, &self.close_press, false, CLOSE, None, theme);
            if let Some(id) = self.close_id() {
                bind(c, &id, at, theme);
            }
            return;
        }
        if !self.has_pin(theme, w) {
            return;
        }
        let icon = if self.pinned { PIN_CLOSE } else { PIN_OPEN };
        self.paint(c, at, &self.pin_press, self.pinned, icon, None, theme);
        if let Some(id) = self.pin_id() {
            bind(c, &id, at, theme);
        }
    }

    /// The drawer's menu button at `at`, on `layer`: its three bars at the
    /// box's edge (the kit's glyph fills its 32 box as the spans do).
    pub fn render_menu(&self, c: &mut Compositor, layer: LayerId, at: Rect, theme: &Theme, w: f32) {
        if self.mode(theme, w) != DockMode::Drawer {
            return;
        }
        let ink = theme.navigation.link.active.icon;
        c.on_layer(layer, |c| {
            self.paint(
                c,
                at,
                &self.menu_press,
                false,
                MENU,
                Some((ink, at.w)),
                theme,
            );
            if let Some(id) = self.menu_id() {
                bind(c, &id, at, theme);
            }
        });
    }

    /// One button at `at`: the link's surface its press and `on` give,
    /// the icon in its tone (or `glyph`'s tone and edge), the ring.
    #[allow(clippy::too_many_arguments)]
    fn paint(
        &self,
        c: &mut Compositor,
        at: Rect,
        p: &Press,
        on: bool,
        icon: &str,
        glyph: Option<(Color, f32)>,
        theme: &Theme,
    ) {
        let r = theme.navigation.dock;
        let link = &theme.navigation.link;
        let surface = if p.pressed {
            Some(&link.pressed)
        } else if on {
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
        let (tone, size) = match glyph {
            Some((tone, size)) => (tone, size),
            None => (surface.unwrap_or(&link.rest).icon, r.button_icon),
        };
        let (ix, iy) = (at.x + (at.w - size) / 2.0, at.y + (at.h - size) / 2.0);
        if let Some(n) = icons::icon_at(icon, size, tone.0, ix, iy) {
            c.push(n);
        }
        if p.focused {
            c.push(focus_ring(at, r.button_radius, theme));
        }
    }
}

/// Binds `id` where a press on the button drawn at `at` lands.
fn bind(c: &mut Compositor, id: &SemId, at: Rect, theme: &Theme) {
    let hit = hit_rect(at, theme);
    c.bind(id, [hit.x, hit.y, hit.w, hit.h]);
}
