//! Card page: the kit's CardsPage (templates/CardsPage/index.tsx), group
//! for group, in the showcase mold: the pricing cards (Free, Studio,
//! Creator), the slide item at rest and active, the asset card, the scene
//! card and its skeleton, the explore tag, the explore card, and the
//! ratio and the two asset items. The widgets are
//! `engine::widgets::content`'s `CatalogCard` and `Item`; every visual
//! value comes from the theme (`theme.disclosure.catalog`, `.item`). A
//! photo of the kit is a rectangle in the catalog's image gradient.
//!
//! Each card is as tall as the kit's (`h-150`, `h-50`, `h-100`, `h-120`,
//! `h-59`, square) or as its widget needs, and each widget as wide as the
//! kit draws it or as its card leaves: the rows drop a column where a card
//! would get narrower than `showcase::CARD_MIN_W`.
//!
//! Projected: a group per title, `card_page.group[<g>]`; in it each card,
//! `...card[<c>]`, a button with `selected` whose `activate` chooses it as
//! a click does; a pricing card a group around its plan's button
//! `...card[<c>].plan`; the asset items options with `select`.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::styling::theme::Theme;
use engine::widgets::content::{CatalogCard, CatalogKind, Item, ItemShape, Plan};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect, rounded_rect_stroke};
use engine::widgets::recipe::focus_ring_inside;

use super::{After, Component, child_id, root_id, showcase, under};

/// The ratio a square card takes alone in its row (`aspect-[1.8]`).
const WIDE_RATIO: f32 = 1.8;

/// What a card of the page holds.
enum Body {
    Catalog(CatalogCard),
    Item(Item),
    /// The ratio label of the last group (CardsPage :170-177).
    Ratio(&'static str),
}

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    body: Body,
}

struct Group {
    title: &'static str,
    span: usize,
    /// The kit's card height (`h-150` is 600); `None` is square while
    /// the row keeps more than one card, and 1.8 times as wide as tall
    /// once a card takes the whole row (`aspect-square
    /// max-[1023px]:aspect-[1.8]`, CardsPage :160, where the kit's span
    /// 3 card goes full width).
    height: Option<f32>,
    cards: Vec<Card>,
}

pub struct CardPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The card, plan or item a reader or Tab focused.
    focus: Option<SemId>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn plan(plan: Plan, title: &str, description: &str, price: &str, features: [&str; 5]) -> Body {
    Body::Catalog(CatalogCard::new(CatalogKind::Price {
        plan,
        title: title.into(),
        description: description.into(),
        price: price.into(),
        features: features.iter().map(|f| (*f).to_string()).collect(),
    }))
}

