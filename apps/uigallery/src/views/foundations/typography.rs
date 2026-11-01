//! Typography page, the hoff core kit's TypographyPage: its five groups
//! (Headline, Body, Heading, Title, Paragraph) with a title and a bottom
//! rule, each a bordered table of the kit's styles (style, font, weight,
//! size / line height, letter spacing), the style's name set in the style
//! itself. Each style is drawn in the active theme's family and tracking
//! (`theme.typography`), so the page shows the face the theme draws with;
//! where the theme does not hold a kit value, the row says so: a size its
//! ramp has no step for ("kit only", the specimen dimmed) or a tracking it
//! does not use ("kit -3%"). Where the table is too narrow for five
//! columns, each row stacks: the specimen, then its readout in one line. A
//! last group, "Engine", shows what the text engine does with that face
//! (`type_engine`).

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect_stroke};

use super::super::components::showcase::{self, Grid, GroupLayout};
use super::type_engine;

/// The face a style without a family resolves to: the engine's default
/// sans (`drawing/text/fonts.rs`).
const DEFAULT_FACE: &str = "Inclusive Sans";
/// The table's text: 11/16 medium (`text-[0.6875rem] leading-[1rem]
/// font-medium`), the header row at half opacity.
const TABLE_LINE_H: f32 = 16.0;
const TABLE_WEIGHT: u16 = 500;
const HEADER_ALPHA: f32 = 0.5;
const HEADERS: [&str; 5] = [
    "Style",
    "Font",
    "Font weight",
    "Size/Line height",
    "Letter spacing",
];
/// The style column takes this many shares of the row; the other four,
/// one each.
const STYLE_SHARES: f32 = 2.0;

/// One row of the kit's TypographyPage: the name it writes and what its
/// classes set, size and line height in px, weight, tracking in em (each
/// row's line of `templates/TypographyPage/index.tsx` is in the comment
/// above it).
struct KitStyle {
    name: &'static str,
    size: f32,
    line_h: f32,
    weight: u16,
    tracking: f32,
}

const fn kit(name: &'static str, size: f32, line_h: f32, weight: u16, tracking: f32) -> KitStyle {
    KitStyle {
        name,
        size,
        line_h,
        weight,
        tracking,
    }
}

/// The kit's five groups and their rows, as its classes draw them (h5,
/// h6 and Title are `font-medium` though the kit's table reads
/// "Regular").
const KIT: [(&str, &[KitStyle]); 5] = [
    (
        "Headline",
        &[
            // :25
            kit("h1", 48.0, 55.2, 400, -0.03),
            // :34
            kit("h2", 40.0, 48.0, 400, -0.03),
            // :43
            kit("h3", 32.0, 40.0, 400, -0.03),
            // :52
            kit("h4", 28.0, 36.0, 400, -0.03),
            // :61
            kit("h5", 24.0, 32.0, 500, -0.03),
            // :70
            kit("h6", 20.0, 28.0, 500, -0.03),
        ],
    ),
    (
        "Body",
        &[
            // :94
            kit("Body, X Small", 10.0, 16.0, 400, -0.01),
            // :103
            kit("Body, Small", 11.0, 16.0, 500, -0.01),
            // :112
            kit("Body, Medium", 12.0, 16.0, 500, -0.01),
            // :121
            kit("Body Medium, Strong", 12.0, 16.0, 600, -0.01),
            // :130
            kit("Body Large, Strong", 13.0, 16.0, 600, -0.01),
            // :139
            kit("Body Large, Medium", 13.0, 16.0, 500, -0.01),
            // :148
            kit("Body Large", 13.0, 16.0, 400, -0.01),
        ],
    ),
    (
        "Heading",
        &[
            // :172
            kit("Heading", 14.0, 20.0, 500, -0.01),
            // :181
            kit("Heading, Strong", 14.0, 20.0, 600, -0.02),
        ],
    ),
    (
        "Title",
        &[
            // :205
            kit("Title", 15.0, 24.0, 500, -0.02),
            // :214
            kit("Title, Strong", 15.0, 24.0, 600, -0.02),
            // :223
            kit("Title, Large", 18.0, 27.0, 400, -0.02),
        ],
    ),
    (
        "Paragraph",
        &[
            // :247
            kit("Paragraph, small", 13.0, 19.52, 400, -0.01),
            // :256
            kit("Paragraph, medium", 15.0, 22.56, 500, -0.01),
        ],
    ),
];

