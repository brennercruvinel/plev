//! The adaptive foundation (adr-0086): the person's and the system's
//! preferences, read once into [`Prefs`], become tokens in one place,
//! [`Theme::adapted`]. A widget keeps reading only `&Theme`. With the
//! default preferences the adapted theme is the base, slot for slot.
//!
//! task-104 front M brings the contrast: [`Contrast::More`] holds the
//! palette's text at AAA (7:1) and its marks at 4.5:1 on every surface
//! (`legibility::settle`), makes the glass opaque, and widens the focus
//! ring and the control's edge. Every slot the base derived from its
//! palette derives again from the adapted one; a slot a theme copies from
//! its kit (the core's) keeps its own until its front adds an adapt step.

use super::Theme;
use super::legibility::Bar;
use crate::control::accessibility::preference;
use crate::styling::color::Color;
use crate::widgets::buttons::tokens::ButtonTokens;
use crate::widgets::content::tokens::DisclosureTokens;
use crate::widgets::form::tokens::FormTokens;
use crate::widgets::header::tokens::HeaderTokens;
use crate::widgets::message::tokens::MessageTokens;
use crate::widgets::nav::tokens::NavigationTokens;
use crate::widgets::overlay::tokens::OverlayTokens;
use crate::widgets::selection::tokens::SelectionTokens;

/// How much contrast the person asks for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Contrast {
    /// The theme as designed (WCAG AA, `legibility::Bar::AA`).
    #[default]
    Standard,
    /// More contrast: the system's "increase contrast" or the app's choice.
    More,
}

/// The preferences a theme adapts to. Each front adds its field (text
/// scale, density, input, zoom) with its slot's adapt step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Prefs {
    pub contrast: Contrast,
}

impl Prefs {
    /// What this thread knows of the system: the contrast the last
    /// `preference::follow_contrast` read, or a flag set
    /// (`preference::set_more_contrast`).
    pub fn from_system() -> Self {
        Self {
            contrast: if preference::more_contrast() {
                Contrast::More
            } else {
                Contrast::Standard
            },
        }
    }
}

/// `c` laid on `under` (opaque), as one opaque color: what the window
/// shows, blended in linear light as the GPU blends into the sRGB surface.
fn opaque(c: Color, under: Color) -> Color {
    let encode = |l: f32| {
        if l <= 0.003_130_8 {
            l * 12.92
        } else {
            1.055 * l.powf(1.0 / 2.4) - 0.055
        }
    };
    let (top, low) = (c.to_linear_array(), under.to_linear_array());
    let a = top[3];
    let mix = |i: usize| encode(top[i] * a + low[i] * (1.0 - a));
    Color([mix(0), mix(1), mix(2), 1.0])
}

impl Theme {
    /// This theme adapted to `prefs`: the only place a preference becomes
    /// a token (adr-0086). With `Prefs::default()` it is this theme.
    pub fn adapted(&self, prefs: &Prefs) -> Theme {
        let mut t = self.clone();
        if prefs.contrast == Contrast::More {
            t.more_contrast(self);
        }
        t
    }

    /// `Contrast::More` over `base` (which `self` is a copy of).
    fn more_contrast(&mut self, base: &Theme) {
        let bg = self.colors.bg;
        let g = &mut self.glass;
        for c in [
            &mut g.surface,
            &mut g.surface_hover,
            &mut g.surface_active,
            &mut g.button,
            &mut g.tabs,
            &mut g.button_hover,
            &mut g.field,
            &mut g.popover,
            &mut g.tooltip,
        ] {
            *c = opaque(*c, bg);
        }
        self.settle_colors(Bar::MORE);
        // the edge a person must see, and the focus ring a step wider
        let k = &mut self.control;
        k.edge_width = k.edge_width.max(k.edge_width_strong);
        k.focus_ring_width += k.edge_width_strong.min(k.focus_ring_width);
        // what the base derived from its palette, derived again
        if *base.button == ButtonTokens::derive(&base.colors) {
            *self.button = ButtonTokens::derive(&self.colors);
        }
        macro_rules! again {
            ($slot:ident, $ty:ty) => {
                if *base.$slot == <$ty>::derive(base) {
                    *self.$slot = <$ty>::derive(self);
                }
            };
        }
        again!(form, FormTokens);
        again!(overlay, OverlayTokens);
        again!(message, MessageTokens);
        again!(disclosure, DisclosureTokens);
        again!(selection, SelectionTokens);
        again!(navigation, NavigationTokens);
        again!(header, HeaderTokens);
        let link = &base.navigation.sidebar_link;
        if *link == NavigationTokens::sidebar_link_of_before(base) {
            self.navigation.sidebar_link = NavigationTokens::sidebar_link_of_before(self);
        }
    }
}
