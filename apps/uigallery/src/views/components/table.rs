//! Data Table page: virtualized rows on a panel; columns drop by
//! priority when the page is narrow.
//!
//! Projected (task-75 phase 2b): the table, `table_page.table`, named
//! "files", with a row per row it holds (never the 2000), as the comps
//! projects them, each named by its cells; a row's `select` selects it as a
//! click does. One row is the table's stop in the Tab sequence; the arrow
//! keys move the focus along the rows, the table scrolling to keep the
//! focused one in view, and the focus follows the wheel (task-75 phase
//! 2d).

use accord::{Request, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::content::{Column, Panel, Table};
use engine::widgets::prelude::{Align, EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, key_under, root_id, under};

/// What row `row` says in column `col`: the page's sample data, one
/// source for the drawing and the projection.
fn cell(row: usize, col: usize) -> String {
    match col {
        0 => format!("crates/comps/src/file_{row}.rs"),
        1 => ["modified", "added", "renamed"][row % 3].to_string(),
        2 => format!("{} KB", (row * 37) % 900 + 3),
        // in u64: a usize of 32 bits (wasm32) wrapped the product, and the
        // web drew other ids than the desktop (task-77)
        _ => format!("{:07x}", row as u64 * 2654435761 % 0xfffffff),
    }
}

/// The columns, in order.
const COLUMNS: usize = 4;

pub struct TablePage {
    root: Option<SemId>,
    table: Table,
}

impl TablePage {
    pub fn new(theme: &Theme) -> Self {
        let mut table = Table::new(
            vec![
                Column::new("file", theme)
                    .weight(2.0)
                    .min_w(theme.size.field_min_w),
                Column::new("status", theme)
                    .min_w(theme.size.field_min_w / 2.0)
                    .priority(2),
                Column::new("size", theme)
                    .align(Align::End)
                    .min_w(theme.size.field_min_w / 2.0)
                    .priority(1),
                Column::new("hash", theme)
                    .min_w(theme.size.field_min_w)
                    .priority(0),
            ],
            theme,
        );
        table.set_row_count(2_000);
        let root = root_id("table_page");
        let table = match child_id(root.as_ref(), "table") {
            Some(id) => table.sem(id),
            None => table,
        };
        Self { root, table }
    }

    /// The row behind `id`, `table_page.table[<row>]`, while the table
    /// holds it.
    fn row_at(&self, id: &SemId) -> Option<usize> {
        let row = key_under(self.table.sem_id()?, id)?.parse().ok()?;
        self.table
            .rows
            .visible_range()
            .contains(&row)
            .then_some(row)
    }

    /// The row behind `id`, `table_page.table[<row>]`, held or not: the
    /// focus may go to any, and the table scrolls to it.
    fn any_row(&self, id: &SemId) -> Option<usize> {
        let row: usize = key_under(self.table.sem_id()?, id)?.parse().ok()?;
        (row < self.table.row_count()).then_some(row)
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        let h = Table::header_h(theme) + Table::row_h(theme) * 8.0;
        Rect::new(content.x, content.y, content.w, h)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        Self::rect(content, theme).h + theme.spacing.sm
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let r = self
            .table
            .handle_event(event, Self::rect(content, theme), theme);
        if blurred {
            return r.merge(EventResult::changed());
        }
        r
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.table.tick(dt)
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        Panel::new().render(c, rect.outset(theme.spacing.sm), theme);
        let text = theme.colors.text_mid.0;
        let dim = theme.glass.text_faint.0;
        self.table.render_with(c, rect, theme, |c, row, at, col| {
            let (align, color) = match col {
                0 => (Align::Start, text),
                2 => (Align::End, dim),
                _ => (Align::Start, dim),
            };
            Table::draw_cell(
                c,
                LayerId::DEFAULT,
                at,
                &cell(row, col),
                align,
                color,
                theme,
            );
        });
    }
}

impl Component for TablePage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(id) = self.table.sem_id() else {
            return;
        };
        let row_label = |row| {
            (0..COLUMNS)
                .map(|col| cell(row, col))
                .collect::<Vec<_>>()
                .join(", ")
        };
        self.table.project(b, id, "files", Some(root), row_label);
        if let Some(at) = self.table.rows.focused() {
            b.focus(id.key(at.to_string()));
        }
    }

    /// A row's `select` selects it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.row_at(&req.target), req.action.as_str()) {
            (Some(row), "select") => {
                self.table.rows.selected = Some(row);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the table page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// A row takes the focus (its ring), scrolled into view; another node
    /// of the page takes it from the table.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let row = self.any_row(id);
        self.table.rows.set_focused(row);
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.table.rows.focused().is_some();
        self.table.rows.set_focused(None);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_drops_columns_when_the_page_is_narrow() {
        let theme = Theme::hoff();
        let page = TablePage::new(&theme);
        let wide = page.table.column_layout(1272.0, &theme);
        assert_eq!(wide.visible.len(), 4);
        let narrow = page.table.column_layout(340.0, &theme);
        assert!(narrow.visible.len() < 4);
        assert_eq!(narrow.visible[0], 0, "the file column never drops");
    }
}
