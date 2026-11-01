//! Toggle page, on the showcase mold of the kit's ButtonsPage
//! (`showcase`): groups with a title and a bottom rule, cards in rows, one
//! state per card behind a chip that names it. The widget is
//! `engine::widgets::selection::Toggle`, the kit's toolbar action
//! (`toolbar` off, `toolbar_active` on); every visual value comes from
//! the theme.
//!
//! The page shows the ranges the widget meets: the icon-only action, a
//! label, an icon with a label, and long and translated labels that
//! truncate inside their card. The columns of a row drop as the content
//! narrows, and a toggle never gets wider than its card.
//!
//! Projected: a group per title, `toggle_page.group[<g>]`, and in it each
//! card's toggle, `toggle_page.group[<g>].card[<c>]`, a `Role::Toggle`
//! with its `on` state, described by the card's chip. Its `toggle` flips
//! it as a click does.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::widgets::selection::Toggle;

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    toggle: Toggle,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    span: usize,
    cards: Vec<Card>,
}

pub struct TogglePage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The card, `(group, card)`, whose toggle a reader or the keyboard
    /// focused (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize)>,
}

/// `toggle_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `toggle_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// The four states of one form: off and on live, both disabled.
fn states(title: &'static str, make: impl Fn() -> Toggle) -> Group {
    let card = |title, toggle| Card { title, toggle };
    Group {
        title,
        span: 4,
        cards: vec![
            card("Off: Default, Hover, Press, Focus", make()),
            card("On: Default, Hover, Focus", make().on(true)),
            card("Disabled", make().disabled(true)),
            card("Disabled, on", make().on(true).disabled(true)),
        ],
    }
}

