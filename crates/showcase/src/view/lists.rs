//! Lists section: 10,000-row virtualized list + tree view.

use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use plev::theme::Theme;
use plev::ui::icons;
use plev::ui::widgets::{EventResult, Rect, Tree, TreeNode, VirtualList, WidgetEvent};

use super::{group_label, panel, text};

const LABEL_H: f32 = 24.0;
const ROW_H: f32 = 28.0;

pub struct ListsSection {
    pub list: VirtualList,
    pub tree: Tree,
}

impl ListsSection {
    pub fn new() -> Self {
        let mut list = VirtualList::new(ROW_H);
        list.set_item_count(10_000);

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
                    TreeNode::leaf(8, "Inter-Regular.ttf"),
                    TreeNode::leaf(9, "codicons.ttf"),
                ],
            ),
            TreeNode::leaf(10, "Cargo.toml").icon("settings"),
            TreeNode::leaf(11, "README.md"),
        ]);

        Self { list, tree }
    }

    /// Panel + inner bounds of the virtual list (left half).
    pub fn list_bounds(&self, content: Rect) -> Rect {
        let w = (content.w * 0.52).min(460.0);
        Rect::new(
            content.x + 1.0,
            content.y + LABEL_H + 1.0,
            w,
            content.h - LABEL_H - 14.0,
        )
    }

    fn tree_bounds(&self, content: Rect) -> Rect {
        let list = self.list_bounds(content);
        let x = list.x + list.w + 40.0;
        Rect::new(x, list.y, (content.x + content.w - x).max(180.0), list.h)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let mut r = self.list.handle_event(event, self.list_bounds(content));
        r = r.merge(self.tree.handle_event(event, self.tree_bounds(content)));
        r
    }

    /// Scrollbar fade. Returns `true` while animating.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.list.tick(dt)
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        list_layer: LayerId,
        content: Rect,
        theme: &Theme,
    ) {
        let list_bounds = self.list_bounds(content);
        let tree_bounds = self.tree_bounds(content);

        group_label(
            c,
            "VIRTUALIZED LIST — 10,000 ROWS",
            content.x,
            content.y,
            theme,
        );
        group_label(c, "TREE", tree_bounds.x, content.y, theme);

        // Panels behind both widgets (default layer; rows live on the
        // clipped list layer above it).
        panel(
            c,
            Rect::new(
                list_bounds.x - 1.0,
                list_bounds.y - 1.0,
                list_bounds.w + 2.0,
                list_bounds.h + 2.0,
            ),
            theme,
        );
        panel(
            c,
            Rect::new(
                tree_bounds.x - 8.0,
                tree_bounds.y - 1.0,
                tree_bounds.w + 16.0,
                tree_bounds.h + 2.0,
            ),
            theme,
        );

        let dim = theme.colors.text_dim.0;
        let fg = theme.colors.text.0;
        let mid = theme.colors.text_mid.0;
        self.list.render_with_to_layer(
            c,
            list_layer,
            list_bounds,
            theme,
            |c, index, rect, _hovered, selected| {
                if let Some(node) = icons::icon_at(
                    if index % 9 == 0 { "folder" } else { "file" },
                    14.0,
                    mid,
                    rect.x + 12.0,
                    rect.y + (rect.h - 14.0) / 2.0,
                ) {
                    c.push_to_layer(list_layer, node);
                }
                c.push_to_layer(
                    list_layer,
                    SceneNode::Text {
                        key: TextNodeKey::new(&format!("Item {index}"), 13.0, 13.0 * 1.3, None)
                            .with_weight(if selected { 600 } else { 400 }),
                        x: rect.x + 34.0,
                        y: rect.y + (rect.h - 13.0 * 1.3) / 2.0,
                        color: fg,
                    },
                );
                c.push_to_layer(
                    list_layer,
                    SceneNode::Text {
                        key: TextNodeKey::new(
                            &format!("#{:04}", index % 10_000),
                            11.0,
                            11.0 * 1.3,
                            None,
                        ),
                        x: rect.x + rect.w - 60.0,
                        y: rect.y + (rect.h - 11.0 * 1.3) / 2.0,
                        color: dim,
                    },
                );
            },
        );

        self.tree.render(c, tree_bounds, theme);

        if let Some(i) = self.list.selected {
            text(
                c,
                &format!("selected: Item {i}"),
                11.0,
                400,
                list_bounds.x,
                list_bounds.y + list_bounds.h + 8.0,
                dim,
            );
        }
    }
}
