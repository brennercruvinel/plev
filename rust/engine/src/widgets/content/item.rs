//! The item of a list, in the kit's two shapes: the Folder row (an icon in
//! a box, a label that wraps, an optional dim line under it, a wash on
//! hover) and the AssetItem tile (a thumbnail in a bordered square well,
//! or a photo filling it, an add mark in its corner on hover). The visual
//! values are `theme.disclosure.item`'s; the photo is a rectangle in the
//! catalog's `image` gradient (`theme.disclosure.catalog.palette`).
//!
//! An item is chosen: a click on it selects it (`clicked`), as a list's
//! row is. With a [`SemId`] ([`Item::sem`]) the render binds the bounds it
//! hit-tests with, and [`Item::project`] declares it a `Role::Option` with
//! `selected` and a `select` that reads it (the app runs it,
//! [`Item::set_selected`]). The add mark only marks the tile under the
//! pointer: the kit gives it no action, and neither does this.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::tokens::{DisclosureSurface, ItemRecipe};
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring_inside, gradient_fill, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The kit's two items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemShape {
    /// The Folder row: full width, as tall as its text.
    #[default]
    Row,
    /// The AssetItem tile: a square of the recipe's edge.
    Tile,
}

/// One item: a row or a tile.
#[derive(Clone, Debug)]
pub struct Item {
    title: String,
    meta: Option<String>,
    icon: Option<&'static str>,
    icon_color: Option<Color>,
    shape: ItemShape,
    full_image: bool,
    selected: bool,
    disabled: bool,
    press: Press,
    sem: Option<SemId>,
}

