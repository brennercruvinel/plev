//! Table page: the static table, every row shown, no scroll, in the
//! showcase mold. The rows are the kit's one table, the spec sheet of its
//! typography page, and the table draws as that sheet does in the core
//! theme (a frame, the titles on no band, a rule over every row); every
//! visual value comes from the theme (`theme.disclosure.table`). The
//! widget is `engine::widgets::content::Table`, the Data Table's, with
//! every row in its rect: a static table never scrolls, so the page keeps
//! the wheel. A narrow card drops the columns of least priority, as the
//! Data Table does.
//!
//! Projected: a group per title, `table_static_page.group[<g>]`; in it
//! each card's table, `...card[<c>]`, named by its chip, with a row per
//! row, `...card[<c>][<row>]`, named by its cells, whose `select` selects
//! it as a click does. A table is one stop of Tab; the arrows walk its
//! rows.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::content::{Column, EmptyState, Table};
use engine::widgets::prelude::{Align, EventResult, Rect, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, key_under, root_id, showcase, under};

/// What a table without rows shows under its header: the empty state,
/// its icon the grid's.
fn no_rows() -> EmptyState {
    EmptyState::new("No rows yet", "Rows you add to the sheet show up here.").icon("layout-grid")
}

/// A row of the sheet: style, font, weight, size and line height, letter
/// spacing.
type Row = [&'static str; 5];

/// The kit's spec sheet (templates/TypographyPage/index.tsx:24-150).
const SHEET: &[Row] = &[
    ["h1", "Inter", "Regular", "48/115%", "-3%"],
    ["h2", "Inter", "Regular", "40/48", "-3%"],
    ["h3", "Inter", "Regular", "32/40", "-3%"],
    ["h4", "Inter", "Regular", "28/36", "-3%"],
    ["h5", "Inter", "Regular", "24/32", "-3%"],
    ["h6", "Inter", "Regular", "20/28", "-3%"],
    ["Body, X Small", "Inter", "Regular", "10/16", "-1%"],
    ["Body, Small", "Inter", "Medium", "11/16", "-1%"],
    ["Body, Medium", "Inter", "Medium", "12/16", "-1%"],
    ["Body Medium, Strong", "Inter", "Semibold", "12/16", "-1%"],
];

/// Rows whose text is longer than any cell: each cut to its cell.
const LONG: &[Row] = &[
    [
        "Paragraph, small, the body of a long comment thread",
        "Inter Variable, optical size",
        "Regular and Medium",
        "13/20 on mobile, 14/20 on desktop",
        "-1% on every size",
    ],
    [
        "Title, Large, the heading of a settings dialog",
        "Inter Display",
        "Semibold",
        "24/32 on desktop",
        "-2% under 24 px",
    ],
];

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    rows: &'static [Row],
    table: Table,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct TableStaticPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

/// `table_static_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `table_static_page.group[<g>].card[<c>]`, the card's table.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// The sheet's columns: the style never drops; then the size, the weight,
/// the font, and the letter spacing first.
fn columns(theme: &Theme) -> Vec<Column> {
    let min = theme.size.field_min_w;
    vec![
        Column::new("Style", theme).weight(2.0).min_w(min),
        Column::new("Font", theme).min_w(min / 2.0).priority(1),
        Column::new("Font weight", theme)
            .min_w(min / 2.0)
            .priority(2),
        Column::new("Size/Line height", theme)
            .min_w(min / 2.0)
            .priority(3),
        Column::new("Letter spacing", theme)
            .min_w(min / 2.0)
            .priority(0),
    ]
}

impl TableStaticPage {
    pub fn new(theme: &Theme) -> Self {
        let card = |title, rows: &'static [Row]| {
            let mut table = Table::new(columns(theme), theme);
            table.set_row_count(rows.len());
            if rows.is_empty() {
                table = table.empty(no_rows());
            }
            Card { title, rows, table }
        };
        let mut groups = vec![
            Group {
                title: "Table",
                columns: 1,
                cards: vec![{
                    let mut sheet = card(
                        "Default, Hover, Selected, Focus: the kit's type sheet",
                        SHEET,
                    );
                    sheet.table.rows.selected = Some(1);
                    sheet
                }],
            },
            Group {
                title: "Table, ranges",
                columns: 3,
                cards: vec![
                    card("Narrow: columns drop by priority", &SHEET[..4]),
                    card("Long content: cells cut", LONG),
                    card("One row: the frame holds it", &SHEET[..1]),
                    card("Empty: no rows", &[]),
                ],
            },
        ];
        let root = root_id("table_static_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        let table = std::mem::replace(&mut card.table, Table::new(vec![], theme));
                        card.table = table.sem(id);
                    }
                }
            }
        }
        Self { root, groups }
    }

    /// Every row in the table's rect; without rows, its header and its
    /// empty state `w` wide.
    fn table_h(card: &Card, w: f32, theme: &Theme) -> f32 {
        if card.rows.is_empty() {
            return card.table.empty_height(w, theme);
        }
        Table::header_h(theme) + card.rows.len() as f32 * Table::row_h(theme)
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.title, g.columns, g.cards.len()))
            .collect();
        fit_layout(&spans, content, theme, |g, c, w| {
            Self::table_h(&self.groups[g].cards[c], w, theme)
        })
    }

    /// The table of card `(g, c)` and the row behind `id` under it, the
    /// row in range or not.
    fn row_of(&self, id: &SemId) -> Option<(usize, usize, Option<usize>)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let table = card.table.sem_id()?;
                if table == id {
                    return Some((g, c, None));
                }
                let row: usize = key_under(table, id)?.parse().ok()?;
                (row < card.rows.len()).then_some((g, c, Some(row)))
            })
        })
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |placed| placed.rule_y - content.y)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        // Every row is in view: the wheel is the page's.
        if matches!(event, WidgetEvent::Scroll { .. }) {
            return EventResult::IGNORED;
        }
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (group, placed) in self.groups.iter_mut().zip(&layout) {
            for (card, (_, body)) in group.cards.iter_mut().zip(&placed.cards) {
                let at = Rect::new(body.x, body.y, body.w, Self::table_h(card, body.w, theme));
                result = result.merge(card.table.handle_event(event, at, theme));
            }
        }
        result
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        let look = theme.disclosure.table;
        for (i, (group, placed)) in self.groups.iter_mut().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for (card, (rect, body)) in group.cards.iter_mut().zip(&placed.cards) {
                showcase::card(c, *rect, card.title, theme);
                let at = Rect::new(body.x, body.y, body.w, Self::table_h(card, body.w, theme));
                let rows = card.rows;
                card.table.render_with(c, at, theme, |c, row, cell, col| {
                    let color = if col == 0 { look.cell } else { look.cell_dim };
                    let text = rows.get(row).map_or("", |r| r[col]);
                    let (layer, align) = (LayerId::DEFAULT, Align::Start);
                    Table::draw_cell(c, layer, cell, text, align, color.0, theme);
                });
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for TableStaticPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                let Some(id) = card.table.sem_id() else {
                    continue;
                };
                let rows = card.rows;
                let label = |row: usize| rows.get(row).map_or(String::new(), |r| r.join(", "));
                card.table.project(b, id, card.title, Some(&at), label);
                if let Some(row) = card.table.rows.focused() {
                    b.focus(id.key(row.to_string()));
                }
            }
        }
    }

    /// A row's `select` selects it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.row_of(&req.target), req.action.as_str()) {
            (Some((g, c, Some(row))), "select") => {
                self.groups[g].cards[c].table.rows.selected = Some(row);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the table page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// A row takes the focus (its ring); any other node of the page takes
    /// it from every table.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self.row_of(id);
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let row = at.filter(|(fg, fc, _)| (*fg, *fc) == (g, c));
                card.table.rows.set_focused(row.and_then(|(_, _, r)| r));
            }
        }
        true
    }

    fn blur(&mut self) -> bool {
        let mut had = false;
        for card in self.groups.iter_mut().flat_map(|g| g.cards.iter_mut()) {
            had |= card.table.rows.focused().is_some();
            card.table.rows.set_focused(None);
        }
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff:
    /// every table inside its card, its style column always shown, the
    /// columns apart and inside the table, and fewer of them in a narrow
    /// card than in the wide one.
    #[test]
    fn every_table_fits_its_card_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = TableStaticPage::new(&theme);
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                        let what = format!("{w}px, {}", card.title);
                        let h = TableStaticPage::table_h(card, body.w, &theme);
                        assert!(body.x >= rect.x && body.right() <= rect.right(), "{what}");
                        assert!(body.y + h <= rect.bottom() + 0.01, "{what}: too tall");
                        let cols = card.table.column_layout(body.w, &theme);
                        assert_eq!(cols.visible[0], 0, "{what}: the style stays");
                        let mut end = 0.0;
                        for (x, cw) in &cols.spans {
                            assert!(*x >= end && x + cw <= body.w + 0.01, "{what}");
                            end = x + cw;
                        }
                    }
                }
                let narrow = page.layout(content, &theme)[1].cards[0].1.w;
                let wide = page.layout(content, &theme)[0].cards[0].1.w;
                let count = |w| page.groups[0].cards[0].table.column_layout(w, &theme);
                assert!(count(narrow).visible.len() <= count(wide).visible.len());
            }
            let small = page.groups[0].cards[0].table.column_layout(280.0, &theme);
            assert!(small.visible.len() < 5, "a phone drops columns");
        }
    }

    /// A static table keeps no wheel: the page scrolls over it.
    #[test]
    fn the_wheel_over_a_table_is_the_page_s() {
        let theme = Theme::hoff_core();
        let mut page = TableStaticPage::new(&theme);
        let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let (_, body) = page.layout(content, &theme)[0].cards[0];
        let (x, y) = (body.x + 10.0, body.y + 60.0);
        let wheel = WidgetEvent::Scroll { x, y, delta: 40.0 };
        assert!(!page.handle_event(&wheel, content, &theme).handled);
    }

    #[test]
    fn a_click_on_a_row_selects_it() {
        let theme = Theme::hoff_core();
        let mut page = TableStaticPage::new(&theme);
        let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let (_, body) = page.layout(content, &theme)[0].cards[0];
        let y = body.y + Table::header_h(&theme) + Table::row_h(&theme) * 2.5;
        let down = WidgetEvent::MouseDown {
            x: body.x + 10.0,
            y,
        };
        let mut c = Compositor::new();
        c.begin_frame();
        page.render(&mut c, content, &theme);
        assert!(page.handle_event(&down, content, &theme).clicked);
        assert_eq!(page.groups[0].cards[0].table.selected(), Some(2));
    }
}
