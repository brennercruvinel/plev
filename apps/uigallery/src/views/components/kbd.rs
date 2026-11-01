//! Kbd page: the kit's key cap, as its menus and its search draw it, in
//! the showcase mold, and in context on the nav links that carry a
//! shortcut. The widgets are `engine::widgets::content::Kbd` and
//! `engine::widgets::nav::NavLink`; every visual value comes from the
//! theme (`theme.disclosure.kbd`).
//!
//! The cards are as tall as what they hold: caps flow and wrap, links
//! stack; the rows drop a column where a card would get narrower than
//! the theme's least card width (`showcase::card_min_w`).
//!
//! Projected: a group per title, `kbd_page.group[<g>]`; in it each card,
//! `...card[<c>]`, a group named by its chip; in that the caps,
//! `...card[<c>].key[<i>]`, labels named by their keys, and the links,
//! `...card[<c>].link[<i>]`, whose `activate` makes one active as a click
//! does, each described by its shortcut.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{Kbd, KbdSize};
use engine::widgets::nav::NavLink;
use engine::widgets::{EventResult, Rect, WidgetEvent};

use super::accordion::fit_layout;
use super::{After, Component, child_id, root_id, showcase, under};

enum Body {
    Caps(Vec<Kbd>),
    Links(Vec<NavLink>),
}

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    body: Body,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

/// A group laid out: the title's y, each card's rect with its parts',
/// and the rule's y.
struct Placed {
    title_y: f32,
    cards: Vec<(Rect, Vec<Rect>)>,
    rule_y: f32,
}

pub struct KbdPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The link, `(group, card, link)`, a reader or the keyboard focused.
    focus: Option<(usize, usize, usize)>,
}

/// `kbd_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `kbd_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// `...card[<c>].key[<i>]` or `...card[<c>].link[<i>]`.
fn part_id(root: &SemId, g: usize, c: usize, part: &str, i: usize) -> Option<SemId> {
    let family = card_id(root, g, c)?.child(part).ok()?;
    Some(family.key(i.to_string()))
}

fn caps(keys: &[&str], size: KbdSize, disabled: bool) -> Body {
    Body::Caps(
        keys.iter()
            .map(|k| Kbd::new(*k).size(size).disabled(disabled))
            .collect(),
    )
}

