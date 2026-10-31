//! Breadcrumb page, on the showcase mold of the kit's ButtonsPage
//! (`showcase`): groups with a title and a bottom rule, cards in rows, a
//! chip naming the states each card's trail shows. The widget is
//! `engine::widgets::nav::Breadcrumb`; the kit has none, so its recipe
//! (`theme.navigation.breadcrumb`) derives from the kit's sidebar link.
//!
//! The page shows the ranges the trail meets: a trail that fits, the
//! current place alone, a deep trail whose leading crumbs fold into an
//! ellipsis, and a translated one. A trail is never wider than its card:
//! it folds, and the current place truncates where even it does not fit.
//!
//! Projected: a group per title, `breadcrumb_page.group[<g>]`, and in it
//! each card's trail (`breadcrumb_page.trail`, `.single`, `.folded`,
//! `.translated`) with its crumbs as the engine projects them (links,
//! the last one where the trail is), described by the card's chip. A
//! crumb's `activate` is what its click does here, the echo, which is a
//! status, `breadcrumb_page.clicked`, a reader hears as it changes.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::Theme;
use engine::widgets::nav::Breadcrumb;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, key_under, root_id, under};

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    /// The trail's name under the page root.
    name: &'static str,
    trail: Breadcrumb,
}

struct Group {
    title: &'static str,
    span: usize,
    cards: Vec<Card>,
}

pub struct BreadcrumbPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The crumb clicked last: `(group, card, crumb)`.
    clicked: Option<(usize, usize, usize)>,
    /// The crumb a screen reader or the keyboard focused (its ring
    /// shows); Escape or a click lets it go.
    focus: Option<(usize, usize, usize)>,
}

fn card(title: &'static str, name: &'static str, crumbs: &[&str]) -> Card {
    Card {
        title,
        name,
        trail: Breadcrumb::new(crumbs.iter().copied()),
    }
}

impl BreadcrumbPage {
    pub fn new() -> Self {
        let mut groups = vec![
            Group {
                title: "Breadcrumb",
                span: 2,
                cards: vec![
                    card(
                        "Default, Hover, Press, Focus, Current",
                        "trail",
                        &["Home", "Projects", "plev", "Components"],
                    ),
                    card("Current place only", "single", &["Home"]),
                ],
            },
            Group {
                title: "Long content",
                span: 2,
                cards: vec![
                    card(
                        "Folded, Hover, Focus",
                        "folded",
                        &[
                            "Workspace",
                            "brennercruvinel",
                            "plev",
                            "apps",
                            "uigallery",
                            "src",
                            "views",
                            "components",
                            "breadcrumb",
                            "tests.rs",
                        ],
                    ),
                    card(
                        "Translated",
                        "translated",
                        &[
                            "Início",
                            "Configurações",
                            "Privacidade e segurança",
                            "Permissões do aplicativo",
                        ],
                    ),
                ],
            },
        ];
        let root = root_id("breadcrumb_page");
        for group in &mut groups {
            for card in &mut group.cards {
                if let Some(id) = child_id(root.as_ref(), card.name) {
                    card.trail = card.trail.clone().sem(id);
                }
            }
        }
        Self {
            root,
            groups,
            clicked: None,
            focus: None,
        }
    }

