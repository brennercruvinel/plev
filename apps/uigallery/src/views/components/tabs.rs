//! Tabs page, on the showcase mold of the kit's ButtonsPage, beside the
//! Toggle Group it shares a drawing with: the kit's tab strip
//! (`components/Tabs`, the well with its `#E2E2E2` border and the raised
//! thumb), with three and with two tabs, and with long and translated
//! labels that truncate inside their segments. Every visual value comes
//! from the theme (`theme.selection.segmented`).
//!
//! Projected (task-75 phase 2b): a group per title,
//! `tabs_page.group[<g>]`, and in it each card's strip, a tab list named
//! by the card and described by its chip, with a tab per label; a tab's
//! `select` makes it the active one as a press does. The first strip keeps
//! the id it always had, `tabs_page.tabs`; the others are
//! `tabs_page.strip[<g>-<c>]`. The focus is on the tab a screen reader or
//! the keyboard moved it to (its ring, task-75 phase 2c), which the arrow
//! keys move along its strip.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, Tabs, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, key_under, root_id, under};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    /// The strip's accessible name.
    name: &'static str,
    tabs: Tabs,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    span: usize,
    cards: Vec<Card>,
}

pub struct TabsPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The tab, `(group, card, tab)`, a reader or the keyboard focused.
    focus: Option<(usize, usize, usize)>,
}

/// `tabs_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// The strip of card `(g, c)`: `tabs_page.tabs` for the first, as it
/// always was, `tabs_page.strip[<g>-<c>]` for the others.
fn strip_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    if (g, c) == (0, 0) {
        child_id(Some(root), "tabs")
    } else {
        child_id(Some(root), "strip").map(|family| family.key(format!("{g}-{c}")))
    }
}

fn card(title: &'static str, name: &'static str, labels: &[&str], active: usize) -> Card {
    let mut tabs = Tabs::new(labels.iter().copied());
    tabs.active = active;
    Card { title, name, tabs }
}

