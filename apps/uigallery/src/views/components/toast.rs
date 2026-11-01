//! Toast page, in the showcase mold: the triggers, one button per intent,
//! whose click queues a live toast (the view owns the toast manager);
//! then the hoff core kit's toast at rest, drawn in its card as the queue
//! draws it (`Toast::draw`), once per intent; then the content a toast
//! meets, a message that wraps and six scripts with emoji. A drawn
//! toast is as wide as the kit's (`size.toast_w`) or its card's body, and
//! as tall as its message wraps.
//!
//! Projected: a group per title, `toast_page.group[<g>]`; in it each card
//! as a group named by its chip, `…card[<c>]`; a drawn toast in it,
//! `…card[<c>].toast`, a label named by its message with its intent and
//! no action (a sample at rest is read where it sits, not announced: only
//! a queued toast is a status or an alert); each trigger,
//! `toast_page.button[<intent>]`, whose `activate` asks the view for the
//! toast a click queues.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::{Intent, Theme};
use engine::widgets::feedback::Toast;
use engine::widgets::prelude::{Button, EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, flow, root_id, showcase, under};

const LONG: &str = "Failed to push to origin: the remote rejected the update because the \
     branch moved since you fetched. Pull, resolve what conflicts and push again.";

/// Six scripts and emoji in one message (adr-0071).
const SCRIPTS: &str = "Olá · مرحبا · שלום · नमस्ते · 你好 · 안녕 🎉 synced";

/// A toast drawn at rest in its card.
struct Drawn {
    message: &'static str,
    intent: Intent,
    sem: Option<SemId>,
}

enum Shown {
    Toast(Drawn),
    Triggers,
}

