//! Stat page, on the showcase mold of the kit's ButtonsPage
//! (`showcase`): groups with a title and a bottom rule, cards in rows, a
//! chip naming what each card shows. The widget is
//! `engine::widgets::content::Stat`; the kit has none, so its recipe
//! (`theme.navigation.stat`) derives from the kit's pricing card.
//!
//! The page shows the ranges a readout meets: the three sizes, the
//! three directions of a delta, a long label that truncates and a
//! translated readout whose delta wraps under its value when the card
//! is narrow.
//!
//! Projected: a group per title, `stat_page.group[<g>]`, and in it each
//! card's readout, `stat_page.group[<g>].card[<c>]`, a label named by its
//! label, its value and unit as its value, its delta as its description.
//! A readout is read, not operated: no action.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::content::{Stat, StatSize};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

struct Card {
    title: &'static str,
    stat: Stat,
}

struct Group {
    title: &'static str,
    span: usize,
    cards: Vec<Card>,
}

pub struct StatPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

/// `stat_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `stat_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl StatPage {
    pub fn new() -> Self {
        let card = |title, stat| Card { title, stat };
        let mut groups = vec![
            Group {
                title: "Stat",
                span: 3,
                cards: vec![
                    card("Small", Stat::new("38 412", "chunks").size(StatSize::Sm)),
                    card(
                        "Medium, unit, delta",
                        Stat::new("1.3", "on disk")
                            .unit("GB")
                            .delta("+12%", Intent::Constructive),
                    ),
                    card(
                        "Large, unit, delta",
                        Stat::new("97", "recall@10")
                            .unit("%")
                            .size(StatSize::Lg)
                            .delta("-2", Intent::Destructive),
                    ),
                ],
            },
            Group {
                title: "Delta",
                span: 3,
                cards: vec![
                    card(
                        "Rising",
                        Stat::new("2 481", "agents online").delta("+8%", Intent::Constructive),
                    ),
                    card(
                        "Falling",
                        Stat::new("312", "ms to first frame").delta("-41", Intent::Destructive),
                    ),
                    card(
                        "Steady",
                        Stat::new("99.9", "uptime")
                            .unit("%")
                            .delta("±0", Intent::Neutral),
                    ),
                ],
            },
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    card(
                        "Long label, truncated",
                        Stat::new(
                            "12 840",
                            "requests answered from the cache in the last hour",
                        )
                        .delta("+3.1%", Intent::Constructive),
                    ),
                    card(
                        "Translated, delta wraps",
                        Stat::new("1,3", "armazenamento usado")
                            .unit("TB")
                            .size(StatSize::Lg)
                            .delta("+12% em relação à semana passada", Intent::Constructive),
                    ),
                ],
            },
        ];
        let root = root_id("stat_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.stat = card.stat.clone().sem(id);
                    }
                }
            }
        }
        Self { root, groups }
    }

    /// The readout's rect in a card `card_w` wide: as wide as it asks,
    /// never wider than the card less its inset (the label truncates,
    /// the delta wraps), never narrower than its value row; as tall as it
    /// is at that width.
    fn widget_size(stat: &Stat, card_w: f32, theme: &Theme) -> (f32, f32) {
        let room = card_w - theme.spacing.sm * 2.0;
        let w = stat
            .preferred_size(theme)
            .0
            .min(room)
            .max(stat.min_width(theme));
        (w, stat.height_at(w, theme))
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| {
            let w = showcase::card_width(g.span, content, theme);
            let heights = g
                .cards
                .iter()
                .map(|c| showcase::card_height(Self::widget_size(&c.stat, w, theme).1, theme))
                .collect();
            (g.span, heights)
        });
        showcase::layout_sized(groups, content, theme)
    }

    fn widget_rect(stat: &Stat, card: Rect, theme: &Theme) -> Rect {
        showcase::centered(Self::widget_size(stat, card.w, theme), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    pub fn handle_event(
        &mut self,
        _event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                showcase::card(c, *rect, card.title, theme);
                let r = Self::widget_rect(&card.stat, *rect, theme);
                card.stat.render(c, r, theme);
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for StatPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                if let Some(id) = card.stat.sem_id() {
                    card.stat.project(b, id, Some(&at));
                }
            }
        }
    }

    /// A readout offers nothing to run.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the stat page has no {} on {}",
            req.action, req.target
        ))
    }

    /// A reader lands on a readout to hear it; nothing here draws a
    /// focus, so the page takes it and keeps none.
    fn focus(&mut self, id: &SemId) -> bool {
        self.root.as_ref().is_some_and(|root| under(root, id))
    }
}

#[cfg(test)]
mod tests;
