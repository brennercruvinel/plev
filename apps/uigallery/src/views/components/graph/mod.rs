//! Graph page, on the showcase mold of the kit's ButtonsPage
//! (`showcase`): groups with a title and a bottom rule, cards in rows, a
//! chip naming what each card shows. The widget is
//! `engine::widgets::graph::GraphView`; the kit has none, so its tones
//! (`theme.navigation.graph`, resolved as adr-0081 says: the look names a
//! tone, the theme resolves it) derive from the kit's palette.
//!
//! The page shows a twelve-node graph with three kinds of edge (drag to
//! pan, wheel to zoom, click a node to select it and light its edges),
//! and a second one whose kinds take the success and danger tones. A
//! canvas takes its card's width; the card grows to hold it.
//!
//! Projected: a group per title, `graph_page.group[<g>]`, and in it each
//! card's canvas, `graph_page.group[<g>].card[<c>]`, a canvas with its
//! `selected` node and a list item per node, `...card[<c>][<i>]`, that
//! offers `select` as a click does. Pan and zoom move the view, not the
//! graph, and are not declared.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::graph::{Edge, Graph, Node};
use engine::widgets::prelude::{EdgeTone, EventResult, GraphView, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::{After, Component, child_id, key_under, root_id, under};

struct Card {
    title: &'static str,
    /// What a reader hears the canvas is.
    name: &'static str,
    labels: Vec<String>,
    view: GraphView,
}

struct Group {
    title: &'static str,
    span: usize,
    cards: Vec<Card>,
}

pub struct GraphPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The node, `(group, card, node)`, a reader or the keyboard focused
    /// (its ring shows); Escape or a click lets it go.
    focus: Option<(usize, usize, usize)>,
}

/// `graph_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `graph_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// A next chain over `n` nodes plus the `extra` edges, each `(from, to,
/// kind)`.
fn chain(n: usize, extra: &[(usize, usize, usize)]) -> Graph {
    let edge = |from, to, kind: usize| Edge {
        kind,
        ..Edge::new(from, to)
    };
    let mut edges: Vec<Edge> = (0..n - 1).map(|i| edge(i, i + 1, 0)).collect();
    edges.extend(extra.iter().map(|&(a, b, k)| edge(a, b, k)));
    Graph {
        nodes: (0..n).map(|i| Node::new(i.to_string())).collect(),
        edges,
        groups: Vec::new(),
    }
}

fn card(title: &'static str, name: &'static str, data: &Graph, tones: &[(u8, EdgeTone)]) -> Card {
    let mut view = GraphView::new();
    for &(kind, tone) in tones {
        view.set_edge_tone(kind, tone);
    }
    view.set_graph(data);
    let labels = data
        .nodes
        .iter()
        .map(|n| format!("node {}", n.label))
        .collect();
    Card {
        title,
        name,
        labels,
        view,
    }
}

impl GraphPage {
    pub fn new() -> Self {
        // A next chain plus two cross links (accent) and one citation
        // (info), as the page had; then one graph of passes and fails.
        let demo = chain(12, &[(0, 6, 1), (3, 9, 1), (5, 1, 2)]);
        let checks = chain(8, &[(0, 4, 3), (2, 6, 4), (1, 7, 3)]);
        let mut groups = vec![
            Group {
                title: "Graph",
                span: 1,
                cards: vec![card(
                    "Default, Hover, Selected, Focus, Pan, Zoom",
                    "Notes and their links",
                    &demo,
                    &[],
                )],
            },
            Group {
                title: "Edge tones",
                span: 1,
                cards: vec![card(
                    "Structure, Success, Danger",
                    "Checks and their outcomes",
                    &checks,
                    &[(3, EdgeTone::Success), (4, EdgeTone::Danger)],
                )],
            },
        ];
        // The demo opens with a node selected, so the Selected state its
        // chip names shows: its edges hot, its ring drawn.
        groups[0].cards[0].view.select(Some(0));
        let root = root_id("graph_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.view.set_sem(id);
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

    /// The canvas's size in a card `card_w` wide: the card's width less
    /// its inset, one and a half least card widths tall.
    fn canvas_size(card_w: f32, theme: &Theme) -> (f32, f32) {
        let w = (card_w - theme.spacing.sm * 2.0).max(0.0);
        (w, theme.size.card_min_w * 1.5)
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let groups = self.groups.iter().map(|g| {
            let w = showcase::card_width(g.span, content, theme);
            let h = showcase::card_height(Self::canvas_size(w, theme).1, theme);
            (g.span, vec![h; g.cards.len()])
        });
        showcase::layout_sized(groups, content, theme)
    }

    fn canvas_rect(card: Rect, theme: &Theme) -> Rect {
        showcase::centered(Self::canvas_size(card.w, theme), card)
    }

    /// The node behind `id`, `graph_page.group[<g>].card[<c>][<i>]`.
    fn node_at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let i: usize = key_under(card.view.sem_id()?, id)?.parse().ok()?;
                (i < card.labels.len()).then_some((g, c, i))
            })
        })
    }

    fn view_mut(&mut self, (g, c): (usize, usize)) -> Option<&mut GraphView> {
        Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.view)
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
                let bounds = Self::canvas_rect(*rect, theme);
                result = result.merge(card.view.handle_event(event, bounds));
            }
        }
        result
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (g, (group, at)) in self.groups.iter_mut().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, at.title_y, theme);
            for (card, rect) in group.cards.iter_mut().zip(&at.cards) {
                showcase::card(c, *rect, card.title, theme);
                card.view.render(c, Self::canvas_rect(*rect, theme), theme);
            }
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
    }
}

impl Component for GraphPage {
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
                if let Some(id) = card.view.sem_id() {
                    let labels = &card.labels;
                    card.view
                        .project(b, id, card.name, Some(&at), |i| labels[i].clone())
                        .description(card.title);
                }
            }
        }
        let focused = self.focus.and_then(|(g, c, i)| {
            let id = self.groups.get(g)?.cards.get(c)?.view.sem_id()?;
            Some(id.key(i.to_string()))
        });
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A node's `select` selects it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let at = self.node_at(&req.target);
        match (at, req.action.as_str()) {
            (Some((g, c, i)), "select") => {
                if let Some(view) = self.view_mut((g, c)) {
                    view.select(Some(i));
                }
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the graph page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.focus = self.node_at(id);
        let at = self.focus;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let node = at.filter(|&(fg, fc, _)| (fg, fc) == (g, c));
                card.view.set_focused(node.map(|(_, _, i)| i));
            }
        }
        true
    }

    fn blur(&mut self) -> bool {
        for group in &mut self.groups {
            for card in &mut group.cards {
                card.view.set_focused(None);
            }
        }
        self.focus.take().is_some()
    }
}

#[cfg(test)]
mod tests;