impl TabsPage {
    pub fn new() -> Self {
        let mut groups = vec![
            Group {
                title: "Tabs",
                span: 2,
                cards: vec![
                    card(
                        "Default, Hover, Press, Focus",
                        "sections",
                        &["Account", "Appearance", "About"],
                        0,
                    ),
                    card("Two tabs", "view", &["Preview", "Code"], 1),
                ],
            },
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    card(
                        "Long labels",
                        "show",
                        &[
                            "Everything",
                            "Only the active ones",
                            "Completed and archived",
                        ],
                        0,
                    ),
                    card(
                        "Translated labels",
                        "Bereiche",
                        &["Konto", "Darstellung", "Über"],
                        1,
                    ),
                ],
            },
        ];
        let root = root_id("tabs_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = strip_id(root, g, c) {
                        card.tabs = card.tabs.clone().sem(id);
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

    /// The tab behind `id`: `(group, card, tab)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.cards().find_map(|((g, c), card)| {
            let key = key_under(card.tabs.sem_id()?, id)?;
            let i = key.parse::<usize>().ok()?;
            (i < card.tabs.labels.len()).then_some((g, c, i))
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let tab = at.filter(|a| (a.0, a.1) == (g, c)).map(|a| a.2);
                card.tabs.set_focused_tab(tab);
            }
        }
        self.focus = at;
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| (g.span, g.cards.len()));
        showcase::layout_measured(groups, content, theme, |g, c, _| {
            self.groups[g].cards[c].tabs.preferred_size(theme).1
        })
    }

    /// The strip's rect in its card: its own width within the card less
    /// its inset on both sides (the labels truncate), never narrower than
    /// its ellipses, centered.
    fn strip_rect(tabs: &Tabs, card: Rect, theme: &Theme) -> Rect {
        let room = showcase::room(card.w, theme);
        let (w, h) = tabs.preferred_size(theme);
        showcase::centered((w.min(room).max(tabs.min_width(theme)), h), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    pub fn handle_escape(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.handle_escape() {
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (group, at) in self.groups.iter_mut().zip(&layout) {
            for (card, rect) in group.cards.iter_mut().zip(&at.cards) {
                let bounds = Self::strip_rect(&card.tabs, *rect, theme);
                result = result.merge(card.tabs.handle_event(event, bounds, theme));
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
                card.tabs
                    .render(c, Self::strip_rect(&card.tabs, *rect, theme), theme);
            }
            if i != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for TabsPage {
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
                if let Some(id) = card.tabs.sem_id() {
                    card.tabs
                        .project(b, id, card.name, Some(&at))
                        .description(card.title);
                }
            }
        }
        let focused = self.focus.and_then(|(g, c, i)| {
            let card = self.groups.get(g)?.cards.get(c)?;
            Some(card.tabs.sem_id()?.key(i.to_string()))
        });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// A tab's `select` makes it the active one, as its press does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, i)), "select") => {
                self.groups[g].cards[c].tabs.active = i;
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the tabs page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// A tab takes the focus (its ring); another node of the page takes
    /// it from the strips.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.at(id));
        true
    }

    fn blur(&mut self) -> bool {
        self.handle_escape()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The width sweep of the epic, 280 to 1600 every 8 px under the core
    /// and the hoff themes: every card in the content, the cards of a row
    /// apart, every strip inside its card and under its chip, every tab
    /// inside its strip, every label whole or truncated, never cut.
    #[test]
    fn every_strip_fits_its_card_from_narrow_to_wide() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = TabsPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let mut prev_rule = content.y;
                for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
                    assert!(at.title_y >= prev_rule, "{w}: groups in order");
                    prev_rule = at.rule_y;
                    for (n, (card, r)) in group.cards.iter().zip(&at.cards).enumerate() {
                        assert!(r.x >= content.x && r.right() <= content.right() + 0.5);
                        if let Some(next) = at.cards.get(n + 1)
                            && next.y == r.y
                        {
                            assert!(next.x >= r.right(), "{w}: cards overlap");
                        }
                        let inset = showcase::inset(&theme);
                        let chip =
                            showcase::chip_rect(r.x + inset, r.y + inset, card.title, &theme);
                        let b = TabsPage::strip_rect(&card.tabs, *r, &theme);
                        assert!(b.x >= r.x && b.right() <= r.right() + 0.5, "{w}");
                        assert!(b.y >= chip.bottom() && b.bottom() <= r.bottom(), "{w}");
                        for (i, it) in card.tabs.item_rects(b, &theme).iter().enumerate() {
                            assert!(it.x >= b.x - 0.5 && it.right() <= b.right() + 0.5);
                            let shown = card.tabs.shown_label(i, *it, &theme);
                            if shown != card.tabs.labels[i] {
                                assert!(shown.ends_with('…'), "{w}: {shown}");
                            }
                        }
                    }
                }
            }
        }
    }

    /// The long label truncates in a narrow content and draws whole in a
    /// wide one: the composition, not a breakpoint, decides.
    #[test]
    fn a_long_label_truncates_only_when_the_card_is_narrow() {
        let theme = Theme::hoff_core();
        let page = TabsPage::new();
        let tabs = &page.groups[1].cards[0].tabs;
        let shown_at = |w: f32| {
            let layout = page.layout(Rect::new(0.0, 0.0, w, 800.0), &theme);
            let b = TabsPage::strip_rect(tabs, layout[1].cards[0], &theme);
            tabs.shown_label(2, tabs.item_rects(b, &theme)[2], &theme)
        };
        assert!(shown_at(280.0).ends_with('…'));
        assert_eq!(shown_at(1600.0), tabs.labels[2]);
    }

    /// A press on a tab of the second strip switches it and leaves the
    /// first alone; the focus lands on a tab and Escape lets it go.
    #[test]
    fn a_press_switches_its_own_strip_and_the_focus_rings_one_tab() {
        let theme = Theme::hoff_core();
        let mut page = TabsPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let card = page.layout(content, &theme)[0].cards[1];
        let tabs = &page.groups[0].cards[1].tabs;
        let b = TabsPage::strip_rect(tabs, card, &theme);
        let (x, y) = tabs.item_rects(b, &theme)[0].center();
        assert!(
            page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme)
                .clicked
        );
        assert_eq!(page.groups[0].cards[1].tabs.active, 0);
        assert_eq!(page.groups[0].cards[0].tabs.active, 0, "the first strip");

        let id = page.groups[1].cards[1].tabs.sem_id().unwrap().key("2");
        assert!(page.focus(&id));
        assert_eq!(page.focus, Some((1, 1, 2)));
        assert_eq!(page.groups[1].cards[1].tabs.focused_tab(), Some(2));
        assert!(page.handle_escape());
        assert_eq!(page.groups[1].cards[1].tabs.focused_tab(), None);
    }
}
