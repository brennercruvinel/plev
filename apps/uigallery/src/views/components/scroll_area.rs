//! Scroll Area page: a 10,000-row virtualized list with the fading
//! scrollbar. Rows draw on a clipped layer the view sets up.
//!
//! Projected (task-75 phase 2b): the list, `scroll_area_page.list`, named
//! "items", with an item per row it holds (never the 10,000), as the comps
//! projects them, each named by its text; an item's `select` selects it as
//! a click does. One item is the list's stop in the Tab sequence; the
//! arrow keys move the focus along the items, the list scrolling to keep
//! the focused one in view, and the focus follows the wheel (task-75
//! phase 2d).

use accord::{Request, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::styling::theme::Theme;
use engine::styling::theme::TypeRole;
use engine::widgets::icons;
use engine::widgets::prelude::{EventResult, Rect, VirtualList, WidgetEvent};

use super::{After, Component, child_id, key_under, panel, root_id, text, under};

/// What row `index` says: one source for the drawing and the projection.
fn row_text(index: usize) -> String {
    format!("Item {index}")
}

pub struct ScrollAreaPage {
    root: Option<SemId>,
    pub list: VirtualList,
}

impl ScrollAreaPage {
    pub fn new(theme: &Theme) -> Self {
        let mut list = VirtualList::new(Self::row_h(theme));
        list.set_item_count(10_000);
        let root = root_id("scroll_area_page");
        let list = match child_id(root.as_ref(), "list") {
            Some(id) => list.sem(id),
            None => list,
        };
        Self { root, list }
    }

    /// A row: one `Xs` control, in the theme the page draws in.
    fn row_h(theme: &Theme) -> f32 {
        theme
            .control
            .height(engine::styling::theme::ControlSize::Xs)
    }

    /// The row behind `id`, `scroll_area_page.list[<index>]`, while the
    /// list holds it.
    fn row_at(&self, id: &SemId) -> Option<usize> {
        let row = key_under(self.list.sem_id()?, id)?.parse().ok()?;
        self.list.visible_range().contains(&row).then_some(row)
    }

    /// The item behind `id`, `scroll_area_page.list[<index>]`, held or
    /// not: the focus may go to any, and the list scrolls to it.
    fn item_at(&self, id: &SemId) -> Option<usize> {
        let row: usize = key_under(self.list.sem_id()?, id)?.parse().ok()?;
        (row < self.list.item_count()).then_some(row)
    }

    /// The list's bounds: the readable width, the viewport's height.
    pub fn bounds(content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            content.w.min(theme.size.readable_max_w),
            content.h - theme.spacing.lg,
        )
    }

    /// The page sizes itself to the viewport; the list scrolls inside.
    pub fn content_height(&self, content: Rect, _theme: &Theme) -> f32 {
        content.h
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.list.set_item_height(Self::row_h(theme));
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let r = self
            .list
            .handle_event(event, Self::bounds(content, theme), theme);
        if blurred {
            return r.merge(EventResult::changed());
        }
        r
    }

    /// Scrollbar fade. `true` while animating.
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
        self.list.set_item_height(Self::row_h(theme));
        let bounds = Self::bounds(content, theme);
        panel(c, bounds.outset(1.0), theme);
        let dim = theme.colors.text_dim.0;
        let fg = theme.colors.text.0;
        let mid = theme.colors.text_mid.0;
        let body = theme.typography.role(TypeRole::Body);
        let small = theme.typography.run(TypeRole::Caption.regular());
        let icon = theme.control.icon(engine::styling::theme::IconSize::Sm);
        let pad = theme.spacing.md;
        self.list.render_with_to_layer(
            c,
            list_layer,
            bounds,
            theme,
            |c, index, rect, _hovered, selected| {
                let name = if index % 9 == 0 { "folder" } else { "file" };
                if let Some(node) = icons::icon_at(
                    name,
                    icon,
                    mid,
                    rect.x + pad,
                    rect.y + (rect.h - icon) / 2.0,
                ) {
                    c.push_to_layer(list_layer, node);
                }
                c.push_to_layer(
                    list_layer,
                    SceneNode::Text {
                        key: TextNodeKey::from_style(&row_text(index), &body, None)
                            .with_weight(if selected { 600 } else { body.font_weight }),
                        x: rect.x + pad + icon + theme.spacing.sm,
                        y: rect.y + (rect.h - body.line_height) / 2.0,
                        color: fg,
                    },
                );
                let tag = format!("#{:04}", index % 10_000);
                let (tag_w, _) =
                    engine::drawing::text::TextMeasurer::measure_styled(&tag, &small, None);
                c.push_to_layer(
                    list_layer,
                    SceneNode::Text {
                        key: TextNodeKey::from_style(&tag, &small, None),
                        x: rect.right() - pad - tag_w - theme.control.scrollbar_w_hover,
                        y: rect.y + (rect.h - small.line_height) / 2.0,
                        color: dim,
                    },
                );
            },
        );
        if let Some(i) = self.list.selected {
            text(
                c,
                &format!("selected: Item {i}"),
                TypeRole::Caption.regular(),
                (bounds.x, bounds.bottom() + theme.spacing.sm),
                dim,
                theme,
            );
        }
    }
}

impl Component for ScrollAreaPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(id) = self.list.sem_id() else {
            return;
        };
        self.list.project(b, id, "items", Some(root), row_text);
        if let Some(at) = self.list.focused() {
            b.focus(id.key(at.to_string()));
        }
    }

    /// An item's `select` selects it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.row_at(&req.target), req.action.as_str()) {
            (Some(row), "select") => {
                self.list.selected = Some(row);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the scroll area page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// An item takes the focus (its ring), scrolled into view; another
    /// node of the page takes it from the list.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.list.set_focused(self.item_at(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.list.focused().is_some();
        self.list.set_focused(None);
        had
    }
}