/// A row of the page: the kit's style set in the theme's face and
/// tracking, and whether the theme's ramp has its size at all.
struct Row {
    kit: &'static KitStyle,
    style: TextStyle,
    /// The size is a step of `theme.typography`; a row whose size is not
    /// is a gap of the theme against the kit, drawn and marked as one.
    in_ramp: bool,
}

/// The theme's ramp: the sizes it names.
fn ramp(theme: &Theme) -> [f32; 7] {
    let t = &theme.typography;
    [
        t.small, t.caption, t.body_sm, t.body, t.title_sm, t.title, t.display,
    ]
}

/// The groups of the page and their rows, the kit's styles in the theme.
fn groups(theme: &Theme) -> Vec<(&'static str, Vec<Row>)> {
    let t = &theme.typography;
    let steps = ramp(theme);
    KIT.iter()
        .map(|(title, styles)| {
            let rows = styles
                .iter()
                .map(|kit| Row {
                    kit,
                    style: t.exact(kit.size, kit.line_h, kit.weight),
                    in_ramp: steps.iter().any(|s| (s - kit.size).abs() < 0.01),
                })
                .collect();
            (*title, rows)
        })
        .collect()
}

fn weight_name(weight: u16) -> String {
    match weight {
        300 => "Light".into(),
        400 => "Regular".into(),
        500 => "Medium".into(),
        600 => "Semibold".into(),
        700 => "Bold".into(),
        other => other.to_string(),
    }
}

/// A px value as the kit writes it: whole where it is whole.
fn px(v: f32) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r.fract() == 0.0 {
        format!("{r:.0}")
    } else {
        format!("{r}")
    }
}

/// The four readouts of a row, after its name: the face, the weight, the
/// size over the line height and the tracking as drawn, each kit value
/// the theme does not hold named beside it.
fn readouts(row: &Row) -> [String; 4] {
    let style = &row.style;
    let percent = |em: f32| (em * 100.0).round() + 0.0;
    let tracking = if style.font_size > 0.0 {
        percent(style.letter_spacing / style.font_size)
    } else {
        0.0
    };
    let kit_tracking = percent(row.kit.tracking);
    let size = format!("{}/{}", px(style.font_size), px(style.line_height));
    [
        style
            .font_family
            .clone()
            .unwrap_or_else(|| DEFAULT_FACE.into()),
        weight_name(style.font_weight),
        if row.in_ramp {
            size
        } else {
            format!("{size} · kit only")
        },
        if tracking == kit_tracking {
            format!("{tracking}%")
        } else {
            format!("{tracking}% · kit {kit_tracking}%")
        },
    ]
}

