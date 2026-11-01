//! Badge page, in the showcase mold: the hoff core kit's chip label
//! (`Card`) at rest, raised and per intent; the count pill on its run,
//! one digit to many and per intent; the notification dot
//! (`HeaderSimple`) per intent; then the content a badge meets: a long
//! label, six scripts with emoji, a capped count. A card is as tall as its
//! badges measure (`showcase::layout_sized`); the badges flow in rows
//! across the card's body, and a label wider than the body ellipsizes.
//!
//! Projected: a group per title, `badge_page.group[<g>]`; in it each card
//! as a list named by its chip, `…card[<c>]`; in the list each badge,
//! `…card[<c>].badge[<i>]`, a label read by its words with its intent. A
//! badge does nothing; a reader lands on it.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::prelude::{Badge, EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, flow, root_id, showcase, under};

const LONG: &str = "Waiting for the second reviewer before the merge";

/// Six scripts and emoji in one label (adr-0071).
const SCRIPTS: &str = "Olá · مرحبا · שלום · नमस्ते · 你好 · 안녕 🎉";

const INTENTS: [(Intent, &str); 4] = [
    (Intent::Neutral, "draft"),
    (Intent::Informational, "synced"),
    (Intent::Constructive, "staged"),
    (Intent::Destructive, "conflict"),
];

