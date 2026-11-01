//! Message Scroller page, in the showcase mold: the kit's comment popup
//! (`components/Comment`) as a thread that scrolls, short (it fits), long
//! (it scrolls under the wheel, a touch drag and the keyboard, one scroll
//! state) and empty. A card is as tall as its thread (`showcase::layout_sized`),
//! never less than the kit's `h-50`; a thread grows to its cap and then
//! scrolls.
//!
//! Projected: a group per title, `message_scroller_page.group[<g>]`; in it
//! each card's thread as a list named by its chip,
//! `…card[<c>]`, with every message, shown or scrolled away,
//! `…card[<c>][<i>]`, and a message's like chip, `…[<i>].reaction`. A
//! reader or the keyboard landing on a message scrolls it into view; a
//! like chip presses and nothing else happens, as in the kit.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::message::message::{Message, MessageKind};
use engine::widgets::message::scroller::MessageScroller;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, root_id, showcase, under};

const BODIES: [&str; 4] = [
    "I love this bag, it looks great. Could you make it with leather material? 🤔",
    "@Luna Thank, I’m working on it",
    "These draggable sliders look really cool. Maybe these could be displayed when you hold \
     shift, to rotate exactly on the X or Y.",
    "Olá! مرحبا بالعالم · שלום · नमस्ते · 你好世界 · こんにちは · 안녕 🎉",
];

const PEOPLE: [&str; 4] = ["Luna Starfield", "Sam", "Randomdash", "سارة"];

/// The `n`-message thread the long card scrolls; every fourth a quick
/// comment with its like chip.
fn thread(n: usize) -> Vec<Message> {
    (0..n)
        .map(|i| {
            let m = Message::new(PEOPLE[i % 4], format!("{}m", n - i), BODIES[i % 4])
                .divider(true)
                .verified(i % 4 == 0);
            if i % 4 == 2 {
                m.kind(MessageKind::Quick).reaction((i + 1).to_string())
            } else {
                m
            }
        })
        .collect()
}

struct Card {
    title: &'static str,
    scroller: MessageScroller,
}

pub struct MessageScrollerPage {
    root: Option<SemId>,
    /// One group, "Message scroller"; its cards.
    cards: Vec<Card>,
    /// The message (card, index) or its chip a reader focused.
    focus: Option<(usize, usize, bool)>,
    /// A focus landed on a message the next layout scrolls into view.
    reveal: Option<(usize, usize)>,
}

const GROUP: &str = "Message scroller";
const COLUMNS: usize = 3;

impl MessageScrollerPage {
    pub fn new() -> Self {
        let root = root_id("message_scroller_page");
        let card_id = |c: usize| {
            child_id(root.as_ref(), "group")
                .map(|g| g.key("0"))
                .and_then(|g| child_id(Some(&g), "card"))
                .map(|f| f.key(c.to_string()))
        };
        let cards = [
            ("Short, it fits", thread(2)),
            ("Long, it scrolls", thread(12)),
            ("Empty", Vec::new()),
        ]
        .into_iter()
        .enumerate()
        .map(|(c, (title, messages))| {
            let scroller = MessageScroller::new(title, "No comments yet", messages);
            let scroller = match card_id(c) {
                Some(id) => scroller.sem(id),
                None => scroller,
            };
            Card { title, scroller }
        })
        .collect();
        Self {
            root,
            cards,
            focus: None,
            reveal: None,
        }
    }

    fn group_id(root: &SemId) -> Option<SemId> {
        child_id(Some(root), "group").map(|g| g.key("0"))
    }

    /// The message, or its chip, whose id is `id`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, bool)> {
        self.cards.iter().enumerate().find_map(|(c, card)| {
            card.scroller
                .messages()
                .iter()
                .enumerate()
                .find_map(|(i, m)| {
                    if m.sem_id() == Some(id) {
                        Some((c, i, false))
                    } else if m.reaction_sem_id() == Some(id) {
                        Some((c, i, true))
                    } else {
                        None
                    }
                })
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, bool)>) {
        self.focus = at;
        for (c, card) in self.cards.iter_mut().enumerate() {
            for (i, m) in card.scroller.messages_mut().iter_mut().enumerate() {
                m.set_focused(at == Some((c, i, false)));
                m.set_reaction_focused(at == Some((c, i, true)));
            }
        }
        self.reveal = at.map(|(c, i, _)| (c, i));
    }

    fn lane_w(card_w: f32, theme: &Theme) -> f32 {
        showcase::room(card_w, theme).min(theme.size.readable_max_w)
    }