/// The table's measures, from the theme.
struct Metrics {
    /// Cell padding (`p-5`).
    pad: f32,
    /// Between a stacked row's specimen and its readout.
    gap: f32,
    text: TextStyle,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        let t = &theme.typography;
        Self {
            pad: theme.spacing.lg,
            gap: theme.spacing.sm,
            text: t.exact(t.caption, TABLE_LINE_H, TABLE_WEIGHT),
        }
    }

    /// The style column's width in a table `w` wide.
    fn style_w(w: f32) -> f32 {
        w * STYLE_SHARES / (STYLE_SHARES + 4.0)
    }

    /// Whether a table `w` wide stacks its rows: a specimen would not fit
    /// its column.
    fn stacked(&self, w: f32, rows: &[Row]) -> bool {
        let room = Self::style_w(w) - self.pad * 2.0;
        let col = (w - Self::style_w(w)) / 4.0 - self.pad * 2.0;
        let wide = |text: &str, style: &TextStyle, room: f32| {
            TextMeasurer::measure_styled(text, style, None).0 > room
        };
        rows.iter().any(|r| {
            wide(r.kit.name, &r.style, room) || readouts(r).iter().any(|v| wide(v, &self.text, col))
        })
    }

    /// A row's height: its specimen, and in a stacked table its readout a
    /// gap under it (a style as tall as its size leaves its descenders no
    /// room of their own).
    fn row_h(&self, style: &TextStyle, stacked: bool) -> f32 {
        let readout = if stacked {
            self.gap + self.text.line_height
        } else {
            0.0
        };
        self.pad * 2.0 + style.line_height.max(self.text.line_height) + readout
    }

    fn header_h(&self) -> f32 {
        self.pad * 2.0 + self.text.line_height
    }
}

pub struct TypographyPage;

impl TypographyPage {
    pub fn new() -> Self {
        Self
    }

