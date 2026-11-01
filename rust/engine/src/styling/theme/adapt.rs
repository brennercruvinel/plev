//! The person's and the platform's preferences, read once into [`Prefs`],
//! become tokens in one place: [`Theme::adapted`] (ADR 0086). A widget
//! keeps reading only `&Theme`; with `Prefs::default()` every slot is the
//! base theme's, value for value.
//!
//! Each preference is a field and a step of `adapted`; a front adds its
//! own (contrast, density, input...) beside the text scale, never a
//! factor of its own elsewhere.
//!
//! task-104 front M brings the contrast: [`Contrast::More`] holds the
//! palette's text at AAA (7:1) and its marks at 4.5:1 on every surface
//! (`legibility::settle`), makes the glass opaque, holds the active
//! border and the fields' edges at 3:1 (`Theme::hold_edges`; the
//! standard themes keep the kit's, task-104 option B), and widens the
//! focus ring and the control's edge. Every slot the base derived from its
//! palette derives again from the adapted one; a slot a theme copies from
//! its kit (the core's) keeps its own until its front adds an adapt step.

use super::Theme;
use super::legibility::Bar;
use super::roles::{TypeRole, TypeRun};
use super::text_scale::TextScale;
use super::tokens::TypographyScale;
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

impl Contrast {
    /// What the system says (`preference::follow_contrast`, which this
    /// thread then keeps); `Standard` where it cannot be read.
    pub fn from_system() -> Self {
        match preference::follow_contrast() {
            Some(true) => Self::More,
            _ => Self::Standard,
        }
    }
}

/// What the person asked of every app, as the platform said it or as the
/// app overrode it (a launch argument, a test).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Prefs {
    /// Dynamic type, font scaling: how much larger text is drawn.
    pub text_scale: TextScale,
    /// Increase contrast.
    pub contrast: Contrast,
}

impl Prefs {
    /// The preferences with `text_scale` set.
    pub fn with_text_scale(mut self, text_scale: TextScale) -> Self {
        self.text_scale = text_scale;
        self
    }

    /// The preferences with `contrast` set.
    pub fn with_contrast(mut self, contrast: Contrast) -> Self {
        self.contrast = contrast;
        self
    }
}

impl Theme {
    /// This theme as `prefs` ask: a base theme (a built-in constructor's
    /// or `named`'s), adapted once. The identity for the default
    /// preferences.
    pub fn adapted(&self, prefs: &Prefs) -> Theme {
        let mut theme = self.clone();
        // the contrast first: it derives slots again from the palette,
        // which the text scale then grows
        if prefs.contrast == Contrast::More {
            theme.more_contrast(self);
        }
        theme.adapt_text(prefs.text_scale);
        theme
    }

    /// The person's text size this theme draws at (ADR 0086): a measure
    /// that follows a text, an icon beside a label, grows by
    /// `theme.text_scale().metric(v, relative_to)`.
    pub fn text_scale(&self) -> &TextScale {
        &self.typography.text_scale
    }

    /// The text scale: every style takes it ([`TypographyScale::exact`]),
    /// and every height that holds a line of text grows by what its line
    /// grew, `h + (lh' − lh)`. That is `max(token, lh' + 2·pad)` with the
    /// token's own padding, `pad = (token − lh) / 2`: the kit's measure
    /// stays the minimum, the padding is kept, and padding and gaps never
    /// scale (ADR 0086).
    fn adapt_text(&mut self, scale: TextScale) {
        if scale == self.typography.text_scale {
            return;
        }
        let mut base = self.typography.clone();
        base.text_scale = TextScale::IDENTITY;
        self.typography.text_scale = scale;
        let g = Grow {
            base: &base,
            now: &self.typography,
        };
        // the generic ladder: fields, table rows, the controls that take a
        // `ControlSize`; a label's line, the run the kit writes most
        let label = TypeRole::Label.run();
        for h in self.control.heights.iter_mut() {
            *h = g.h(*h, label);
        }
        for m in self.button.sizes.iter_mut() {
            m.height = g.h(m.height, m.text);
        }
        let sel = &mut *self.selection;
        for m in sel.segmented.sizes.iter_mut() {
            m.height = g.h(m.height, m.text);
        }
        sel.radio.row_h = g.h(sel.radio.row_h, sel.radio.text);
        sel.folders.row_h = g.h(sel.folders.row_h, sel.folders.text);

        let form = &mut *self.form;
        for input in [
            &mut form.input,
            &mut form.search,
            &mut form.inset,
            &mut form.header,
            &mut form.combo,
            &mut form.bare,
        ] {
            input.height = g.h(input.height, input.text);
        }
        form.edit.box_h = g.h(form.edit.box_h, form.edit.text);
        let select = &mut form.select;
        select.height = g.h(select.height, select.text);
        select.small_height = g.h(select.small_height, select.small_text);
        select.panel.row_h = g.h(select.panel.row_h, select.panel.text);

        let overlay = &mut *self.overlay;
        for menu in [
            &mut overlay.menu,
            &mut overlay.file_menu,
            &mut overlay.filter_menu,
            &mut overlay.upload_menu,
        ] {
            menu.item_h = g.h(menu.item_h, menu.label);
        }
        let popover = &mut overlay.popover;
        popover.trigger.height = g.h(popover.trigger.height, popover.trigger.value);
        popover.line.height = g.h(popover.line.height, popover.line.label);

        let nav = &mut *self.navigation;
        for link in [&mut nav.link, &mut nav.sidebar_link] {
            link.height = g.h(link.height, link.label);
        }
        let canvas = &mut nav.sidebars.canvas;
        canvas.row_h = g.h(canvas.row_h, canvas.label);
        canvas.design.header_h = g.h(canvas.design.header_h, canvas.design.header);
        let zoom = &mut nav.tool.zoom;
        zoom.line_h = g.h(zoom.line_h, zoom.label);

        let tree = &mut self.disclosure.tree;
        tree.row_h = g.h(tree.row_h, tree.text_type);
    }
}

/// The two scales a height grows between: the theme's as designed and
/// the adapted one.
struct Grow<'a> {
    base: &'a TypographyScale,
    now: &'a TypographyScale,
}

impl Grow<'_> {
    /// `h` grown by what the line of `run` grew, to a whole pixel; `h`
    /// itself where the line did not grow.
    fn h(&self, h: f32, run: TypeRun) -> f32 {
        let grew = self.now.run(run).line_height - self.base.run(run).line_height;
        if grew > 0.0 { (h + grew).ceil() } else { h }
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
        // the fields' edges at 3:1, the kit's included
        self.hold_edges(Bar::MORE);
    }
}
