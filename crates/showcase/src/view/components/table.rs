//! Data Table page: virtualized rows on a panel; columns drop by
//! priority when the page is narrow.

use comps::content::{Column, Panel, Table};
use comps::prelude::{Align, EventResult, Rect, WidgetEvent};
use engine::compositor::{Compositor, LayerId};
use engine::theme::Theme;

pub struct TablePage {
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
        Self { table }
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
        self.table
            .handle_event(event, Self::rect(content, theme), theme)
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.table.tick(dt)
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        Panel::new().render(c, rect.outset(theme.spacing.sm), theme);
        let text = theme.colors.text_mid.0;
        let dim = theme.glass.text_faint.0;
        self.table.render_with(c, rect, theme, |c, row, cell, col| {
            let (label, align, color) = match col {
                0 => (
                    format!("crates/comps/src/file_{row}.rs"),
                    Align::Start,
                    text,
                ),
                1 => (
                    ["modified", "added", "renamed"][row % 3].to_string(),
                    Align::Start,
                    dim,
                ),
                2 => (format!("{} KB", (row * 37) % 900 + 3), Align::End, dim),
                _ => (
                    format!("{:07x}", row * 2654435761u64 as usize % 0xfffffff),
                    Align::Start,
                    dim,
                ),
            };
            Table::draw_cell(c, LayerId::DEFAULT, cell, &label, align, color, theme);
        });
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
