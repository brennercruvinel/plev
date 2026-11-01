//! Bubble page, in the showcase mold: the chat bubble of the hoff core
//! kit (its comment pin, holding text) read and unread, from the other
//! person, from oneself and as a note, a conversation, then the content a
//! bubble meets: long text, six scripts with emoji. A card is as tall as
//! its bubbles measure (`showcase::layout_sized`), never less than the kit's
//! `h-50`; a bubble grows up to its share of the card's width.
//!
//! Projected: a group per title, `bubble_page.group[<g>]`; in it each
//! card as a list named by its chip, `…card[<c>]`; in the list each
//! bubble, `…card[<c>].bubble[<i>]`, read by its words with its `unread`
//! state. A bubble does nothing; a reader lands on it.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::message::bubble::{Bubble, BubbleSide};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, root_id, showcase, under};

const LONG: &str = "These draggable sliders look really cool. Maybe these could be displayed \
     when you hold shift, to rotate exactly on the X or Y.";

/// Six scripts and emoji in one bubble (adr-0071).
const SCRIPTS: &str = "Olá! مرحبا · שלום · नमस्ते · 你好 · こんにちは · 안녕 🎉👍🏽";

struct Card {
    title: &'static str,
    bubbles: Vec<Bubble>,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct BubblePage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The bubble a reader focused, `(group, card, bubble)`.
    focus: Option<(usize, usize, usize)>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn bubble_id(card: &SemId, i: usize) -> Option<SemId> {
    child_id(Some(card), "bubble").map(|family| family.key(i.to_string()))
}

fn card(title: &'static str, bubbles: Vec<Bubble>) -> Card {
    Card { title, bubbles }
}

fn from(text: &str) -> Bubble {
    Bubble::new(text)
}

fn mine(text: &str) -> Bubble {
    Bubble::new(text).side(BubbleSide::End)
}

impl BubblePage {
    pub fn new() -> Self {
        let mut groups = vec![
            Group {
                title: "Bubble",
                columns: 2,
                cards: vec![
                    card("Read, Focus", vec![from("I love this bag")]),
                    card(
                        "Unread",
                        vec![from("Could you make it in leather?").unread(true)],
                    ),
                    card("Mine", vec![mine("Thanks, I'm working on it")]),
                    card("Note", vec![from("Sam joined").side(BubbleSide::Center)]),
                ],
            },
            Group {
                title: "Conversation",
                columns: 1,
                cards: vec![card(
                    "A thread, newest unread",
                    vec![
                        from("I love this bag, it looks great."),
                        from("Could you make it with leather material? 🤔"),
                        mine("Thanks, I'm working on it"),
                        from("Great, send me a photo").unread(true),
                    ],
                )],
            },
            Group {
                title: "Content",
                columns: 2,
                cards: vec![
                    card("Long text", vec![from(LONG), mine(LONG)]),
                    card("Mixed scripts", vec![from(SCRIPTS), mine(SCRIPTS)]),
                ],
            },
        ];
        let root = root_id("bubble_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(cid) = card_id(root, g, c) else {
                        continue;
                    };
                    for (i, b) in std::mem::take(&mut card.bubbles).into_iter().enumerate() {
                        card.bubbles.push(match bubble_id(&cid, i) {
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
                let i = card.bubbles.iter().position(|b| b.sem_id() == Some(id))?;
                Some((g, c, i))
            })
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                for (i, b) in card.bubbles.iter_mut().enumerate() {
                    b.set_focused(at == Some((g, c, i)));
                }
            }
        }
    }

    /// The width a card's bubbles share: its inside, up to a readable
    /// column.
    fn lane_w(card_w: f32, theme: &Theme) -> f32 {
        showcase::room(card_w, theme).min(theme.size.readable_max_w)
    }