impl KbdPage {
    pub fn new() -> Self {
        let links = [
            ("Explore", "layout-grid", "1"),
            ("Assets", "layers", "2"),
            ("Likes", "heart", "3"),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (label, icon, hint))| {
            NavLink::new(label).icon(icon).shortcut(hint).active(i == 0)
        })
        .collect();
        let groups = vec![
            Group {
                title: "Key cap",
                columns: 2,
                cards: vec![
                    Card {
                        title: "Small: a menu's shortcuts",
                        body: caps(&["⇧⌘E", "⌘N", "⌘D"], KbdSize::Sm, false),
                    },
                    Card {
                        title: "Medium: the search's",
                        body: caps(&["⌘ K"], KbdSize::Md, false),
                    },
                ],
            },
            Group {
                title: "Key cap in context",
                columns: 2,
                cards: vec![
                    Card {
                        title: "Nav link shortcut: Default, Hover, Active, Focus",
                        body: Body::Links(links),
                    },
                    Card {
                        title: "Disabled",
                        body: caps(&["⌘ +", "⌘ -", "⌘ 0"], KbdSize::Sm, true),
                    },
                ],
            },
            Group {
                title: "Key cap, long content",
                columns: 1,
                cards: vec![Card {
                    title: "Chords that wrap",
                    body: caps(
                        &[
                            "⌃⌥⇧⌘ Delete",
                            "Fn F12",
                            "⌘ Shift Option Return",
                            "Ctrl Alt Page Down",
                            "⌥ Space",
                            "⇧ Tab",
                        ],
                        KbdSize::Md,
                        false,
                    ),
                }],
            },
        ];
        let mut page = Self {
            root: root_id("kbd_page"),
            groups,
            focus: None,
        };
        if let Some(root) = page.root.clone() {
            for (g, group) in page.groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Body::Links(links) = &mut card.body {
                        for (i, link) in links.iter_mut().enumerate() {
                            if let Some(id) = part_id(&root, g, c, "link", i) {
                                *link = link.clone().sem(id);
                            }
                        }
                    }
                }
            }
        }
        page
    }

    /// The link behind `id`: `(group, card, link)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .find_map(|(c, card)| match &card.body {
                    Body::Links(links) => {
                        let i = links.iter().position(|l| l.sem_id() == Some(id))?;
                        Some((g, c, i))
                    }
                    Body::Caps(_) => None,
                })
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                if let Body::Links(links) = &mut card.body {
                    for (i, link) in links.iter_mut().enumerate() {
                        link.set_focused(at == Some((g, c, i)));
                    }
                }
            }
        }
    }

    /// Link `i` of a card becomes the active one.
    fn activate(links: &mut [NavLink], i: usize) {
        for (k, link) in links.iter_mut().enumerate() {
            link.active = k == i;
        }
    }

    /// The parts of a card `w` wide from `(x, y)`: caps left to right,
    /// wrapping; links one under the other, full width.
    fn place(body: &Body, x: f32, y: f32, w: f32, theme: &Theme) -> Vec<Rect> {
        match body {
            Body::Links(links) => {
                let h = NavLink::height(theme);
                (0..links.len())
                    .map(|i| Rect::new(x, y + i as f32 * h, w, h))
                    .collect()
            }
            Body::Caps(keys) => {
                let gap = theme.spacing.sm;
                let (mut cx, mut cy, mut line) = (x, y, 0.0_f32);
                keys.iter()
                    .map(|key| {
                        let (kw, kh) = key.preferred_size(theme);
                        let kw = kw.min(w);
                        if cx > x && cx + kw > x + w {
                            cx = x;
                            cy += line + gap;
                            line = 0.0;
                        }
                        let r = Rect::new(cx, cy, kw, kh);
                        cx += kw + gap;
                        line = line.max(kh);
                        r
                    })
                    .collect()
            }
        }
    }

    /// Every group placed in `content`: cards as tall as their chip and
    /// their parts, a row as tall as its tallest card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Placed> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.title, g.columns, g.cards.len()))
            .collect();
        let fitted = fit_layout(&spans, content, theme, |g, c, w| {
            let card = &self.groups[g].cards[c];
            Self::place(&card.body, 0.0, 0.0, w, theme)
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
                        (rect, Self::place(&card.body, body.x, body.y, body.w, theme))
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
                let Body::Links(links) = &mut card.body else {
                    continue;
                };
                let mut chosen = None;
                for (i, (link, rect)) in links.iter_mut().zip(rects).enumerate() {
                    let r = link.handle_event(event, *rect);
                    if r.clicked {
                        chosen = Some(i);
                    }
                    result = result.merge(r);
                }
                if let Some(i) = chosen {
                    Self::activate(links, i);
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
                match &card.body {
                    Body::Caps(keys) => {
                        for (key, at) in keys.iter().zip(rects) {
                            key.render(c, *at, theme);
                        }
                    }
                    Body::Links(links) => {
                        for (link, at) in links.iter().zip(rects) {
                            link.render(c, *at, theme);
                        }
                    }
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for KbdPage {
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
                match &card.body {
                    Body::Caps(keys) => {
                        for (i, key) in keys.iter().enumerate() {
                            if let Some(id) = part_id(root, g, c, "key", i) {
                                key.project(b, &id, Some(&card_at));
                            }
                        }
                    }
                    Body::Links(links) => {
                        for link in links {
                            let Some(id) = link.sem_id() else {
                                continue;
                            };
                            let mut node = link.project(b, id, Some(&card_at));
                            if let Some(keys) = &link.shortcut {
                                node.description(format!("shortcut {keys}"));
                            }
                        }
                    }
                }
            }
        }
        let focused =
            self.focus
                .and_then(|(g, c, i)| match &self.groups.get(g)?.cards.get(c)?.body {
                    Body::Links(links) => links.get(i)?.sem_id(),
                    Body::Caps(_) => None,
                });
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A link's `activate` makes it the active one, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, i)), "activate") => {
                if let Body::Links(links) = &mut self.groups[g].cards[c].body {
                    Self::activate(links, i);
                }
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the kbd page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.at(id));
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

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// card in the content, its caps and links in it and apart, no two
    /// cards of a row overlapping.
    #[test]
    fn nothing_is_cut_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = KbdPage::new();
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
                            assert!(inside(*at, *rect), "{what}: part {n}");
                            for other in &rects[n + 1..] {
                                assert!(apart(*at, *other), "{what}: parts overlap");
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
    fn the_caps_are_labels_and_the_links_say_their_shortcut() {
        let page = KbdPage::new();
        let root = page.root.clone().expect("the root parses");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Kbd");
        page.project(&mut b, &root);
        let seen = b.finish().expect("a valid tree");
        let key = SemId::parse("kbd_page.group[0].card[0].key[0]").unwrap();
        let node = seen.get(&key).expect("a cap");
        assert_eq!((node.role, node.label.as_str()), (Role::Label, "⇧⌘E"));
        let link = SemId::parse("kbd_page.group[1].card[0].link[1]").unwrap();
        let node = seen.get(&link).expect("a link");
        assert_eq!(node.description.as_deref(), Some("shortcut 2"));
    }

    #[test]
    fn a_click_on_a_link_makes_it_active() {
        let theme = Theme::hoff_core();
        let mut page = KbdPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let (x, y) = page.layout(content, &theme)[1].cards[0].1[2].center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(
            page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme)
                .clicked
        );
        let Body::Links(links) = &page.groups[1].cards[0].body else {
            panic!("the links card");
        };
        assert!(links[2].active && !links[0].active);
    }
}
