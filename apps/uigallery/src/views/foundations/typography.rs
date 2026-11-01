//! Typography page: the type scale by role, the one every component reads
//! (`theme.typography`, `engine::styling::theme::TypeRole`).
//!
//! - **Roles**: a row per role, its name set in the active theme's style
//!   at the page's width, then the same role in the four base themes
//!   (core, hoff, dark, light), each with its size / line height and
//!   weight, so a theme's gap against another reads at a glance.
//! - **Fluid**: the title roles (display to h3), which scale with the
//!   width of their container between a minimum and their full size, set
//!   at five widths of the range.
//! - **Engine**: what the text engine does with the face (`type_engine`).
//!
//! Where a table is too narrow for its columns, each row stacks: the
//! specimen, then its readouts in one line.

use std::sync::OnceLock;

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::{FLUID_RANGE, Theme, TypeRole, TypographyScale};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect_stroke};

use super::super::components::showcase::{self, Grid, GroupLayout};
use super::type_engine;

/// The face a style without a family resolves to: the engine's default
/// sans (`drawing/text/fonts.rs`).
const DEFAULT_FACE: &str = "Inclusive Sans";
const HEADER_ALPHA: f32 = 0.5;
/// The first column takes this many shares of the row; the others, one.
const LEAD_SHARES: f32 = 2.0;
/// What a theme column sets in the role.
const SPECIMEN: &str = "Ag 123";
/// The widths the fluid table sets its roles at: the range, in quarters.
const SWEEP: usize = 5;

