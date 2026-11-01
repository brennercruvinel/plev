//! Item page: the kit's two items, the Folder row of its home sidebar and
//! the AssetItem tile of its asset panels, in the showcase mold. Each
//! card holds a few items to choose from: rows stack, tiles wrap. The
//! widget is `engine::widgets::content::Item`; every visual value comes
//! from the theme (`theme.disclosure.item`).
//!
//! The cards are as tall as what they hold; the rows drop a column where
//! a card would get narrower than the theme's least card width (`showcase::card_min_w`).
//!
//! Projected: a group per title, `item_page.group[<g>]`; in it each card,
//! `...card[<c>]`, a group named by its chip; in that the items,
//! `...card[<c>].item[<i>]`, options whose `select` chooses one (the
//! card's other items let go) as a click does. The arrows walk the
//! options of a card.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Theme, hoff_core};
use engine::widgets::content::{Item, ItemShape};
use engine::widgets::{EventResult, Rect, WidgetEvent};

use super::accordion::fit_layout;
use super::{After, Component, child_id, root_id, showcase, under};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    items: Vec<Item>,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

/// A group laid out: the title's y, each card's rect with its items',
/// and the rule's y.
struct Placed {
    title_y: f32,
    cards: Vec<(Rect, Vec<Rect>)>,
    rule_y: f32,
}

pub struct ItemPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The item, `(group, card, item)`, a reader or the keyboard focused.
    focus: Option<(usize, usize, usize)>,
}

/// `item_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `item_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn folders() -> Vec<Item> {
    vec![
        Item::new("Untitled Folder")
            .icon("folder")
            .icon_color(hoff_core::ORANGE),
        Item::new("3D Icons")
            .icon("folder")
            .icon_color(hoff_core::GREEN),
    ]
}

fn tiles() -> Vec<Item> {
    ["Cube", "Sphere", "Torus"]
        .into_iter()
        .zip(["square", "circle", "layers"])
        .map(|(title, icon)| Item::new(title).shape(ItemShape::Tile).icon(icon))
        .collect()
}

