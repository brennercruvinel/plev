//! Tree page: branches expand and collapse, leaves select, in the
//! showcase mold. Two cards: a project (every state) and a deep tree of
//! long names, cut to what each row has left. The look is the theme's
//! (`theme.disclosure.tree`): the kit's scene list in the core theme, the
//! glass rows elsewhere. A card is as tall as the rows its tree shows.
//!
//! Projected (task-75 phase 2b): a group, `tree_page.group[0]`; in it
//! each card's tree, named by its chip, with an item per visible row, as
//! the comps projects it: an item's `select` selects it and a branch's
//! `toggle_expanded` opens or closes it. A click on a leaf is its
//! `select`; on a branch, both (the comps tree toggles and selects it,
//! `Tree::handle_event`). The first tree keeps its id, `tree_page.tree`;
//! the second is `tree_page.group[0].card[1]`.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, Tree, TreeNode, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, key_under, root_id, showcase, under};

const GROUP: &str = "Tree";

struct Card {
    title: &'static str,
    tree: Tree,
}

pub struct TreePage {
    root: Option<SemId>,
    cards: Vec<Card>,
    /// The row a screen reader or the keyboard focused, `(card, node id)`
    /// (its ring shows while the row does); Escape or a click lets it go.
    focus: Option<(usize, u64)>,
}

fn project_tree() -> Tree {
    Tree::new(vec![
        TreeNode::branch(
            1,
            "src",
            vec![
                TreeNode::branch(
                    2,
                    "ui",
                    vec![
                        TreeNode::leaf(3, "widgets.rs").icon("code"),
                        TreeNode::leaf(4, "icons.rs").icon("code"),
                    ],
                )
                .expanded(true),
                TreeNode::leaf(5, "main.rs").icon("code"),
                TreeNode::leaf(6, "renderer.rs").icon("code"),
            ],
        )
        .expanded(true),
        TreeNode::branch(
            7,
            "assets",
            vec![
                TreeNode::leaf(8, "InclusiveSans-Regular.ttf"),
                TreeNode::leaf(9, "codicons.ttf"),
            ],
        ),
        TreeNode::leaf(10, "Cargo.toml").icon("settings"),
        TreeNode::leaf(11, "README.md"),
    ])
}

/// Five levels down, each name longer than a narrow card holds.
fn deep_tree() -> Tree {
    let mut node =
        TreeNode::leaf(6, "the_render_graph_of_every_frame_the_window_draws.rs").icon("code");
    let names = [
        "presentation_layer_of_the_scene",
        "compositor_and_its_layers",
        "drawing_backends_for_every_target",
        "engine_crate_with_a_long_name",
        "workspace_of_the_whole_project",
    ];
    for (depth, name) in names.into_iter().enumerate() {
        node = TreeNode::branch(5 - depth as u64, name, vec![node]).expanded(true);
    }
    Tree::new(vec![node])
}

impl TreePage {
    pub fn new() -> Self {
        let root = root_id("tree_page");
        let ids = [
            child_id(root.as_ref(), "tree"),
            child_id(
                child_id(root.as_ref(), "group")
                    .map(|g| g.key("0"))
                    .as_ref(),
                "card",
            )
            .map(|c| c.key("1")),
        ];
        let trees = [
            (
                "Default, Hover, Selected, Focus, Expand: a project",
                project_tree(),
            ),
            ("Deep and long: names cut", deep_tree()),
        ];
        let cards = trees
            .into_iter()
            .zip(ids)
            .map(|((title, tree), id)| Card {
                title,
                tree: match id {
                    Some(id) => tree.sem(id),
                    None => tree,
                },
            })
            .collect();
        Self {
            root,
            cards,
            focus: None,
        }
    }

    /// The card and the row behind `id`, `<tree>[<node id>]`, while it
    /// shows.
    fn row_at(&self, id: &SemId) -> Option<(usize, u64)> {
        self.cards.iter().enumerate().find_map(|(c, card)| {
            let node = key_under(card.tree.sem_id()?, id)?.parse().ok()?;
            Self::shows(&card.tree, node).then_some((c, node))
        })
    }

