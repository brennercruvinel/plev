//! Message page, in the mold of the hoff core kit's ButtonsPage: groups
//! with a title and a bottom rule, cards in rows, one state per card
//! behind a chip that names it. The widget is
//! `engine::widgets::message::message::Message` in the kit's two
//! layouts (the thread row of its comment page, the quick comment of its
//! modal page), then the content a message meets: long text, six scripts
//! with emoji, a long name, an empty body. Every visual value comes from
//! the theme; a card is as tall as the messages it holds measure, never
//! less than the kit's `h-50`. The mention card is pending: the kit
//! colors a mention inside the body, the text engine draws one color per
//! paragraph (the color per run is an open decision of task-104).
//!
//! Projected (task-75 phase 2b): a group per title,
//! `message_page.group[<g>]`; in it each card as a list named by its
//! chip, `…card[<c>]`; in the list each message as the widget projects
//! it, `…card[<c>].message[<i>]`, and its like chip,
//! `…message[<i>].reaction`. A like chip here does nothing but press, as
//! in the kit, so its `activate` changes nothing.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::message::message::{Message, MessageKind};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, root_id, showcase, under};

const LUNA: &str = "I love this bag, it looks great. Could you make it with leather material? 🤔";

const DRAGGABLE: &str = "These draggable sliders look really cool. Maybe these could be \
     displayed when you hold shift, to rotate exactly on the X or Y. But by default I don't \
     think we need these controllers, you could just rotate the object by clicking and \
     dragging anywhere as expected on any 3D tool (theoretically).";

/// Six scripts and emoji in one body (adr-0071).
const SCRIPTS: &str =
    "Olá! مرحبا بالعالم · שלום עולם · नमस्ते दुनिया · 你好世界 · こんにちは · 안녕하세요 🎉👍🏽";

struct Card {
    /// The chip: which state or content this card shows.
    title: &'static str,
    messages: Vec<Message>,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    columns: usize,
    cards: Vec<Card>,
}

/// A focusable part of the page: a message, or its like chip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spot {
    g: usize,
    c: usize,
    m: usize,
    chip: bool,
}

/// Where a group lands: the title's y, each card with its messages'
/// rects, the rule's y.
struct Placed {
    title_y: f32,
    cards: Vec<(Rect, Vec<Rect>)>,
    rule_y: f32,
}

pub struct MessagePage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// What a screen reader focused (its ring shows); Escape or a click
    /// lets it go.
    focus: Option<Spot>,
}

/// `message_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `message_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// `…card[<c>].message[<i>]`.
fn message_id(card: &SemId, i: usize) -> Option<SemId> {
    child_id(Some(card), "message").map(|family| family.key(i.to_string()))
}

fn card(title: &'static str, messages: Vec<Message>) -> Card {
    Card { title, messages }
}

/// A row of the kit's thread, closed by its hairline.
fn thread(author: &str, time: &str, body: &str) -> Message {
    Message::new(author, time, body).divider(true)
}

fn quick(disabled: bool) -> Message {
    Message::new("Randomdash", "1h", DRAGGABLE)
        .kind(MessageKind::Quick)
        .reaction("2")
        .reaction_disabled(disabled)
}

