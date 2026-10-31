//! Button Group page, on the showcase mold of the kit's ButtonsPage: the
//! kit's toolbar shell (`engine::widgets::selection::ButtonGroup`) with
//! its tool picker (exclusive toggles), actions, a disabled group, free
//! toggles and long labels that truncate and wrap. Every visual value
//! comes from the theme.
//!
//! Projected: a group per title, `button_group_page.group[<g>]`, and in it
//! each card's group, `button_group_page.group[<g>].card[<c>]`, a
//! `Role::Toolbar` with its items, `...card[<c>].item[<i>]`, as the items
//! project themselves (a toggle, a button), described by the card's chip.
//! A toggle's `toggle` presses it as a click does (in the tool picker it
//! turns on and the others off); an action's `activate` presses and
//! changes nothing.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, IconButton, IconButtonVariant, Rect, WidgetEvent};
use engine::widgets::selection::{ButtonGroup, GroupItem, Toggle};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    /// The group's accessible name.
    name: &'static str,
    group: ButtonGroup,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    span: usize,
    cards: Vec<Card>,
}

pub struct ButtonGroupPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The item, `(group, card, item)`, a reader or the keyboard focused
    /// (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize, usize)>,
}

/// `button_group_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `button_group_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn tool(label: &str, icon: &'static str) -> GroupItem {
    GroupItem::Toggle(Toggle::icon_only(icon, label))
}

fn action(label: &str, icon: &'static str) -> GroupItem {
    GroupItem::Action {
        button: IconButton::new(icon).variant(IconButtonVariant::Toolbar),
        label: label.into(),
    }
}

/// The kit's toolbar: a tool picker, then undo (disabled, as in the kit)
/// and redo.
fn toolbar(disabled: bool) -> ButtonGroup {
    let mut items = vec![
        tool("Select", "square"),
        tool("Search", "search"),
        tool("Comment", "send"),
        GroupItem::Separator,
        GroupItem::Action {
            button: IconButton::new("undo")
                .variant(IconButtonVariant::Toolbar)
                .disabled(true),
            label: "Undo".into(),
        },
        action("Redo", "redo"),
    ];
    if let Some(GroupItem::Toggle(t)) = items.first_mut() {
        t.on = true;
    }
    if disabled {
        for item in &mut items {
            match item {
                GroupItem::Toggle(t) => t.disabled = true,
                GroupItem::Action { button, .. } => button.disabled = true,
                GroupItem::Separator => {}
            }
        }
    }
    ButtonGroup::new(items).exclusive(true)
}

