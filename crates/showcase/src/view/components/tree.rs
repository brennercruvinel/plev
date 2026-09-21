//! Tree page: branches expand and collapse, leaves select.

use comps::prelude::{EventResult, Rect, Tree, TreeNode, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::panel;

pub struct TreePage {
    tree: Tree,
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
        Self { tree }
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
        self.tree
            .handle_event(event, Self::bounds(content, theme), theme)
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let bounds = Self::bounds(content, theme);
        panel(c, bounds.outset(theme.spacing.sm), theme);
        self.tree.render(c, bounds, theme);
    }
}
