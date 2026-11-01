//! Toggle Group page, on the showcase mold of the kit's ButtonsPage: the
//! kit's segmented control (`engine::widgets::selection::ToggleGroup`),
//! at both of its steps, disabled, and with long and translated labels
//! that truncate inside their segments. The thumb slides on the theme's
//! spring; every visual value comes from the theme.
//!
//! The page is a [`ChoicePage`], which the Radio Group page shares: cards
//! of one-of-many widgets ([`Choice`]), each projected as a `Role::Group`
//! of `Role::Radio` items, `<root>.group[<g>].card[<c>][<i>]`, described
//! by the card's chip. An item's `select` chooses it as a click does.

use accord::{NodeBuilder, Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::widgets::selection::{SegmentSize, ToggleGroup};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, key_under, root_id, under};

/// A one-of-many widget as the choice pages show it.
pub trait Choice: Clone {
    fn count(&self) -> usize;
    #[cfg(test)]
    fn label(&self, i: usize) -> &str;
    #[cfg(test)]
    fn chosen(&self) -> usize;
    /// Choose `i`, as a click does; whether the choice moved.
    fn choose(&mut self, i: usize) -> bool;
    fn set_focus(&mut self, i: Option<usize>);
    fn focus_at(&self) -> Option<usize>;
    fn id(&self) -> Option<&SemId>;
    fn with_id(self, id: SemId) -> Self;
    /// Its size given `room` across.
    fn size_in(&self, room: f32, theme: &Theme) -> (f32, f32);
    #[cfg(test)]
    fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect>;
    #[cfg(test)]
    fn shown(&self, i: usize, item: Rect, theme: &Theme) -> String;
    fn pointer(&mut self, event: &WidgetEvent, bounds: Rect, theme: &Theme) -> EventResult;
    fn draw(&self, c: &mut Compositor, bounds: Rect, theme: &Theme);
    fn declare<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        name: &str,
        parent: &SemId,
    ) -> NodeBuilder<'b>;
    /// Advance a motion; `true` while it still moves.
    fn advance(&mut self, _dt: f32) -> bool {
        false
    }
}

impl Choice for ToggleGroup {
    fn count(&self) -> usize {
        self.items.len()
    }
    #[cfg(test)]
    fn label(&self, i: usize) -> &str {
        &self.items[i]
    }
    #[cfg(test)]
    fn chosen(&self) -> usize {
        self.selected
    }
    fn choose(&mut self, i: usize) -> bool {
        self.select(i)
    }
    fn set_focus(&mut self, i: Option<usize>) {
        self.set_focused(i);
    }
    fn focus_at(&self) -> Option<usize> {
        self.focused()
    }
    fn id(&self) -> Option<&SemId> {
        self.sem_id()
    }
    fn with_id(self, id: SemId) -> Self {
        self.sem(id)
    }
    /// Its intrinsic width, never wider than `room` (the labels
    /// truncate), never narrower than its ellipses.
    fn size_in(&self, room: f32, theme: &Theme) -> (f32, f32) {
        let (w, h) = self.preferred_size(theme);
        (w.min(room).max(self.min_width(theme)), h)
    }
    #[cfg(test)]
    fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        self.segment_rects(bounds, theme)
    }
    #[cfg(test)]
    fn shown(&self, i: usize, item: Rect, theme: &Theme) -> String {
        self.shown_label(i, item, theme)
    }
    fn pointer(&mut self, event: &WidgetEvent, bounds: Rect, theme: &Theme) -> EventResult {
        self.handle_event(event, bounds, theme)
    }
    fn draw(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render(c, bounds, theme);
    }
    fn declare<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        name: &str,
        parent: &SemId,
    ) -> NodeBuilder<'b> {
        self.project(b, id, name, Some(parent))
    }
    fn advance(&mut self, dt: f32) -> bool {
        self.tick(dt)
    }
}

pub struct Card<W> {
    /// The chip: which states this card shows.
    pub title: &'static str,
    /// The widget's accessible name.
    pub name: &'static str,
    pub widget: W,
}

pub struct Group<W> {
    pub title: &'static str,
    /// Cards per row at full width.
    pub span: usize,
    pub cards: Vec<Card<W>>,
}

/// A page of choice widgets on the showcase mold.
pub struct ChoicePage<W> {
    root: Option<SemId>,
    groups: Vec<Group<W>>,
    /// The item, `(group, card, item)`, a reader or the keyboard focused
    /// (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize, usize)>,
}

