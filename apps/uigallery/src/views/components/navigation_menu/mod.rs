//! Navigation Menu page, on the showcase mold of the kit's ButtonsPage
//! (`showcase`): groups with a title and a bottom rule, cards in rows, a
//! chip naming the states each card shows. The widget is
//! `engine::widgets::nav::NavLink` on its own, the kit's home sidebar
//! link (`theme.navigation.link`).
//!
//! The page shows the ranges the link meets: a menu of four with the
//! active mark, the same menu in its rail form, a trailing shortcut and
//! a count, a long label that truncates and a translated one. A click
//! anywhere moves the one active mark, like a real sidebar.
//!
//! Projected (task-75 phase 2b): each link, `navigation_menu_page.
//! link[<i>]`, as the engine projects it, under a group per title,
//! `navigation_menu_page.group[<g>]`, described by its shortcut hint or
//! its badge; `activate` moves the active mark as a click does. The rail
//! is the menu's first four links in another form: drawn and clickable,
//! with no id of its own.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::nav::NavLink;
use engine::widgets::prelude::{Badge, EventResult, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

/// What a card shows: a column of the page's links (indices), or the
/// rail form of the menu.
enum Shows {
    Links(Vec<usize>),
    Rail,
}

struct Card {
    title: &'static str,
    shows: Shows,
}

struct Group {
    title: &'static str,
    span: usize,
    cards: Vec<Card>,
}

pub struct NavigationMenuPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    links: Vec<NavLink>,
    /// The menu's links in the rail form, mirroring `links[..MENU]`.
    rail: Vec<NavLink>,
    /// The link a screen reader or the keyboard focused (its ring shows);
    /// Escape or a click lets it go.
    focus: Option<usize>,
}

/// The menu: the first links of the page, the ones the rail mirrors.
const MENU: usize = 4;

/// One card's place: its rect and the rects of what it draws.
struct Placed {
    card: Rect,
    rows: Vec<Rect>,
}

