//! The kit's catalog cards, its CardsPage: the pricing card (PriceCard),
//! the asset card (AssetCard), the scene card (SceneCard) and its
//! skeleton (SceneCardSkeleton), the explore card (ExploreCard), its
//! small tag (ExploreCardSm) and the slide item. The visual values are
//! `theme.disclosure.catalog`'s; a photo of the kit is a rectangle in the
//! palette's `image` gradient.
//!
//! Each card is as wide as the kit draws it, or as wide as it is given
//! when that is less, and as tall as what it holds at that width. A card
//! is chosen: a click on it, or on a plan's button, selects it
//! (`clicked`). With a [`SemId`] ([`CatalogCard::sem`]) the render binds
//! where it hit-tests, and [`CatalogCard::project`] declares it a
//! `Role::Button` with `selected` and an `activate` that reads it; a
//! pricing card is a `Role::Group` around its plan's button,
//! `<id>.plan`. The hover's actions only show under the pointer, as in
//! the kit, and take no action of their own; the one under the pointer
//! hovers as the kit's does.

mod explore;
mod price;
mod tiles;

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::tokens::{CatalogPalette, CatalogTokens};
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Button, Press};
use crate::widgets::icons;
use crate::widgets::recipe::{gradient_fill, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// A pricing card's plan: the kit's three (PriceCard :14-16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plan {
    /// The plan in use: its button faded.
    Free,
    /// The featured plan: on the image, lifted.
    Creator,
    /// The plan to take: the dark button.
    Studio,
}

/// What a catalog card shows.
#[derive(Clone, Debug, PartialEq)]
pub enum CatalogKind {
    Price {
        plan: Plan,
        title: String,
        description: String,
        price: String,
        features: Vec<String>,
    },
    Asset,
    Scene {
        title: String,
        category: String,
        video: bool,
    },
    SceneSkeleton,
    Explore {
        title: String,
    },
    ExploreSm {
        title: String,
        description: String,
    },
    Slide,
}

#[derive(Clone, Debug)]
pub struct CatalogCard {
    pub kind: CatalogKind,
    press: Press,
    /// The hover's action under the pointer, by index in
    /// [`CatalogCard::actions`].
    hot: Option<usize>,
    selected: bool,
    /// A plan's button (pricing cards only).
    button: Option<Button>,
    sem: Option<SemId>,
}

impl CatalogCard {
    pub fn new(kind: CatalogKind) -> Self {
        let button = match &kind {
            CatalogKind::Price { plan, .. } => Some(price::button(*plan)),
            _ => None,
        };
        Self {
            kind,
            press: Press::default(),
            hot: None,
            selected: false,
            button,
            sem: None,
        }
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn selected(mut self, on: bool) -> Self {
        self.selected = on;
        self
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, on: bool) -> bool {
        let changed = self.selected != on;
        self.selected = on;
        changed
    }

    /// Shows the hover (the pointer over it), as a pinned state.
    pub fn set_hovered(&mut self, on: bool) {
        self.press.hovered = on;
        if !on {
            self.hot = None;
        }
    }

    /// The hover's action under the pointer, by index in
    /// [`CatalogCard::actions`].
    pub fn hot_action(&self) -> Option<usize> {
        self.hot
    }

    /// The buttons the hover shows in `bounds`: the asset card's add,
    /// download and share, the explore card's three, left to right; none
    /// for the others.
    pub fn actions(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        match self.kind {
            CatalogKind::Asset => tiles::asset_actions(bounds, theme).to_vec(),
            CatalogKind::Explore { .. } => explore::explore_actions(bounds, theme).to_vec(),
            _ => Vec::new(),
        }
    }

    pub fn is_hovered(&self) -> bool {
        self.press.hovered
    }

    /// What the projection and a reader call it.
    pub fn label(&self) -> &str {
        match &self.kind {
            CatalogKind::Price { title, .. }
            | CatalogKind::Scene { title, .. }
            | CatalogKind::Explore { title }
            | CatalogKind::ExploreSm { title, .. } => title,
            CatalogKind::Asset => "Asset",
            CatalogKind::SceneSkeleton => "Loading scene",
            CatalogKind::Slide => "Slide",
        }
    }

    /// Its size in `w` of room: the kit's width or less, the height of
    /// what it holds at that width.
    pub fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let t = &theme.disclosure.catalog;
        match &self.kind {
            CatalogKind::Price { .. } => price::size(self, w, theme),
            CatalogKind::Asset => {
                let w = t.tile.asset_w.min(w);
                (w, w)
            }
            CatalogKind::Scene { .. } | CatalogKind::SceneSkeleton => tiles::scene_size(w, theme),
            CatalogKind::Explore { .. } => {
                let w = t.explore.width.min(w);
                (w, w / t.explore.ratio)
            }
            CatalogKind::ExploreSm { .. } => {
                let e = &t.explore;
                (e.tag_w.min(w), e.tag_pad * 2.0 + e.thumb + EDGE * 2.0)
            }
            CatalogKind::Slide => {
                let s = t.explore.slide + EDGE * 2.0;
                (s.min(w), s)
            }
        }
    }