impl ItemPage {
    pub fn new() -> Self {
        let mark = |mut items: Vec<Item>, selected: Option<usize>, disabled: bool| {
            for (i, item) in items.iter_mut().enumerate() {
                *item = item
                    .clone()
                    .selected(selected == Some(i))
                    .disabled(disabled);
            }
            items
        };
        let mut groups = vec![
            Group {
                title: "Folder row",
                columns: 3,
                cards: vec![
                    Card {
                        title: "Default, Hover, Focus",
                        items: folders(),
                    },
                    Card {
                        title: "Selected",
                        items: mark(folders(), Some(1), false),
                    },
                    Card {
                        title: "Disabled",
                        items: mark(folders(), None, true),
                    },
                ],
            },
            Group {
                title: "Asset tile",
                columns: 3,
                cards: vec![
                    Card {
                        title: "Default, Hover, Focus",
                        items: tiles(),
                    },
                    Card {
                        title: "Selected",
                        items: mark(tiles(), Some(0), false),
                    },
                    Card {
                        title: "Disabled",
                        items: mark(tiles(), None, true),
                    },
                ],
            },
            Group {
                title: "Item, long content",
                columns: 1,
                cards: vec![Card {
                    title: "Wrapped label and meta",
                    items: vec![
                        Item::new(
                            "Materials imported from the shared library of the studio, \
                             sorted by the project that uses them",
                        )
                        .icon("folder")
                        .icon_color(hoff_core::BLUE)
                        .meta("128 files, edited by three people this week"),
                        Item::new("Scenes").icon("folder").meta("Empty"),
                    ],
                }],
            },
        ];
        let root = root_id("item_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let family = card_id(root, g, c).and_then(|id| id.child("item").ok());
                    let Some(family) = family else {
                        continue;
                    };
                    for (i, item) in card.items.iter_mut().enumerate() {
                        *item = item.clone().sem(family.key(i.to_string()));
                    }
                }
            }
        }
        Self {
            root,
            groups,
            focus: None,
        }
    }

    /// The item behind `id`: `(group, card, item)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let i = card.items.iter().position(|it| it.sem_id() == Some(id))?;
                Some((g, c, i))
            })
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                for (i, item) in card.items.iter_mut().enumerate() {
                    item.set_focused(at == Some((g, c, i)));
                }
            }
        }
    }

    /// Selects item `i` of a card and lets its others go; `false` when it
    /// is disabled.
    fn choose(card: &mut Card, i: usize) -> bool {
        if !card
            .items
            .get_mut(i)
            .is_some_and(|item| item.set_selected(true))
        {
            return false;
        }
        for (k, item) in card.items.iter_mut().enumerate() {
            if k != i {
                item.set_selected(false);
            }
        }
        true
    }

    /// The items of a card `w` wide, from `(x, y)`: rows one under the
    /// other, tiles left to right, wrapping.
    fn place(items: &[Item], x: f32, y: f32, w: f32, theme: &Theme) -> Vec<Rect> {
        let gap = theme.spacing.xs;
        let (mut cx, mut cy, mut line) = (x, y, 0.0_f32);
        items
            .iter()
            .map(|item| {
                let (iw, ih) = item.size(w, theme);
                if cx > x && cx + iw > x + w {
                    cx = x;
                    cy += line + gap;
                    line = 0.0;
                }
                let r = Rect::new(cx, cy, iw.min(w), ih);
                line = line.max(ih);
                if iw >= w {
                    cx = x;
                    cy += ih + gap;
                    line = 0.0;
                } else {
                    cx += iw + gap;
                }
                r
            })
            .collect()
    }

    /// Every group placed in `content`: cards as tall as their chip and
    /// their parts, a row as tall as its tallest card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Placed> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.columns, g.cards.len()))
            .collect();
        let fitted = fit_layout(&spans, content, theme, |g, c, w| {
            let card = &self.groups[g].cards[c];
            Self::place(&card.items, 0.0, 0.0, w, theme)
                .iter()
                .map(Rect::bottom)
                .fold(0.0, f32::max)
        });
        fitted
            .into_iter()
            .zip(&self.groups)
            .map(|(f, group)| Placed {
                title_y: f.title_y,
                rule_y: f.rule_y,
                cards: f
                    .cards
                    .into_iter()
                    .zip(&group.cards)
                    .map(|((rect, body), card)| {
                        (
                            rect,
                            Self::place(&card.items, body.x, body.y, body.w, theme),
                        )
                    })
                    .collect(),
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |placed| placed.rule_y - content.y)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (group, placed) in self.groups.iter_mut().zip(&layout) {
            for (card, (_, rects)) in group.cards.iter_mut().zip(&placed.cards) {
                let mut chosen = None;
                for (i, (item, rect)) in card.items.iter_mut().zip(rects).enumerate() {
                    let r = item.handle_event(event, *rect);
                    if r.clicked {
                        chosen = Some(i);
                    }
                    result = result.merge(r);
                }
                if let Some(i) = chosen {
                    Self::choose(card, i);
                }
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, placed)) in self
            .groups
            .iter()
            .zip(self.layout(content, theme))
            .enumerate()
        {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for (card, (rect, rects)) in group.cards.iter().zip(&placed.cards) {
                showcase::card(c, *rect, card.title, theme);
                for (item, at) in card.items.iter().zip(rects) {
                    item.render(c, *at, theme);
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for ItemPage {
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
                let Some(card_at) = card_id(root, g, c) else {
                    continue;
                };
                b.node(&card_at, Role::Group, card.title).parent(&at);
                for item in &card.items {
                    if let Some(id) = item.sem_id() {
                        item.project(b, id, Some(&card_at));
                    }
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c, i)| self.groups.get(g)?.cards.get(c)?.items.get(i)?.sem_id());
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// An item's `select` chooses it in its card, as its click does; a
    /// disabled one refuses.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, i)), "select") => {
                if Self::choose(&mut self.groups[g].cards[c], i) {
                    Ok(After::Nothing)
                } else {
                    Err(format!("{} is disabled", req.target))
                }
            }
            _ => Err(format!(
                "the item page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self
            .at(id)
            .filter(|(g, c, i)| !self.groups[*g].cards[*c].items[*i].is_disabled());
        self.set_focus(at);
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(inner: Rect, outer: Rect) -> bool {
        const EPS: f32 = 0.01;
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.right() <= outer.right() + EPS
            && inner.bottom() <= outer.bottom() + EPS
    }

    fn apart(a: Rect, b: Rect) -> bool {
        a.right() <= b.x || b.right() <= a.x || a.bottom() <= b.y || b.bottom() <= a.y
    }

    fn id(text: &str) -> SemId {
        SemId::parse(text).expect("the id parses")
    }

    fn ask(page: &mut ItemPage, at: &str, action: &str) -> Result<After, String> {
        let root = page.root.clone().expect("the root parses");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Item");
        page.project(&mut b, &root);
        let seen = b.finish().expect("the page projects a valid tree");
        let req = Request {
            actor: accord::Actor::Test {
                name: "item".into(),
            },
            target: id(at),
            action: accord::ActionName::parse(action).expect("an action name"),
            param: None,
            observed: accord::Observation {
                revision: seen.revision(),
                values: Vec::new(),
            },
            ask: None,
        };
        page.run(&req)
    }

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// card in the content, its items in it and apart from each other, no
    /// two cards of a row overlapping.
    #[test]
    fn nothing_is_cut_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = ItemPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let mut floor = content.y;
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    assert!(placed.title_y >= floor, "{w}: {}", group.title);
                    for (k, (card, (rect, rects))) in
                        group.cards.iter().zip(&placed.cards).enumerate()
                    {
                        let what = format!("{w}px, {}", card.title);
                        assert!(
                            rect.x >= 0.0 && rect.right() <= content.right() + 0.01,
                            "{what}"
                        );
                        for (n, at) in rects.iter().enumerate() {
                            assert!(inside(*at, *rect), "{what}: item {n}");
                            for other in &rects[n + 1..] {
                                assert!(apart(*at, *other), "{what}: items overlap");
                            }
                        }
                        for (other, _) in &placed.cards[k + 1..] {
                            assert!(apart(*rect, *other), "{what}: cards overlap");
                        }
                    }
                    floor = placed.rule_y;
                }
            }
        }
    }

    #[test]
    fn selecting_one_lets_the_others_of_its_card_go_and_a_disabled_one_refuses() {
        let mut page = ItemPage::new();
        let card = "item_page.group[0].card[1]";
        assert_eq!(
            ask(&mut page, &format!("{card}.item[0]"), "select"),
            Ok(After::Nothing)
        );
        let items = &page.groups[0].cards[1].items;
        assert!(items[0].is_selected() && !items[1].is_selected());
        assert!(ask(&mut page, "item_page.group[0].card[2].item[0]", "select").is_err());
        assert!(ask(&mut page, &format!("{card}.item[0]"), "activate").is_err());
    }

    #[test]
    fn a_click_on_a_tile_chooses_it() {
        let theme = Theme::hoff_core();
        let mut page = ItemPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let (x, y) = page.layout(content, &theme)[1].cards[1].1[2].center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(
            page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme)
                .clicked
        );
        let items = &page.groups[1].cards[1].items;
        assert!(items[2].is_selected() && !items[0].is_selected());
    }

    #[test]
    fn a_disabled_item_takes_no_ring() {
        let mut page = ItemPage::new();
        assert!(page.focus(&id("item_page.group[0].card[2].item[0]")));
        assert_eq!(page.focus, None);
        assert!(page.focus(&id("item_page.group[0].card[0].item[1]")));
        assert_eq!(page.focus, Some((0, 0, 1)));
        assert!(page.blur());
    }
}
