//! Alert page, in the showcase mold: the inline alert of the hoff core
//! kit (its notification row) by intent, the kit's notifications (a
//! comment with its content, an invite with its answers, a like read, a
//! video), a list of them as the kit's notifications dropdown stacks
//! them, then the content an alert meets: long text, six scripts with
//! emoji. A card is as tall as its alerts measure (`showcase::layout_sized`),
//! never less than the kit's `h-50`; an alert takes the card's inside up
//! to a readable column.
//!
//! An alert is read by a click on it, and answered by a click on one of
//! its buttons, which reads it too: its unread dot goes.
//!
//! Projected: a group per title, `alert_page.group[<g>]`; in it each card
//! as a list named by its chip, `…card[<c>]`; in the list each alert as
//! the widget projects it, `…card[<c>].alert[<i>]`, with `activate`
//! (read), and its answers, `…alert[<i>].action[<j>]`, with `activate`
//! (choose).

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::buttons::ButtonVariant;
use engine::widgets::feedback::alert::{Alert, NotificationKind};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, root_id, showcase, under};

const DRAGGABLE: &str = "These draggable sliders look really cool. Maybe these could be \
     displayed when you hold shift, to rotate exactly on the X or Y. But by default I don't \
     think we need these controllers, you could just rotate the object by clicking and \
     dragging anywhere as expected on any 3D tool (theoretically).";

/// Six scripts and emoji (adr-0071).
const SCRIPTS: &str =
    "Olá! مرحبا بالعالم · שלום עולם · नमस्ते दुनिया · 你好世界 · こんにちは · 안녕하세요 🎉👍🏽";

struct Card {
    /// The chip: which state or content this card shows.
    title: &'static str,
    alerts: Vec<Alert>,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    columns: usize,
    cards: Vec<Card>,
}

/// A focusable part of the page: an alert, or one of its answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spot {
    g: usize,
    c: usize,
    a: usize,
    action: Option<usize>,
}

/// Where a group lands: the title's y, each card with its alerts' rects,
/// the rule's y.
struct Placed {
    title_y: f32,
    cards: Vec<(Rect, Vec<Rect>)>,
    rule_y: f32,
}

pub struct AlertPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// What a screen reader focused (its ring shows); Escape or a click
    /// lets it go.
    focus: Option<Spot>,
}

/// `alert_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `alert_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// `…card[<c>].alert[<i>]`.
fn alert_id(card: &SemId, i: usize) -> Option<SemId> {
    child_id(Some(card), "alert").map(|family| family.key(i.to_string()))
}

fn card(title: &'static str, alerts: Vec<Alert>) -> Card {
    Card { title, alerts }
}

/// The kit's comment notification, its content clamped.
fn comment() -> Alert {
    Alert::new(
        Intent::Neutral,
        "Randomdash",
        "Commented on Classic Car in Studio",
    )
    .kind(NotificationKind::Comment)
    .time("1h ago")
    .avatar("Randomdash")
    .content(DRAGGABLE)
    .unread(true)
}

/// The kit's invite, its two answers.
fn invite() -> Alert {
    Alert::new(
        Intent::Constructive,
        "3D object is generated",
        "Invited you to edit Minimalist Architecture Scene",
    )
    .kind(NotificationKind::Invite)
    .time("4h ago")
    .avatar("Hoff")
    .unread(true)
    .action("Decline", ButtonVariant::Primary)
    .action("Accept", ButtonVariant::Secondary)
}

fn like() -> Alert {
    Alert::new(
        Intent::Informational,
        "Marina",
        "Liked Classic Car in Studio",
    )
    .kind(NotificationKind::Like)
    .time("10h ago")
    .avatar("Marina")
}

fn video() -> Alert {
    Alert::new(
        Intent::Informational,
        "Animation is generated",
        "12s – 720p",
    )
    .kind(NotificationKind::Video)
    .time("1d ago")
    .avatar("Hoff")
    .content("Prompt: Create 3D character dancing")
}