    /// The plan's button in `bounds` (pricing cards only).
    pub fn button_rect(&self, bounds: Rect, theme: &Theme) -> Option<Rect> {
        match &self.kind {
            CatalogKind::Price { .. } => Some(price::layout(self, bounds, theme).button),
            _ => None,
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut out = EventResult::IGNORED;
        if let Some(at) = self.button_rect(bounds, theme)
            && let Some(button) = &mut self.button
        {
            let r = button.handle_event(event, at);
            if r.clicked {
                self.selected = true;
                return r;
            }
            out = r;
        }
        let mut r = self.press.handle(event, bounds, false);
        if let WidgetEvent::MouseMove { x, y } = *event {
            let hot = if self.press.hovered {
                self.actions(bounds, theme)
                    .iter()
                    .position(|a| a.contains(x, y))
            } else {
                None
            };
            if hot != self.hot {
                self.hot = hot;
                r = EventResult::changed();
            }
        }
        if r.clicked && self.button.is_none() {
            self.selected = true;
            return r;
        }
        if r.handled || r.changed {
            return r;
        }
        out
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        match &self.kind {
            CatalogKind::Price { .. } => price::render(self, c, bounds, theme),
            CatalogKind::Asset => tiles::asset(self, c, bounds, theme),
            CatalogKind::Scene { .. } | CatalogKind::SceneSkeleton => {
                tiles::scene(self, c, bounds, theme)
            }
            CatalogKind::Explore { .. } => explore::card(self, c, bounds, theme),
            CatalogKind::ExploreSm { .. } => explore::tag(self, c, bounds, theme),
            CatalogKind::Slide => explore::slide(self, c, bounds, theme),
        }
    }

    /// Declares the card as `id`: a `Role::Button` named by its label with
    /// `selected` and an `activate` that reads it, or, a pricing card, a
    /// `Role::Group` around its plan's button `<id>.plan`, enabled unless
    /// it is the plan in use.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        if let (Some(button), CatalogKind::Price { plan, .. }) = (&self.button, &self.kind)
            && let Ok(plan_id) = id.child("plan")
        {
            let mut node = b.node(&plan_id, Role::Button, button.label.as_str());
            node.parent(id)
                .enabled(*plan != Plan::Free)
                .state("selected", self.selected);
            node.action("activate").reads(format!("{id}.plan:selected"));
            let mut group = b.node(id, Role::Group, self.label());
            if let Some(parent) = parent {
                group.parent(parent);
            }
            return group;
        }
        let mut node = b.node(id, Role::Button, self.label());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("selected", self.selected);
        node.action("activate").reads(format!("{id}:selected"));
        node
    }
}

/// The kit's `border` is one pixel.
const EDGE: f32 = 1.0;

fn palette(theme: &Theme) -> &CatalogPalette {
    &theme.disclosure.catalog.palette
}

fn tokens(theme: &Theme) -> &CatalogTokens {
    &theme.disclosure.catalog
}

/// A filled, bordered, rounded box.
fn boxed(c: &mut Compositor, r: Rect, radius: f32, fill: Color, border: Color) {
    c.push(rounded_rect(r.x, r.y, r.w, r.h, radius, fill.0));
    c.push(rounded_rect_stroke(
        r.x, r.y, r.w, r.h, radius, border.0, EDGE,
    ));
}

/// A photo: a rectangle in the image gradient.
fn photo(c: &mut Compositor, r: Rect, radius: f32, theme: &Theme) {
    let [top, bottom] = palette(theme).image;
    c.push(gradient_fill(r, radius, top.0, bottom.0));
}

/// The hover of a button on a photo: the glass fill and its edge.
fn glass(c: &mut Compositor, r: Rect, radius: f32, theme: &Theme) {
    let k = palette(theme);
    boxed(c, r, radius, k.glass, k.glass_line);
}

/// The scrim a hover lays from the bottom of an image, `h` tall, at
/// `alpha` at its foot.
fn scrim(c: &mut Compositor, r: Rect, h: f32, alpha: f32, radius: f32, theme: &Theme) {
    let s = palette(theme).scrim;
    let at = Rect::new(r.x, r.bottom() - h, r.w, h);
    let foot = [s.0[0], s.0[1], s.0[2], alpha];
    let top = [s.0[0], s.0[1], s.0[2], 0.0];
    c.push(gradient_fill(at, radius, top, foot));
}

fn text(
    c: &mut Compositor,
    s: &str,
    style: &TextStyle,
    x: f32,
    y: f32,
    color: Color,
    w: Option<f32>,
) {
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(s, style, w),
        x,
        y,
        color: color.0,
    });
}

fn icon(c: &mut Compositor, name: &str, size: f32, color: Color, x: f32, y: f32) {
    if let Some(node) = icons::icon_at(name, size, color.0, x, y) {
        c.push(node);
    }
}

#[cfg(test)]
mod tests;