pub type ToggleGroupPage = ChoicePage<ToggleGroup>;

/// `<root>.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `<root>.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl<W: Choice> ChoicePage<W> {
    /// The page `name` over `groups`, each widget given its card's id.
    pub fn with(name: &str, mut groups: Vec<Group<W>>) -> Self {
        let root = root_id(name);
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.widget = card.widget.clone().with_id(id);
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

    fn cards(&self) -> impl Iterator<Item = ((usize, usize), &Card<W>)> {
        self.groups.iter().enumerate().flat_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .map(move |(c, card)| ((g, c), card))
        })
    }

    /// The item behind `id`: `(group, card, item)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.cards().find_map(|((g, c), card)| {
            let parent = card.widget.id()?;
            let i = key_under(parent, id)?.parse::<usize>().ok()?;
            (i < card.widget.count()).then_some((g, c, i))
        })
    }

    fn widget_mut(&mut self, g: usize, c: usize) -> Option<&mut W> {
        Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.widget)
    }

    /// The focus on one item, or on none; a disabled widget refuses it.
    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let item = at.filter(|a| (a.0, a.1) == (g, c)).map(|a| a.2);
                card.widget.set_focus(item);
            }
        }
        self.focus = at.filter(|&(g, c, i)| {
            self.groups
                .get(g)
                .and_then(|group| group.cards.get(c))
                .is_some_and(|card| card.widget.focus_at() == Some(i))
        });
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| (g.span, g.cards.len()));
        showcase::layout(groups, content, theme)
    }

    /// The widget's rect in its card: its size within the card less its
    /// inset on both sides, centered.
    fn widget_rect(widget: &W, card: Rect, theme: &Theme) -> Rect {
        let room = card.w - theme.spacing.sm * 2.0;
        showcase::centered(widget.size_in(room, theme), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// Advance the widgets' motion; `true` while one still moves.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for group in &mut self.groups {
            for card in &mut group.cards {
                moving |= card.widget.advance(dt);
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
                let bounds = Self::widget_rect(&card.widget, *rect, theme);
                result = result.merge(card.widget.pointer(event, bounds, theme));
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
                card.widget
                    .draw(c, Self::widget_rect(&card.widget, *rect, theme), theme);
            }
            if i != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl<W: Choice> Component for ChoicePage<W> {
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
                if let Some(id) = card.widget.id() {
                    card.widget
                        .declare(b, id, card.name, &at)
                        .description(card.title);
                }
            }
        }
        let focused = self.focus.and_then(|(g, c, i)| {
            let card = self.groups.get(g)?.cards.get(c)?;
            Some(card.widget.id()?.key(i.to_string()))
        });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// An item's `select` chooses it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let chose = match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, i)), "select") => self.widget_mut(g, c).is_some_and(|w| w.choose(i)),
            _ => false,
        };
        if chose {
            Ok(After::Nothing)
        } else {
            Err(format!("the page has no {} on {}", req.action, req.target))
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

const VIEWS: [&str; 3] = ["Day", "Week", "Month"];

/// A step of the kit: live, and disabled.
fn step(title: &'static str, size: SegmentSize, theme: &Theme) -> Group<ToggleGroup> {
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
                widget: make(),
            },
            Card {
                title: "Disabled",
                name: "view",
                widget: make().disabled(true),
            },
        ],
    }
}

