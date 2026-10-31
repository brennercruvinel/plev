//! Pagination page: page numbers in cells between two arrows, derived
//! from the kit's language, in the showcase mold. Each card holds a pager
//! as wide as the card: a wide one shows every page or the ends and the
//! pages around the current one, a narrow one the arrows and the current
//! page. The widget is `engine::widgets::content::Pagination`; every
//! visual value comes from the theme (`theme.disclosure.pager`).
//!
//! Projected: a group per title, `pagination_page.group[<g>]`; in it each
//! card's pager, `...card[<c>]`, a group named by its chip and described
//! by "Page k of N"; in that the arrows (`...previous`, `...next`) and the
//! pages it shows (`...page[<k>]`), buttons whose `activate` goes there
//! as a click does.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{PageSlot, Pagination};
use engine::widgets::{EventResult, Rect, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, root_id, showcase, under};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    pager: Pagination,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct PaginationPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The cell, `(group, card, slot)`, a reader or the keyboard focused.
    focus: Option<(usize, usize, PageSlot)>,
}

/// `pagination_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `pagination_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl PaginationPage {
    pub fn new() -> Self {
        let card = |title, pager| Card { title, pager };
        let mut groups = vec![
            Group {
                title: "Pagination",
                columns: 1,
                cards: vec![
                    card("Default, Hover, Focus: ten pages", Pagination::new(10)),
                    card(
                        "Current in the middle: forty pages",
                        Pagination::new(40).current(19),
                    ),
                ],
            },
            Group {
                title: "Pagination, ranges",
                columns: 3,
                cards: vec![
                    card("Narrow", Pagination::new(40).current(7)),
                    card("Three pages, the last", Pagination::new(3).current(2)),
                    card("One page", Pagination::new(1)),
                ],
            },
        ];
        let root = root_id("pagination_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.pager = std::mem::take(&mut card.pager).sem(id);
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

    /// The cell behind `id`: `(group, card, slot)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, PageSlot)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .find_map(|(c, card)| card.pager.slot_of(id).map(|s| (g, c, s)))
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, PageSlot)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let slot = at
                    .filter(|(fg, fc, _)| (*fg, *fc) == (g, c))
                    .map(|(_, _, s)| s);
                card.pager.set_focused(slot);
            }
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.columns, g.cards.len()))
            .collect();
        fit_layout(&spans, content, theme, |g, c, _| {
            self.groups[g].cards[c].pager.height(theme)
        })
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
            for (card, (_, body)) in group.cards.iter_mut().zip(&placed.cards) {
                result = result.merge(card.pager.handle_event(event, *body, theme));
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
            for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                showcase::card(c, *rect, card.title, theme);
                card.pager.render(c, *body, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for PaginationPage {
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
                if let Some(id) = card.pager.sem_id() {
                    card.pager.project(b, id, card.title, Some(&at));
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c, s)| self.groups.get(g)?.cards.get(c)?.pager.slot_id(s));
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// A cell's `activate` goes to its page, or one page on or back, as
    /// its click does; an arrow with nowhere to go refuses.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let done = match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, slot)), "activate") => {
                let pager = &mut self.groups[g].cards[c].pager;
                match slot {
                    PageSlot::Previous => pager.back(),
                    PageSlot::Next => pager.forward(),
                    PageSlot::Page(k) => pager.go(k) || pager.current_page() == k,
                    PageSlot::Gap => false,
                }
            }
            _ => {
                return Err(format!(
                    "the pagination page has no {} on {}",
                    req.action, req.target
                ));
            }
        };
        if done {
            Ok(After::Nothing)
        } else {
            Err(format!("{} has nowhere to go", req.target))
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

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// pager's cells inside its card and apart, the current page always
    /// among them.
    #[test]
    fn every_pager_fits_its_card_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = PaginationPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                        let what = format!("{w}px, {}", card.title);
                        let cells = card.pager.layout(body.x, body.y, body.w, &theme);
                        let current = PageSlot::Page(card.pager.current_page());
                        assert!(cells.iter().any(|(s, _)| *s == current), "{what}: current");
                        for (n, (_, at)) in cells.iter().enumerate() {
                            assert!(
                                at.x >= rect.x && at.right() <= rect.right() + 0.01,
                                "{what}"
                            );
                            for (_, other) in &cells[n + 1..] {
                                assert!(at.right() <= other.x, "{what}: cells overlap");
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_click_on_a_cell_goes_there() {
        let theme = Theme::hoff_core();
        let mut page = PaginationPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let (_, body) = page.layout(content, &theme)[0].cards[0];
        let cells = page.groups[0].cards[0]
            .pager
            .layout(body.x, body.y, body.w, &theme);
        let (_, at) = cells.iter().find(|(s, _)| *s == PageSlot::Page(4)).unwrap();
        let (x, y) = at.center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(
            page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme)
                .clicked
        );
        assert_eq!(page.groups[0].cards[0].pager.current_page(), 4);
    }
}