impl Item {
    /// A row titled `title`.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            meta: None,
            icon: None,
            icon_color: None,
            shape: ItemShape::Row,
            full_image: false,
            selected: false,
            disabled: false,
            press: Press::default(),
            sem: None,
        }
    }

    pub fn shape(mut self, shape: ItemShape) -> Self {
        self.shape = shape;
        self
    }

    /// The row's leading icon, or the tile's thumbnail.
    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(name);
        self
    }

    /// The tile's photo fills it inside its border, in place of the
    /// thumbnail (AssetItem's `fullSizeImage`, its `size-full
    /// object-cover` in an `overflow-hidden` well).
    pub fn full_image(mut self, on: bool) -> Self {
        self.full_image = on;
        self
    }

    /// The icon's color (the kit's Folder takes the folder's own).
    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = Some(color);
        self
    }

    /// A dim line under the row's label.
    pub fn meta(mut self, text: impl Into<String>) -> Self {
        self.meta = Some(text.into());
        self
    }

    pub fn selected(mut self, on: bool) -> Self {
        self.selected = on;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        if on {
            self.press = Press::default();
        }
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Selects it or lets it go; `false` when disabled.
    pub fn set_selected(&mut self, on: bool) -> bool {
        if self.disabled {
            return false;
        }
        self.selected = on;
        true
    }

    /// The pointer over it, for an owner that hit-tests its rows itself
    /// (a virtualized list).
    pub fn set_hovered(&mut self, on: bool) {
        self.press.hovered = on && !self.disabled;
    }

    pub fn set_focused(&mut self, on: bool) {
        self.press.focused = on;
    }

    pub fn focused(&self) -> bool {
        self.press.focused
    }

    fn label_style(r: &ItemRecipe, theme: &Theme) -> TextStyle {
        theme
            .typography
            .exact(r.font_size, r.line_height, r.font_weight)
    }

    fn meta_style(r: &ItemRecipe, theme: &Theme) -> TextStyle {
        theme
            .typography
            .exact(r.meta_size, r.meta_line_height, r.meta_weight)
    }

    /// The width the row's text wraps at, in a row `w` wide.
    fn text_w(&self, r: &ItemRecipe, w: f32) -> f32 {
        let lead = if self.icon.is_some() {
            r.icon_box + r.gap
        } else {
            0.0
        };
        (w - (r.row_border + r.row_pad) * 2.0 - lead).max(0.0)
    }

    /// The row's label and meta heights at width `w`.
    fn text_h(&self, r: &ItemRecipe, w: f32, theme: &Theme) -> (f32, f32) {
        let tw = self.text_w(r, w);
        let label = Self::label_style(r, theme);
        let (_, label_h) = TextMeasurer::measure_styled(&self.title, &label, Some(tw));
        let meta_h = self.meta.as_deref().map_or(0.0, |meta| {
            TextMeasurer::measure_styled(meta, &Self::meta_style(r, theme), Some(tw)).1
        });
        (label_h.max(label.line_height), meta_h)
    }

    /// Its size at width `w`: a row takes the width and is as tall as its
    /// text or its icon box; a tile is the recipe's square.
    pub fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let r = &theme.disclosure.item;
        match self.shape {
            ItemShape::Tile => (r.tile, r.tile),
            ItemShape::Row => {
                let (label_h, meta_h) = self.text_h(r, w, theme);
                let inner = (label_h + meta_h).max(r.icon_box);
                (w, inner + (r.row_border + r.row_pad) * 2.0)
            }
        }
    }

    /// Hover and press; a release inside selects it (`clicked`). A
    /// disabled item reacts to nothing.
    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        let r = self.press.handle(event, bounds, self.disabled);
        if r.clicked {
            self.selected = true;
        }
        r
    }

    fn surface<'r>(&self, r: &'r ItemRecipe) -> &'r DisclosureSurface {
        match (self.shape, self.selected, self.press.hovered) {
            (ItemShape::Row, true, _) => &r.row_selected,
            (ItemShape::Row, false, true) => &r.row_hover,
            (ItemShape::Row, false, false) => &r.row_rest,
            (ItemShape::Tile, true, _) => &r.tile_selected,
            (ItemShape::Tile, false, true) => &r.tile_hover,
            (ItemShape::Tile, false, false) => &r.tile_rest,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let r = &theme.disclosure.item;
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let alpha = if self.disabled {
            theme.glass.disabled_alpha
        } else {
            1.0
        };
        let (radius, border) = match self.shape {
            ItemShape::Row => (r.row_radius, r.row_border),
            ItemShape::Tile => (r.tile_radius, r.tile_border),
        };
        let radius = radius.min(bounds.h / 2.0);
        let surface = self.surface(r);
        if surface.fill.0[3] > 0.0 {
            let fill = faded(surface.fill, alpha);
            c.push(rounded_rect(
                bounds.x, bounds.y, bounds.w, bounds.h, radius, fill,
            ));
        }
        if border > 0.0 && surface.border.0[3] > 0.0 {
            c.push(rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                faded(surface.border, alpha),
                border,
            ));
        }
        match self.shape {
            ItemShape::Row => self.paint_row(c, bounds, r, alpha, theme),
            ItemShape::Tile => self.paint_tile(c, bounds, radius, alpha, theme),
        }
        if self.press.focused && !self.disabled {
            c.push(focus_ring_inside(bounds, radius, theme));
        }
    }

    fn paint_row(&self, c: &mut Compositor, at: Rect, r: &ItemRecipe, alpha: f32, theme: &Theme) {
        let edge = r.row_border + r.row_pad;
        let inner = Rect::new(
            at.x + edge,
            at.y + edge,
            (at.w - edge * 2.0).max(0.0),
            (at.h - edge * 2.0).max(0.0),
        );
        let mut x = inner.x;
        if let Some(name) = self.icon {
            let inset = (r.icon_box - r.icon) / 2.0;
            let y = inner.y + (inner.h - r.icon_box) / 2.0 + inset;
            let color = faded(self.icon_color.unwrap_or(r.icon_color), alpha);
            if let Some(node) = icons::icon_at(name, r.icon, color, x + inset, y) {
                c.push(node);
            }
            x += r.icon_box + r.gap;
        }
        let tw = self.text_w(r, at.w);
        let (label_h, meta_h) = self.text_h(r, at.w, theme);
        let mut y = inner.y + (inner.h - label_h - meta_h) / 2.0;
        c.push_clip(x, inner.y, tw, inner.h);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&self.title, &Self::label_style(r, theme), Some(tw)),
            x,
            y,
            color: faded(r.label, alpha),
        });
        y += label_h;
        if let Some(meta) = &self.meta {
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(meta, &Self::meta_style(r, theme), Some(tw)),
                x,
                y,
                color: faded(r.meta, alpha),
            });
        }
        c.pop_clip();
    }

    fn paint_tile(&self, c: &mut Compositor, at: Rect, radius: f32, alpha: f32, theme: &Theme) {
        let r = &theme.disclosure.item;
        if self.full_image {
            let b = r.tile_border;
            let [top, bottom] = theme.disclosure.catalog.palette.image;
            c.push(gradient_fill(
                Rect::new(at.x + b, at.y + b, at.w - b * 2.0, at.h - b * 2.0),
                (radius - b).max(0.0),
                faded(top, alpha),
                faded(bottom, alpha),
            ));
        } else if let Some(name) = self.icon {
            let color = faded(self.icon_color.unwrap_or(r.icon_color), alpha);
            let (x, y) = (at.x + (at.w - r.thumb) / 2.0, at.y + (at.h - r.thumb) / 2.0);
            if let Some(node) = icons::icon_at(name, r.thumb, color, x, y) {
                c.push(node);
            }
        }
        if self.press.hovered && !self.disabled {
            let add = Rect::new(
                at.right() - r.add_inset - r.add,
                at.y + r.add_inset,
                r.add,
                r.add,
            );
            let [top, bottom] = r.add_fill;
            c.push(gradient_fill(add, r.add / 2.0, top.0, bottom.0));
            let inset = (r.add - r.add_glyph) / 2.0;
            if let Some(node) = icons::icon_at(
                "plus",
                r.add_glyph,
                r.add_glyph_color.0,
                add.x + inset,
                add.y + inset,
            ) {
                c.push(node);
            }
        }
    }

    /// Declares the item as `id`: a `Role::Option` named by its title,
    /// enabled unless disabled, with `selected` and a `select` that reads
    /// it. Pass the id given to [`Item::sem`].
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Option, self.title.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.enabled(!self.disabled)
            .state("selected", self.selected);
        node.action("select").reads(format!("{id}:selected"));
        if let Some(meta) = &self.meta {
            node.description(meta.as_str());
        }
        node
    }
}

