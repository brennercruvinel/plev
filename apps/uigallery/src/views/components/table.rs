//! Data Table page: virtualized rows in the showcase mold. Each card
//! holds a table as wide as the card and a few rows tall, over as many
//! rows as its dataset has (2,000 for the first); the rows scroll inside
//! it and are clipped to it, and a narrow card drops the columns of least
//! priority. The look is the theme's (`theme.disclosure.table`): the
//! kit's spec sheet in the core theme, the glass band elsewhere.
//!
//! Projected (task-75 phase 2b): a group per title,
//! `table_page.group[<g>]`; in it each card's table, named by its chip,
//! with a row per row it holds (never the 2,000), each named by its
//! cells; a row's `select` selects it as a click does. The first table
//! keeps the id it had before the mold, `table_page.table`; the others
//! are `table_page.group[<g>].card[<c>]`. A table is one stop in the Tab
//! sequence; the arrow keys move the focus along its rows, the table
//! scrolling to keep the focused one in view, and the focus follows the
//! wheel (task-75 phase 2d).

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::styling::theme::Theme;
use engine::widgets::content::{Column, EmptyState, Table};
use engine::widgets::prelude::{Align, EventResult, Rect, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, key_under, root_id, showcase, under};

/// What row `row` of the files under `dir` says in column `col`: the
/// page's sample data, one source for the drawing and the projection.
fn cell(dir: &str, row: usize, col: usize) -> String {
    match col {
        0 => format!("crates/{dir}/src/file_{row}.rs"),
        1 => ["modified", "added", "renamed"][row % 3].to_string(),
        2 => format!("{} KB", (row * 37) % 900 + 3),
        // in u64: a usize of 32 bits (wasm32) wrapped the product, and the
        // web drew other ids than the desktop (task-77)
        _ => format!("{:07x}", row as u64 * 2654435761 % 0xfffffff),
    }
}

/// The columns, in order.
const COLUMNS: usize = 4;

/// What a table without rows shows under its header: the empty state,
/// its icon the grid's.
fn no_rows() -> EmptyState {
    EmptyState::new("No rows yet", "Rows land here once the dataset has any.").icon("layout-grid")
}

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    /// The directory its files are under.
    dir: &'static str,
    /// How many rows tall its rect is.
    shown: usize,
    table: Table,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct TablePage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

/// `table_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// The table of card `(g, c)`: `table_page.table` for the first,
/// `table_page.group[<g>].card[<c>]` for the others.
fn table_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    if (g, c) == (0, 0) {
        return child_id(Some(root), "table");
    }
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// The files' columns: the file never drops; the hash drops first, then
/// the size, then the status.
fn columns(theme: &Theme) -> Vec<Column> {
    let min = theme.size.field_min_w;
    vec![
        Column::new("file", theme).weight(2.0).min_w(min),
        Column::new("status", theme).min_w(min / 2.0).priority(2),
        Column::new("size", theme)
            .align(Align::End)
            .min_w(min / 2.0)
            .priority(1),
        Column::new("hash", theme).min_w(min).priority(0),
    ]
}