impl CardPage {
    pub fn new() -> Self {
        let card = |title, body| Card { title, body };
        let catalog = |kind| Body::Catalog(CatalogCard::new(kind));
        let groups = vec![
            Group {
                title: "Pricing card",
                span: 3,
                height: Some(600.0),
                cards: vec![
                    card(
                        "Free",
                        plan(
                            Plan::Free,
                            "Free",
                            "Perfect for hobbyists, students, or early-stage creators.",
                            "0",
                            [
                                "Access to a basic asset library",
                                "15-second video animation",
                                "Generate up to 10 scenes/month",
                                "Watermarked exports",
                                "Try basic AI prompts",
                            ],
                        ),
                    ),
                    card(
                        "Studio",
                        plan(
                            Plan::Studio,
                            "Studio",
                            "For teams and studios that need power, speed.",
                            "40",
                            [
                                "Everything in Creator",
                                "Unlimited 3D scene generation",
                                "Full access to premium asset library",
                                "Animations up to 60 seconds",
                                "Unlimited Video AI models",
                            ],
                        ),
                    ),
                    card(
                        "Creator",
                        plan(
                            Plan::Creator,
                            "Creator",
                            "For indie creators, and startups who need high-quality output",
                            "20",
                            [
                                "Everything in Free",
                                "Unlimited 3D scene generation",
                                "Premium asset library access",
                                "Animations up to 30 seconds",
                                "20+ Video AI models",
                            ],
                        ),
                    ),
                ],
            },
            Group {
                title: "Slide item",
                span: 2,
                height: Some(200.0),
                cards: vec![
                    card("Default, Hover", catalog(CatalogKind::Slide)),
                    card(
                        "Active",
                        Body::Catalog(CatalogCard::new(CatalogKind::Slide).selected(true)),
                    ),
                ],
            },
            Group {
                title: "Asset",
                span: 1,
                height: Some(400.0),
                cards: vec![card("Default, Hover", catalog(CatalogKind::Asset))],
            },
            Group {
                title: "Asset",
                span: 2,
                height: Some(480.0),
                cards: vec![
                    card(
                        "Default, Hover",
                        catalog(CatalogKind::Scene {
                            title: "Futuristic Humanoid Robot".into(),
                            category: "Untitled Folder".into(),
                            video: false,
                        }),
                    ),
                    card("Skeleton", catalog(CatalogKind::SceneSkeleton)),
                ],
            },
            Group {
                title: "Explore tags",
                span: 1,
                height: Some(236.0),
                cards: vec![card(
                    "Default, Hover",
                    catalog(CatalogKind::ExploreSm {
                        title: "Product Design".into(),
                        description: "Minimalist, soft".into(),
                    }),
                )],
            },
            Group {
                title: "Explore tags",
                span: 1,
                height: Some(480.0),
                cards: vec![card(
                    "Default, Hover",
                    catalog(CatalogKind::Explore {
                        title: "Tran Mau Tri Tam".into(),
                    }),
                )],
            },
            Group {
                title: "Asset",
                span: 3,
                height: None,
                cards: vec![
                    card("Ratio", Body::Ratio("3:4")),
                    card(
                        "Material, Add",
                        Body::Item(Item::new("Material").shape(ItemShape::Tile).icon("layers")),
                    ),
                    card(
                        "Style, Add",
                        Body::Item(Item::new("Style").shape(ItemShape::Tile).full_image(true)),
                    ),
                ],
            },
        ];
        let mut page = Self {
            root: root_id("card_page"),
            groups,
            focus: None,
        };
        if let Some(root) = page.root.clone() {
            for (g, group) in page.groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(id) = card_id(&root, g, c) else {
                        continue;
                    };
                    match &mut card.body {
                        Body::Catalog(w) => *w = w.clone().sem(id),
                        Body::Item(w) => *w = w.clone().sem(id),
                        Body::Ratio(_) => {}
                    }
                }
            }
        }
        page
    }

    /// The widget's size in `w` of room.
    fn widget_size(body: &Body, w: f32, theme: &Theme) -> (f32, f32) {
        match body {
            Body::Catalog(card) => card.size(w, theme),
            Body::Item(item) => item.size(w, theme),
            Body::Ratio(_) => {
                let s = theme.disclosure.item.tile;
                (s, s)
            }
        }
    }

    /// Each group's cards, and in each card the widget's rect.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Rect>)> {
        let grids = self.groups.iter().map(|g| {
            let card_w = showcase::card_width(g.span, content, theme);
            let alone =
                showcase::columns_at(g.span, showcase::CARD_MIN_W, content.w, theme.spacing.sm)
                    == 1;
            let square = if alone && g.span > 1 {
                card_w / WIDE_RATIO
            } else {
                card_w
            };
            let room = showcase::body(Rect::new(0.0, 0.0, card_w, 0.0), theme).w;
            let heights = g
                .cards
                .iter()
                .map(|c| {
                    let (_, h) = Self::widget_size(&c.body, room, theme);
                    showcase::card_height(h, theme).max(g.height.unwrap_or(square))
                })
                .collect();
            showcase::Grid {
                span: g.span,
                min_w: showcase::CARD_MIN_W,
                heights,
            }
        });
        showcase::layout_grids(grids, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let rects = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        let room = showcase::body(*rect, theme).w;
                        showcase::centered(Self::widget_size(&card.body, room, theme), *rect)
                    })
                    .collect();
                (placed, rects)
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |(placed, _)| placed.rule_y - content.y)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let layout = self.layout(content, theme);
        let mut out = EventResult::IGNORED;
        for (group, (_, rects)) in self.groups.iter_mut().zip(&layout) {
            for (card, at) in group.cards.iter_mut().zip(rects) {
                let r = match &mut card.body {
                    Body::Catalog(w) => w.handle_event(event, *at, theme),
                    Body::Item(w) => w.handle_event(event, *at),
                    Body::Ratio(_) => EventResult::IGNORED,
                };
                if r.clicked {
                    return r;
                }
                if r.changed {
                    out = r;
                }
            }
        }
        out
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, rects))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), at) in group.cards.iter().zip(&placed.cards).zip(rects) {
                showcase::card(c, *rect, card.title, theme);
                match &card.body {
                    Body::Catalog(w) => w.render(c, *at, theme),
                    Body::Item(w) => w.render(c, *at, theme),
                    Body::Ratio(label) => ratio(c, *at, label, theme),
                }
                if let Some(ring) = self.focus_rect(card, *at, theme) {
                    c.push(focus_ring_inside(ring, theme.shape.nav, theme));
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }

    /// Where the focus ring goes when `card` holds the focus: the plan's
    /// button, or the widget.
    fn focus_rect(&self, card: &Card, at: Rect, theme: &Theme) -> Option<Rect> {
        let focus = self.focus.as_ref()?;
        match &card.body {
            Body::Catalog(w) => {
                let id = w.sem_id()?;
                if id == focus {
                    return Some(at);
                }
                let plan = id.child("plan").ok()?;
                (plan == *focus).then(|| w.button_rect(at, theme)).flatten()
            }
            Body::Item(w) => (w.sem_id()? == focus).then_some(at),
            Body::Ratio(_) => None,
        }
    }

    /// `(group, card, plan)` of an id of the page.
    fn at(&self, id: &SemId) -> Option<(usize, usize, bool)> {
        let root = self.root.as_ref()?;
        for (g, group) in self.groups.iter().enumerate() {
            for c in 0..group.cards.len() {
                let card = card_id(root, g, c)?;
                if &card == id {
                    return Some((g, c, false));
                }
                if card.child("plan").ok().as_ref() == Some(id) {
                    return Some((g, c, true));
                }
            }
        }
        None
    }
}