    /// The crumb behind `id`, `breadcrumb_page.<trail>[<i>]`.
    fn crumb_at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let i: usize = key_under(card.trail.sem_id()?, id)?.parse().ok()?;
                (i < card.trail.crumbs.len()).then_some((g, c, i))
            })
        })
    }

    fn trail(&self, (g, c): (usize, usize)) -> Option<&Breadcrumb> {
        Some(&self.groups.get(g)?.cards.get(c)?.trail)
    }

    /// What the page echoes for the crumb clicked last: the first trail
    /// by its index alone, the others by their card.
    fn echo(&self, (g, c, i): (usize, usize, usize)) -> String {
        match self.groups.get(g).and_then(|group| group.cards.get(c)) {
            Some(card) if (g, c) != (0, 0) => format!("clicked crumb {i} in {}", card.name),
            _ => format!("clicked crumb {i}"),
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| (g.span, g.cards.len()));
        showcase::layout(groups, content, theme)
    }

    /// The trail's rect in its card: as wide as it asks, never wider than
    /// the card less its inset on both sides (it folds), centered.
    fn widget_rect(trail: &Breadcrumb, card: Rect, theme: &Theme) -> Rect {
        let room = card.w - theme.spacing.sm * 2.0;
        let w = trail.preferred_width(theme).min(room);
        showcase::centered((w, Breadcrumb::height(theme)), card)
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
        let mut clicked = None;
        for (g, (group, at)) in self.groups.iter_mut().zip(&layout).enumerate() {
            for (c, (card, rect)) in group.cards.iter_mut().zip(&at.cards).enumerate() {
                let bounds = Self::widget_rect(&card.trail, *rect, theme);
                let (r, crumb) = card.trail.handle_event(event, bounds, theme);
                if let Some(i) = crumb {
                    clicked = Some((g, c, i));
                }
                result = result.merge(r);
            }
        }
        if clicked.is_some() {
            self.clicked = clicked;
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (g, (group, at)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (k, (card, rect)) in group.cards.iter().zip(&at.cards).enumerate() {
                showcase::card(c, *rect, card.title, theme);
                let bounds = Self::widget_rect(&card.trail, *rect, theme);
                card.trail.render(c, bounds, theme);
                let echoed = self.clicked.filter(|&(eg, ec, _)| (eg, ec) == (g, k));
                if let Some(at) = echoed {
                    Self::render_echo(c, &self.echo(at), bounds, *rect, theme);
                }
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }

    /// The echo under the trail it names, centered in its card.
    fn render_echo(c: &mut Compositor, note: &str, trail: Rect, card: Rect, theme: &Theme) {
        let style = theme.typography.caption_r();
        let room = card.w - theme.spacing.sm * 2.0;
        let shown = TextMeasurer::truncate_to_width(note, &style, room);
        let (w, _) = TextMeasurer::measure_styled(&shown, &style, None);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&shown, &style, None),
            x: card.x + (card.w - w) / 2.0,
            y: trail.bottom() + theme.spacing.sm,
            color: theme.colors.text_mid.0,
        });
    }
}

impl Component for BreadcrumbPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let family = child_id(Some(root), "group");
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = family.as_ref().map(|f| f.key(g.to_string())) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                if let Some(id) = card.trail.sem_id() {
                    card.trail
                        .project(b, id, "path", Some(&at))
                        .description(card.title);
                }
            }
        }
        if let (Some(at), Some(echo)) = (self.clicked, child_id(Some(root), "clicked")) {
            b.node(&echo, Role::Status, self.echo(at)).parent(root);
        }
        let focused = self.focus.and_then(|(g, c, i)| {
            let id = self.trail((g, c))?.sem_id()?;
            Some(id.key(i.to_string()))
        });
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A crumb's `activate` is echoed, as its click is; the current
    /// place is not a target.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let at = self
            .crumb_at(&req.target)
            .filter(|&(g, c, i)| self.trail((g, c)).is_some_and(|t| i + 1 < t.crumbs.len()));
        match (at, req.action.as_str()) {
            (Some(at), "activate") => {
                self.clicked = Some(at);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the breadcrumb page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self
            .crumb_at(id)
            .filter(|&(g, c, i)| self.trail((g, c)).is_some_and(|t| i + 1 < t.crumbs.len()));
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let crumb = at.filter(|&(fg, fc, _)| (fg, fc) == (g, c));
                card.trail.set_focused(crumb.map(|(_, _, i)| i));
            }
        }
        true
    }

    fn blur(&mut self) -> bool {
        for group in &mut self.groups {
            for card in &mut group.cards {
                card.trail.set_focused(None);
            }
        }
        self.focus.take().is_some()
    }
}

#[cfg(test)]
mod tests;
