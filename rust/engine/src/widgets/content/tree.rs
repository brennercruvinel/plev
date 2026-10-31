use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::icons;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::list::reachable;
use super::tokens::{DisclosureSurface, TreeRecipe};
use crate::widgets::recipe::{focus_ring_inside, rounded_rect, rounded_rect_stroke, shadow_node};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// The row height the hit test and the render share
/// (`theme.disclosure.tree`).
fn row_h(theme: &Theme) -> f32 {
    theme.disclosure.tree.row_h
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
/// [`VirtualList`](crate::widgets::content::VirtualList)-style clipping for huge trees).
///
/// With a [`SemId`] ([`Tree::sem`]) the render binds the tree and each row
/// it draws to the rects it hit-tests them with, and [`Tree::project`]
/// declares the tree in a semantic projection.
#[derive(Clone, Debug)]
pub struct Tree {
    pub roots: Vec<TreeNode>,
    pub selected: Option<u64>,
    hovered_row: Option<usize>,
    focused: Option<u64>,
    sem: Option<SemId>,
}

impl Tree {
    pub fn new(roots: Vec<TreeNode>) -> Self {
        Self {
            roots,
            selected: None,
            hovered_row: None,
            focused: None,
            sem: None,
        }
    }

    /// The logical identity the render binds the tree to
    /// (`Compositor::bind`); each drawn row binds as `<id>[<node id>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Keyboard focus on the row of node `id`, or on none, driven by the
    /// owning view (plev has no global focus chain): the accent ring inside
    /// that row while it shows (a row under a collapsed branch shows none).
    pub fn set_focused(&mut self, id: Option<u64>) {
        self.focused = id;
    }

    pub fn focused(&self) -> Option<u64> {
        self.focused
    }

    /// Declares this tree as `id` in a projection: a `Role::Tree` named
    /// `label` and one `Role::TreeItem` `<id>[<node id>]` per visible row
    /// (the children of collapsed branches are not rows until expanded),
    /// nested as the tree nests them: a root's parent is the tree, a
    /// child's parent is its branch's item. Each item carries `selected`
    /// and a `select` that reads it; a branch also carries `expanded` and
    /// a `toggle_expanded` that reads it. Hover is never projected. It only
    /// reads the widget; the app owns it and runs the actions
    /// ([`Tree::toggle`], `selected`). Pass the id given to [`Tree::sem`]
    /// so the nodes and the bindings name the same controls. Returns the
    /// tree's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        // Depth first, siblings in order: the order the rows draw in.
        let mut stack: Vec<(&TreeNode, SemId)> =
            self.roots.iter().rev().map(|n| (n, id.clone())).collect();
        while let Some((node, owner)) = stack.pop() {
            let item = id.key(node.id.to_string());
            let mut row = b.node(&item, Role::TreeItem, node.label.as_str());
            row.parent(&owner)
                .state("selected", self.selected == Some(node.id));
            if node.is_branch() {
                row.state("expanded", node.expanded);
            }
            row.action("select").reads(format!("{item}:selected"));
            if node.is_branch() {
                row.action("toggle_expanded")
                    .reads(format!("{item}:expanded"));
                if node.expanded {
                    stack.extend(node.children.iter().rev().map(|c| (c, item.clone())));
                }
            }
        }
        let mut node = b.node(id, Role::Tree, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }

    pub fn row_height(&self, theme: &Theme) -> f32 {
        row_h(theme)
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
        self.visible_rows().len() as f32 * row_h(theme)
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
        let i = ((y - bounds.y) / row_h(theme)).floor();
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
        let r = &theme.disclosure.tree;
        let style = theme
            .typography
            .exact(r.font_size, r.line_height, r.font_weight);
        if let Some(id) = &self.sem {
            compositor.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        for (i, row) in self.visible_rows().iter().enumerate() {
            let ry = bounds.y + i as f32 * r.row_h;
            if ry + r.row_h > bounds.y + bounds.h + r.row_h {
                break;
            }
            let row_rect = Rect::new(bounds.x, ry, bounds.w, r.row_h);
            // The row binds where the hit test finds it: inside the bounds.
            if let Some(id) = &self.sem
                && let Some(at) = reachable(row_rect, bounds)
            {
                compositor.bind(&id.key(row.id.to_string()), [at.x, at.y, at.w, at.h]);
            }
            let is_selected = self.selected == Some(row.id);
            let is_hovered = self.hovered_row == Some(i);
            if is_selected || is_hovered {
                let surface = if is_selected { &r.selected } else { &r.hover };
                paint_surface(compositor, row_rect, r, surface);
            }
            if self.focused == Some(row.id) {
                let radius = r.radius.min(row_rect.h / 2.0);
                compositor.push(focus_ring_inside(row_rect, radius, theme));
            }
            let span = (bounds.x, bounds.right());
            self.paint_row(compositor, row, span, ry, is_selected, &style, r);
        }
    }

    /// The chevron, the icon (in its tile) and the label of a row at
    /// `ry`, indented by its depth, between the bounds' left and right
    /// edges `x`: a label too long for what is left is cut.
    #[allow(clippy::too_many_arguments)]
    fn paint_row(
        &self,
        compositor: &mut Compositor,
        row: &TreeRow,
        (x, right): (f32, f32),
        ry: f32,
        selected: bool,
        style: &TextStyle,
        r: &TreeRecipe,
    ) {
        let mut cx = x + r.pad_x + row.depth as f32 * r.indent;
        if row.is_branch {
            let chevron = if row.expanded {
                "chevron-down"
            } else {
                "chevron-right"
            };
            let y = ry + (r.row_h - r.chevron) / 2.0;
            let color = with_alpha(r.chevron_color, 1.0);
            if let Some(node) = icons::icon_at(chevron, r.chevron, color, cx, y) {
                compositor.push(node);
            }
        }
        cx += r.chevron + r.chevron_gap;

        let icon_name = row.icon.unwrap_or(if row.is_branch {
            if row.expanded {
                "folder-open"
            } else {
                "folder"
            }
        } else {
            "file"
        });
        let mut box_w = r.icon;
        if let Some(tile) = &r.tile {
            let ty = ry + (r.row_h - tile.size) / 2.0;
            let tile_rect = Rect::new(cx, ty, tile.size, tile.size);
            if selected
                && let Some(node) = shadow_node(tile_rect, tile.radius, &tile.shadow_selected)
            {
                compositor.push(node);
            }
            let fill = if selected { tile.selected } else { tile.rest };
            let radius = tile.radius.min(tile.size / 2.0);
            compositor.push(rounded_rect(cx, ty, tile.size, tile.size, radius, fill.0));
            box_w = tile.size;
        }
        let icon_color = if selected {
            r.icon_selected
        } else {
            r.icon_color
        };
        let (ix, iy) = (cx + (box_w - r.icon) / 2.0, ry + (r.row_h - r.icon) / 2.0);
        let color = with_alpha(icon_color, 1.0);
        if let Some(node) = icons::icon_at(icon_name, r.icon, color, ix, iy) {
            compositor.push(node);
        }
        cx += box_w + r.label_gap;

        let label = if selected { r.label_selected } else { r.label };
        let room = (right - r.pad_x - cx).max(0.0);
        let shown = TextMeasurer::truncate_to_width(&row.label, style, room);
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(&shown, style, None),
            x: cx,
            y: ry + TextMeasurer::vertical_center(style, r.row_h),
            color: with_alpha(label, 1.0),
        });
    }
}

/// The surface under a hovered or selected row, inset by the seam.
fn paint_surface(c: &mut Compositor, row: Rect, r: &TreeRecipe, surface: &DisclosureSurface) {
    let [sx, sy] = r.seam;
    let at = Rect::new(row.x + sx, row.y + sy, row.w - sx * 2.0, row.h - sy * 2.0);
    let radius = r.radius.min(row.h / 2.0);
    c.push(rounded_rect(at.x, at.y, at.w, at.h, radius, surface.fill.0));
    if r.border > 0.0 && surface.border.0[3] > 0.0 {
        let edge = surface.border.0;
        c.push(rounded_rect_stroke(
            at.x, at.y, at.w, at.h, radius, edge, r.border,
        ));
    }
}
