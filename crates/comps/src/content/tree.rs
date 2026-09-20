use crate::icons;
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{IconSize, Theme};

use crate::core::{EventResult, Rect, WidgetEvent, with_alpha};

/// The per-theme geometry the hit test and the render share: a dense
/// base-2r row (line box plus `xs` above and below), one `lg` step of
/// indent per depth, small icons, `xs` gaps.
struct Metrics {
    row_h: f32,
    indent: f32,
    icon: f32,
    pad_x: f32,
    gap: f32,
    style: TextStyle,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        let style = theme.typography.base_2r();
        Self {
            row_h: (style.line_height + theme.spacing.xs * 2.0).round(),
            indent: theme.spacing.lg,
            icon: theme.control.icon(IconSize::Sm),
            pad_x: theme.spacing.sm,
            gap: theme.spacing.xs,
            style,
        }
    }
}

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
/// [`VirtualList`](crate::content::VirtualList)-style clipping for huge trees).
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

    pub fn row_height(&self, theme: &Theme) -> f32 {
        Metrics::of(theme).row_h
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
    pub fn content_height(&self, theme: &Theme) -> f32 {
        self.visible_rows().len() as f32 * Metrics::of(theme).row_h
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

    fn row_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        if !bounds.contains(x, y) {
            return None;
        }
        let i = ((y - bounds.y) / Metrics::of(theme).row_h).floor();
        if i < 0.0 {
            return None;
        }
        let i = i as usize;
        (i < self.visible_rows().len()).then_some(i)
    }

    /// Handle events. Clicking a branch toggles it; clicking a leaf
    /// selects it (selection id readable via `self.selected`).
    /// Rows are laid out from the theme, so the event path takes the same
    /// `theme` the render path draws with.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.row_at(x, y, bounds, theme);
                if hit != self.hovered_row {
                    self.hovered_row = hit;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                let Some(i) = self.row_at(x, y, bounds, theme) else {
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
        let m = Metrics::of(theme);
        let style = &m.style;
        for (i, row) in self.visible_rows().iter().enumerate() {
            let ry = bounds.y + i as f32 * m.row_h;
            if ry + m.row_h > bounds.y + bounds.h + m.row_h {
                break;
            }
            let row_rect = Rect::new(bounds.x, ry, bounds.w, m.row_h);
            let is_selected = self.selected == Some(row.id);
            let is_hovered = self.hovered_row == Some(i);

            // HOFF rows: hover / selected white glass at the nav radius,
            // inset a hair so neighbors keep a seam. Row icons pushed
            // later stack on top (push order preserved).
            if is_selected || is_hovered {
                let seam_x = m.gap / 2.0;
                let seam_y = theme.control.edge_width;
                compositor.push(crate::recipe::rounded_rect(
                    row_rect.x + seam_x,
                    row_rect.y + seam_y,
                    row_rect.w - seam_x * 2.0,
                    row_rect.h - seam_y * 2.0,
                    theme.shape.nav.min(row_rect.h / 2.0),
                    if is_selected {
                        theme.glass.surface_active.0
                    } else {
                        theme.glass.surface_hover.0
                    },
                ));
            }

            let mut cx = bounds.x + m.pad_x + row.depth as f32 * m.indent;

            if row.is_branch {
                let chevron = if row.expanded {
                    "chevron-down"
                } else {
                    "chevron-right"
                };
                if let Some(node) = icons::icon_at(
                    chevron,
                    m.icon,
                    with_alpha(theme.colors.text_dim, 1.0),
                    cx,
                    ry + (m.row_h - m.icon) / 2.0,
                ) {
                    compositor.push(node);
                }
            }
            cx += m.icon + m.gap;

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
                m.icon,
                with_alpha(theme.colors.text_mid, 1.0),
                cx,
                ry + (m.row_h - m.icon) / 2.0,
            ) {
                compositor.push(node);
            }
            cx += m.icon + m.gap * 2.0;

            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(&row.label, style, None),
                x: cx,
                y: ry + TextMeasurer::vertical_center(style, m.row_h),
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
