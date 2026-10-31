//! Buttons page, the hoff core kit's ButtonsPage: groups with a title and
//! a bottom rule, cards in rows, one state per card behind a chip that
//! names it. The widgets are `engine::widgets::buttons` (labeled, icon-only,
//! submit) and, last, the page's Tooltip bubble at rest; every visual value
//! comes from the theme.
//!
//! Projected (task-75 phase 2b): a group per title,
//! `button_page.group[<g>]`, and in it each card's widget,
//! `button_page.group[<g>].card[<c>]`, as the comps projects it, described
//! by the card's chip (the states it shows); an icon-only button is named
//! by its icon, the submit button "submit", the tooltip a label named by
//! its text. A button here does nothing but press, so its `activate`
//! changes nothing. The Icon Button page is this
//! page with the icon-only groups alone ([`ButtonPage::icons`]).

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::{Theme, hoff_core};
use engine::widgets::prelude::{
    Button, ButtonSize, ButtonVariant, EventResult, IconButton, IconButtonSize, IconButtonVariant,
    Rect, SubmitButton, Tooltip, WidgetEvent,
};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, root_id, under};

enum Spec {
    Button(Button),
    Icon(IconButton),
    Submit(SubmitButton),
    /// The bubble alone, shown as the kit's card shows it: no anchor, no
    /// delay. It binds its id itself (the widget has none).
    Tooltip(Tooltip, Option<SemId>),
}

impl Spec {
    fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        match self {
            Spec::Button(b) => b.preferred_size(theme),
            Spec::Icon(b) => b.preferred_size(theme),
            Spec::Submit(b) => b.preferred_size(theme),
            Spec::Tooltip(t, _) => t.size(theme),
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        match self {
            Spec::Button(b) => b.handle_event(event, bounds),
            Spec::Icon(b) => b.handle_event(event, bounds),
            Spec::Submit(b) => b.handle_event(event, bounds),
            Spec::Tooltip(..) => EventResult::IGNORED,
        }
    }

    fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        match self {
            Spec::Button(b) => b.render(c, bounds, theme),
            Spec::Icon(b) => b.render(c, bounds, theme),
            Spec::Submit(b) => b.render(c, bounds, theme),
            Spec::Tooltip(t, id) => {
                if let Some(id) = id {
                    c.bind_to_layer(
                        LayerId::DEFAULT,
                        id,
                        [bounds.x, bounds.y, bounds.w, bounds.h],
                    );
                }
                t.render_at(c, LayerId::DEFAULT, theme, bounds);
            }
        }
    }

    /// The widget with the id its draw binds to.
    fn sem(self, id: SemId) -> Self {
        match self {
            Spec::Button(b) => Spec::Button(b.sem(id)),
            Spec::Icon(b) => Spec::Icon(b.sem(id)),
            Spec::Submit(b) => Spec::Submit(b.sem(id)),
            Spec::Tooltip(t, _) => Spec::Tooltip(t, Some(id)),
        }
    }

    fn sem_id(&self) -> Option<&SemId> {
        match self {
            Spec::Button(b) => b.sem_id(),
            Spec::Icon(b) => b.sem_id(),
            Spec::Submit(b) => b.sem_id(),
            Spec::Tooltip(_, id) => id.as_ref(),
        }
    }

    fn set_focused(&mut self, on: bool) {
        match self {
            Spec::Button(b) => b.set_focused(on),
            Spec::Icon(b) => b.set_focused(on),
            Spec::Submit(b) => b.set_focused(on),
            // A label: the reader's focus reads it, nothing rings.
            Spec::Tooltip(..) => {}
        }
    }

    /// The widget as the comps projects it, under `parent`, described by
    /// `states`: a labeled button by its label, an icon-only one by its
    /// icon, the submit button as "submit", the tooltip as a label named
    /// by its text.
    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId, states: &str) {
        let mut node = match self {
            Spec::Button(w) => w.project(b, id, Some(parent)),
            Spec::Icon(w) => w.project(b, id, w.icon, Some(parent)),
            Spec::Submit(w) => w.project(b, id, "submit", Some(parent)),
            Spec::Tooltip(t, _) => {
                let mut node = b.node(id, Role::Label, t.text.as_str());
                node.parent(parent);
                node
            }
        };
        node.description(states);
    }
}

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    /// Columns of the row the card takes, out of the row's `span`.
    spec: Spec,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width (the page's `span`).
    columns: usize,
    cards: Vec<Card>,
}