/// The ratio label (CardsPage :170): `size-15 rounded-[0.625rem]
/// bg-[#F1F1F1] border border-[#ECECEC]`, `text-[0.75rem] leading-[1rem]
/// font-medium text-[#7B7B7B]` (the catalog's muted text, for AAA).
fn ratio(c: &mut Compositor, at: Rect, label: &str, theme: &Theme) {
    let k = &theme.disclosure.catalog.palette;
    let tile = &theme.disclosure.item;
    let style = theme.typography.exact(12.0, 16.0, 500);
    c.push(rounded_rect(
        at.x,
        at.y,
        at.w,
        at.h,
        tile.tile_radius,
        k.wash.0,
    ));
    c.push(rounded_rect_stroke(
        at.x,
        at.y,
        at.w,
        at.h,
        tile.tile_radius,
        k.line.0,
        tile.tile_border,
    ));
    let (w, h) = engine::drawing::text::TextMeasurer::measure_styled(label, &style, None);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(label, &style, None),
        x: at.x + (at.w - w) / 2.0,
        y: at.y + (at.h - h) / 2.0,
        color: k.muted.0,
    });
}

impl Component for CardPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for (c, card) in group.cards.iter().enumerate() {
                let Some(id) = card_id(root, g, c) else {
                    continue;
                };
                match &card.body {
                    Body::Catalog(w) => {
                        w.project(b, &id, Some(&at)).description(card.title);
                    }
                    Body::Item(w) => {
                        w.project(b, &id, Some(&at)).description(card.title);
                    }
                    Body::Ratio(label) => {
                        b.node(&id, Role::Label, *label)
                            .parent(&at)
                            .description(card.title);
                    }
                }
            }
        }
        if let Some(id) = &self.focus {
            b.focus(id);
        }
    }

    fn run(&mut self, req: &Request) -> Result<After, String> {
        let refused = || format!("the card page has no {} on {}", req.action, req.target);
        let (g, c, on_plan) = self.at(&req.target).ok_or_else(refused)?;
        match (
            &mut self.groups[g].cards[c].body,
            req.action.as_str(),
            on_plan,
        ) {
            (Body::Catalog(w), "activate", plan) => {
                let priced = matches!(w.kind, CatalogKind::Price { .. });
                if priced != plan {
                    return Err(refused());
                }
                if matches!(
                    w.kind,
                    CatalogKind::Price {
                        plan: Plan::Free,
                        ..
                    }
                ) {
                    return Err(format!("{} is the plan in use", req.target));
                }
                w.set_selected(true);
                Ok(After::Nothing)
            }
            (Body::Item(w), "select", false) => {
                w.set_selected(true);
                Ok(After::Nothing)
            }
            _ => Err(refused()),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let takes = self
            .at(id)
            .is_some_and(|(g, c, plan)| match &self.groups[g].cards[c].body {
                Body::Catalog(w) => matches!(w.kind, CatalogKind::Price { .. }) == plan,
                Body::Item(_) => !plan,
                Body::Ratio(_) => false,
            });
        self.focus = takes.then(|| id.clone());
        true
    }

    fn blur(&mut self) -> bool {
        self.focus.take().is_some()
    }
}

#[cfg(test)]
mod tests;
