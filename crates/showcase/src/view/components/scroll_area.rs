//! Scroll Area page: a 10,000-row virtualized list with the fading
//! scrollbar. Rows draw on a clipped layer the view sets up.
//!
//! Projected (task-75 phase 2b): the list, `scroll_area_page.list`, named
//! "items", with an item per row it holds (never the 10,000), as the comps
//! projects them, each named by its text; an item's `select` selects it as
//! a click does.

use comps::icons;
use comps::prelude::{EventResult, Rect, VirtualList, WidgetEvent};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::theme::Theme;
use sem::{Request, SemId, SnapshotBuilder};

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
        let mut list = VirtualList::new(theme.control.height(engine::theme::ControlSize::Xs));
        list.set_item_count(10_000);
        let root = root_id("scroll_area_page");
        let list = match child_id(root.as_ref(), "list") {
            Some(id) => list.sem(id),
            None => list,
        };
        Self { root, list }
    }

    /// The row behind `id`, `scroll_area_page.list[<index>]`, while the
    /// list holds it.
    fn row_at(&self, id: &SemId) -> Option<usize> {
        let row = key_under(self.list.sem_id()?, id)?.parse().ok()?;
        self.list.visible_range().contains(&row).then_some(row)
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
        self.list
            .handle_event(event, Self::bounds(content, theme), theme)
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
        let bounds = Self::bounds(content, theme);
        panel(c, bounds.outset(1.0), theme);
        let dim = theme.colors.text_dim.0;
        let fg = theme.colors.text.0;
        let mid = theme.colors.text_mid.0;
        let body = theme.typography.base_2r();
        let small = theme.typography.caption_r();
        let icon = theme.control.icon(engine::theme::IconSize::Sm);
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
                let (tag_w, _) = engine::text::TextMeasurer::measure_styled(&tag, &small, None);
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
            let style = theme.typography.caption_r();
            text(
                c,
                &format!("selected: Item {i}"),
                style.font_size,
                style.font_weight,
                bounds.x,
                bounds.bottom() + theme.spacing.sm,
                dim,
            );
        }
    }
}

impl Component for ScrollAreaPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        if let Some(id) = self.list.sem_id() {
            self.list.project(b, id, "items", Some(root), row_text);
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

    /// Nothing of the list takes focus (an item does not); the page's
    /// nodes are its own all the same.
    fn focus(&mut self, id: &SemId) -> bool {
        self.root.as_ref().is_some_and(|root| under(root, id))
    }
}