/// The four base themes the role table compares.
fn scales() -> &'static [(&'static str, TypographyScale); 4] {
    static SCALES: OnceLock<[(&'static str, TypographyScale); 4]> = OnceLock::new();
    SCALES.get_or_init(|| {
        [
            ("core", Theme::hoff_core().typography),
            ("hoff", Theme::hoff().typography),
            ("dark", Theme::dark().typography),
            ("light", Theme::light().typography),
        ]
    })
}

fn sweep() -> [f32; SWEEP] {
    let [from, to] = FLUID_RANGE;
    std::array::from_fn(|i| from + (to - from) * i as f32 / (SWEEP - 1) as f32)
}

/// A px value: whole where it is whole, else to a tenth.
fn px(v: f32) -> String {
    let r = (v * 10.0).round() / 10.0;
    if r.fract() == 0.0 {
        format!("{r:.0}")
    } else {
        format!("{r}")
    }
}

/// `12/16 · 500`.
fn measure(style: &TextStyle) -> String {
    format!(
        "{}/{} · {}",
        px(style.font_size),
        px(style.line_height),
        style.font_weight
    )
}

/// One cell: a text in a style, and its readout under it.
struct Cell {
    text: String,
    style: TextStyle,
    readout: String,
}

/// A table: its header row and its rows of cells, the first the lead.
struct Table {
    title: &'static str,
    headers: Vec<String>,
    rows: Vec<Vec<Cell>>,
}

/// The role table: each role in the active theme at `width`, then in the
/// four base themes.
fn roles_table(theme: &Theme, width: f32) -> Table {
    let t = &theme.typography;
    let face = t.family.unwrap_or(DEFAULT_FACE);
    let mut headers = vec!["Role".to_string()];
    headers.extend(scales().iter().map(|(name, _)| name.to_string()));
    let rows = TypeRole::ALL
        .iter()
        .map(|&role| {
            let style = t.role_at(role, width);
            let em = (t.spec(role).tracking_em * 100.0).round() + 0.0;
            let lead_face = if role == TypeRole::Code {
                t.mono_family
            } else {
                face
            };
            let mut row = vec![Cell {
                text: role.name().to_string(),
                readout: format!("{lead_face} · {em}%"),
                style,
            }];
            row.extend(scales().iter().map(|(_, scale)| {
                let style = scale.role(role);
                Cell {
                    text: SPECIMEN.to_string(),
                    readout: measure(&style),
                    style,
                }
            }));
            row
        })
        .collect();
    Table {
        title: "Roles",
        headers,
        rows,
    }
}

/// The fluid table: each title role of the active theme at the sweep's
/// widths.
fn fluid_table(theme: &Theme) -> Table {
    let t = &theme.typography;
    let mut headers = vec!["Role".to_string()];
    headers.extend(sweep().iter().map(|w| format!("{} px", px(*w))));
    let rows = TypeRole::FLUID
        .iter()
        .map(|&role| {
            let [min, _] = t.roles.fluid_min(role).unwrap_or([0.0; 2]);
            let mut row = vec![Cell {
                text: role.name().to_string(),
                style: t.role(TypeRole::Heading),
                readout: format!("{} to {}", px(min), px(t.spec(role).size)),
            }];
            row.extend(sweep().iter().map(|&w| {
                let style = t.role_at(role, w);
                Cell {
                    text: "Ag".to_string(),
                    readout: measure(&style),
                    style,
                }
            }));
            row
        })
        .collect();
    Table {
        title: "Fluid",
        headers,
        rows,
    }
}

fn tables(theme: &Theme, width: f32) -> [Table; 2] {
    [roles_table(theme, width), fluid_table(theme)]
}

/// The table's measures, from the theme.
struct Metrics {
    pad: f32,
    gap: f32,
    text: TextStyle,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        Self {
            pad: theme.spacing.lg,
            gap: theme.spacing.xs,
            text: theme.typography.role(TypeRole::Caption),
        }
    }

    /// Each column's x offset and width in a table `w` wide with `n`
    /// columns.
    fn columns(w: f32, n: usize) -> Vec<(f32, f32)> {
        let unit = w / (LEAD_SHARES + (n - 1) as f32);
        (0..n)
            .map(|i| match i {
                0 => (0.0, unit * LEAD_SHARES),
                _ => (unit * (LEAD_SHARES + (i - 1) as f32), unit),
            })
            .collect()
    }

    /// Whether a table `w` wide stacks: a cell would not fit its column.
    fn stacked(&self, w: f32, table: &Table) -> bool {
        let cols = Self::columns(w, table.headers.len());
        let wide = |text: &str, style: &TextStyle, room: f32| {
            TextMeasurer::measure_styled(text, style, None).0 > room
        };
        table.rows.iter().any(|row| {
            row.iter().zip(&cols).any(|(cell, (_, cw))| {
                let room = cw - self.pad * 2.0;
                wide(&cell.text, &cell.style, room) || wide(&cell.readout, &self.text, room)
            })
        })
    }

    fn row_h(&self, row: &[Cell], stacked: bool) -> f32 {
        let line = if stacked {
            row[0].style.line_height
        } else {
            row.iter().map(|c| c.style.line_height).fold(0.0, f32::max)
        };
        self.pad * 2.0 + line + self.gap + self.text.line_height
    }

    fn header_h(&self) -> f32 {
        self.pad * 2.0 + self.text.line_height
    }

    fn height(&self, w: f32, table: &Table) -> f32 {
        let stacked = self.stacked(w, table);
        let header = if stacked { 0.0 } else { self.header_h() };
        header
            + table
                .rows
                .iter()
                .map(|r| self.row_h(r, stacked))
                .sum::<f32>()
    }
}

pub struct TypographyPage;

impl TypographyPage {
    pub fn new() -> Self {
        Self
    }

    /// One full-width card per table, then the engine's cards.
    fn layout(content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let m = Metrics::of(theme);
        let grids = tables(theme, content.w)
            .map(|table| Grid::new(1, 0.0, vec![m.height(content.w, &table)]));
        let grids = grids.into_iter().chain([type_engine::grid(content, theme)]);
        showcase::layout_grids(grids, content, theme)
    }

    /// Natural height of the page (page scrolling needs it).
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&Self::layout(content, theme), content)
    }

    /// A specimen page: nothing to hit.
    pub fn handle_event(&mut self, _event: &WidgetEvent, _content: Rect) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let m = Metrics::of(theme);
        let layout = Self::layout(content, theme);
        let all = tables(theme, content.w);
        let last = layout.len().saturating_sub(1);
        for (g, group) in layout.iter().enumerate() {
            match all.get(g) {
                Some(table) => {
                    showcase::title(c, table.title, content, group.title_y, theme);
                    if let Some(rect) = group.cards.first() {
                        draw_table(c, *rect, table, &m, theme);
                    }
                }
                None => {
                    showcase::title(c, "Engine", content, group.title_y, theme);
                    type_engine::draw(c, &group.cards, theme);
                }
            }
            if g < last {
                showcase::rule(c, content, group.rule_y, theme);
            }
        }
    }
}

/// Text at `(x, y)`, cut with an ellipsis to `w`.
fn cell(
    c: &mut Compositor,
    text: &str,
    style: &TextStyle,
    (x, y): (f32, f32),
    w: f32,
    color: [f32; 4],
) {
    let text = TextMeasurer::truncate_to_width(text, style, w.max(0.0));
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&text, style, None),
        x,
        y,
        color,
    });
}