    /// Whether the row of `node` shows (its branches are expanded).
    fn shows(tree: &Tree, node: u64) -> bool {
        tree.visible_rows().iter().any(|r| r.id == node)
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        fit_layout(&[(2, self.cards.len())], content, theme, |_, c, _| {
            self.cards[c].tree.content_height(theme)
        })
    }

    /// Each tree's rect.
    fn rects(&self, content: Rect, theme: &Theme) -> Vec<Rect> {
        let layout = self.layout(content, theme);
        let placed = layout.first().map(|g| g.cards.as_slice()).unwrap_or(&[]);
        let rects = self.cards.iter().zip(placed).map(|(card, (_, body))| {
            Rect::new(body.x, body.y, body.w, card.tree.content_height(theme))
        });
        rects.collect()
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
        let rects = self.rects(content, theme);
        for (card, at) in self.cards.iter_mut().zip(rects) {
            result = result.merge(card.tree.handle_event(event, at, theme));
        }
        result
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let Some(placed) = layout.first() else {
            return;
        };
        showcase::title(c, GROUP, content, placed.title_y, theme);
        for (card, (rect, body)) in self.cards.iter().zip(&placed.cards) {
            showcase::card(c, *rect, card.title, theme);
            let at = Rect::new(body.x, body.y, body.w, card.tree.content_height(theme));
            card.tree.render(c, at, theme);
        }
    }
}

impl Component for TreePage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(group) = child_id(Some(root), "group").map(|g| g.key("0")) else {
            return;
        };
        b.node(&group, Role::Group, GROUP).parent(root);
        for card in &self.cards {
            if let Some(id) = card.tree.sem_id() {
                card.tree.project(b, id, card.title, Some(&group));
            }
        }
        let focused = self
            .focus
            .filter(|(c, n)| Self::shows(&self.cards[*c].tree, *n))
            .and_then(|(c, n)| Some(self.cards[c].tree.sem_id()?.key(n.to_string())));
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// An item's `select` selects it; a branch's `toggle_expanded` opens or
    /// closes it: what a click on a leaf, and on a branch, do.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let row = self.row_at(&req.target);
        match (row, req.action.as_str()) {
            (Some((c, node)), "select") => self.cards[c].tree.selected = Some(node),
            (Some((c, node)), "toggle_expanded") if self.cards[c].tree.toggle(node) => {}
            _ => {
                return Err(format!(
                    "the tree page has no {} on {}",
                    req.action, req.target
                ));
            }
        }
        Ok(After::Nothing)
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.focus = self.row_at(id);
        for (c, card) in self.cards.iter_mut().enumerate() {
            let node = self.focus.filter(|(fc, _)| *fc == c).map(|(_, n)| n);
            card.tree.set_focused(node);
        }
        true
    }

    fn blur(&mut self) -> bool {
        for card in &mut self.cards {
            card.tree.set_focused(None);
        }
        self.focus.take().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::drawing::compositor::{LayerId, SceneNode};
    use engine::drawing::text::TextMeasurer;

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff:
    /// every label the trees draw ends inside its tree.
    #[test]
    fn every_label_fits_its_tree_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let mut page = TreePage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let rects = page.rects(content, &theme);
                let mut c = Compositor::new();
                c.begin_frame();
                page.render(&mut c, content, &theme);
                let r = &theme.disclosure.tree;
                let style = theme.typography.run(r.text_type);
                for node in c.layer(LayerId::DEFAULT).unwrap().nodes() {
                    let SceneNode::Text { key, x, y, .. } = node else {
                        continue;
                    };
                    let Some(at) = rects.iter().find(|r| r.contains(*x + 0.5, *y + 1.0)) else {
                        continue;
                    };
                    let (tw, _) = TextMeasurer::measure_styled(&key.text, &style, None);
                    assert!(x + tw <= at.right() + 0.5, "{w}px: {} overflows", key.text);
                }
            }
        }
    }
}
