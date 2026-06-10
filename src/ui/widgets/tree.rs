use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, with_alpha};

const FONT: f32 = 13.0;
const ROW_H: f32 = 26.0;
const INDENT: f32 = 16.0;
const ICON: f32 = 14.0;
const CHEVRON: f32 = 12.0;
const PAD_X: f32 = 6.0;

/// A node in a [`Tree`].
#[derive(Clone, Debug)]
pub struct TreeNode {
    /// Opaque id reported on selection.
    pub id: u64,
    pub label: String,
    /// Leading icon name; branches default to folder/folder-open, leaves
    /// to "file" when `None`.
    pub icon: Option<&'static str>,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

impl TreeNode {
    pub fn leaf(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
        }
    }

    pub fn branch(id: u64, label: impl Into<String>, children: Vec<TreeNode>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            children,
            expanded: false,
        }
    }

    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(name);
        self
    }

    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }

    fn is_branch(&self) -> bool {
        !self.children.is_empty()
    }
}

/// A row of the flattened (visible) tree.
#[derive(Clone, Debug)]
pub struct TreeRow {
    pub id: u64,
    pub depth: usize,
    pub label: String,
    pub icon: Option<&'static str>,
    pub is_branch: bool,
    pub expanded: bool,
}

/// Tree view with expand/collapse, indentation, and icons.
///
/// Rows are laid out top-down from `bounds.y`; the caller decides whether
/// to wrap it in a scrollable region (pair with
/// [`VirtualList`](super::VirtualList)-style clipping for huge trees).
#[derive(Clone, Debug)]
pub struct Tree {
    pub roots: Vec<TreeNode>,
    pub selected: Option<u64>,
    hovered_row: Option<usize>,
}

impl Tree {
    pub fn new(roots: Vec<TreeNode>) -> Self {
        Self {
            roots,
            selected: None,
            hovered_row: None,
        }
    }

    pub fn row_height(&self) -> f32 {
        ROW_H
    }

    /// Currently visible rows (expanded branches only), top to bottom.
    pub fn visible_rows(&self) -> Vec<TreeRow> {
        let mut rows = Vec::new();
        fn walk(nodes: &[TreeNode], depth: usize, rows: &mut Vec<TreeRow>) {
            for node in nodes {
                rows.push(TreeRow {
                    id: node.id,
                    depth,
                    label: node.label.clone(),
                    icon: node.icon,
                    is_branch: node.is_branch(),
                    expanded: node.expanded,
                });
                if node.is_branch() && node.expanded {
                    walk(&node.children, depth + 1, rows);
                }
            }
        }
        walk(&self.roots, 0, &mut rows);
        rows
    }

    /// Total height of the visible rows.
    pub fn content_height(&self) -> f32 {
        self.visible_rows().len() as f32 * ROW_H
    }

    fn node_mut(nodes: &mut [TreeNode], id: u64) -> Option<&mut TreeNode> {
        for node in nodes {
            if node.id == id {
                return Some(node);
            }
            if let Some(found) = Self::node_mut(&mut node.children, id) {
                return Some(found);
            }
        }
        None
    }

    /// Toggle a branch's expanded state. Returns `true` if found.
    pub fn toggle(&mut self, id: u64) -> bool {
        match Self::node_mut(&mut self.roots, id) {
            Some(node) if node.is_branch() => {
                node.expanded = !node.expanded;
                true
            }
            _ => false,
        }
    }

    fn row_at(&self, x: f32, y: f32, bounds: Rect) -> Option<usize> {
        if !bounds.contains(x, y) {
            return None;
        }
        let i = ((y - bounds.y) / ROW_H).floor();
        if i < 0.0 {
            return None;
        }
        let i = i as usize;
        (i < self.visible_rows().len()).then_some(i)
    }

    /// Handle events. Clicking a branch toggles it; clicking a leaf
    /// selects it (selection id readable via `self.selected`).
    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.row_at(x, y, bounds);
                if hit != self.hovered_row {
                    self.hovered_row = hit;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                let Some(i) = self.row_at(x, y, bounds) else {
                    return EventResult::IGNORED;
                };
                let row = &self.visible_rows()[i];
                let id = row.id;
                if row.is_branch {
                    self.toggle(id);
                    self.selected = Some(id);
                } else {
                    self.selected = Some(id);
                }
                EventResult::clicked()
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let line_height = FONT * 1.3;
        for (i, row) in self.visible_rows().iter().enumerate() {
            let ry = bounds.y + i as f32 * ROW_H;
            if ry + ROW_H > bounds.y + bounds.h + ROW_H {
                break;
            }
            let row_rect = Rect::new(bounds.x, ry, bounds.w, ROW_H);
            let is_selected = self.selected == Some(row.id);
            let is_hovered = self.hovered_row == Some(i);

            // HOFF rows: hover .05 / selected .10 white glass, radius 12.
            // Row icons pushed later stack on top (push order preserved).
            if is_selected {
                compositor.push(super::rounded_rect(
                    row_rect.x + 2.0,
                    row_rect.y + 1.0,
                    row_rect.w - 4.0,
                    row_rect.h - 2.0,
                    theme.radius.md.min(row_rect.h / 2.0),
                    theme.glass.surface_active.0,
                ));
            } else if is_hovered {
                compositor.push(super::rounded_rect(
                    row_rect.x + 2.0,
                    row_rect.y + 1.0,
                    row_rect.w - 4.0,
                    row_rect.h - 2.0,
                    theme.radius.md.min(row_rect.h / 2.0),
                    theme.glass.surface_hover.0,
                ));
            }

            let mut cx = bounds.x + PAD_X + row.depth as f32 * INDENT;

            if row.is_branch {
                let chevron = if row.expanded {
                    "chevron-down"
                } else {
                    "chevron-right"
                };
                if let Some(node) = icons::icon_at(
                    chevron,
                    CHEVRON,
                    with_alpha(theme.colors.text_dim, 1.0),
                    cx,
                    ry + (ROW_H - CHEVRON) / 2.0,
                ) {
                    compositor.push(node);
                }
            }
            cx += CHEVRON + 4.0;

            let icon_name = row.icon.unwrap_or(if row.is_branch {
                if row.expanded {
                    "folder-open"
                } else {
                    "folder"
                }
            } else {
                "file"
            });
            if let Some(node) = icons::icon_at(
                icon_name,
                ICON,
                with_alpha(theme.colors.text_mid, 1.0),
                cx,
                ry + (ROW_H - ICON) / 2.0,
            ) {
                compositor.push(node);
            }
            cx += ICON + 6.0;

            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(&row.label, FONT, line_height, None),
                x: cx,
                y: ry + (ROW_H - line_height) / 2.0,
                color: with_alpha(
                    if is_selected {
                        theme.colors.text
                    } else {
                        theme.colors.text_mid
                    },
                    1.0,
                ),
            });
        }
    }
}