/// A table in `rect`: the border, the header (unless stacked), and a row
/// per line under a hairline.
fn draw_table(c: &mut Compositor, rect: Rect, table: &Table, m: &Metrics, theme: &Theme) {
    let line = theme.control.edge_width;
    let divider = theme.colors.divider.0;
    c.push(rounded_rect_stroke(
        rect.x, rect.y, rect.w, rect.h, 0.0, divider, line,
    ));
    let stacked = m.stacked(rect.w, table);
    let cols = Metrics::columns(rect.w, table.headers.len());
    let dim = theme.colors.text_mid.0;
    let ink = theme.colors.text.0;
    let mut y = rect.y;
    if !stacked {
        let faded = [dim[0], dim[1], dim[2], dim[3] * HEADER_ALPHA];
        for (head, (cx, cw)) in table.headers.iter().zip(&cols) {
            let at = (rect.x + cx + m.pad, y + m.pad);
            cell(c, head, &m.text, at, cw - m.pad * 2.0, faded);
        }
        y += m.header_h();
    }
    for (r, row) in table.rows.iter().enumerate() {
        if r > 0 || !stacked {
            c.push(SceneNode::Rect {
                x: rect.x,
                y,
                w: rect.w,
                h: line,
                color: divider,
            });
        }
        let h = m.row_h(row, stacked);
        let top = y + m.pad;
        if stacked {
            let room = rect.w - m.pad * 2.0;
            let lead = &row[0];
            cell(c, &lead.text, &lead.style, (rect.x + m.pad, top), room, ink);
            let readout = row
                .iter()
                .map(|cell| cell.readout.as_str())
                .collect::<Vec<_>>()
                .join(" · ");
            let ry = top + lead.style.line_height + m.gap;
            cell(c, &readout, &m.text, (rect.x + m.pad, ry), room, dim);
        } else {
            let line_h = h - m.pad * 2.0 - m.gap - m.text.line_height;
            for (i, (item, (cx, cw))) in row.iter().zip(&cols).enumerate() {
                let x = rect.x + cx + m.pad;
                let room = cw - m.pad * 2.0;
                let color = if i == 0 { ink } else { dim };
                // Each specimen sits on the row's line; its readout under.
                let sy = top + line_h - item.style.line_height;
                cell(c, &item.text, &item.style, (x, sy), room, color);
                let ry = top + line_h + m.gap;
                cell(c, &item.readout, &m.text, (x, ry), room, dim);
            }
        }
        y += h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A row per role, the lead in the active theme's face, a column per
    /// base theme with its own measure.
    #[test]
    fn the_role_table_has_every_role_in_the_four_themes() {
        let core = Theme::hoff_core();
        let t = roles_table(&core, 1280.0);
        assert_eq!(t.headers, ["Role", "core", "hoff", "dark", "light"]);
        assert_eq!(t.rows.len(), TypeRole::ALL.len());
        let label = &t.rows[TypeRole::ALL
            .iter()
            .position(|r| *r == TypeRole::Label)
            .unwrap()];
        assert_eq!(label[0].text, "label");
        assert_eq!(label[0].readout, "Inter · -2%");
        assert_eq!(label[1].readout, "12/16 · 500");
        assert_eq!(label[2].readout, "12/16 · 500");
        let hoff = roles_table(&Theme::hoff(), 1280.0);
        assert!(hoff.rows[0][0].readout.starts_with(DEFAULT_FACE));
    }

    /// The fluid table sets each title role at the sweep's widths, from
    /// its minimum to its full size.
    #[test]
    fn the_fluid_table_sweeps_the_range() {
        let core = Theme::hoff_core();
        let t = fluid_table(&core);
        assert_eq!(t.rows.len(), TypeRole::FLUID.len());
        let h1 = &t.rows[1];
        assert_eq!(h1[0].text, "h1");
        assert_eq!(h1[1].readout, "32/40 · 400");
        assert_eq!(h1[3].readout, "40/48 · 400");
        assert_eq!(h1[SWEEP].readout, "48/56 · 400");
    }

    /// Wide tables keep their columns and a header; narrow ones stack; no
    /// table leaves the content, from 280 to 1600.
    #[test]
    fn tables_fit_from_narrow_to_wide() {
        for theme in [Theme::hoff_core(), Theme::hoff(), Theme::dark()] {
            let m = Metrics::of(&theme);
            let [roles, _] = tables(&theme, 280.0);
            assert!(m.stacked(280.0, &roles));
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let layout = TypographyPage::layout(content, &theme);
                for group in &layout[..layout.len() - 1] {
                    let table = group.cards[0];
                    assert!(table.right() <= content.right() + 0.5, "{w}");
                    assert!(group.rule_y > table.bottom());
                }
                let engine = layout.last().expect("the engine group");
                assert_eq!(engine.cards.len(), type_engine::CARDS.len());
                let mut c = Compositor::new();
                TypographyPage::new().render(&mut c, content, &theme);
            }
        }
    }
}