pub struct ButtonPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The card, `(group, card)`, whose widget a screen reader focused
    /// (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize)>,
}

/// `button_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `button_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

const STATES: &str = "Default, Hover, Press, Focus";

/// The page's usual group: one card for the live states, one disabled.
fn pair(title: &'static str, make: impl Fn(bool) -> Spec) -> Group {
    Group {
        title,
        columns: 2,
        cards: vec![
            Card {
                title: STATES,
                spec: make(false),
            },
            Card {
                title: "Disabled",
                spec: make(true),
            },
        ],
    }
}

impl ButtonPage {
    /// The Button page: every group of the kit's ButtonsPage.
    pub fn new() -> Self {
        Self::with("button_page", Self::groups())
    }

    /// The Icon Button page: the ButtonsPage's groups of icon-only buttons
    /// (`:186`, `:219`, `:252`, `:320`), the same cards as on the Button
    /// page.
    pub fn icons() -> Self {
        let mut groups = Self::groups();
        groups.retain(|g| g.cards.iter().all(|c| matches!(c.spec, Spec::Icon(_))));
        Self::with("icon_button_page", groups)
    }

    /// Every group of the kit's ButtonsPage, in its order.
    fn groups() -> Vec<Group> {
        let button = |variant: ButtonVariant, size: ButtonSize, icon: Option<&'static str>| {
            move |disabled: bool| {
                let mut b = Button::new("Button")
                    .variant(variant)
                    .size(size)
                    .disabled(disabled);
                if let Some(icon) = icon {
                    b = b.icon(icon);
                }
                Spec::Button(b)
            }
        };
        let icon = |variant: IconButtonVariant, size: IconButtonSize, name: &'static str| {
            move |disabled: bool| {
                Spec::Icon(
                    IconButton::new(name)
                        .variant(variant)
                        .size(size)
                        .disabled(disabled),
                )
            }
        };
        let mut groups = vec![
            pair(
                "Button, medium",
                button(ButtonVariant::Primary, ButtonSize::Md, None),
            ),
            pair(
                "Button, medium with icon",
                button(ButtonVariant::Primary, ButtonSize::Md, Some("link")),
            ),
            pair(
                "Button, small",
                button(ButtonVariant::Primary, ButtonSize::Sm, None),
            ),
            pair(
                "Button, small with icon",
                button(ButtonVariant::Primary, ButtonSize::Sm, Some("link")),
            ),
            pair(
                "Call to action",
                button(ButtonVariant::Secondary, ButtonSize::Md, None),
            ),
            pair(
                "Warning button",
                button(ButtonVariant::Warning, ButtonSize::Md, None),
            ),
            pair("Sign in button", |disabled| {
                Spec::Button(
                    Button::new("Sign in with Google")
                        .icon("globe")
                        .disabled(disabled),
                )
            }),
            pair(
                "Ghost icon button",
                icon(IconButtonVariant::Outline, IconButtonSize::Lg, "x"),
            ),
            pair(
                "Large icon button",
                icon(IconButtonVariant::Ghost, IconButtonSize::Md, "plus"),
            ),
            pair(
                "Small icon button",
                icon(IconButtonVariant::Ghost, IconButtonSize::Sm, "plus"),
            ),
            pair("Like button", |disabled| {
                Spec::Button(
                    Button::new("200")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Lg)
                        .icon("heart")
                        .icon_color(hoff_core::RED.0)
                        .disabled(disabled),
                )
            }),
        ];
        groups.push(Group {
            title: "Toolbar button",
            columns: 3,
            cards: vec![
                Card {
                    title: "Default, Hover",
                    spec: Spec::Icon(
                        IconButton::new("send")
                            .variant(IconButtonVariant::Toolbar)
                            .size(IconButtonSize::Lg),
                    ),
                },
                Card {
                    title: "Disabled",
                    spec: Spec::Icon(
                        IconButton::new("send")
                            .variant(IconButtonVariant::Toolbar)
                            .size(IconButtonSize::Lg)
                            .disabled(true),
                    ),
                },
                Card {
                    title: "Active",
                    spec: Spec::Icon(
                        IconButton::new("send")
                            .variant(IconButtonVariant::Toolbar)
                            .size(IconButtonSize::Lg)
                            .active(true),
                    ),
                },
            ],
        });
        groups.push(pair("Like comment", |disabled| {
            Spec::Button(
                Button::new("2")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Xs)
                    .icon("thumbs-up")
                    .disabled(disabled),
            )
        }));
        groups.push(Group {
            title: "Submit button",
            columns: 3,
            cards: vec![
                Card {
                    title: "Default, Hover, Focus",
                    spec: Spec::Submit(SubmitButton::new()),
                },
                Card {
                    title: "Disabled",
                    spec: Spec::Submit(SubmitButton::new().disabled(true)),
                },
                Card {
                    title: "Active",
                    spec: Spec::Submit(SubmitButton::new().active(true)),
                },
            ],
        });
        // `<Tooltip>Copy prompt</Tooltip>`, one card the row's width.
        groups.push(Group {
            title: "Tooltip",
            columns: 1,
            cards: vec![Card {
                title: "Text",
                spec: Spec::Tooltip(Tooltip::new("Copy prompt"), None),
            }],
        });
        groups
    }

    /// The page of `groups` under the root `name`, each card's widget
    /// with its id, `<name>.group[<g>].card[<c>]`.
    fn with(name: &str, mut groups: Vec<Group>) -> Self {
        let root = root_id(name);
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in std::mem::take(&mut group.cards).into_iter().enumerate() {
                    let spec = match card_id(root, g, c) {
                        Some(id) => card.spec.sem(id),
                        None => card.spec,
                    };
                    group.cards.push(Card { spec, ..card });
                }
            }
        }
        Self {
            root,
            groups,
            focus: None,
        }
    }

    /// The card whose widget is `id`.
    fn at(&self, id: &SemId) -> Option<(usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            let c = group
                .cards
                .iter()
                .position(|c| c.spec.sem_id() == Some(id))?;
            Some((g, c))
        })
    }

    /// The reader's focus on the widget of one card, or on none.
    fn set_focus(&mut self, at: Option<(usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                card.spec.set_focused(at == Some((g, c)));
            }
        }
    }

    /// Per group: the title's top, the card rects, the rule (the
    /// showcase mold).
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| (g.columns, g.cards.len()));
        showcase::layout(groups, content, theme)
    }

    fn widget_rect(spec: &Spec, card: Rect, theme: &Theme) -> Rect {
        showcase::centered(spec.preferred_size(theme), card)
    }

    /// Center of the first card's widget (the view tests probe hover
    /// there).
    #[cfg(test)]
    pub(crate) fn first_widget_center(&self, content: Rect, theme: &Theme) -> (f32, f32) {
        let card = self.layout(content, theme)[0].cards[0];
        let r = Self::widget_rect(&self.groups[0].cards[0].spec, card, theme);
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
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
                let bounds = Self::widget_rect(&card.spec, *rect, theme);
                result = result.merge(card.spec.handle_event(event, bounds));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len() - 1;
        for (i, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                showcase::card(c, *rect, card.title, theme);
                card.spec
                    .render(c, Self::widget_rect(&card.spec, *rect, theme), theme);
            }
            // The rule under the group (`border-b`, none on the last).
            if i != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for ButtonPage {
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
                if let Some(id) = card.spec.sem_id() {
                    card.spec.project(b, id, &at, card.title);
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c)| self.groups.get(g)?.cards.get(c))
            .and_then(|card| card.spec.sem_id());
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A button's `activate`: it presses, and nothing else happens, as
    /// for a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let pressable =
            |(g, c): (usize, usize)| !matches!(self.groups[g].cards[c].spec, Spec::Tooltip(..));
        match (self.at(&req.target), req.action.as_str()) {
            (Some(at), "activate") if pressable(at) => Ok(After::Nothing),
            _ => Err(format!(
                "the button page has no {} on {}",
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
    use super::super::showcase::{CARD_H, CARD_MIN_W};
    use super::*;

    /// Narrow content: rows drop to one column instead of squeezing the
    /// cards under `CARD_MIN_W`.
    #[test]
    fn cards_stack_in_narrow_content() {
        let section = ButtonPage::new();
        let content = Rect::new(0.0, 0.0, 500.0, 900.0);
        let layout = section.layout(content, &Theme::hoff_core());
        for group in &layout {
            for r in &group.cards {
                assert_eq!(r.x, content.x);
                assert!(r.w >= CARD_MIN_W);
            }
        }
    }

    /// Wide content: two-column groups sit side by side, three-column
    /// groups in threes, and the groups follow each other down the page.
    #[test]
    fn cards_fill_rows_in_wide_content() {
        let section = ButtonPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let layout = section.layout(content, &Theme::hoff_core());
        let first = &layout[0].cards;
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].y, first[1].y);
        assert!(first[1].x > first[0].x);
        let toolbar = &layout[11].cards;
        assert_eq!(toolbar.len(), 3);
        assert_eq!(toolbar[0].y, toolbar[2].y);
        let mut prev = 0.0;
        for group in &layout {
            assert!(group.title_y > prev);
            prev = group.rule_y;
        }
        assert!(section.content_height(content, &Theme::hoff_core()) > 14.0 * CARD_H);
    }

    /// Every card's widget gets pointer events at its centered rect.
    #[test]
    fn hover_reaches_the_centered_widget() {
        let mut section = ButtonPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let theme = Theme::hoff_core();
        let card = section.layout(content, &theme)[0].cards[0];
        let center = (card.x + card.w / 2.0, card.y + card.h / 2.0);
        let r = section.handle_event(
            &WidgetEvent::MouseMove {
                x: center.0,
                y: center.1,
            },
            content,
            &theme,
        );
        assert!(r.changed);
    }

    /// The Icon Button page is the Button page's icon-only groups: the
    /// three icon buttons live and disabled, the toolbar tool at rest,
    /// disabled and active, each widget centered in its card.
    #[test]
    fn the_icon_page_is_the_icon_groups_of_the_buttons_page() {
        let page = ButtonPage::icons();
        let titles: Vec<&str> = page.groups.iter().map(|g| g.title).collect();
        assert_eq!(
            titles,
            [
                "Ghost icon button",
                "Large icon button",
                "Small icon button",
                "Toolbar button"
            ]
        );
        let toolbar = &page.groups[3];
        let states: Vec<&str> = toolbar.cards.iter().map(|c| c.title).collect();
        assert_eq!(states, ["Default, Hover", "Disabled", "Active"]);
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1000.0, 800.0);
        for (group, at) in page.groups.iter().zip(page.layout(content, &theme)) {
            for (card, rect) in group.cards.iter().zip(&at.cards) {
                let r = ButtonPage::widget_rect(&card.spec, *rect, &theme);
                let (cy, wy) = (rect.y + rect.h / 2.0, r.y + r.h / 2.0);
                assert!((cy - wy).abs() <= 0.5, "{}: centered", group.title);
            }
        }
        let root = page.root.as_ref().map(|r| r.as_str().to_string());
        assert_eq!(root.as_deref(), Some("icon_button_page"));
    }

    /// The ButtonsPage ends with its Tooltip: one card, "Text", the
    /// bubble "Copy prompt" at its own size, projected as a label (not a
    /// button: it has no `activate`).
    #[test]
    fn the_page_ends_with_the_kit_s_tooltip() {
        let mut page = ButtonPage::new();
        let theme = Theme::hoff_core();
        let last = page.groups.last().expect("groups");
        assert_eq!(last.title, "Tooltip");
        let card = &last.cards[0];
        assert_eq!(card.title, "Text");
        let Spec::Tooltip(tip, Some(id)) = &card.spec else {
            panic!("the tooltip card holds the bubble with its id");
        };
        assert_eq!(card.spec.preferred_size(&theme), tip.size(&theme));
        let id = id.clone();
        let root = page.root.clone().expect("the root parses");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Button");
        page.project(&mut b, &root);
        let snap = b.finish().expect("a valid tree");
        let node = snap.get(&id).expect("the bubble is projected");
        assert_eq!(
            (node.role, node.label.as_str()),
            (Role::Label, "Copy prompt")
        );
        let req = Request {
            actor: accord::Actor::Test {
                name: "button".into(),
            },
            target: id,
            action: accord::ActionName::parse("activate").expect("an action name"),
            param: None,
            observed: accord::Observation {
                revision: snap.revision(),
                values: Vec::new(),
            },
            ask: None,
        };
        assert!(page.run(&req).is_err(), "a tooltip does not press");
    }
}