    /// The group's layout (`showcase::layout_sized`, each card as tall as
    /// its thread) and each card's thread rect, centered in it.
    fn layout(&self, content: Rect, theme: &Theme) -> (showcase::GroupLayout, Vec<Rect>) {
        let w = Self::lane_w(showcase::card_width(COLUMNS, content, theme), theme);
        let cards = self
            .cards
            .iter()
            .map(|card| showcase::card_height(card.scroller.height(w, theme), theme))
            .collect();
        let group = showcase::layout_sized([(GROUP, COLUMNS, cards)], content, theme)
            .pop()
            .expect("one group");
        let threads = group
            .cards
            .iter()
            .zip(&self.cards)
            .map(|(rect, card)| {
                let w = Self::lane_w(rect.w, theme);
                showcase::centered((w, card.scroller.height(w, theme)), *rect)
            })
            .collect();
        (group, threads)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let (group, _) = self.layout(content, theme);
        showcase::content_height(&[group], content)
    }

    /// Scrolls a focused message into view, once its thread is laid out.
    fn apply_reveal(&mut self, content: Rect, theme: &Theme) -> bool {
        let Some((c, i)) = self.reveal.take() else {
            return false;
        };
        let (_, threads) = self.layout(content, theme);
        self.cards[c].scroller.reveal(i, threads[c], theme)
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
        let (_, threads) = self.layout(content, theme);
        for (card, rect) in self.cards.iter_mut().zip(threads) {
            result = result.merge(card.scroller.handle_event(event, rect, theme));
        }
        result
    }

    /// The scrollbars' fade. `true` while one animates.
    pub fn tick(&mut self, dt: f32) -> bool {
        // Every bar ticks, even after one says it moves.
        let mut moving = false;
        for card in &mut self.cards {
            moving |= card.scroller.tick(dt);
        }
        moving
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        self.apply_reveal(content, theme);
        let (group, threads) = self.layout(content, theme);
        showcase::title(c, GROUP, content, group.title_y, theme);
        for ((card, rect), thread) in self.cards.iter().zip(&group.cards).zip(threads) {
            showcase::card(c, *rect, card.title, theme);
            card.scroller.render(c, thread, theme);
        }
    }
}

impl Component for MessageScrollerPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(gid) = Self::group_id(root) else {
            return;
        };
        b.node(&gid, Role::Group, GROUP).parent(root);
        for card in &self.cards {
            if let Some(id) = card.scroller.sem_id() {
                card.scroller.project(b, id, Some(&gid));
            }
        }
        let focused = self.focus.and_then(|(c, i, chip)| {
            let m = self.cards.get(c)?.scroller.messages().get(i)?;
            if chip {
                m.reaction_sem_id()
            } else {
                m.sem_id()
            }
        });
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A like chip's `activate`: it presses, and nothing else happens.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some((_, _, true)), "activate") => Ok(After::Nothing),
            _ => Err(format!(
                "the message scroller page has no {} on {}",
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
        self.reveal = None;
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

    /// The width sweep, 280 to 1600 every 8 px: every thread inside its
    /// card, every card inside the content, the long thread at its cap.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = MessageScrollerPage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let (group, threads) = page.layout(content, &theme);
                for (card, thread) in group.cards.iter().zip(&threads) {
                    let lane = Rect::new(content.x, card.y, content.w, card.h);
                    assert!(within(*card, lane), "{w}: card out of the content");
                    assert!(within(*thread, *card), "{w}: thread out of its card");
                }
                assert!(threads[1].h <= theme.message.scroller.max_h + EPS, "{w}");
            }
        }
    }

    /// A reader landing on a message below the fold scrolls it into view
    /// at the next frame; Escape lets the focus go; only a chip runs.
    #[test]
    fn a_focused_message_scrolls_into_view() {
        let mut page = MessageScrollerPage::new();
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1000.0, 800.0);
        let target = page.cards[1].scroller.messages()[10]
            .sem_id()
            .cloned()
            .unwrap();
        assert!(page.focus(&target));
        assert!(page.apply_reveal(content, &theme), "it scrolls");
        let (_, threads) = page.layout(content, &theme);
        let row = page.cards[1].scroller.rows(threads[1], &theme)[10];
        assert!(within(row, threads[1]), "{row:?} shows in {:?}", threads[1]);
        assert!(page.blur());
        assert!(page.focus.is_none() && page.reveal.is_none());
    }
}