fn faded(c: Color, alpha: f32) -> [f32; 4] {
    [c.0[0], c.0[1], c.0[2], c.0[3] * alpha]
}

#[cfg(test)]
mod tests {
    use accord::Value;

    use super::*;

    fn core() -> Theme {
        Theme::hoff_core()
    }

    #[test]
    fn a_row_is_as_tall_as_the_kit_and_grows_with_its_text() {
        let theme = core();
        let row = Item::new("Untitled Folder").icon("folder");
        // border, p-0.75 around the size-8 icon box.
        assert_eq!(row.size(240.0, &theme), (240.0, 40.0));
        let long = Item::new("3D Icons imported from the shared library of the studio")
            .icon("folder")
            .meta("12 files, edited today");
        assert!(long.size(140.0, &theme).1 > long.size(900.0, &theme).1);
        let tile = Item::new("Cube").shape(ItemShape::Tile);
        assert_eq!(tile.size(240.0, &theme), (60.0, 60.0));
    }

    #[test]
    fn a_click_selects_it_and_a_disabled_one_stays() {
        let bounds = Rect::new(0.0, 0.0, 240.0, 40.0);
        let (x, y) = bounds.center();
        let mut item = Item::new("Untitled Folder");
        assert!(
            item.handle_event(&WidgetEvent::MouseMove { x, y }, bounds)
                .changed
        );
        item.handle_event(&WidgetEvent::MouseDown { x, y }, bounds);
        assert!(
            item.handle_event(&WidgetEvent::MouseUp { x, y }, bounds)
                .clicked
        );
        assert!(item.is_selected());
        let mut off = Item::new("Untitled Folder").disabled(true);
        off.handle_event(&WidgetEvent::MouseDown { x, y }, bounds);
        assert_eq!(
            off.handle_event(&WidgetEvent::MouseUp { x, y }, bounds),
            EventResult::IGNORED
        );
        assert!(!off.is_selected() && !off.set_selected(true));
    }

