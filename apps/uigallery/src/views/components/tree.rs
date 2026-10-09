//! Tree page: branches expand and collapse, leaves select.
//!
//! Projected (task-75 phase 2b): the tree, `tree_page.tree`, named
//! "files", with an item per visible row, as the comps projects it: an
//! item's `select` selects it and a branch's `toggle_expanded` opens or
//! closes it. A click on a leaf is its `select`; on a branch, both (the
//! comps tree toggles and selects it, `Tree::handle_event`).

use accord::{Request, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, Tree, TreeNode, WidgetEvent};

use super::{After, Component, child_id, key_under, panel, root_id, under};

pub struct TreePage {
    root: Option<SemId>,
    tree: Tree,
    /// The row a screen reader or the keyboard focused, by node id (its
    /// ring shows while the row does); Escape or a click lets it go.
    focus: Option<u64>,
}

impl TreePage {
    pub fn new() -> Self {
        let tree = Tree::new(vec![
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
        ]);
        let root = root_id("tree_page");
        let tree = match child_id(root.as_ref(), "tree") {
            Some(id) => tree.sem(id),
            None => tree,
        };
        Self {
            root,
            tree,
            focus: None,
        }
    }

    /// The row behind `id`, `tree_page.tree[<node id>]`, while it shows.
    fn row_at(&self, id: &SemId) -> Option<u64> {
        let node = key_under(self.tree.sem_id()?, id)?.parse().ok()?;
        self.shows(node).then_some(node)
    }

    /// Whether the row of `node` shows (its branches are expanded).
    fn shows(&self, node: u64) -> bool {
        self.tree.visible_rows().iter().any(|r| r.id == node)
    }

    fn bounds(content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x + theme.spacing.sm,
            content.y,
            content.w.min(theme.size.readable_max_w / 2.0) - theme.spacing.sm * 2.0,
            theme.size.card_min_w * 1.5,
        )
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.size.card_min_w * 1.5
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let r = self
            .tree
            .handle_event(event, Self::bounds(content, theme), theme);
        if blurred {
            return r.merge(EventResult::changed());
        }
        r
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let bounds = Self::bounds(content, theme);
        panel(c, bounds.outset(theme.spacing.sm), theme);
        self.tree.render(c, bounds, theme);
    }
}

impl Component for TreePage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(id) = self.tree.sem_id() else {
            return;
        };
        self.tree.project(b, id, "files", Some(root));
        if let Some(node) = self.focus.filter(|n| self.shows(*n)) {
            b.focus(id.key(node.to_string()));
        }
    }

    /// An item's `select` selects it; a branch's `toggle_expanded` opens or
    /// closes it: what a click on a leaf, and on a branch, do.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let row = self.row_at(&req.target);
        match (row, req.action.as_str()) {
            (Some(node), "select") => self.tree.selected = Some(node),
            (Some(node), "toggle_expanded") if self.tree.toggle(node) => {}
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
        self.tree.set_focused(self.focus);
        true
    }

    fn blur(&mut self) -> bool {
        self.tree.set_focused(None);
        self.focus.take().is_some()
    }
}