impl MessagePage {
    pub fn new() -> Self {
        let luna = || thread("Luna Starfield", "2s", LUNA).verified(true);
        let sam = || thread("Sam", "1s", "@Luna Thank, I’m working on it");
        let mut groups = vec![
            Group {
                title: "Comment",
                columns: 2,
                cards: vec![
                    card("Default, Focus", vec![luna()]),
                    card("Conversation", vec![luna(), sam()]),
                    card("Mention (pending)", vec![sam()]),
                ],
            },
            Group {
                title: "Quick comment",
                columns: 2,
                cards: vec![
                    card("Default, Hover, Press, Focus", vec![quick(false)]),
                    card("Disabled", vec![quick(true)]),
                ],
            },
            Group {
                title: "Content",
                columns: 2,
                cards: vec![
                    card(
                        "Long text",
                        vec![thread("Luna Starfield", "4m", DRAGGABLE).verified(true)],
                    ),
                    card("Mixed scripts", vec![thread("سارة الأحمد", "3m", SCRIPTS)]),
                    card(
                        "Long name, the time kept",
                        vec![thread(
                            "Maria Eduarda Albuquerque de Souza Vasconcelos Starfield",
                            "12 min ago",
                            LUNA,
                        )],
                    ),
                    card("Empty body", vec![thread("Sam", "now", "")]),
                ],
            },
        ];
        let root = root_id("message_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(cid) = card_id(root, g, c) else {
                        continue;
                    };
                    for (i, m) in std::mem::take(&mut card.messages).into_iter().enumerate() {
                        card.messages.push(match message_id(&cid, i) {
                            Some(id) => m.sem(id),
                            None => m,
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

    /// The message, or its like chip, whose id is `id`.
    fn at(&self, id: &SemId) -> Option<Spot> {
        self.spots()
            .find(|(_, m, chip)| {
                let sem = if *chip {
                    m.reaction_sem_id()
                } else {
                    m.sem_id()
                };
                sem == Some(id)
            })
            .map(|(spot, _, _)| spot)
    }

    /// Every message with its spot, and its chip when it has one.
    fn spots(&self) -> impl Iterator<Item = (Spot, &Message, bool)> + '_ {
        self.groups.iter().enumerate().flat_map(|(g, group)| {
            group.cards.iter().enumerate().flat_map(move |(c, card)| {
                card.messages.iter().enumerate().flat_map(move |(m, msg)| {
                    let row = (
                        Spot {
                            g,
                            c,
                            m,
                            chip: false,
                        },
                        msg,
                        false,
                    );
                    let chip = msg.has_reaction().then_some((
                        Spot {
                            g,
                            c,
                            m,
                            chip: true,
                        },
                        msg,
                        true,
                    ));
                    std::iter::once(row).chain(chip)
                })
            })
        })
    }

    /// The reader's focus on one message or chip, or on none.
    fn set_focus(&mut self, at: Option<Spot>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                for (m, msg) in card.messages.iter_mut().enumerate() {
                    let here = |chip| at == Some(Spot { g, c, m, chip });
                    msg.set_focused(here(false));
                    msg.set_reaction_focused(here(true));
                }
            }
        }
    }

    /// The width a card's messages take: the card's inside, up to a
    /// readable column.
    fn message_w(card_w: f32, theme: &Theme) -> f32 {
        showcase::room(card_w, theme).min(theme.size.readable_max_w)
    }

    fn heights(card: &Card, w: f32, theme: &Theme) -> Vec<f32> {
        card.messages.iter().map(|m| m.height(w, theme)).collect()
    }

    /// The showcase's groups with each card as tall as its messages
    /// (`showcase::layout_sized`), the messages stacked as one block
    /// centered in the card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Placed> {
        let sized = self.groups.iter().map(|g| {
            let w = Self::message_w(showcase::card_width(g.columns, content, theme), theme);
            let cards = g.cards.iter().map(|card| {
                let stack: f32 = Self::heights(card, w, theme).iter().sum();
                showcase::card_height(stack, theme)
            });
            (g.title, g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| Placed {
                title_y: placed.title_y,
                cards: placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        let w = Self::message_w(rect.w, theme);
                        let heights = Self::heights(card, w, theme);
                        let block = showcase::centered((w, heights.iter().sum()), *rect);
                        let mut y = block.y;
                        let rows = heights
                            .iter()
                            .map(|h| {
                                let r = Rect::new(block.x, y, w, *h);
                                y += h;
                                r
                            })
                            .collect();
                        (*rect, rows)
                    })
                    .collect(),
                rule_y: placed.rule_y,
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |p| p.rule_y - content.y)
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
                for (m, r) in card.messages.iter_mut().zip(rects) {
                    result = result.merge(m.handle_event(event, *r, theme));
                }
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, placed)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for (card, (rect, rects)) in group.cards.iter().zip(&placed.cards) {
                showcase::card(c, *rect, card.title, theme);
                for (m, r) in card.messages.iter().zip(rects) {
                    m.render(c, *r, theme);
                }
            }
            // The rule under the group (`border-b`, none on the last).
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for MessagePage {
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
                for m in &card.messages {
                    if let Some(id) = m.sem_id() {
                        m.project(b, id, Some(&cid));
                    }
                }
            }
        }
        let focused = self.focus.and_then(|spot| {
            let m = self
                .groups
                .get(spot.g)?
                .cards
                .get(spot.c)?
                .messages
                .get(spot.m)?;
            if spot.chip {
                m.reaction_sem_id()
            } else {
                m.sem_id()
            }
        });
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A like chip's `activate`: it presses, and nothing else happens,
    /// as for a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(Spot { chip: true, .. }), "activate") => Ok(After::Nothing),
            _ => Err(format!(
                "the message page has no {} on {}",
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
    use accord::Actor;

    const EPS: f32 = 0.5;

    fn within(inner: Rect, outer: Rect) -> bool {
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.right() <= outer.right() + EPS
            && inner.bottom() <= outer.bottom() + EPS
    }

    fn overlap(a: Rect, b: Rect) -> bool {
        a.x < b.right() - EPS
            && b.x < a.right() - EPS
            && a.y < b.bottom() - EPS
            && b.y < a.bottom() - EPS
    }

    /// Every message has the id its draw binds, under the page's root.
    #[test]
    fn every_message_and_chip_has_an_id_under_the_root() {
        let page = MessagePage::new();
        let root = page.root().cloned().expect("the root parses");
        for (spot, m, chip) in page.spots() {
            let id = if chip {
                m.reaction_sem_id()
            } else {
                m.sem_id()
            };
            let id = id.unwrap_or_else(|| panic!("{spot:?} has no id"));
            assert!(under(&root, id), "{id}");
            assert_eq!(page.at(id), Some(spot), "{id}");
        }
    }

    /// The width sweep, 280 to 1600 every 8 px, with the long, the mixed
    /// and the empty content: every card in the content, every message in
    /// its card and never narrower than its parts, no two cards over each
    /// other, the groups down the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = MessagePage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let layout = page.layout(content, &theme);
                let mut prev = content.y;
                for (g, (group, placed)) in page.groups.iter().zip(&layout).enumerate() {
                    assert!(placed.title_y >= prev, "{w}: group {g} climbs");
                    prev = placed.rule_y;
                    for (i, (card, rects)) in placed.cards.iter().enumerate() {
                        let messages = &group.cards[i].messages;
                        let tall = Rect::new(content.x, card.y, content.w, card.h);
                        assert!(
                            within(*card, tall),
                            "{w}: card {g}.{i} {card:?} out of the content"
                        );
                        for (m, r) in messages.iter().zip(rects) {
                            assert!(
                                within(*r, *card),
                                "{w}: message {r:?} out of card {g}.{i} {card:?}"
                            );
                            assert!(r.w >= m.min_width(&theme), "{w}: {g}.{i} too narrow");
                        }
                        for (j, (other, _)) in placed.cards.iter().enumerate().skip(i + 1) {
                            assert!(!overlap(*card, *other), "{w}: cards {g}.{i} and {g}.{j}");
                        }
                        assert!(
                            placed.rule_y >= card.bottom(),
                            "{w}: rule over card {g}.{i}"
                        );
                    }
                }
                let h = page.content_height(content, &theme);
                assert!(h.is_finite() && h > 0.0, "{w}: {h}");
            }
        }
    }

    /// Narrow content stacks the cards one a row; wide content puts them
    /// side by side; the long-text card grows past the kit's height.
    #[test]
    fn cards_stack_when_narrow_and_grow_with_their_content() {
        let page = MessagePage::new();
        let theme = Theme::hoff_core();
        let narrow = page.layout(Rect::new(0.0, 0.0, 400.0, 800.0), &theme);
        assert!(narrow[0].cards.iter().all(|(r, _)| r.x == 0.0));
        let wide = page.layout(Rect::new(0.0, 0.0, 1200.0, 800.0), &theme);
        let (a, b) = (wide[0].cards[0].0, wide[0].cards[1].0);
        assert_eq!(a.y, b.y);
        assert!(b.x > a.x);
        let long = narrow[2].cards[0].0;
        assert!(
            long.h > showcase::card_min_h(&theme),
            "the long text grows its card: {long:?}"
        );
    }

    /// The reader's focus lands on a message or a chip, Escape lets it go,
    /// and only a chip's `activate` runs.
    #[test]
    fn focus_lands_on_a_message_and_only_a_chip_activates() {
        let mut page = MessagePage::new();
        let root = page.root().cloned().unwrap();
        let first = page.groups[0].cards[0].messages[0]
            .sem_id()
            .cloned()
            .unwrap();
        assert!(page.focus(&first));
        assert!(page.groups[0].cards[0].messages[0].is_focused());
        assert!(page.blur());
        assert!(!page.groups[0].cards[0].messages[0].is_focused());
        assert!(!page.blur(), "nothing left to let go");

        let chip = page.groups[1].cards[0].messages[0]
            .reaction_sem_id()
            .cloned()
            .unwrap();
        let seen = {
            let mut b = SnapshotBuilder::new();
            b.node(&root, Role::Region, "Message");
            page.project(&mut b, &root);
            b.finish().expect("the projection validates")
        };
        let ask = |target: &SemId| {
            let actor = Actor::Test {
                name: "message page".to_owned(),
            };
            Request::observe(&seen, actor, target.as_str(), "activate", None)
        };
        let press = ask(&chip).expect("a chip declares activate");
        assert_eq!(page.run(&press), Ok(After::Nothing));
        assert!(ask(&first).is_err(), "a message declares no action");
        // A request that reached the page for a node it does not run.
        let mut stray = press.clone();
        stray.target = first.clone();
        assert!(page.run(&stray).is_err());
    }
}
