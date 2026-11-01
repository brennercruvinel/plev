//! The dock of a sidebar ([`SidebarDock`](crate::widgets::nav::SidebarDock)),
//! `theme.navigation.dock`: how wide the panel is, how near the window's
//! edge a pointer reveals it, how long it waits before it shows and
//! before it hides, how it slides, how narrow the page may get beside it
//! before it turns into a drawer, and the box of its pin and of the
//! drawer's menu button.
//!
//! The kit has no such behavior (its sidebars are always there), so the
//! dock is one rule in every theme, [`DockRecipe::derive`], over the
//! theme's spacing, durations and sizes, the sidebars' width and the
//! navigation link. [`DockRecipe::hoff_core`] writes the same rule in
//! the kit's terms (tailwind's steps and the core scales), and a test
//! holds the two equal in the core theme.

use crate::compose::animation::Easing;
use crate::styling::theme::Theme;
use crate::styling::theme::scales::{DurationScale, SizeTokens};

use super::NavLinkRecipe;

/// tailwind's `ease-in-out` (`cubic-bezier(0.4, 0, 0.2, 1)`), the curve
/// of the theme's transitions (`theme.duration`).
const EASE_IN_OUT: Easing = Easing::CubicBezier(0.4, 0.0, 0.2, 1.0);

/// What the dock reads from a theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockRecipe {
    /// The panel's width: the sidebars' (`w-60` in the kit).
    pub width: f32,
    /// The strip at the window's edge a pointer reveals a peeking
    /// sidebar from: the `sm` step of the spacing.
    pub zone: f32,
    /// The pause before a peeking sidebar shows (a pointer that only
    /// grazes the edge reveals nothing): the fast duration, a hover's.
    pub enter_delay: f32,
    /// The pause before it hides once the pointer left it: the slow
    /// duration, so a pointer that overshoots by a hair can come back.
    pub leave_delay: f32,
    /// The slide: the slow duration (a sidebar's collapse, the
    /// `DurationScale` says), on the theme's transition curve; with
    /// reduced motion it is a cut.
    pub slide: f32,
    pub slide_easing: Easing,
    /// The narrowest the page may be beside a pinned sidebar, its
    /// gutters aside: a card at its narrowest. Narrower, the sidebar is a
    /// drawer.
    pub content_min_w: f32,
    /// The pin and the drawer's menu button: the navigation link's icon
    /// box (its tile in the kit, `size-8 rounded-lg`), its radius and its
    /// icon. Their tones are the link's surfaces (`navigation.link`).
    pub button: f32,
    pub button_radius: f32,
    pub button_icon: f32,
}

impl DockRecipe {
    /// The rule, in any theme: the panel is the sidebars' `width`, the
    /// buttons are `link`'s icon box.
    pub fn derive(theme: &Theme, width: f32, link: &NavLinkRecipe) -> Self {
        Self::of(width, link, theme.spacing.sm, &theme.duration, &theme.size)
    }

    /// The kit's terms: `w-2` (tailwind's spacing 2) for the strip, the
    /// core durations (`duration-100`, `duration-300`) and sizes.
    pub fn hoff_core(width: f32, link: &NavLinkRecipe) -> Self {
        Self::of(
            width,
            link,
            8.0,
            &DurationScale::hoff(),
            &SizeTokens::hoff(),
        )
    }

    fn of(
        width: f32,
        link: &NavLinkRecipe,
        zone: f32,
        duration: &DurationScale,
        size: &SizeTokens,
    ) -> Self {
        Self {
            width,
            zone,
            enter_delay: duration.fast,
            leave_delay: duration.slow,
            slide: duration.slow,
            slide_easing: EASE_IN_OUT,
            content_min_w: size.card_min_w,
            button: link.icon_box(),
            button_radius: link.tile.map_or(link.radius, |t| t.radius),
            button_icon: link.tile.map_or(link.icon, |t| t.icon),
        }
    }

    /// Whether a window `w` wide holds the panel and the page beside it:
    /// the page's narrowest content inside the gutter of the width it
    /// would get. A rule on the content, not on the screen's class.
    pub fn fits(&self, w: f32, theme: &Theme) -> bool {
        let page = w - self.width;
        let gutter = theme.layout.gutter_at(page.max(0.0));
        page >= self.content_min_w + gutter * 2.0
    }
}