struct Card {
    title: &'static str,
    shown: Shown,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct ToastPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    buttons: Vec<(Intent, Button)>,
    /// The button a screen reader focused (its ring shows); Escape or a
    /// click lets it go.
    focus: Option<usize>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn drawn(title: &'static str, message: &'static str, intent: Intent) -> Card {
    Card {
        title,
        shown: Shown::Toast(Drawn {
            message,
            intent,
            sem: None,
        }),
    }
}

fn groups() -> Vec<Group> {
    let at_rest = |intent| ToastPage::message(intent);
    vec![
        Group {
            title: "Trigger",
            columns: 1,
            cards: vec![Card {
                title: "Click, Activate",
                shown: Shown::Triggers,
            }],
        },
        Group {
            title: "Intent",
            columns: 2,
            cards: vec![
                drawn("Neutral", at_rest(Intent::Neutral), Intent::Neutral),
                drawn(
                    "Success",
                    at_rest(Intent::Constructive),
                    Intent::Constructive,
                ),
                drawn("Error", at_rest(Intent::Destructive), Intent::Destructive),
                drawn(
                    "Info",
                    at_rest(Intent::Informational),
                    Intent::Informational,
                ),
            ],
        },
        Group {
            title: "Content",
            columns: 2,
            cards: vec![
                drawn("Long message", LONG, Intent::Destructive),
                drawn("Mixed scripts", SCRIPTS, Intent::Informational),
            ],
        },
    ]
}

impl ToastPage {
    pub fn new() -> Self {
        let root = root_id("toast_page");
        let family = child_id(root.as_ref(), "button");
        let button = |key: &str, label: &str, intent: Intent| {
            let b = Button::new(label).intent(intent);
            let b = match &family {
                Some(f) => b.sem(f.key(key)),
                None => b,
            };
            (intent, b)
        };
        let mut groups = groups();
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Shown::Toast(d) = &mut card.shown {
                        d.sem = child_id(card_id(root, g, c).as_ref(), "toast");
                    }
                }
            }
        }
        Self {
            buttons: vec![
                button("neutral", "Neutral", Intent::Neutral),
                button("success", "Success", Intent::Constructive),
                button("error", "Error", Intent::Destructive),
                button("info", "Info", Intent::Informational),
            ],
            groups,
            root,
            focus: None,
        }
    }

    /// The button behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.buttons
            .iter()
            .position(|(_, b)| b.sem_id() == Some(id))
    }

    /// The reader's focus on the button at `at`, or on none.
    fn set_focus(&mut self, at: Option<usize>) {
        self.focus = at;
        for (i, (_, b)) in self.buttons.iter_mut().enumerate() {
            b.set_focused(at == Some(i));
        }
    }

    /// The message a toast of `intent` carries.
    pub fn message(intent: Intent) -> &'static str {
        match intent {
            Intent::Neutral => "Neutral toast: plain information.",
            Intent::Constructive => "Saved! Everything went fine.",
            Intent::Destructive => "Failed to push: remote rejected.",
            Intent::Informational => "3 new commits fetched from origin.",
        }
    }

    /// A card's content from the origin of a body `w` wide: a drawn toast
    /// as wide as the kit's or the body, or the triggers flowed.
    fn content(&self, card: &Card, w: f32, theme: &Theme) -> (Vec<Rect>, f32) {
        match &card.shown {
            Shown::Toast(d) => {
                let tw = theme.size.toast_w.min(w);
                let h = Toast::height(d.message, tw, theme);
                (vec![Rect::new(0.0, 0.0, tw, h)], h)
            }
            Shown::Triggers => flow(
                Rect::new(0.0, 0.0, w, 0.0),
                0.0,
                theme.spacing.md,
                self.buttons.iter().map(|(_, b)| {
                    let (bw, bh) = b.preferred_size(theme);
                    (bw.min(w), bh)
                }),
            ),
        }
    }

    fn body_w(card_w: f32, theme: &Theme) -> f32 {
        showcase::body(Rect::new(0.0, 0.0, card_w, 0.0), theme).w
    }

    /// Per group: its layout (each card as tall as its content), and per
    /// card its content's rects, the block centered in the card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Vec<Rect>>)> {
        let sized = self.groups.iter().map(|g| {
            let w = Self::body_w(showcase::card_width(g.columns, content, theme), theme);
            let cards = g
                .cards
                .iter()
                .map(|card| showcase::card_height(self.content(card, w, theme).1, theme));
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
                        let (rects, h) = self.content(card, Self::body_w(rect.w, theme), theme);
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

    /// The triggers' rects at this layout.
    fn trigger_rects(&self, content: Rect, theme: &Theme) -> Vec<Rect> {
        self.layout(content, theme)
            .into_iter()
            .zip(&self.groups)
            .flat_map(|((_, rects), group)| {
                group
                    .cards
                    .iter()
                    .zip(rects)
                    .filter(|(card, _)| matches!(card.shown, Shown::Triggers))
                    .flat_map(|(_, rects)| rects)
                    .collect::<Vec<_>>()
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

    /// Route an event; the second value is the intent of a clicked button.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<Intent>) {
        let rects = self.trigger_rects(content, theme);
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let mut queued = None;
        for ((intent, button), rect) in self.buttons.iter_mut().zip(rects) {
            let r = button.handle_event(event, rect);
            if r.clicked {
                queued = Some(*intent);
            }
            result = result.merge(r);
        }
        (result, queued)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, rects))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), rects) in group.cards.iter().zip(&placed.cards).zip(rects) {
                showcase::card(c, *rect, card.title, theme);
                match &card.shown {
                    Shown::Toast(d) => {
                        if let Some(r) = rects.first() {
                            if let Some(id) = &d.sem {
                                c.bind(id, [r.x, r.y, r.w, r.h]);
                            }
                            Toast::draw(c, LayerId::DEFAULT, *r, d.message, d.intent, 1.0, theme);
                        }
                    }
                    Shown::Triggers => {
                        for ((_, b), r) in self.buttons.iter().zip(rects) {
                            b.render(c, *r, theme);
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

impl Component for ToastPage {
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
                b.node(&cid, Role::Group, card.title).parent(&gid);
                match &card.shown {
                    Shown::Toast(d) => {
                        if let Some(id) = &d.sem {
                            b.node(id, Role::Label, d.message)
                                .parent(&cid)
                                .state("intent", intent_word(d.intent));
                        }
                    }
                    Shown::Triggers => {
                        for (_, button) in &self.buttons {
                            if let Some(id) = button.sem_id() {
                                button.project(b, id, Some(&cid));
                            }
                        }
                    }
                }
            }
        }
        let focused = self.focus.and_then(|i| self.buttons.get(i));
        if let Some(id) = focused.and_then(|(_, button)| button.sem_id()) {
            b.focus(id);
        }
    }

    /// A button's `activate` queues its toast, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(i), "activate") => Ok(After::Toast(self.buttons[i].0)),
            _ => Err(format!(
                "the toast page has no {} on {}",
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

/// The intent as a reader hears it.
fn intent_word(intent: Intent) -> &'static str {
    match intent {
        Intent::Neutral => "neutral",
        Intent::Informational => "informational",
        Intent::Constructive => "constructive",
        Intent::Destructive => "destructive",
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

    /// The width sweep, 280 to 1600 every 8 px, under the core, hoff and a
    /// theme whose message run is larger: every drawn toast and trigger
    /// inside its card's body and as tall as its message wraps, the cards
    /// inside the content, the groups down the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = ToastPage::new();
        let mut large = Theme::hoff_core();
        large.message.alert.detail.run = engine::styling::theme::TypeRole::H6.run();
        for theme in [Theme::hoff_core(), Theme::hoff(), large] {
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
                        assert!(within(*rect, lane), "{w}: {} out", card.title);
                        let body = showcase::body(*rect, &theme);
                        for r in rects {
                            assert!(within(*r, *rect), "{w}: {} out of its card", card.title);
                            assert!(r.w <= body.w + EPS, "{w}: {} wider", card.title);
                        }
                        if let (Shown::Toast(d), Some(r)) = (&card.shown, rects.first()) {
                            assert_eq!(r.h, Toast::height(d.message, r.w, &theme));
                        }
                    }
                }
                assert_eq!(page.trigger_rects(content, &theme).len(), 4);
            }
        }
    }

    /// A drawn toast is a label named by its message, with its intent, and
    /// runs nothing; a trigger's `activate` queues its intent's toast.
    #[test]
    fn a_drawn_toast_is_read_and_a_trigger_queues() {
        let mut page = ToastPage::new();
        let root = page.root().cloned().expect("the root parses");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Toast");
        page.project(&mut b, &root);
        let seen = b.finish().expect("the projection validates");
        let Shown::Toast(error) = &page.groups[1].cards[2].shown else {
            panic!("the error card draws a toast");
        };
        let id = error.sem.clone().expect("an id");
        let node = seen.get(&id).expect("the drawn toast");
        assert_eq!(node.role, Role::Label, "a sample is not announced");
        assert_eq!(node.label, ToastPage::message(Intent::Destructive));
        assert!(format!("{:?}", node.state).contains("destructive"));
        assert!(node.actions.is_empty());
        let revision = seen.revision();
        let ask = |target: SemId| Request {
            actor: accord::Actor::Test {
                name: "toast".into(),
            },
            target,
            action: accord::ActionName::parse("activate").expect("an action name"),
            param: None,
            observed: accord::Observation {
                revision,
                values: Vec::new(),
            },
            ask: None,
        };
        let button = page.buttons[1].1.sem_id().cloned().unwrap();
        assert!(matches!(
            page.run(&ask(button)),
            Ok(After::Toast(Intent::Constructive))
        ));
        assert!(page.run(&ask(id)).is_err());
    }
}
