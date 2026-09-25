//! Buttons page, the hoff core kit's ButtonsPage: groups with a title and
//! a bottom rule, cards in rows, one state per card behind a chip that
//! names it. The widgets are `comps::action` (labeled, icon-only,
//! submit); every visual value comes from the theme.
//!
//! Projected (task-75 phase 2b): a group per title,
//! `button_page.group[<g>]`, and in it each card's widget,
//! `button_page.group[<g>].card[<c>]`, as the comps projects it, described
//! by the card's chip (the states it shows); an icon-only button is named
//! by its icon, the submit button "submit". A button here does nothing but
//! press, so its `activate` changes nothing.

use comps::prelude::{
    Button, ButtonSize, ButtonVariant, EventResult, IconButton, IconButtonSize, IconButtonVariant,
    Rect, SubmitButton, WidgetEvent, rounded_rect, rounded_rect_stroke,
};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::TextMeasurer;
use engine::theme::{Theme, hoff_core};
use sem::{Request, Role, SemId, SnapshotBuilder};

use super::{After, Component, child_id, root_id, under};

/// A card is as tall as the page's `h-50`.
const CARD_H: f32 = 200.0;
/// Narrowest a card column may get before the row drops a column.
const CARD_MIN_W: f32 = 300.0;

enum Spec {
    Button(Button),
    Icon(IconButton),
    Submit(SubmitButton),
}