struct Card {
    title: &'static str,
    badges: Vec<Badge>,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct BadgePage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The badge a reader focused, `(group, card, badge)`.
    focus: Option<(usize, usize, usize)>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn badge_id(card: &SemId, i: usize) -> Option<SemId> {
    child_id(Some(card), "badge").map(|family| family.key(i.to_string()))
}

fn card(title: &'static str, badges: Vec<Badge>) -> Card {
    Card { title, badges }
}

fn groups() -> Vec<Group> {
    let tags = INTENTS.map(|(i, word)| Badge::tag(word).intent(i));
    let counts = INTENTS.map(|(i, _)| Badge::count("4").intent(i));
    let dots = INTENTS.map(|(i, word)| Badge::dot(word).intent(i));
    vec![
        Group {
            title: "Tag",
            columns: 3,
            cards: vec![
                card("Rest, Focus", vec![Badge::tag("Card")]),
                card("Raised", vec![Badge::tag("Card").raised(true)]),
                card("Intents", tags.to_vec()),
            ],
        },
        Group {
            title: "Count",
            columns: 3,
            cards: vec![
                card("One digit", vec![Badge::count("3")]),
                card("More", vec![Badge::count("12"), Badge::count("128")]),
                card("Intents", counts.to_vec()),
            ],
        },
        Group {
            title: "Dot",
            columns: 2,
            cards: vec![
                card("New", vec![Badge::dot("New")]),
                card("Intents", dots.to_vec()),
            ],
        },
        Group {
            title: "Content",
            columns: 3,
            cards: vec![
                card(
                    "Long label",
                    vec![Badge::tag(LONG).intent(Intent::Informational)],
                ),
                card("Mixed scripts", vec![Badge::tag(SCRIPTS)]),
                card("Capped count", vec![Badge::count("99+")]),
            ],
        },
    ]
}

impl BadgePage {
    pub fn new() -> Self {
        let mut groups = groups();
        let root = root_id("badge_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(cid) = card_id(root, g, c) else {
                        continue;
                    };
                    for (i, b) in std::mem::take(&mut card.badges).into_iter().enumerate() {
                        card.badges.push(match badge_id(&cid, i) {
                            Some(id) => b.sem(id),
                            None => b,
                        });
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

    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let i = card.badges.iter().position(|b| b.sem_id() == Some(id))?;
                Some((g, c, i))
            })
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                for (i, b) in card.badges.iter_mut().enumerate() {
                    b.set_focused(at == Some((g, c, i)));
                }
            }
        }
    }

    /// A card's badges flowed across a body `w` wide, from the origin:
    /// each at its preferred size, no wider than the body.
    fn flowed(card: &Card, w: f32, theme: &Theme) -> (Vec<Rect>, f32) {
        let body = Rect::new(0.0, 0.0, w, 0.0);
        let sizes = card.badges.iter().map(|b| {
            let (bw, bh) = b.preferred_size(theme);
            (bw.min(w), bh)
        });
        flow(body, 0.0, theme.spacing.sm, sizes)
    }

    /// The width a card's body leaves its badges.
    fn body_w(card_w: f32, theme: &Theme) -> f32 {
        showcase::body(Rect::new(0.0, 0.0, card_w, 0.0), theme).w
    }

    /// Per group: its layout (each card as tall as its badges), and per
    /// card its badges' rects, the flowed block centered in the card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Vec<Rect>>)> {
        let sized = self.groups.iter().map(|g| {
            let w = Self::body_w(showcase::card_width(g.columns, content, theme), theme);
            let cards = g
                .cards
                .iter()
                .map(|card| showcase::card_height(Self::flowed(card, w, theme).1, theme));
            (g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let rects = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        let w = Self::body_w(rect.w, theme);
                        let (rects, h) = Self::flowed(card, w, theme);
                        let used = rects.iter().map(|r| r.right()).fold(0.0, f32::max);
                        let block = showcase::centered((used, h), *rect);
                        rects
                            .into_iter()
                            .map(|r| Rect::new(block.x + r.x, block.y + r.y, r.w, r.h))
                            .collect()
                    })
                    .collect();
                (placed, rects)
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let layout: Vec<_> = self
            .layout(content, theme)
            .into_iter()
            .map(|(g, _)| g)
            .collect();
        showcase::content_height(&layout, content)
    }

    /// A badge takes no pointer; a click lets the reader's focus go.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            return EventResult::changed();
        }
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, rects))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), rects) in group.cards.iter().zip(&placed.cards).zip(rects) {
                showcase::card(c, *rect, card.title, theme);
                for (b, r) in card.badges.iter().zip(rects) {
                    b.render(c, *r, theme);
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for BadgePage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(gid) = group_id(root, g) else {
                continue;
            };
            b.node(&gid, Role::Group, group.title).parent(root);
            for (c, card) in group.cards.iter().enumerate() {
                let Some(cid) = card_id(root, g, c) else {
                    continue;
                };
                b.node(&cid, Role::List, card.title).parent(&gid);
                for badge in &card.badges {
                    if let Some(id) = badge.sem_id() {
                        badge.project(b, id, Some(&cid));
                    }
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c, i)| self.groups.get(g)?.cards.get(c)?.badges.get(i)?.sem_id());
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A badge declares no action.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the badge page has no {} on {}",
            req.action, req.target
        ))
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

    const EPS: f32 = 0.5;

    fn within(inner: Rect, outer: Rect) -> bool {
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.right() <= outer.right() + EPS
            && inner.bottom() <= outer.bottom() + EPS
    }

    /// The width sweep, 280 to 1600 every 8 px, under three text scales
    /// (the core's, and a theme whose type ramp is larger and smaller):
    /// every badge inside its card's body, every card inside the content,
    /// the groups down the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = BadgePage::new();
        let scaled = |role: engine::styling::theme::TypeRole| {
            let mut t = Theme::hoff_core();
            t.message.badge.tag.text.run = role.run();
            t
        };
        use engine::styling::theme::TypeRole::{Small, Title};
        for theme in [
            Theme::hoff_core(),
            Theme::hoff(),
            scaled(Title),
            scaled(Small),
        ] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let mut prev = content.y;
                for (g, (group, (placed, rects))) in page
                    .groups
                    .iter()
                    .zip(page.layout(content, &theme))
                    .enumerate()
                {
                    assert!(placed.title_y >= prev, "{w}: group {g} climbs");
                    prev = placed.rule_y;
                    for ((card, rect), rects) in group.cards.iter().zip(&placed.cards).zip(&rects) {
                        let lane = Rect::new(content.x, rect.y, content.w, rect.h);
                        assert!(
                            within(*rect, lane),
                            "{w}: {} out of the content",
                            card.title
                        );
                        let body = showcase::body(*rect, &theme);
                        for (b, r) in card.badges.iter().zip(rects) {
                            let drawn = b.rect(*r, &theme);
                            assert!(within(drawn, *rect), "{w}: {:?} out of its card", b.label);
                            assert!(
                                drawn.w <= body.w + EPS,
                                "{w}: {:?} wider than the body",
                                b.label
                            );
                        }
                    }
                }
            }
        }
    }

    /// The ids parse, the reader's focus lands on a badge and goes, and a
    /// badge is a label with no action.
    #[test]
    fn a_badge_takes_the_reader_s_focus_and_runs_nothing() {
        let mut page = BadgePage::new();
        let root = page.root().cloned().expect("the root parses");
        let first = page.groups[0].cards[0].badges[0].sem_id().cloned().unwrap();
        assert!(under(&root, &first));
        assert!(page.focus(&first));
        assert!(page.groups[0].cards[0].badges[0].is_focused());
        assert!(page.blur());
        assert!(!page.groups[0].cards[0].badges[0].is_focused());
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Badge");
        page.project(&mut b, &root);
        let seen = b.finish().expect("the projection validates");
        let node = seen.get(&first).expect("the badge");
        assert_eq!(node.role, Role::Label);
        assert!(node.actions.is_empty());
    }
}