    /// One full-width card per group, as tall as its table, then the
    /// engine's cards.
    fn layout(content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let m = Metrics::of(theme);
        let tables = groups(theme).into_iter().map(|(_, rows)| {
            let stacked = m.stacked(content.w, &rows);
            let body: f32 = rows.iter().map(|r| m.row_h(&r.style, stacked)).sum();
            let header = if stacked { 0.0 } else { m.header_h() };
            Grid::new(1, 0.0, vec![header + body])
        });
        let grids = tables.chain([type_engine::grid(content, theme)]);
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
        let all = groups(theme);
        let last = layout.len().saturating_sub(1);
        for (g, group) in layout.iter().enumerate() {
            match all.get(g) {
                Some((title, rows)) => {
                    showcase::title(c, title, content, group.title_y, theme);
                    if let Some(table) = group.cards.first() {
                        draw_table(c, *table, rows, &m, theme);
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
    x: f32,
    y: f32,
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

/// One group's table in `rect`: the border, the header (unless stacked),
/// and a row per style under a hairline.
fn draw_table(c: &mut Compositor, rect: Rect, rows: &[Row], m: &Metrics, theme: &Theme) {
    let line = theme.control.edge_width;
    let divider = theme.colors.divider.0;
    c.push(rounded_rect_stroke(
        rect.x, rect.y, rect.w, rect.h, 0.0, divider, line,
    ));
    let stacked = m.stacked(rect.w, rows);
    let style_w = Metrics::style_w(rect.w);
    let col_w = (rect.w - style_w) / 4.0;
    let col_x = |i: usize| match i {
        0 => rect.x,
        n => rect.x + style_w + (n - 1) as f32 * col_w,
    };
    let col_room = |i: usize| (if i == 0 { style_w } else { col_w }) - m.pad * 2.0;
    let dim = theme.colors.text_mid.0;
    let mut y = rect.y;
    if !stacked {
        let faded = [dim[0], dim[1], dim[2], dim[3] * HEADER_ALPHA];
        for (i, head) in HEADERS.iter().enumerate() {
            cell(
                c,
                head,
                &m.text,
                col_x(i) + m.pad,
                y + m.pad,
                col_room(i),
                faded,
            );
        }
        y += m.header_h();
    }
    for (r, row) in rows.iter().enumerate() {
        let (name, style) = (row.kit.name, &row.style);
        if r > 0 || !stacked {
            c.push(SceneNode::Rect {
                x: rect.x,
                y,
                w: rect.w,
                h: line,
                color: divider,
            });
        }
        let h = m.row_h(style, stacked);
        let row_line = style.line_height.max(m.text.line_height);
        let text_y = y + m.pad;
        let room = if stacked {
            rect.w - m.pad * 2.0
        } else {
            col_room(0)
        };
        // A kit style the theme's ramp lacks is drawn, dimmed: the gap.
        let ink = if row.in_ramp {
            theme.colors.text
        } else {
            theme.colors.text_mid
        };
        cell(c, name, style, rect.x + m.pad, text_y, room, ink.0);
        let readouts = readouts(row);
        if stacked {
            let joined = readouts.join(" · ");
            cell(
                c,
                &joined,
                &m.text,
                rect.x + m.pad,
                text_y + row_line + m.gap,
                room,
                dim,
            );
        } else {
            // The readouts sit on the specimen's first line.
            let ry = text_y + (style.line_height - m.text.line_height).max(0.0) / 2.0;
            for (i, value) in readouts.iter().enumerate() {
                cell(
                    c,
                    value,
                    &m.text,
                    col_x(i + 1) + m.pad,
                    ry,
                    col_room(i + 1),
                    dim,
                );
            }
        }
        y += h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page sets every style in the active theme's face: Inter under
    /// the core kit, the engine's sans under HOFF.
    #[test]
    fn the_readout_names_the_active_theme_face() {
        let core = Theme::hoff_core();
        let (_, rows) = &groups(&core)[0];
        assert_eq!(readouts(&rows[0])[0], "Inter");
        let hoff = Theme::hoff();
        let (_, rows) = &groups(&hoff)[0];
        assert_eq!(readouts(&rows[0])[0], DEFAULT_FACE);
    }

    /// The kit's rows, in its five groups, with its values; a size the
    /// core ramp lacks and a tracking it does not hold read as gaps.
    #[test]
    fn the_rows_are_the_kit_styles_with_the_gaps_named() {
        let core = Theme::hoff_core();
        let all = groups(&core);
        let titles: Vec<&str> = all.iter().map(|(t, _)| *t).collect();
        assert_eq!(
            titles,
            ["Headline", "Body", "Heading", "Title", "Paragraph"]
        );
        let rows: usize = all.iter().map(|(_, r)| r.len()).sum();
        assert_eq!(rows, 20);
        // h1: 48/115%, a step of the ramp; -3% where the theme tracks -2%.
        let h1 = &all[0].1[0];
        assert!(h1.in_ramp);
        let [_, weight, size, tracking] = readouts(h1);
        assert_eq!((weight.as_str(), size.as_str()), ("Regular", "48/55.2"));
        assert_eq!(tracking, "-2% · kit -3%");
        // h2: 40/48, no step of the core ramp.
        let h2 = &all[0].1[1];
        assert!(!h2.in_ramp);
        assert_eq!(readouts(h2)[2], "40/48 · kit only");
        // Title, Strong: -2% as the theme.
        assert_eq!(readouts(&all[3].1[1])[3], "-2%");
    }

    /// Wide tables keep five columns and a header; narrow ones stack; no
    /// table leaves the content, from 280 to 1600.
    #[test]
    fn tables_fit_from_narrow_to_wide() {
        for theme in [Theme::hoff_core(), Theme::hoff(), Theme::dark()] {
            let m = Metrics::of(&theme);
            let rows = &groups(&theme)[0].1;
            assert!(!m.stacked(1000.0, rows));
            assert!(m.stacked(280.0, rows));
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let layout = TypographyPage::layout(content, &theme);
                // The tables; the engine's cards are the last group.
                for group in &layout[..layout.len() - 1] {
                    let table = group.cards[0];
                    assert!(table.right() <= content.right() + 0.5, "{w}");
                    assert!(group.rule_y > table.bottom());
                }
                let engine = layout.last().expect("the engine group");
                assert_eq!(engine.cards.len(), type_engine::CARDS.len());
                for card in &engine.cards {
                    assert!(card.right() <= content.right() + 0.5, "{w}: {card:?}");
                }
                let mut c = Compositor::new();
                TypographyPage::new().render(&mut c, content, &theme);
            }
        }
    }
}