impl TablePage {
    pub fn new(theme: &Theme) -> Self {
        let card = |title, dir, rows, shown| {
            let mut table = Table::new(columns(theme), theme);
            table.set_row_count(rows);
            if rows == 0 {
                table = table.empty(no_rows());
            }
            Card {
                title,
                dir,
                shown,
                table,
            }
        };
        let mut groups = vec![
            Group {
                title: "Data Table",
                columns: 1,
                cards: vec![card(
                    "Default, Hover, Selected, Focus, Scroll: 2,000 rows",
                    "comps",
                    2_000,
                    8,
                )],
            },
            Group {
                title: "Data Table, ranges",
                columns: 2,
                cards: vec![
                    card("Narrow: columns drop by priority", "screen", 500, 5),
                    card("Few rows: nothing to scroll", "accord", 3, 5),
                    card("Empty: no rows", "accord", 0, 0),
                ],
            },
        ];
        let root = root_id("table_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = table_id(root, g, c) {
                        let table = std::mem::replace(&mut card.table, Table::new(vec![], theme));
                        card.table = table.sem(id);
                    }
                }
            }
        }
        Self { root, groups }
    }

    /// The header and the rows it shows; without rows, the header and
    /// its empty state `w` wide.
    fn table_h(card: &Card, w: f32, theme: &Theme) -> f32 {
        if card.table.row_count() == 0 {
            return card.table.empty_height(w, theme);
        }
        Table::header_h(theme) + card.shown as f32 * Table::row_h(theme)
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

    /// Each table's rect, in the order of the cards.
    fn rects(&self, content: Rect, theme: &Theme) -> Vec<Rect> {
        let layout = self.layout(content, theme);
        let cards = self.groups.iter().zip(&layout).flat_map(|(group, placed)| {
            group
                .cards
                .iter()
                .zip(&placed.cards)
                .map(|(card, (_, body))| {
                    Rect::new(body.x, body.y, body.w, Self::table_h(card, body.w, theme))
                })
        });
        cards.collect()
    }

    fn cards_mut(&mut self) -> impl Iterator<Item = &mut Card> {
        self.groups.iter_mut().flat_map(|g| g.cards.iter_mut())
    }

    /// The card whose table holds the row behind `id`, and the row, held
    /// or not: the focus may go to any, and the table scrolls to it.
    fn row_of(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let row: usize = key_under(card.table.sem_id()?, id)?.parse().ok()?;
                (row < card.table.row_count()).then_some((g, c, row))
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
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let rects = self.rects(content, theme);
        for (card, at) in self.cards_mut().zip(rects) {
            result = result.merge(card.table.handle_event(event, at, theme));
        }
        result
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for card in self.cards_mut() {
            moving |= card.table.tick(dt);
        }
        moving
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
                // the rows overscan past the rect: they end at its edge
                c.push(SceneNode::PushClip {
                    x: at.x,
                    y: at.y,
                    w: at.w,
                    h: at.h,
                });
                let dir = card.dir;
                card.table
                    .render_with(c, at, theme, |c, row, cell_at, col| {
                        let (align, color) = match col {
                            0 => (Align::Start, look.cell),
                            2 => (Align::End, look.cell_dim),
                            _ => (Align::Start, look.cell_dim),
                        };
                        let text = cell(dir, row, col);
                        let layer = LayerId::DEFAULT;
                        Table::draw_cell(c, layer, cell_at, &text, align, color.0, theme);
                    });
                c.push(SceneNode::PopClip);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for TablePage {
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
                let dir = card.dir;
                let row_label = |row| {
                    (0..COLUMNS)
                        .map(|col| cell(dir, row, col))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                card.table.project(b, id, card.title, Some(&at), row_label);
                if let Some(row) = card.table.rows.focused() {
                    b.focus(id.key(row.to_string()));
                }
            }
        }
    }

    /// A row's `select` selects it, as its click does, while the table
    /// holds it.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let held = self.row_of(&req.target).filter(|(g, c, row)| {
            let table = &self.groups[*g].cards[*c].table;
            table.rows.visible_range().contains(row)
        });
        match (held, req.action.as_str()) {
            (Some((g, c, row)), "select") => {
                self.groups[g].cards[c].table.rows.selected = Some(row);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the table page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// A row takes the focus (its ring), scrolled into view; another node
    /// of the page takes it from every table.
    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let at = self.row_of(id);
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let row = at.filter(|(fg, fc, _)| (*fg, *fc) == (g, c));
                card.table.rows.set_focused(row.map(|(_, _, r)| r));
            }
        }
        true
    }

    fn blur(&mut self) -> bool {
        let mut had = false;
        for card in self.cards_mut() {
            had |= card.table.rows.focused().is_some();
            card.table.rows.set_focused(None);
        }
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// table inside its card with the file column always shown and its
    /// columns apart; a phone drops columns.
    #[test]
    fn every_table_fits_its_card_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = TablePage::new(&theme);
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                        let what = format!("{w}px, {}", card.title);
                        let h = TablePage::table_h(card, body.w, &theme);
                        assert!(body.x >= rect.x && body.right() <= rect.right(), "{what}");
                        assert!(body.y + h <= rect.bottom() + 0.01, "{what}: too tall");
                        let cols = card.table.column_layout(body.w, &theme);
                        assert_eq!(cols.visible[0], 0, "{what}: the file stays");
                        let mut end = 0.0;
                        for (x, cw) in &cols.spans {
                            assert!(*x >= end && x + cw <= body.w + 0.01, "{what}");
                            end = x + cw;
                        }
                    }
                }
            }
            let narrow = page.groups[0].cards[0].table.column_layout(280.0, &theme);
            assert!(narrow.visible.len() < 4, "a phone drops columns");
        }
    }

    /// The rows past a table's rect are clipped: every row text node the
    /// page draws sits inside an open clip.
    #[test]
    fn the_rows_end_at_the_table_s_edge() {
        let theme = Theme::hoff();
        let mut page = TablePage::new(&theme);
        let mut c = Compositor::new();
        c.begin_frame();
        page.render(&mut c, Rect::new(0.0, 0.0, 1200.0, 800.0), &theme);
        let mut depth = 0;
        for node in c.layer(LayerId::DEFAULT).unwrap().nodes() {
            match node {
                SceneNode::PushClip { .. } => depth += 1,
                SceneNode::PopClip => depth -= 1,
                SceneNode::Text { key, .. } if key.text.starts_with("crates/") => {
                    assert_eq!(depth, 1, "{} drawn unclipped", key.text);
                }
                _ => {}
            }
        }
    }

    /// The gallery builds its pages under the core theme and draws them
    /// under whichever is chosen: a list or a table built under core and
    /// drawn under hoff draws as one built under hoff (rows at hoff's
    /// height), and so does every page whose rows they are.
    #[test]
    fn rows_follow_the_theme_they_are_drawn_in() {
        use super::super::{
            item::ItemPage, scroll_area::ScrollAreaPage, table_static::TableStaticPage,
        };
        let (core, hoff) = (Theme::hoff_core(), Theme::hoff());
        assert_ne!(
            Table::row_h(&core),
            Table::row_h(&hoff),
            "the ladders differ"
        );
        let content = Rect::new(0.0, 0.0, 1200.0, 800.0);
        let scene = |draw: &mut dyn FnMut(&mut Compositor, LayerId)| {
            let mut c = Compositor::new();
            c.begin_frame();
            let list = c.create_layer(10);
            draw(&mut c, list);
            [LayerId::DEFAULT, list].map(|l| c.layer(l).unwrap().nodes().to_vec())
        };
        let (mut a, mut b) = (TablePage::new(&core), TablePage::new(&hoff));
        assert_eq!(
            scene(&mut |c, _| a.render(c, content, &hoff)),
            scene(&mut |c, _| b.render(c, content, &hoff)),
            "Data Table"
        );
        let (mut a, mut b) = (TableStaticPage::new(&core), TableStaticPage::new(&hoff));
        assert_eq!(
            scene(&mut |c, _| a.render(c, content, &hoff)),
            scene(&mut |c, _| b.render(c, content, &hoff)),
            "Table"
        );
        let (mut a, mut b) = (ScrollAreaPage::new(&core), ScrollAreaPage::new(&hoff));
        assert_eq!(
            scene(&mut |c, l| a.render(c, l, content, &hoff)),
            scene(&mut |c, l| b.render(c, l, content, &hoff)),
            "Scroll Area"
        );
        let item = ItemPage::new();
        let one = scene(&mut |c, _| item.render(c, content, &hoff));
        assert_eq!(
            one,
            scene(&mut |c, _| ItemPage::new().render(c, content, &hoff)),
            "Item"
        );
    }
}
