//! The person's and the platform's preferences, read once into [`Prefs`],
//! become tokens in one place: [`Theme::adapted`] (ADR 0086). A widget
//! keeps reading only `&Theme`; with `Prefs::default()` every slot is the
//! base theme's, value for value.
//!
//! Each preference is a field and a step of `adapted`; a front adds its
//! own (contrast, density, input...) beside the text scale, never a
//! factor of its own elsewhere.

use super::Theme;
use super::roles::{TypeRole, TypeRun};
use super::text_scale::TextScale;
use super::tokens::TypographyScale;

/// What the person asked of every app, as the platform said it or as the
/// app overrode it (a launch argument, a test).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Prefs {
    /// Dynamic type, font scaling: how much larger text is drawn.
    pub text_scale: TextScale,
}

impl Prefs {
    /// The preferences with `text_scale` set.
    pub fn with_text_scale(mut self, text_scale: TextScale) -> Self {
        self.text_scale = text_scale;
        self
    }
}

impl Theme {
    /// This theme as `prefs` ask: a base theme (a built-in constructor's
    /// or `named`'s), adapted once. The identity for the default
    /// preferences.
    pub fn adapted(&self, prefs: &Prefs) -> Theme {
        let mut theme = self.clone();
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
        // an icon beside a label grows with the label's size
        // (`@ScaledMetric(relativeTo:)`), its gap does not
        for m in self.button.sizes.iter_mut() {
            m.height = g.h(m.height, m.text);
            m.icon = g.metric(m.icon, m.text);
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
            menu.icon = g.metric(menu.icon, menu.label);
        }
        let popover = &mut overlay.popover;
        popover.trigger.height = g.h(popover.trigger.height, popover.trigger.value);
        popover.line.height = g.h(popover.line.height, popover.line.label);

        let nav = &mut *self.navigation;
        for link in [&mut nav.link, &mut nav.sidebar_link] {
            link.height = g.h(link.height, link.label);
            link.icon = g.metric(link.icon, link.label);
            link.slot = link.slot.max(link.icon);
            if let Some(tile) = &mut link.tile {
                // the tile grows by the line, as the link holding it; its
                // icon by the label, never past the tile's own inset
                let inset = tile.size - tile.icon;
                tile.size = g.h(tile.size, link.label);
                tile.icon = g.metric(tile.icon, link.label).min(tile.size - inset);
            }
        }
        let canvas = &mut nav.sidebars.canvas;
        canvas.row_h = g.h(canvas.row_h, canvas.label);
        let design = &mut canvas.design;
        design.header_h = g.h(design.header_h, design.header);
        // the design sidebar's wells hold a line of their text; the unit
        // and the value chips hold a word of it beside a field
        design.field_h = g.h(design.field_h, design.field_text);
        for w in [
            &mut design.unit_w,
            &mut design.chip_w,
            &mut design.chip_w_narrow,
        ] {
            *w = g.metric(*w, design.field_text);
        }
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

    /// `v`, a measure beside a text of `run` (an icon by its label),
    /// grown by what the run's size grew (`TextScale::metric`), to a
    /// whole pixel; `v` itself where the size did not grow.
    fn metric(&self, v: f32, run: TypeRun) -> f32 {
        let (was, now) = (self.base.run(run).font_size, self.now.run(run).font_size);
        if now > was && was > 0.0 {
            (v * now / was).round()
        } else {
            v
        }
    }
}
