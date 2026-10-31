//! Toggle Group page, on the showcase mold of the kit's ButtonsPage: the
//! kit's segmented control (`engine::widgets::selection::ToggleGroup`),
//! at both of its steps, disabled, and with long and translated labels
//! that truncate inside their segments. The thumb slides on the theme's
//! spring; every visual value comes from the theme.
//!
//! Projected: a group per title, `toggle_group_page.group[<g>]`, and in it
//! each card's toggle group, `toggle_group_page.group[<g>].card[<c>]`, a
//! `Role::Group` of `Role::Radio` segments (`...card[<c>][<i>]`), described
//! by the card's chip. A segment's `select` chooses it as a click does.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::widgets::selection::{SegmentSize, ToggleGroup};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, key_under, root_id, under};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    /// The group's accessible name.
    name: &'static str,
    group: ToggleGroup,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    span: usize,
    cards: Vec<Card>,
}

pub struct ToggleGroupPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The segment, `(group, card, segment)`, a reader or the keyboard
    /// focused (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize, usize)>,
}

/// `toggle_group_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `toggle_group_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

const VIEWS: [&str; 3] = ["Day", "Week", "Month"];

/// A step of the kit: live, and disabled.
fn step(title: &'static str, size: SegmentSize, theme: &Theme) -> Group {
    let make = || {
        ToggleGroup::new(VIEWS)
            .size(size)
            .selected(1)
            .with_motion(&theme.motion)
    };
    Group {
        title,
        span: 2,
        cards: vec![
            Card {
                title: "Default, Hover, Press, Focus",
                name: "view",
                group: make(),
            },
            Card {
                title: "Disabled",
                name: "view",
                group: make().disabled(true),
            },
        ],
    }
}