    /// Each bubble's row height in a lane `w` wide: the bubble and the gap
    /// under it, none under the last.
    fn rows(card: &Card, w: f32, theme: &Theme) -> Vec<f32> {
        let last = card.bubbles.len().saturating_sub(1);
        card.bubbles
            .iter()
            .enumerate()
            .map(|(i, b)| b.size(w, theme).1 + if i < last { theme.spacing.sm } else { 0.0 })
            .collect()
    }

    /// Per group: its layout (`showcase::layout_sized`, each card as tall
    /// as its bubbles), and per card the rows of its bubbles, stacked as
    /// one block centered in the card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Vec<Rect>>)> {
        let sized = self.groups.iter().map(|g| {
            let w = Self::lane_w(showcase::card_width(g.columns, content, theme), theme);
            let cards = g
                .cards
                .iter()
                .map(|card| showcase::card_height(Self::rows(card, w, theme).iter().sum(), theme));
            (g.title, g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let rows = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        let w = Self::lane_w(rect.w, theme);
                        let heights = Self::rows(card, w, theme);
                        let block = showcase::centered((w, heights.iter().sum()), *rect);
                        let mut y = block.y;
                        heights
                            .iter()
                            .map(|h| {
                                let r = Rect::new(block.x, y, w, *h);
                                y += h;
                                r
                            })
                            .collect()
                    })
                    .collect();
                (placed, rows)
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

    /// A bubble takes no pointer; a click lets the reader's focus go.
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
        for (i, (group, (placed, rows))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), rows) in group.cards.iter().zip(&placed.cards).zip(rows) {
                showcase::card(c, *rect, card.title, theme);
                for (b, row) in card.bubbles.iter().zip(rows) {
                    b.render(c, *row, theme);
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for BubblePage {
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
                for bubble in &card.bubbles {
                    if let Some(id) = bubble.sem_id() {
                        bubble.project(b, id, Some(&cid));
                    }
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c, i)| self.groups.get(g)?.cards.get(c)?.bubbles.get(i)?.sem_id());
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A bubble declares no action.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the bubble page has no {} on {}",
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

    /// The width sweep, 280 to 1600 every 8 px: every bubble inside its
    /// row and its card, every card inside the content, the groups down
    /// the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = BubblePage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let mut prev = content.y;
                for (g, (group, (placed, rows))) in page
                    .groups
                    .iter()
                    .zip(page.layout(content, &theme))
                    .enumerate()
                {
                    assert!(placed.title_y >= prev, "{w}: group {g} climbs");
                    prev = placed.rule_y;
                    for ((card, rect), rows) in group.cards.iter().zip(&placed.cards).zip(&rows) {
                        let lane = Rect::new(content.x, rect.y, content.w, rect.h);
                        assert!(
                            within(*rect, lane),
                            "{w}: {} out of the content",
                            card.title
                        );
                        for (b, row) in card.bubbles.iter().zip(rows) {
                            let r = b.rect(*row, &theme);
                            assert!(within(r, *row), "{w}: {:?} out of its row", b.text());
                            assert!(within(r, *rect), "{w}: {:?} out of its card", b.text());
                        }
                    }
                }
            }
        }
    }

    /// The ids parse, the reader's focus lands on a bubble and goes, and
    /// nothing runs.
    #[test]
    fn a_bubble_takes_the_reader_s_focus_and_runs_nothing() {
        let mut page = BubblePage::new();
        let root = page.root().cloned().expect("the root parses");
        let first = page.groups[0].cards[0].bubbles[0]
            .sem_id()
            .cloned()
            .unwrap();
        assert!(under(&root, &first));
        assert!(page.focus(&first));
        assert!(page.groups[0].cards[0].bubbles[0].is_focused());
        assert!(page.blur());
        assert!(!page.groups[0].cards[0].bubbles[0].is_focused());
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Bubble");
        page.project(&mut b, &root);
        let seen = b.finish().expect("the projection validates");
        assert!(seen.get(&first).is_some_and(|n| n.actions.is_empty()));
    }
}