impl Spec {
    fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        match self {
            Spec::Button(b) => b.preferred_size(theme),
            Spec::Icon(b) => b.preferred_size(theme),
            Spec::Submit(b) => b.preferred_size(theme),
        }
    }

    fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        match self {
            Spec::Button(b) => b.handle_event(event, bounds),
            Spec::Icon(b) => b.handle_event(event, bounds),
            Spec::Submit(b) => b.handle_event(event, bounds),
        }
    }

    fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        match self {
            Spec::Button(b) => b.render(c, bounds, theme),
            Spec::Icon(b) => b.render(c, bounds, theme),
            Spec::Submit(b) => b.render(c, bounds, theme),
        }
    }

    /// The widget with the id its draw binds to.
    fn sem(self, id: SemId) -> Self {
        match self {
            Spec::Button(b) => Spec::Button(b.sem(id)),
            Spec::Icon(b) => Spec::Icon(b.sem(id)),
            Spec::Submit(b) => Spec::Submit(b.sem(id)),
        }
    }

    fn sem_id(&self) -> Option<&SemId> {
        match self {
            Spec::Button(b) => b.sem_id(),
            Spec::Icon(b) => b.sem_id(),
            Spec::Submit(b) => b.sem_id(),
        }
    }

    fn set_focused(&mut self, on: bool) {
        match self {
            Spec::Button(b) => b.set_focused(on),
            Spec::Icon(b) => b.set_focused(on),
            Spec::Submit(b) => b.set_focused(on),
        }
    }

    /// The widget as the comps projects it, under `parent`, described by
    /// `states`: a labeled button by its label, an icon-only one by its
    /// icon, the submit button as "submit".
    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId, states: &str) {
        let mut node = match self {
            Spec::Button(w) => w.project(b, id, Some(parent)),
            Spec::Icon(w) => w.project(b, id, w.icon, Some(parent)),
            Spec::Submit(w) => w.project(b, id, "submit", Some(parent)),
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
    pub fn new() -> Self {
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
        let root = root_id("button_page");
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

    /// Group padding: the page's `p-12`, vertical only (the content rect
    /// already carries the horizontal gutter).
    fn pad(theme: &Theme) -> f32 {
        theme.spacing.xxl
    }

    /// Space between the group title and its cards (`mb-5`).
    fn title_gap(theme: &Theme) -> f32 {
        theme.spacing.lg
    }

    fn title_style(theme: &Theme) -> engine::text::TextStyle {
        theme.typography.exact(theme.typography.title_sm, 28.0, 500)
    }

    fn chip_style(theme: &Theme) -> engine::text::TextStyle {
        theme.typography.exact(theme.typography.caption, 16.0, 500)
    }

    /// Columns a row actually gets at this width: the page's span until
    /// the cards would get narrower than `CARD_MIN_W`.
    fn columns(span: usize, w: f32, gap: f32) -> usize {
        (1..=span)
            .rev()
            .find(|n| (w - gap * (*n as f32 - 1.0)) / *n as f32 >= CARD_MIN_W)
            .unwrap_or(1)
    }

    /// Per group: the title baseline y, the card rects, the rule y.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(f32, Vec<Rect>, f32)> {
        let pad = Self::pad(theme);
        let gap = theme.spacing.sm;
        let title_h = Self::title_style(theme).line_height;
        let mut y = content.y;
        let mut out = Vec::with_capacity(self.groups.len());
        for group in &self.groups {
            let title_y = y + pad;
            let cols = Self::columns(group.columns, content.w, gap);
            let card_w = (content.w - gap * (cols as f32 - 1.0)) / cols as f32;
            let mut rects = Vec::with_capacity(group.cards.len());
            let first_row = title_y + title_h + Self::title_gap(theme);
            for (i, _) in group.cards.iter().enumerate() {
                let col = i % cols;
                let row = i / cols;
                rects.push(Rect::new(
                    content.x + col as f32 * (card_w + gap),
                    first_row + row as f32 * (CARD_H + gap),
                    card_w,
                    CARD_H,
                ));
            }
            let rows = group.cards.len().div_ceil(cols);
            let rule_y = first_row + rows as f32 * CARD_H + (rows as f32 - 1.0) * gap + pad;
            out.push((title_y, rects, rule_y));
            y = rule_y;
        }
        out
    }

    fn widget_rect(spec: &Spec, card: Rect, theme: &Theme) -> Rect {
        let (w, h) = spec.preferred_size(theme);
        Rect::new(
            (card.x + (card.w - w) / 2.0).round(),
            (card.y + (card.h - h) / 2.0).round(),
            w,
            h,
        )
    }

    /// Center of the first card's widget (the view tests probe hover
    /// there).
    #[cfg(test)]
    pub(crate) fn first_widget_center(&self, content: Rect, theme: &Theme) -> (f32, f32) {
        let card = self.layout(content, theme)[0].1[0];
        let r = Self::widget_rect(&self.groups[0].cards[0].spec, card, theme);
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |(_, _, rule)| rule - content.y)
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
        for (group, (_, rects, _)) in self.groups.iter_mut().zip(&layout) {
            for (card, rect) in group.cards.iter_mut().zip(rects) {
                let bounds = Self::widget_rect(&card.spec, *rect, theme);
                result = result.merge(card.spec.handle_event(event, bounds));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let title_style = Self::title_style(theme);
        let chip_style = Self::chip_style(theme);
        let chip_pad_x = theme.spacing.md;
        let chip_pad_y = theme.spacing.xs;
        let chip_inset = theme.spacing.sm;
        let last = self.groups.len() - 1;
        for (i, (group, (title_y, rects, rule_y))) in self.groups.iter().zip(&layout).enumerate() {
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(group.title, &title_style, None),
                x: content.x,
                y: *title_y,
                color: theme.colors.text.0,
            });
            for (card, rect) in group.cards.iter().zip(rects) {
                // The card: a divider-colored hairline at the card radius.
                c.push(rounded_rect_stroke(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    theme.shape.card,
                    theme.colors.divider.0,
                    1.0,
                ));
                // The chip naming the states.
                let (tw, _) = TextMeasurer::measure_styled(card.title, &chip_style, None);
                let chip = Rect::new(
                    rect.x + chip_inset,
                    rect.y + chip_inset,
                    tw + chip_pad_x * 2.0,
                    chip_style.line_height + chip_pad_y * 2.0,
                );
                c.push(rounded_rect(
                    chip.x,
                    chip.y,
                    chip.w,
                    chip.h,
                    theme.shape.nav,
                    theme.colors.bg_hover.0,
                ));
                c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(card.title, &chip_style, None),
                    x: chip.x + chip_pad_x,
                    y: chip.y + chip_pad_y,
                    color: theme.colors.text.0,
                });
                card.spec
                    .render(c, Self::widget_rect(&card.spec, *rect, theme), theme);
            }
            // The rule under the group (`border-b`, none on the last).
            if i != last {
                c.push(SceneNode::Rect {
                    x: content.x,
                    y: *rule_y,
                    w: content.w,
                    h: 1.0,
                    color: theme.colors.divider.0,
                });
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
        match (self.at(&req.target), req.action.as_str()) {
            (Some(_), "activate") => Ok(After::Nothing),
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
    use super::*;

    /// Narrow content: rows drop to one column instead of squeezing the
    /// cards under `CARD_MIN_W`.
    #[test]
    fn cards_stack_in_narrow_content() {
        let section = ButtonPage::new();
        let content = Rect::new(0.0, 0.0, 500.0, 900.0);
        let layout = section.layout(content, &Theme::hoff_core());
        for (_, rects, _) in &layout {
            for r in rects {
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
        let (_, first, _) = &layout[0];
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].y, first[1].y);
        assert!(first[1].x > first[0].x);
        let toolbar = &layout[11].1;
        assert_eq!(toolbar.len(), 3);
        assert_eq!(toolbar[0].y, toolbar[2].y);
        let mut prev = 0.0;
        for (title_y, _, rule_y) in &layout {
            assert!(*title_y > prev);
            prev = *rule_y;
        }
        assert!(section.content_height(content, &Theme::hoff_core()) > 14.0 * CARD_H);
    }

    /// Every card's widget gets pointer events at its centered rect.
    #[test]
    fn hover_reaches_the_centered_widget() {
        let mut section = ButtonPage::new();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let theme = Theme::hoff_core();
        let card = section.layout(content, &theme)[0].1[0];
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
}