impl ButtonGroupPage {
    pub fn new() -> Self {
        let free = |labels: &[&str]| {
            ButtonGroup::new(
                labels
                    .iter()
                    .map(|l| GroupItem::Toggle(Toggle::new(*l)))
                    .collect::<Vec<_>>(),
            )
        };
        let mut groups = vec![
            Group {
                title: "Toolbar",
                span: 2,
                cards: vec![
                    Card {
                        title: "Default, Hover, Press, Focus, Active",
                        name: "tools",
                        group: toolbar(false),
                    },
                    Card {
                        title: "Disabled",
                        name: "tools",
                        group: toolbar(true),
                    },
                ],
            },
            Group {
                title: "Toggles",
                span: 2,
                cards: vec![
                    Card {
                        title: "Each on its own",
                        name: "view",
                        group: free(&["Grid", "Snap", "Rulers"]),
                    },
                    Card {
                        title: "Long labels, wrapping",
                        name: "show",
                        group: free(&["Hidden files", "Comments from everyone"]),
                    },
                ],
            },
        ];
        let root = root_id("button_group_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(id) = card_id(root, g, c) else {
                        continue;
                    };
                    let family = child_id(Some(&id), "item");
                    for (i, item) in card.group.items.iter_mut().enumerate() {
                        let Some(at) = family.as_ref().map(|f| f.key(i.to_string())) else {
                            continue;
                        };
                        match item {
                            GroupItem::Toggle(t) => *t = t.clone().sem(at),
                            GroupItem::Action { button, .. } => *button = button.clone().sem(at),
                            GroupItem::Separator => {}
                        }
                    }
                    card.group = card.group.clone().sem(id);
                }
            }
        }
        Self {
            root,
            groups,
            focus: None,
        }
    }

    fn item_id(item: &GroupItem) -> Option<&SemId> {
        match item {
            GroupItem::Toggle(t) => t.sem_id(),
            GroupItem::Action { button, .. } => button.sem_id(),
            GroupItem::Separator => None,
        }
    }

    /// The item behind `id`: `(group, card, item)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let i = card
                    .group
                    .items
                    .iter()
                    .position(|item| Self::item_id(item) == Some(id))?;
                Some((g, c, i))
            })
        })
    }

    /// The focus on one item, or on none; a disabled one refuses it.
    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        let mut took = false;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                for (i, item) in card.group.items.iter_mut().enumerate() {
                    let on = at == Some((g, c, i));
                    match item {
                        GroupItem::Toggle(t) => {
                            t.set_focused(on);
                            took |= on && t.is_focused();
                        }
                        GroupItem::Action { button, .. } => {
                            button.set_focused(on);
                            took |= on && button.is_focused();
                        }
                        GroupItem::Separator => {}
                    }
                }
            }
        }
        self.focus = at.filter(|_| took);
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| (g.span, g.cards.len()));
        showcase::layout(groups, content, theme)
    }

    /// The group's rect in its card: as wide as it needs within the card
    /// less its inset, wrapping below that, centered.
    fn widget_rect(group: &ButtonGroup, card: Rect, theme: &Theme) -> Rect {
        let room = card.w - theme.spacing.sm * 2.0;
        showcase::centered(group.size_within(room, theme), card)
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

impl Component for ButtonGroupPage {
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
            Self::item_id(self.groups.get(g)?.cards.get(c)?.group.items.get(i)?)
        });
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A toggle's `toggle` presses it, as its click does; an action's
    /// `activate` presses and changes nothing.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let at = self.at(&req.target);
        let group =
            at.and_then(|(g, c, _)| Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.group));
        match (group, at, req.action.as_str()) {
            (Some(group), Some((_, _, i)), "toggle")
                if matches!(group.items.get(i), Some(GroupItem::Toggle(_))) =>
            {
                group.press(i);
                Ok(After::Nothing)
            }
            (Some(group), Some((_, _, i)), "activate")
                if matches!(group.items.get(i), Some(GroupItem::Action { .. })) =>
            {
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the button group page has no {} on {}",
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

    fn on(page: &ButtonGroupPage, g: usize, c: usize) -> Vec<bool> {
        page.groups[g].cards[c]
            .group
            .items
            .iter()
            .filter_map(|i| match i {
                GroupItem::Toggle(t) => Some(t.on),
                _ => None,
            })
            .collect()
    }

    /// Every id parses, so the page projects.
    #[test]
    fn the_page_has_its_root_and_an_id_per_item() {
        let page = ButtonGroupPage::new();
        assert!(page.root.is_some());
        for group in &page.groups {
            for card in &group.cards {
                assert!(card.group.sem_id().is_some());
                for item in &card.group.items {
                    let separator = matches!(item, GroupItem::Separator);
                    assert_eq!(ButtonGroupPage::item_id(item).is_some(), !separator);
                }
            }
        }
    }

    /// A click on a tool turns it on and the others off; a click in the
    /// disabled toolbar changes nothing.
    #[test]
    fn a_click_picks_the_tool() {
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let mut page = ButtonGroupPage::new();
        let layout = page.layout(content, &theme);
        let press = |page: &mut ButtonGroupPage, g: usize, c: usize, i: usize| {
            let group = &page.groups[g].cards[c].group;
            let bounds = ButtonGroupPage::widget_rect(group, layout[g].cards[c], &theme);
            let (x, y) = group.item_rects(bounds, &theme)[i].unwrap().center();
            page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
            page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme)
        };
        assert!(press(&mut page, 0, 0, 2).clicked);
        assert_eq!(on(&page, 0, 0), [false, false, true]);
        press(&mut page, 0, 1, 1);
        assert_eq!(on(&page, 0, 1), [true, false, false], "disabled");
        press(&mut page, 1, 0, 1);
        assert_eq!(on(&page, 1, 0), [false, true, false], "free toggles");
    }

    /// The width sweep of the epic, 280 to 1600 every 8 px, under the
    /// core and the hoff themes: every card in the content, the cards of
    /// a row apart, every group inside its card and under its chip, its
    /// items inside it and apart.
    #[test]
    fn nothing_cuts_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = ButtonGroupPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (i, (card, r)) in group.cards.iter().zip(&at.cards).enumerate() {
                        assert!(r.x >= content.x && r.right() <= content.right() + 0.5);
                        if let Some(next) = at.cards.get(i + 1)
                            && next.y == r.y
                        {
                            assert!(next.x >= r.right(), "{w}: cards overlap");
                        }
                        let inset = theme.spacing.sm;
                        let chip =
                            showcase::chip_rect(r.x + inset, r.y + inset, card.title, &theme);
                        let b = ButtonGroupPage::widget_rect(&card.group, *r, &theme);
                        assert!(
                            b.x >= r.x && b.right() <= r.right() + 0.5,
                            "{w}: in its card"
                        );
                        assert!(b.y >= chip.bottom() && b.bottom() <= r.bottom(), "{w}");
                        let items: Vec<Rect> = card
                            .group
                            .item_rects(b, &theme)
                            .into_iter()
                            .flatten()
                            .collect();
                        for (k, a) in items.iter().enumerate() {
                            assert!(a.x >= b.x && a.right() <= b.right() + 0.5, "{w}");
                            assert!(a.y >= b.y && a.bottom() <= b.bottom() + 0.5, "{w}");
                            for o in &items[k + 1..] {
                                let apart = a.right() <= o.x
                                    || o.right() <= a.x
                                    || a.bottom() <= o.y
                                    || o.bottom() <= a.y;
                                assert!(apart, "{w}: {a:?} {o:?}");
                            }
                        }
                    }
                }
            }
        }
    }

    /// The long labels sit in one row in a wide card and wrap in a narrow
    /// one: the composition, not a breakpoint, decides.
    #[test]
    fn the_long_labels_wrap_only_when_the_card_is_narrow() {
        let theme = Theme::hoff_core();
        let page = ButtonGroupPage::new();
        let long = &page.groups[1].cards[1].group;
        let height_at = |w: f32| {
            let layout = page.layout(Rect::new(0.0, 0.0, w, 800.0), &theme);
            ButtonGroupPage::widget_rect(long, layout[1].cards[1], &theme).h
        };
        assert!(height_at(280.0) > height_at(1600.0));
        assert_eq!(height_at(1600.0), long.preferred_size(&theme).1);
    }
}