    #[test]
    fn it_projects_as_an_option_with_selected_and_binds_where_it_draws() {
        let theme = core();
        let id = SemId::parse("items.folder").unwrap();
        let item = Item::new("3D Icons")
            .meta("12 files")
            .selected(true)
            .sem(id.clone());
        let mut b = SnapshotBuilder::new();
        item.project(&mut b, &id, None);
        let seen = b.finish().unwrap();
        let node = seen.get(&id).unwrap();
        assert_eq!(node.role, Role::Option);
        assert_eq!(node.state.get("selected"), Some(&Value::Bool(true)));
        assert_eq!(node.description.as_deref(), Some("12 files"));
        let mut c = Compositor::new();
        c.begin_frame();
        let at = Rect::new(4.0, 8.0, 240.0, 40.0);
        item.render(&mut c, at, &theme);
        assert_eq!(c.binding(&id).unwrap().rect, [4.0, 8.0, 240.0, 40.0]);
    }

    /// The tile shows its add mark only under the pointer.
    #[test]
    fn the_tile_marks_its_add_button_on_hover() {
        let theme = core();
        let at = Rect::new(0.0, 0.0, 60.0, 60.0);
        let paths = |item: &Item| {
            let mut c = Compositor::new();
            c.begin_frame();
            item.render(&mut c, at, &theme);
            c.layer(crate::drawing::compositor::LayerId::DEFAULT)
                .unwrap()
                .nodes()
                .iter()
                .filter(|n| matches!(n, SceneNode::Path { .. }))
                .count()
        };
        let mut tile = Item::new("Cube").shape(ItemShape::Tile).icon("square");
        let rest = paths(&tile);
        tile.set_hovered(true);
        assert_eq!(paths(&tile), rest + 1, "the plus glyph");
    }

    /// A full image fills the tile inside its border, in the catalog's
    /// photo gradient, under the add mark.
    #[test]
    fn a_full_image_fills_the_tile_inside_its_border() {
        let theme = core();
        let at = Rect::new(10.0, 20.0, 60.0, 60.0);
        let mut tile = Item::new("Style").shape(ItemShape::Tile).full_image(true);
        tile.set_hovered(true);
        let mut c = Compositor::new();
        c.begin_frame();
        tile.render(&mut c, at, &theme);
        let nodes = c
            .layer(crate::drawing::compositor::LayerId::DEFAULT)
            .unwrap()
            .nodes();
        let image = nodes
            .iter()
            .position(|n| matches!(n, SceneNode::GradientRect { w, .. } if *w > 50.0))
            .expect("the photo");
        let SceneNode::GradientRect {
            x, y, w, h, color, ..
        } = &nodes[image]
        else {
            unreachable!()
        };
        let edge = theme.disclosure.item.tile_border;
        assert_eq!([*x, *y], [at.x + edge, at.y + edge]);
        assert_eq!([*w, *h], [at.w - edge * 2.0, at.h - edge * 2.0]);
        assert_eq!(*color, theme.disclosure.catalog.palette.image[0].0);
        assert!(
            nodes[image + 1..]
                .iter()
                .any(|n| matches!(n, SceneNode::Path { .. })),
            "the add mark over it"
        );
    }
}