impl NavigationMenuPage {
    pub fn new() -> Self {
        let links = vec![
            NavLink::new("Workspace").icon("layout-grid").active(true),
            NavLink::new("Branches").icon("git-branch"),
            NavLink::new("History").icon("history"),
            NavLink::new("Settings").icon("settings"),
            NavLink::new("Search").icon("search").hint("Ctrl K"),
            NavLink::new("History")
                .icon("history")
                .badge(Badge::count("4")),
            NavLink::new("A label long enough to truncate in a narrow rail")
                .icon("file")
                .hint("9"),
            NavLink::new("Configurações de privacidade").icon("settings"),
        ];
        let rail = links[..MENU].to_vec();
        let root = root_id("navigation_menu_page");
        let family = child_id(root.as_ref(), "link");
        let links = links
            .into_iter()
            .enumerate()
            .map(|(i, link)| match &family {
                Some(f) => link.sem(f.key(i.to_string())),
                None => link,
            })
            .collect();
        let card = |title, shows| Card { title, shows };
        let groups = vec![
            Group {
                title: "Navigation menu",
                span: 2,
                cards: vec![
                    card(
                        "Default, Hover, Press, Active, Focus",
                        Shows::Links((0..MENU).collect()),
                    ),
                    card("Rail", Shows::Rail),
                ],
            },
            Group {
                title: "Trailing content",
                span: 2,
                cards: vec![
                    card("Shortcut hint", Shows::Links(vec![4])),
                    card("Count badge", Shows::Links(vec![5])),
                ],
            },
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    card("Long label, truncated", Shows::Links(vec![6])),
                    card("Translated", Shows::Links(vec![7])),
                ],
            },
        ];
        Self {
            root,
            groups,
            links,
            rail,
            focus: None,
        }
    }

    /// The link behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.links.iter().position(|l| l.sem_id() == Some(id))
    }

    /// The focused link rings, the others do not.
    fn ring(&mut self) {
        for (i, link) in self.links.iter_mut().enumerate() {
            link.set_focused(self.focus == Some(i));
        }
    }

    /// The active mark on link `i`: the click's path and a request's.
    fn activate(&mut self, i: usize) {
        for (k, link) in self.links.iter_mut().enumerate() {
            link.active = k == i;
        }
        for (k, link) in self.rail.iter_mut().enumerate() {
            link.active = k == i;
        }
    }

    /// How many rows a card draws.
    fn rows(shows: &Shows) -> usize {
        match shows {
            Shows::Links(at) => at.len(),
            Shows::Rail => MENU,
        }
    }

    /// The column a card's links take: as wide as the kit sidebar's list
    /// (`w-60` less its `px-5`), never wider than the card less its
    /// inset; a rail is the rail sidebar's.
    fn column_w(shows: &Shows, card_w: f32, theme: &Theme) -> f32 {
        let room = card_w - theme.spacing.sm * 2.0;
        let w = match shows {
            Shows::Links(_) => theme.size.sidebar_w - theme.spacing.lg * 2.0,
            Shows::Rail => theme.size.sidebar_rail_w - theme.spacing.md * 2.0,
        };
        w.min(room)
    }

    fn column_h(rows: usize, theme: &Theme) -> f32 {
        let r = &theme.navigation.link;
        rows as f32 * r.height + rows.saturating_sub(1) as f32 * r.link_gap
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| {
            let heights = g
                .cards
                .iter()
                .map(|c| showcase::card_height(Self::column_h(Self::rows(&c.shows), theme), theme))
                .collect();
            (g.span, heights)
        });
        showcase::layout_sized(groups, content, theme)
    }

    /// Every card with the rows it draws, in the order of the groups.
    fn placed(&self, content: Rect, theme: &Theme) -> Vec<Vec<Placed>> {
        let r = &theme.navigation.link;
        let layout = self.layout(content, theme);
        self.groups
            .iter()
            .zip(&layout)
            .map(|(group, at)| {
                group
                    .cards
                    .iter()
                    .zip(&at.cards)
                    .map(|(card, rect)| {
                        let n = Self::rows(&card.shows);
                        let w = Self::column_w(&card.shows, rect.w, theme);
                        let col = showcase::centered((w, Self::column_h(n, theme)), *rect);
                        let rows = (0..n)
                            .map(|i| {
                                let y = col.y + i as f32 * (r.height + r.link_gap);
                                Rect::new(col.x, y, col.w, r.height)
                            })
                            .collect();
                        Placed { card: *rect, rows }
                    })
                    .collect()
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let placed = self.placed(content, theme);
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let mut clicked = None;
        for (group, cards) in self.groups.iter().zip(&placed) {
            for (card, at) in group.cards.iter().zip(cards) {
                let (links, ids): (&mut [NavLink], Vec<usize>) = match &card.shows {
                    Shows::Links(ids) => (&mut self.links, ids.clone()),
                    Shows::Rail => (&mut self.rail, (0..MENU).collect()),
                };
                for (i, rect) in ids.into_iter().zip(&at.rows) {
                    let r = links[i].handle_event(event, *rect);
                    if r.clicked {
                        clicked = Some(i);
                    }
                    result = result.merge(r);
                }
            }
        }
        if let Some(i) = clicked {
            self.activate(i);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let placed = self.placed(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (g, ((group, at), cards)) in self.groups.iter().zip(&layout).zip(&placed).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, place) in group.cards.iter().zip(cards) {
                showcase::card(c, place.card, card.title, theme);
                let (links, ids): (&[NavLink], Vec<usize>) = match &card.shows {
                    Shows::Links(ids) => (&self.links, ids.clone()),
                    Shows::Rail => (&self.rail, (0..MENU).collect()),
                };
                for (i, rect) in ids.into_iter().zip(&place.rows) {
                    links[i].render(c, *rect, theme);
                }
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for NavigationMenuPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let family = child_id(Some(root), "group");
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = family.as_ref().map(|f| f.key(g.to_string())) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                let Shows::Links(ids) = &card.shows else {
                    continue;
                };
                for link in ids.iter().filter_map(|i| self.links.get(*i)) {
                    let Some(id) = link.sem_id() else {
                        continue;
                    };
                    let mut node = link.project(b, id, Some(&at));
                    let aside = link
                        .hint
                        .as_deref()
                        .or(link.badge.as_ref().map(|b| b.label.as_str()));
                    if let Some(aside) = aside {
                        node.description(aside);
                    }
                }
            }
        }
        let focused = self.focus.and_then(|i| self.links.get(i));
        if let Some(id) = focused.and_then(NavLink::sem_id) {
            b.focus(id);
        }
    }

    /// A link's `activate` moves the active mark to it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(i), "activate") => {
                self.activate(i);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the navigation menu page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.focus = self.at(id);
        self.ring();
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.take().is_some();
        self.ring();
        had
    }
}

#[cfg(test)]
mod tests;
