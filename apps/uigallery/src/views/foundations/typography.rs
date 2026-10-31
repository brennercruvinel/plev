//! Typography page, the hoff core kit's TypographyPage: groups with a
//! title and a bottom rule, each a bordered table of styles (style, font,
//! weight, size / line height, letter spacing), the style's name set in
//! the style itself. The styles are the active theme's
//! (`theme.typography`: its ramp and its named styles, in its family and
//! tracking), so the page shows the face the theme draws with, whatever it
//! is. Where the table is too narrow for five columns, each row stacks:
//! the specimen, then its readout in one line. A last group, "Engine",
//! shows what the text engine does with that face (`type_engine`).

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

/// The groups of the page and their rows, from the theme.
fn groups(theme: &Theme) -> Vec<(&'static str, Vec<(&'static str, TextStyle)>)> {
    let t = &theme.typography;
    vec![
        (
            "Headline",
            vec![
                ("Display", t.h4()),
                ("Title", t.exact(t.title, t.line_height(t.title), 400)),
                ("Title small", t.title()),
            ],
        ),
        (
            "Body",
            vec![
                ("Base", t.base_r()),
                ("Base, Medium", t.base_m()),
                ("Base 2", t.base_2r()),
                ("Base 2, Medium", t.base_2m()),
                ("Base 2, Strong", t.base_2sm()),
                ("Base 2, Bold", t.base_2b()),
                ("Caption", t.caption_r()),
                ("Caption, Strong", t.caption_sm()),
                ("Small", t.small_r()),
                ("Small, Strong", t.small_sm()),
            ],
        ),
        (
            "Paragraph",
            vec![
                ("Body", t.body()),
                ("Body 2", t.body_2r()),
                ("Body 2, Medium", t.body_2m()),
                ("Hairline", t.hairline()),
            ],
        ),
    ]
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

/// The four readouts of a style, after its name.
fn readouts(style: &TextStyle) -> [String; 4] {
    let tracking = if style.font_size > 0.0 {
        (style.letter_spacing / style.font_size * 100.0).round()
    } else {
        0.0
    };
    [
        style
            .font_family
            .clone()
            .unwrap_or_else(|| DEFAULT_FACE.into()),
        weight_name(style.font_weight),
        format!("{}/{}", px(style.font_size), px(style.line_height)),
        format!("{}%", tracking + 0.0),
    ]
}

/// The table's measures, from the theme.
struct Metrics {
    /// Cell padding (`p-5`).
    pad: f32,
    text: TextStyle,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        let t = &theme.typography;
        Self {
            pad: theme.spacing.lg,
            text: t.exact(t.caption, TABLE_LINE_H, TABLE_WEIGHT),
        }
    }

    /// The style column's width in a table `w` wide.
    fn style_w(w: f32) -> f32 {
        w * STYLE_SHARES / (STYLE_SHARES + 4.0)
    }

    /// Whether a table `w` wide stacks its rows: a specimen would not fit
    /// its column.
    fn stacked(&self, w: f32, rows: &[(&str, TextStyle)]) -> bool {
        let room = Self::style_w(w) - self.pad * 2.0;
        rows.iter()
            .any(|(name, s)| TextMeasurer::measure_styled(name, s, None).0 > room)
    }

    /// A row's height: its specimen, and in a stacked table its readout.
    fn row_h(&self, style: &TextStyle, stacked: bool) -> f32 {
        let readout = if stacked { self.text.line_height } else { 0.0 };
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
            let body: f32 = rows.iter().map(|(_, s)| m.row_h(s, stacked)).sum();
            let header = if stacked { 0.0 } else { m.header_h() };
            Grid {
                span: 1,
                min_w: 0.0,
                heights: vec![header + body],
            }
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
fn draw_table(
    c: &mut Compositor,
    rect: Rect,
    rows: &[(&str, TextStyle)],
    m: &Metrics,
    theme: &Theme,
) {
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
    for (r, (name, style)) in rows.iter().enumerate() {
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
        cell(
            c,
            name,
            style,
            rect.x + m.pad,
            text_y,
            room,
            theme.colors.text.0,
        );
        let readouts = readouts(style);
        if stacked {
            let joined = readouts.join(" · ");
            cell(
                c,
                &joined,
                &m.text,
                rect.x + m.pad,
                text_y + row_line,
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
        assert_eq!(readouts(&rows[0].1)[0], "Inter");
        let hoff = Theme::hoff();
        let (_, rows) = &groups(&hoff)[0];
        assert_eq!(readouts(&rows[0].1)[0], DEFAULT_FACE);
    }

    #[test]
    fn the_readouts_are_the_style_measures() {
        let core = Theme::hoff_core();
        let style = core.typography.exact(13.0, 16.0, 600);
        let [_, weight, size, tracking] = readouts(&style);
        assert_eq!(weight, "Semibold");
        assert_eq!(size, "13/16");
        assert_eq!(tracking, "-2%");
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