impl ChoicePage<ToggleGroup> {
    pub fn new(theme: &Theme) -> Self {
        let groups = vec![
            step("Toggle group", SegmentSize::Md, theme),
            step("Toggle group, small", SegmentSize::Sm, theme),
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    Card {
                        title: "Long labels",
                        name: "show",
                        widget: ToggleGroup::new([
                            "Everything",
                            "Only the active ones",
                            "Completed and archived",
                        ])
                        .with_motion(&theme.motion),
                    },
                    Card {
                        title: "Translated labels",
                        name: "Ansicht",
                        widget: ToggleGroup::new(["Tag", "Woche", "Monat", "Jahr"])
                            .selected(2)
                            .with_motion(&theme.motion),
                    },
                ],
            },
        ];
        Self::with("toggle_group_page", groups)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// The checks every choice page passes: every id parses; a click on
    /// the last item of card `(0, 0)` chooses it and the disabled card
    /// `(0, 1)` does not move; the focus lands on a live item and not on
    /// a disabled one; and the width sweep of the epic, 280 to 1600 every
    /// 8 px under the core and the hoff themes: every card in the
    /// content, the cards of a row apart, every widget inside its card and
    /// under its chip, every item inside its widget, every label whole or
    /// truncated, never cut.
    pub(in crate::views) fn check_choice_page<W: Choice>(make: impl Fn(&Theme) -> ChoicePage<W>) {
        let theme = Theme::hoff_core();
        let mut page = make(&theme);
        assert!(page.root.is_some());
        assert!(page.cards().all(|(_, c)| c.widget.id().is_some()));

        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let layout = page.layout(content, &theme);
        let press = |page: &mut ChoicePage<W>, c: usize, i: usize| {
            let widget = &page.groups[0].cards[c].widget;
            let bounds = ChoicePage::widget_rect(widget, layout[0].cards[c], &theme);
            let (x, y) = widget.item_rects(bounds, &theme)[i].center();
            page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
            page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme)
        };
        let last = page.groups[0].cards[0].widget.count() - 1;
        let before = page.groups[0].cards[1].widget.chosen();
        assert!(press(&mut page, 0, last).clicked);
        assert_eq!(page.groups[0].cards[0].widget.chosen(), last);
        press(&mut page, 1, (before + 1) % (last + 1));
        assert_eq!(page.groups[0].cards[1].widget.chosen(), before, "disabled");

        let item =
            |page: &ChoicePage<W>, c: usize| page.groups[0].cards[c].widget.id().unwrap().key("0");
        let (live, off) = (item(&page, 0), item(&page, 1));
        assert!(page.focus(&live));
        assert_eq!(page.focus, Some((0, 0, 0)));
        assert!(page.focus(&off));
        assert_eq!(page.focus, None, "a disabled widget takes no focus");

        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = make(&theme);
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
                        let inset = theme.spacing.sm;
                        let chip =
                            showcase::chip_rect(r.x + inset, r.y + inset, card.title, &theme);
                        let b = ChoicePage::widget_rect(&card.widget, *r, &theme);
                        assert!(
                            b.x >= r.x && b.right() <= r.right() + 0.5,
                            "{w}: in its card"
                        );
                        assert!(b.y >= chip.bottom() && b.bottom() <= r.bottom(), "{w}");
                        for (i, it) in card.widget.item_rects(b, &theme).iter().enumerate() {
                            assert!(it.x >= b.x - 0.5 && it.right() <= b.right() + 0.5, "{w}");
                            let shown = card.widget.shown(i, *it, &theme);
                            if shown != card.widget.label(i) {
                                assert!(shown.ends_with('…'), "{w}: {shown}");
                            }
                        }
                    }
                }
            }
        }
    }

    /// The label `i` of card `(g, c)` truncates in a narrow content and
    /// draws whole in a wide one: the composition, not a breakpoint,
    /// decides.
    pub(in crate::views) fn check_truncates_only_when_narrow<W: Choice>(
        page: &ChoicePage<W>,
        (g, c, i): (usize, usize, usize),
    ) {
        let theme = Theme::hoff_core();
        let widget = &page.groups[g].cards[c].widget;
        let shown_at = |w: f32| {
            let layout = page.layout(Rect::new(0.0, 0.0, w, 800.0), &theme);
            let b = ChoicePage::widget_rect(widget, layout[g].cards[c], &theme);
            widget.shown(i, widget.item_rects(b, &theme)[i], &theme)
        };
        assert!(shown_at(280.0).ends_with('…'));
        assert_eq!(shown_at(1600.0), widget.label(i));
    }

    #[test]
    fn the_toggle_group_page_passes_the_choice_checks() {
        check_choice_page(ToggleGroupPage::new);
    }

    #[test]
    fn the_thumb_slides_after_a_choice() {
        let theme = Theme::hoff_core();
        let mut page = ToggleGroupPage::new(&theme);
        assert!(page.widget_mut(0, 0).unwrap().choose(0));
        assert!(page.tick(1.0 / 60.0), "the thumb slides");
    }

    #[test]
    fn the_long_labels_truncate_only_when_the_card_is_narrow() {
        let theme = Theme::hoff_core();
        check_truncates_only_when_narrow(&ToggleGroupPage::new(&theme), (2, 0, 2));
    }
}