impl AlertPage {
    pub fn new() -> Self {
        let mut groups = vec![
            Group {
                title: "Intent",
                columns: 2,
                cards: vec![
                    card(
                        "Neutral, Focus",
                        vec![
                            Alert::new(Intent::Neutral, "New comment", "On Classic Car in Studio")
                                .time("1h ago"),
                        ],
                    ),
                    card(
                        "Informational",
                        vec![
                            Alert::new(Intent::Informational, "Rendering", "12s – 720p, 40% done")
                                .time("now"),
                        ],
                    ),
                    card(
                        "Constructive, Unread",
                        vec![
                            Alert::new(
                                Intent::Constructive,
                                "Cute Turtle is generated",
                                "Matte texture - Hoff Style",
                            )
                            .time("2h ago")
                            .unread(true),
                        ],
                    ),
                    card(
                        "Destructive",
                        vec![
                            Alert::new(
                                Intent::Destructive,
                                "Export failed",
                                "The scene is larger than 2 GB",
                            )
                            .time("now"),
                        ],
                    ),
                ],
            },
            Group {
                title: "Notification",
                columns: 2,
                cards: vec![
                    card("Comment, Unread", vec![comment()]),
                    card("Invite, Hover, Press", vec![invite()]),
                    card("Like, Read", vec![like()]),
                    card("Video generated", vec![video()]),
                ],
            },
            Group {
                title: "List",
                columns: 1,
                cards: vec![card(
                    "Notifications, newest first",
                    vec![
                        comment(),
                        invite().divider(true),
                        like().divider(true),
                        video().divider(true),
                    ],
                )],
            },
            Group {
                title: "Content",
                columns: 2,
                cards: vec![
                    card(
                        "Long text",
                        vec![
                            Alert::new(Intent::Neutral, DRAGGABLE, DRAGGABLE)
                                .time("12 min ago")
                                .content(DRAGGABLE)
                                .unread(true),
                        ],
                    ),
                    card(
                        "Mixed scripts",
                        vec![
                            Alert::new(Intent::Informational, "سارة الأحمد", SCRIPTS)
                                .time("3m")
                                .avatar("سارة")
                                .content(SCRIPTS),
                        ],
                    ),
                ],
            },
        ];
        let root = root_id("alert_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(cid) = card_id(root, g, c) else {
                        continue;
                    };
                    for (i, a) in std::mem::take(&mut card.alerts).into_iter().enumerate() {
                        card.alerts.push(match alert_id(&cid, i) {
                            Some(id) => a.sem(id),
                            None => a,
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

    /// Every alert with its spot, then each of its answers.
    fn spots(&self) -> impl Iterator<Item = (Spot, Option<&SemId>)> + '_ {
        self.groups.iter().enumerate().flat_map(|(g, group)| {
            group.cards.iter().enumerate().flat_map(move |(c, card)| {
                card.alerts.iter().enumerate().flat_map(move |(a, alert)| {
                    let row = (
                        Spot {
                            g,
                            c,
                            a,
                            action: None,
                        },
                        alert.sem_id(),
                    );
                    let answers = alert.action_ids().enumerate().map(move |(j, id)| {
                        (
                            Spot {
                                g,
                                c,
                                a,
                                action: Some(j),
                            },
                            id,
                        )
                    });
                    std::iter::once(row).chain(answers)
                })
            })
        })
    }

    /// The alert, or its answer, whose id is `id`.
    fn at(&self, id: &SemId) -> Option<Spot> {
        self.spots()
            .find(|(_, sem)| *sem == Some(id))
            .map(|(spot, _)| spot)
    }

    fn alert_mut(&mut self, spot: Spot) -> Option<&mut Alert> {
        self.groups
            .get_mut(spot.g)?
            .cards
            .get_mut(spot.c)?
            .alerts
            .get_mut(spot.a)
    }

    /// The reader's focus on one alert or answer, or on none.
    fn set_focus(&mut self, at: Option<Spot>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                for (a, alert) in card.alerts.iter_mut().enumerate() {
                    let here = at.filter(|s| (s.g, s.c, s.a) == (g, c, a));
                    alert.set_focused(here.is_some_and(|s| s.action.is_none()));
                    alert.set_action_focused(here.and_then(|s| s.action));
                }
            }
        }
    }

    /// The width a card's alerts take: the card's inside, up to a
    /// readable column.
    fn alert_w(card_w: f32, theme: &Theme) -> f32 {
        (card_w - theme.spacing.sm * 2.0)
            .max(0.0)
            .min(theme.size.readable_max_w)
    }

    fn heights(card: &Card, w: f32, theme: &Theme) -> Vec<f32> {
        card.alerts.iter().map(|a| a.height(w, theme)).collect()
    }

    /// The showcase's groups with each card as tall as its alerts
    /// (`showcase::layout_sized`), the alerts stacked as one block
    /// centered in the card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Placed> {
        let sized = self.groups.iter().map(|g| {
            let w = Self::alert_w(showcase::card_width(g.columns, content, theme), theme);
            let cards = g.cards.iter().map(|card| {
                let stack: f32 = Self::heights(card, w, theme).iter().sum();
                showcase::card_height(stack, theme)
            });
            (g.columns, cards.collect())
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
                        let w = Self::alert_w(rect.w, theme);
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

    /// A click lets the reader's focus go; each alert takes the click at
    /// its rect (reads, or chooses an answer).
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
                for (a, r) in card.alerts.iter_mut().zip(rects) {
                    result = result.merge(a.handle_event(event, *r, theme));
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
                for (a, r) in card.alerts.iter().zip(rects) {
                    a.render(c, *r, theme);
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for AlertPage {
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
                for a in &card.alerts {
                    if let Some(id) = a.sem_id() {
                        a.project(b, id, Some(&cid));
                    }
                }
            }
        }
        let focused = self.focus.and_then(|spot| {
            self.spots()
                .find(|(s, _)| *s == spot)
                .and_then(|(_, id)| id.cloned())
        });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// An alert's `activate` reads it; an answer's chooses it, as a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let spot = self
            .at(&req.target)
            .filter(|_| req.action.as_str() == "activate");
        let done = spot.and_then(|spot| {
            let alert = self.alert_mut(spot)?;
            Some(match spot.action {
                Some(j) => alert.choose(j),
                None => {
                    alert.open();
                    true
                }
            })
        });
        match done {
            Some(true) => Ok(After::Nothing),
            _ => Err(format!(
                "the alert page has no {} on {}",
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

    /// Every alert and answer has the id its draw binds, under the root.
    #[test]
    fn every_alert_and_answer_has_an_id_under_the_root() {
        let page = AlertPage::new();
        let root = page.root().cloned().expect("the root parses");
        for (spot, id) in page.spots() {
            let id = id.unwrap_or_else(|| panic!("{spot:?} has no id"));
            assert!(under(&root, id), "{id}");
            assert_eq!(page.at(id), Some(spot), "{id}");
        }
    }

    /// The width sweep, 280 to 1600 every 8 px, with the long and the
    /// mixed content: every card in the content, every alert and its
    /// parts in its card and never narrower than its parts, no two cards
    /// over each other, the groups down the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = AlertPage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let layout = page.layout(content, &theme);
                let mut prev = content.y;
                for (g, (group, placed)) in page.groups.iter().zip(&layout).enumerate() {
                    assert!(placed.title_y >= prev, "{w}: group {g} climbs");
                    prev = placed.rule_y;
                    for (i, (card, rects)) in placed.cards.iter().enumerate() {
                        let tall = Rect::new(content.x, card.y, content.w, card.h);
                        assert!(within(*card, tall), "{w}: card {g}.{i} out of the content");
                        assert!(card.h >= showcase::CARD_H);
                        for (a, r) in group.cards[i].alerts.iter().zip(rects) {
                            assert!(within(*r, *card), "{w}: alert {r:?} out of {g}.{i}");
                            assert!(r.w >= a.min_width(&theme), "{w}: {g}.{i} too narrow");
                            let l = a.layout(*r, &theme);
                            for part in l.actions.iter().chain(l.dot.iter()) {
                                assert!(within(*part, *r), "{w}: {g}.{i} part out");
                            }
                        }
                        for (j, (other, _)) in placed.cards.iter().enumerate().skip(i + 1) {
                            assert!(!overlap(*card, *other), "{w}: cards {g}.{i} and {g}.{j}");
                        }
                        assert!(placed.rule_y >= card.bottom(), "{w}: rule over {g}.{i}");
                    }
                }
                let h = page.content_height(content, &theme);
                assert!(h.is_finite() && h > 0.0, "{w}: {h}");
            }
        }
    }

    /// The reader's focus lands on an alert or an answer and goes; an
    /// alert's `activate` reads it, an answer's chooses it.
    #[test]
    fn focus_lands_and_activate_reads_or_answers() {
        let mut page = AlertPage::new();
        let root = page.root().cloned().unwrap();
        let invite = page.groups[1].cards[1].alerts[0].sem_id().cloned().unwrap();
        let accept = page.groups[1].cards[1].alerts[0]
            .action_ids()
            .nth(1)
            .flatten()
            .cloned()
            .unwrap();
        assert!(page.focus(&accept));
        assert!(page.blur());
        assert!(!page.blur(), "nothing left to let go");

        let seen = {
            let mut b = SnapshotBuilder::new();
            b.node(&root, Role::Region, "Alert");
            page.project(&mut b, &root);
            b.finish().expect("the projection validates")
        };
        let ask = |target: &SemId| {
            let actor = Actor::Test {
                name: "alert page".to_owned(),
            };
            Request::observe(&seen, actor, target.as_str(), "activate", None)
        };
        let press = ask(&accept).expect("an answer declares activate");
        assert_eq!(page.run(&press), Ok(After::Nothing));
        let alert = &page.groups[1].cards[1].alerts[0];
        assert_eq!(alert.chosen(), Some(1));
        assert!(!alert.is_unread());

        let comment = page.groups[1].cards[0].alerts[0].sem_id().cloned().unwrap();
        let read = ask(&comment).expect("an alert declares activate");
        assert_eq!(page.run(&read), Ok(After::Nothing));
        assert!(!page.groups[1].cards[0].alerts[0].is_unread());
        let mut stray = read.clone();
        stray.target = card_id(&root, 0, 0).unwrap();
        assert!(page.run(&stray).is_err());
        assert!(under(&root, &invite));
    }
}