impl TogglePage {
    pub fn new() -> Self {
        let mut groups = vec![
            states("Toggle", || Toggle::icon_only("eye", "Preview")),
            states("Toggle with label", || Toggle::new("Preview")),
            states("Toggle with icon and label", || {
                Toggle::new("Preview").icon("eye")
            }),
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    Card {
                        title: "Long label",
                        toggle: Toggle::new("Show hidden files and folders in every view")
                            .icon("eye"),
                    },
                    Card {
                        title: "Translated label, on",
                        toggle: Toggle::new("Versteckte Dateien und Ordner anzeigen").on(true),
                    },
                ],
            },
        ];
        let root = root_id("toggle_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.toggle = card.toggle.clone().sem(id);
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

    /// The card whose toggle is `id`.
    fn at(&self, id: &SemId) -> Option<(usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            let c = group
                .cards
                .iter()
                .position(|c| c.toggle.sem_id() == Some(id))?;
            Some((g, c))
        })
    }

    fn toggle_mut(&mut self, (g, c): (usize, usize)) -> Option<&mut Toggle> {
        Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.toggle)
    }

    /// The focus on the toggle of one card, or on none; a disabled one
    /// refuses it.
    fn set_focus(&mut self, at: Option<(usize, usize)>) {
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                card.toggle.set_focused(at == Some((g, c)));
            }
        }
        self.focus = at.filter(|&(g, c)| {
            self.groups
                .get(g)
                .and_then(|group| group.cards.get(c))
                .is_some_and(|card| card.toggle.is_focused())
        });
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| (g.span, g.cards.len()));
        showcase::layout_measured(groups, content, theme, |g, c, w| {
            let toggle = &self.groups[g].cards[c].toggle;
            Self::widget_rect(toggle, Rect::new(0.0, 0.0, w, 0.0), theme).h
        })
    }

    /// The toggle's rect in its card: its intrinsic size, never wider
    /// than the card less its inset on both sides (the label truncates),
    /// never narrower than an ellipsis, centered.
    fn widget_rect(toggle: &Toggle, card: Rect, theme: &Theme) -> Rect {
        let (w, h) = toggle.preferred_size(theme);
        let room = showcase::room(card.w, theme);
        let w = w.min(room).max(toggle.min_width(theme));
        showcase::centered((w, h), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
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
                let bounds = Self::widget_rect(&card.toggle, *rect, theme);
                result = result.merge(card.toggle.handle_event(event, bounds));
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
                card.toggle
                    .render(c, Self::widget_rect(&card.toggle, *rect, theme), theme);
            }
            if i != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for TogglePage {
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
                if let Some(id) = card.toggle.sem_id() {
                    card.toggle
                        .project(b, id, Some(&at))
                        .description(card.title);
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c)| self.groups.get(g)?.cards.get(c))
            .and_then(|card| card.toggle.sem_id());
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A toggle's `toggle` flips it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let flipped = match (self.at(&req.target), req.action.as_str()) {
            (Some(at), "toggle") => self.toggle_mut(at).is_some_and(Toggle::toggle),
            _ => false,
        };
        if flipped {
            Ok(After::Nothing)
        } else {
            Err(format!(
                "the toggle page has no {} on {}",
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

    /// Every id parses, so the page projects.
    #[test]
    fn the_page_has_its_root_and_an_id_per_card() {
        let page = TogglePage::new();
        assert!(page.root.is_some());
        for group in &page.groups {
            assert!(group.cards.iter().all(|c| c.toggle.sem_id().is_some()));
        }
    }

    /// A click on a card's toggle flips it, and the same click again
    /// flips it back; a disabled one does not move.
    #[test]
    fn a_click_flips_the_toggle_of_its_card() {
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let mut page = TogglePage::new();
        let layout = page.layout(content, &theme);
        let press = |page: &mut TogglePage, g: usize, c: usize| {
            let card = layout[g].cards[c];
            let r = TogglePage::widget_rect(&page.groups[g].cards[c].toggle, card, &theme);
            let (x, y) = r.center();
            page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
            page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme)
        };
        assert!(press(&mut page, 0, 0).clicked);
        assert!(page.groups[0].cards[0].toggle.on);
        press(&mut page, 0, 0);
        assert!(!page.groups[0].cards[0].toggle.on);
        press(&mut page, 1, 2);
        assert!(!page.groups[1].cards[2].toggle.on, "disabled");
    }

    /// The focus a reader asks for lands on a live toggle and rings it; a
    /// disabled one refuses it, and blur lets it go.
    #[test]
    fn the_focus_rings_a_live_toggle_and_skips_a_disabled_one() {
        let mut page = TogglePage::new();
        let (live, disabled) = (page_id(&page, 0, 0), page_id(&page, 0, 2));
        assert!(page.focus(&live));
        assert_eq!(page.focus, Some((0, 0)));
        assert!(page.groups[0].cards[0].toggle.is_focused());
        assert!(page.focus(&disabled));
        assert_eq!(page.focus, None, "a disabled toggle takes no focus");
        assert!(!page.blur());
    }

    fn page_id(page: &TogglePage, g: usize, c: usize) -> SemId {
        page.groups[g].cards[c].toggle.sem_id().unwrap().clone()
    }

    /// The width sweep of the epic, 280 to 1600 every 8 px, under the
    /// core and the hoff themes: every card inside the content, the cards
    /// of a row side by side without overlap, every toggle inside its
    /// card, and a label that does not fit truncated, never cut.
    #[test]
    fn nothing_cuts_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = TogglePage::new();
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
                        assert!(r.bottom() <= at.rule_y, "{w}: card above its rule");
                        if let Some(next) = at.cards.get(i + 1)
                            && next.y == r.y
                        {
                            assert!(next.x >= r.right(), "{w}: cards overlap");
                        }
                        let t = TogglePage::widget_rect(&card.toggle, *r, &theme);
                        assert!(
                            t.x >= r.x && t.right() <= r.right() + 0.5,
                            "{w}: toggle in its card"
                        );
                        assert!(t.y >= r.y && t.bottom() <= r.bottom(), "{w}");
                        if !card.toggle.icon_only {
                            let shown = card.toggle.shown_label(t, &theme);
                            if card.toggle.preferred_size(&theme).0 <= t.w {
                                assert_eq!(shown, card.toggle.label, "{w}: whole when it fits");
                            } else if shown != card.toggle.label {
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
    fn the_long_label_truncates_only_when_its_card_is_narrow() {
        let theme = Theme::hoff_core();
        let page = TogglePage::new();
        let long = &page.groups[3].cards[0].toggle;
        let shown_at = |w: f32| {
            let layout = page.layout(Rect::new(0.0, 0.0, w, 800.0), &theme);
            let t = TogglePage::widget_rect(long, layout[3].cards[0], &theme);
            long.shown_label(t, &theme)
        };
        assert!(shown_at(280.0).ends_with('…'));
        assert_eq!(shown_at(1600.0), long.label);
    }
}