impl ToggleGroupPage {
    pub fn new(theme: &Theme) -> Self {
        let mut groups = vec![
            step("Toggle group", SegmentSize::Md, theme),
            step("Toggle group, small", SegmentSize::Sm, theme),
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    Card {
                        title: "Long labels",
                        name: "show",
                        group: ToggleGroup::new([
                            "Everything",
                            "Only the active ones",
                            "Completed and archived",
                        ])
                        .with_motion(&theme.motion),
                    },
                    Card {
                        title: "Translated labels",
                        name: "Ansicht",
                        group: ToggleGroup::new(["Tag", "Woche", "Monat", "Jahr"])
                            .selected(2)
                            .with_motion(&theme.motion),
                    },
                ],
            },
        ];
        let root = root_id("toggle_group_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.group = card.group.clone().sem(id);
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

    fn cards(&self) -> impl Iterator<Item = ((usize, usize), &Card)> {
        self.groups.iter().enumerate().flat_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .map(move |(c, card)| ((g, c), card))
        })
    }

    /// The segment behind `id`: `(group, card, segment)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.cards().find_map(|((g, c), card)| {
            let parent = card.group.sem_id()?;
            let i = key_under(parent, id)?.parse::<usize>().ok()?;
            (i < card.group.items.len()).then_some((g, c, i))
        })
    }

    fn group_mut(&mut self, g: usize, c: usize) -> Option<&mut ToggleGroup> {
        Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.group)
    }

    /// The focus on one segment, or on none; a disabled group refuses it.
    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let seg = at.filter(|a| (a.0, a.1) == (g, c)).map(|a| a.2);
                card.group.set_focused(seg);
            }
        }
        self.focus = at.filter(|&(g, c, i)| {
            self.groups
                .get(g)
                .and_then(|group| group.cards.get(c))
                .is_some_and(|card| card.group.focused() == Some(i))
        });
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| (g.span, g.cards.len()));
        showcase::layout(groups, content, theme)
    }

    /// The group's rect in its card: its intrinsic size, never wider than
    /// the card less its inset on both sides (the labels truncate), never
    /// narrower than its ellipses, centered.
    fn widget_rect(group: &ToggleGroup, card: Rect, theme: &Theme) -> Rect {
        let (w, h) = group.preferred_size(theme);
        let room = card.w - theme.spacing.sm * 2.0;
        let w = w.min(room).max(group.min_width(theme));
        showcase::centered((w, h), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// Advance the thumbs; `true` while one still slides.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for group in &mut self.groups {
            for card in &mut group.cards {
                moving |= card.group.tick(dt);
            }
        }
        moving
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
        for (group, at) in self.groups.iter_mut().zip(&layout) {
            for (card, rect) in group.cards.iter_mut().zip(&at.cards) {
                let bounds = Self::widget_rect(&card.group, *rect, theme);
                result = result.merge(card.group.handle_event(event, bounds, theme));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                showcase::card(c, *rect, card.title, theme);
                card.group
                    .render(c, Self::widget_rect(&card.group, *rect, theme), theme);
            }
            if i != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for ToggleGroupPage {
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
                if let Some(id) = card.group.sem_id() {
                    card.group
                        .project(b, id, card.name, Some(&at))
                        .description(card.title);
                }
            }
        }
        let focused = self.focus.and_then(|(g, c, i)| {
            let card = self.groups.get(g)?.cards.get(c)?;
            Some(card.group.sem_id()?.key(i.to_string()))
        });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// A segment's `select` chooses it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let chose = match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, i)), "select") => self.group_mut(g, c).is_some_and(|t| t.select(i)),
            _ => false,
        };
        if chose {
            Ok(After::Nothing)
        } else {
            Err(format!(
                "the toggle group page has no {} on {}",
                req.action, req.target
            ))
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

    fn page() -> (ToggleGroupPage, Theme) {
        let theme = Theme::hoff_core();
        (ToggleGroupPage::new(&theme), theme)
    }

    /// Every id parses, so the page projects.
    #[test]
    fn the_page_has_its_root_and_an_id_per_card() {
        let (page, _) = page();
        assert!(page.root.is_some());
        assert!(page.cards().all(|(_, c)| c.group.sem_id().is_some()));
    }

    /// A click on a segment chooses it and the thumb slides there; the
    /// disabled group does not move.
    #[test]
    fn a_click_chooses_the_segment_and_the_thumb_slides() {
        let (mut page, theme) = page();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let layout = page.layout(content, &theme);
        let press = |page: &mut ToggleGroupPage, g: usize, c: usize, i: usize| {
            let card = layout[g].cards[c];
            let group = &page.groups[g].cards[c].group;
            let bounds = ToggleGroupPage::widget_rect(group, card, &theme);
            let (x, y) = group.segment_rects(bounds, &theme)[i].center();
            page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
            page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme)
        };
        assert!(press(&mut page, 0, 0, 2).clicked);
        assert_eq!(page.groups[0].cards[0].group.selected, 2);
        assert!(page.tick(1.0 / 60.0), "the thumb slides");
        press(&mut page, 0, 1, 0);
        assert_eq!(page.groups[0].cards[1].group.selected, 1, "disabled");
    }

    /// The focus a reader asks for lands on a segment of a live group; a
    /// disabled group refuses it.
    #[test]
    fn the_focus_lands_on_a_segment_and_skips_a_disabled_group() {
        let (mut page, _) = page();
        let seg = |page: &ToggleGroupPage, g: usize, c: usize, i: usize| {
            page.groups[g].cards[c]
                .group
                .sem_id()
                .unwrap()
                .key(i.to_string())
        };
        let (live, off) = (seg(&page, 0, 0, 2), seg(&page, 0, 1, 2));
        assert!(page.focus(&live));
        assert_eq!(page.focus, Some((0, 0, 2)));
        assert_eq!(page.groups[0].cards[0].group.focused(), Some(2));
        assert!(page.focus(&off));
        assert_eq!(page.focus, None);
    }

    /// The width sweep of the epic, 280 to 1600 every 8 px, under the
    /// core and the hoff themes: every card in the content, the cards of
    /// a row apart, every group inside its card, every label inside its
    /// segment, truncated when it does not fit.
    #[test]
    fn nothing_cuts_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = ToggleGroupPage::new(&theme);
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let mut prev_rule = content.y;
                for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
                    assert!(at.title_y >= prev_rule, "{w}: groups in order");
                    prev_rule = at.rule_y;
                    for (i, (card, r)) in group.cards.iter().zip(&at.cards).enumerate() {
                        assert!(
                            r.x >= content.x && r.right() <= content.right() + 0.5,
                            "{w}: card in the content"
                        );
                        if let Some(next) = at.cards.get(i + 1)
                            && next.y == r.y
                        {
                            assert!(next.x >= r.right(), "{w}: cards overlap");
                        }
                        let b = ToggleGroupPage::widget_rect(&card.group, *r, &theme);
                        assert!(
                            b.x >= r.x && b.right() <= r.right() + 0.5,
                            "{w}: group in its card"
                        );
                        assert!(b.y >= r.y && b.bottom() <= r.bottom(), "{w}");
                        for (s, seg) in card.group.segment_rects(b, &theme).iter().enumerate() {
                            let shown = card.group.shown_label(s, *seg, &theme);
                            if shown != card.group.items[s] {
                                assert!(shown.ends_with('…'), "{w}: {shown}");
                            }
                        }
                    }
                }
            }
        }
    }

    /// The long labels truncate in a narrow content and draw whole in a
    /// wide one: the composition, not a breakpoint, decides.
    #[test]
    fn the_long_labels_truncate_only_when_the_card_is_narrow() {
        let (page, theme) = page();
        let long = &page.groups[2].cards[0].group;
        let shown_at = |w: f32| {
            let layout = page.layout(Rect::new(0.0, 0.0, w, 800.0), &theme);
            let b = ToggleGroupPage::widget_rect(long, layout[2].cards[0], &theme);
            let segs = long.segment_rects(b, &theme);
            long.shown_label(2, segs[2], &theme)
        };
        assert!(shown_at(280.0).ends_with('…'));
        assert_eq!(shown_at(1600.0), long.items[2]);
    }
}
