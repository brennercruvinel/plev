//! Progress page, in the showcase mold: the hoff core kit's video player
//! rail (`VideoPlayer`) as a progress bar, empty, part way and full; per
//! intent; then the widths it meets, the kit's narrowest (`min-w-45`) and
//! the whole card. A bar spans its card's body, never under the kit's
//! narrowest unless the body is.
//!
//! Projected: a group per title, `progress_page.group[<g>]`; in it each
//! card as a group named by its chip, `…card[<c>]`; in it the bar,
//! `…card[<c>].bar`, a progress bar with its value from 0 to 1. A bar
//! takes no input.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::prelude::{EventResult, ProgressBar, Rect, WidgetEvent};

use super::{After, Component, child_id, root_id, showcase, under};

struct Card {
    title: &'static str,
    bar: ProgressBar,
    /// Only as wide as the kit's narrowest rail.
    narrow: bool,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct ProgressPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn card(title: &'static str, bar: ProgressBar) -> Card {
    Card {
        title,
        bar,
        narrow: false,
    }
}

fn groups() -> Vec<Group> {
    let at = |v: f32, i: Intent| ProgressBar::new(v).intent(i);
    vec![
        Group {
            title: "Value",
            columns: 3,
            cards: vec![
                card("Empty", ProgressBar::new(0.0)),
                card("Part way", ProgressBar::new(0.4)),
                card("Full", ProgressBar::new(1.0)),
            ],
        },
        Group {
            title: "Intent",
            columns: 2,
            cards: vec![
                card("Neutral", at(0.65, Intent::Neutral)),
                card("Info", at(0.65, Intent::Informational)),
                card("Success", at(1.0, Intent::Constructive)),
                card("Error", at(0.35, Intent::Destructive)),
            ],
        },
        Group {
            title: "Width",
            columns: 2,
            cards: vec![
                Card {
                    title: "Narrowest",
                    bar: ProgressBar::new(0.4),
                    narrow: true,
                },
                card("Whole card", ProgressBar::new(0.4)),
            ],
        },
    ]
}

impl ProgressPage {
    pub fn new() -> Self {
        let mut groups = groups();
        let root = root_id("progress_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = child_id(card_id(root, g, c).as_ref(), "bar") {
                        card.bar = card.bar.clone().sem(id);
                    }
                }
            }
        }
        Self { root, groups }
    }

    /// Per group: its layout, and per card its bar's rect, centered in the
    /// card's body.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Rect>)> {
        let groups = self.groups.iter().map(|g| (g.columns, g.cards.len()));
        let bar_h = ProgressBar::preferred_size(theme).1;
        showcase::layout_measured(groups, content, theme, |_, _, _| bar_h)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let rects = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        let body = showcase::body(*rect, theme).w;
                        let (min_w, h) = ProgressBar::preferred_size(theme);
                        let w = if card.narrow { min_w.min(body) } else { body };
                        showcase::centered((w, h), *rect)
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
        for (i, (group, (placed, rects))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), r) in group.cards.iter().zip(&placed.cards).zip(rects) {
                showcase::card(c, *rect, card.title, theme);
                card.bar.render(c, *r, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for ProgressPage {
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
                if let Some(id) = card.bar.sem_id() {
                    card.bar.project(b, id, Some(&cid), card.title);
                }
            }
        }
    }

    /// A bar declares no action.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the progress page has no {} on {}",
            req.action, req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        self.root.as_ref().is_some_and(|root| under(root, id))
    }

    fn blur(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 0.5;

    /// The width sweep, 280 to 1600 every 8 px, under the core and hoff:
    /// every bar inside its card's body and its parts inside the bar, the
    /// narrow one the kit's narrowest where the body allows.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = ProgressPage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                for (group, (placed, rects)) in page.groups.iter().zip(page.layout(content, &theme))
                {
                    for ((card, rect), r) in group.cards.iter().zip(&placed.cards).zip(&rects) {
                        let body = showcase::body(*rect, &theme);
                        assert!(r.x >= body.x - EPS && r.right() <= body.right() + EPS);
                        let l = card.bar.layout(*r, &theme);
                        for part in [l.track, l.fill, l.head] {
                            assert!(part.x >= r.x - EPS && part.right() <= r.right() + EPS);
                        }
                        if card.narrow && body.w >= theme.message.progress.min_w {
                            assert_eq!(r.w, theme.message.progress.min_w);
                        }
                    }
                }
            }
        }
    }

    /// Each bar is a progress bar with its value, and runs nothing.
    #[test]
    fn a_bar_is_read_with_its_value() {
        let page = ProgressPage::new();
        let root = page.root().cloned().expect("the root parses");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Progress");
        page.project(&mut b, &root);
        let seen = b.finish().expect("the projection validates");
        let id = page.groups[0].cards[2].bar.sem_id().cloned().unwrap();
        let n = seen.get(&id).expect("the full bar");
        assert_eq!(n.role, Role::ProgressBar);
        assert_eq!(n.state.get("value"), Some(&accord::Value::Num(1.0)));
        assert!(n.actions.is_empty());
    }
}
