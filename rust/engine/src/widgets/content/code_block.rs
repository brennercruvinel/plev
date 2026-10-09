//! Code block: monospaced-feel text (the mono ramp step, the single
//! embedded family) on a field-tone surface at the nav radius, wrapped
//! at the block width, with an optional caption line. The `code_style`
//! helper three gallery sections grew is this one widget.
//!
//! Rows (task-97): given `rows`, the block draws them instead of `code`,
//! one line box each, cut at the block's width with an ellipsis, clipped
//! to the area under the caption and moved up by `scroll`. Each row's
//! kind (a diff's context, a line added or removed, a hunk's header) is a
//! wash behind it, from the theme's tokens; its text is the app's, the
//! `+` and `-` of a diff included, so the kind never rests on the hue
//! alone. Without rows the block draws exactly as it did.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;

use crate::widgets::widget::{Rect, with_alpha};
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke};

/// The kind of one row of a code block: what a diff says of the line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineKind {
    #[default]
    Context,
    Added,
    Removed,
    Header,
}

impl LineKind {
    pub const ALL: [LineKind; 4] = [
        LineKind::Context,
        LineKind::Added,
        LineKind::Removed,
        LineKind::Header,
    ];

    /// The wash behind a row of this kind: the design system's intent
    /// wash (the hue of the intent's token at `glass.wash_alpha`, as a
    /// selected chip and a progress fill draw it), success for a line
    /// added and danger for one removed; the hovered glass for a header;
    /// none for context.
    pub fn wash(self, theme: &Theme) -> Option<[f32; 4]> {
        let g = &theme.glass;
        match self {
            LineKind::Context => None,
            LineKind::Added => Some(with_alpha(theme.colors.success, g.wash_alpha)),
            LineKind::Removed => Some(with_alpha(theme.colors.danger, g.wash_alpha)),
            LineKind::Header => Some(g.surface_hover.0),
        }
    }

    /// The ink of a row's text: the primary text for every kind, the text
    /// token that reads at WCAG AA over every wash on the block (the
    /// secondary `text_mid` of the wrapped code does not over a wash).
    pub fn ink(self, theme: &Theme) -> [f32; 4] {
        theme.colors.text.0
    }
}

/// One row of a code block: its text and its kind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeLine {
    pub text: String,
    pub kind: LineKind,
}

impl CodeLine {
    pub fn new(text: impl Into<String>, kind: LineKind) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub code: String,
    /// Small caption above the code (a file name, a language).
    pub caption: Option<String>,
    /// The code as rows, each with its kind; when there are any, the block
    /// draws them instead of `code`.
    pub rows: Vec<CodeLine>,
    /// How far down the rows are scrolled, in px; the block keeps it
    /// within `0..=max_scroll`.
    pub scroll: f32,
}

impl CodeBlock {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            caption: None,
            rows: Vec::new(),
            scroll: 0.0,
        }
    }

    /// A block of rows.
    pub fn lines(rows: Vec<CodeLine>) -> Self {
        Self {
            rows,
            ..Self::new("")
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.mono()
    }

    fn caption_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_sm()
    }

    /// Inner padding: the `md` step.
    fn pad(theme: &Theme) -> f32 {
        theme.spacing.md
    }

    /// The band the caption takes above the code, its gap included; none
    /// without a caption.
    fn caption_band(&self, theme: &Theme) -> f32 {
        self.caption
            .as_ref()
            .map(|_| Self::caption_style(theme).line_height + theme.spacing.xs)
            .unwrap_or(0.0)
    }

    /// Height for a given width: the code wrapped at the inner width plus
    /// the padding and the caption band; with rows, a line box per row
    /// (at least one), whatever the width.
    pub fn height_for(&self, width: f32, theme: &Theme) -> f32 {
        let pad = Self::pad(theme);
        let style = Self::text_style(theme);
        let caption = self.caption_band(theme);
        if !self.rows.is_empty() {
            return pad + caption + self.content_h(theme) + pad;
        }
        let inner = (width - pad * 2.0).max(0.0);
        let (_, th) = TextMeasurer::measure_styled(&self.code, &style, Some(inner));
        pad + caption + th.max(style.line_height) + pad
    }

    /// The height of one row: a line box of the mono step.
    pub fn row_h(theme: &Theme) -> f32 {
        Self::text_style(theme).line_height
    }

    /// The height of every row.
    pub fn content_h(&self, theme: &Theme) -> f32 {
        self.rows.len() as f32 * Self::row_h(theme)
    }

    /// Where the rows are drawn and clipped in a block at `bounds`: the
    /// block's width, between the caption band and the bottom padding.
    pub fn rows_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let pad = Self::pad(theme);
        let top = pad + self.caption_band(theme);
        Rect::new(
            bounds.x,
            bounds.y + top,
            bounds.w,
            (bounds.h - top - pad).max(0.0),
        )
    }

    /// The farthest the rows scroll in a block at `bounds`: what they pass
    /// its rows area by.
    pub fn max_scroll(&self, bounds: Rect, theme: &Theme) -> f32 {
        (self.content_h(theme) - self.rows_rect(bounds, theme).h).max(0.0)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let pad = Self::pad(theme);
        let radius = theme.shape.nav;
        c.push_to_layer(
            layer,
            rounded_rect(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                glass.field.0,
            ),
        );
        c.push_to_layer(
            layer,
            rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                glass.edge_soft.0,
                theme.control.edge_width,
            ),
        );
        let inner = (bounds.w - pad * 2.0).max(0.0);
        let mut y = bounds.y + pad;
        if let Some(caption) = &self.caption {
            let style = Self::caption_style(theme);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(caption, &style, Some(inner)),
                    x: bounds.x + pad,
                    y,
                    color: glass.text_faint.0,
                },
            );
            y += style.line_height + theme.spacing.xs;
        }
        if !self.rows.is_empty() {
            self.render_rows(c, layer, bounds, theme);
            return;
        }
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.code, &Self::text_style(theme), Some(inner)),
                x: bounds.x + pad,
                y,
                color: theme.colors.text_mid.0,
            },
        );
    }

    /// The rows in their area, scrolled: the wash of each row's kind
    /// inside the block's edge, then its text cut at the inner width. Only
    /// the rows that show are pushed.
    fn render_rows(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let area = self.rows_rect(bounds, theme);
        let style = Self::text_style(theme);
        let row_h = style.line_height;
        let pad = Self::pad(theme);
        let edge = theme.control.edge_width;
        let inner = (bounds.w - pad * 2.0).max(0.0);
        let scroll = self.scroll.clamp(0.0, self.max_scroll(bounds, theme));
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: area.x,
                y: area.y,
                w: area.w,
                h: area.h,
            },
        );
        let first = if row_h > 0.0 {
            (scroll / row_h).floor() as usize
        } else {
            0
        };
        for (at, row) in self.rows.iter().enumerate().skip(first) {
            let y = area.y + at as f32 * row_h - scroll;
            if y >= area.y + area.h {
                break;
            }
            if let Some(color) = row.kind.wash(theme) {
                c.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: bounds.x + edge,
                        y,
                        w: (bounds.w - edge * 2.0).max(0.0),
                        h: row_h,
                        color,
                    },
                );
            }
            let shown = TextMeasurer::truncate_to_width(&row.text, &style, inner);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&shown, &style, None),
                    x: bounds.x + pad,
                    y,
                    color: row.kind.ink(theme),
                },
            );
        }
        c.push_to_layer(layer, SceneNode::PopClip);
    }
}

#[cfg(test)]
mod tests {
    use crate::styling::color::Color;
    use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
    use crate::drawing::text::TextMeasurer;
    use crate::styling::theme::Theme;

    use super::{CodeBlock, CodeLine, LineKind};
    use crate::widgets::widget::{Rect, with_alpha};

    fn nodes(block: &CodeBlock, bounds: Rect, theme: &Theme) -> Vec<SceneNode> {
        let mut c = Compositor::new();
        block.render(&mut c, bounds, theme);
        c.layer(LayerId::DEFAULT)
            .map(|l| l.nodes().to_vec())
            .unwrap_or_default()
    }

    /// Each text drawn: its string, where, and its color.
    fn texts(nodes: &[SceneNode]) -> Vec<(String, f32, [f32; 4])> {
        nodes
            .iter()
            .filter_map(|n| match n {
                SceneNode::Text { key, y, color, .. } => Some((key.text.clone(), *y, *color)),
                _ => None,
            })
            .collect()
    }

    /// Each flat rect drawn: the washes behind the rows.
    fn washes(nodes: &[SceneNode]) -> Vec<(f32, [f32; 4])> {
        nodes
            .iter()
            .filter_map(|n| match n {
                SceneNode::Rect { y, color, .. } => Some((*y, *color)),
                _ => None,
            })
            .collect()
    }

    fn diff() -> Vec<CodeLine> {
        vec![
            CodeLine::new("src/lib.rs: @@ -1,3 +1,3 @@", LineKind::Header),
            CodeLine::new(" fn main() {", LineKind::Context),
            CodeLine::new("-    old();", LineKind::Removed),
            CodeLine::new("+    new();", LineKind::Added),
        ]
    }

    /// The five themes the kinds are measured in: the four of task-75's
    /// contrast test and the ide's light one.
    fn themes() -> [(&'static str, Theme); 5] {
        [
            ("core", Theme::hoff_core()),
            ("hoff", Theme::hoff()),
            ("dark", Theme::dark()),
            ("light", Theme::light()),
            ("hoff_light", Theme::hoff_light()),
        ]
    }

    /// Without rows the block draws as it always did: its fill, its edge,
    /// its caption in the faint glass text, and the code wrapped at the
    /// inner width in the secondary text, nothing else.
    #[test]
    fn a_block_without_rows_draws_as_it_did() {
        let theme = Theme::hoff();
        let block = CodeBlock::new("cargo test --workspace").caption("shell");
        let bounds = Rect::new(10.0, 20.0, 300.0, 80.0);
        let drawn = nodes(&block, bounds, &theme);
        assert_eq!(drawn.len(), 4, "{drawn:?}");
        assert!(matches!(
            drawn[0],
            SceneNode::RoundedRect { color, border_width, .. }
                if color == theme.glass.field.0 && border_width == 0.0
        ));
        assert!(matches!(
            drawn[1],
            SceneNode::RoundedRect { border_color, .. } if border_color == theme.glass.edge_soft.0
        ));
        let pad = theme.spacing.md;
        let inner = (300.0 - pad * 2.0).to_bits();
        match &drawn[3] {
            SceneNode::Text { key, x, color, .. } => {
                assert_eq!(key.text, "cargo test --workspace");
                assert_eq!(
                    key.max_width_bits,
                    Some(inner),
                    "wrapped at the inner width"
                );
                assert_eq!((*x, *color), (10.0 + pad, theme.colors.text_mid.0));
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(texts(&drawn)[0].2, theme.glass.text_faint.0, "the caption");
        let caption = theme.typography.caption_sm().line_height + theme.spacing.xs;
        let line = theme.typography.mono().line_height;
        assert_eq!(
            block.height_for(300.0, &theme),
            pad + caption + line + pad,
            "one line of code"
        );
    }

    /// Each row's kind is the wash of its token behind it and the primary
    /// text on it; no color the theme does not hold.
    #[test]
    fn each_kind_draws_its_own_tokens_and_no_other_color() {
        for (name, theme) in themes() {
            let block = CodeBlock::lines(diff()).caption("diff");
            let bounds = Rect::new(0.0, 0.0, 420.0, block.height_for(420.0, &theme));
            let drawn = nodes(&block, bounds, &theme);
            let g = &theme.glass;
            let added = with_alpha(theme.colors.success, g.wash_alpha);
            let removed = with_alpha(theme.colors.danger, g.wash_alpha);
            let rows: Vec<(String, f32, [f32; 4])> = texts(&drawn).into_iter().skip(1).collect();
            assert_eq!(rows.len(), 4, "{name}: every row shows");
            for (row, (text, _, ink)) in diff().iter().zip(&rows) {
                assert_eq!(*text, row.text, "{name}");
                assert_eq!(*ink, theme.colors.text.0, "{name}: {text}");
            }
            let area = block.rows_rect(bounds, &theme);
            let row_h = CodeBlock::row_h(&theme);
            let at = |i: usize| area.y + i as f32 * row_h;
            assert_eq!(
                washes(&drawn),
                [(at(0), g.surface_hover.0), (at(2), removed), (at(3), added)],
                "{name}: a wash behind the header, the removed and the added, none behind context"
            );
            let allowed = [
                g.field.0,
                g.edge_soft.0,
                g.text_faint.0,
                g.surface_hover.0,
                theme.colors.text.0,
                added,
                removed,
            ];
            for n in &drawn {
                let colors: Vec<[f32; 4]> = match n {
                    SceneNode::Rect { color, .. } | SceneNode::Text { color, .. } => vec![*color],
                    SceneNode::RoundedRect {
                        color,
                        border_color,
                        border_width,
                        ..
                    } => {
                        let mut v = vec![*color];
                        if *border_width > 0.0 {
                            v.push(*border_color);
                        }
                        v
                    }
                    SceneNode::PushClip { .. } | SceneNode::PopClip => Vec::new(),
                    other => panic!("{name}: {other:?}"),
                };
                for c in colors {
                    assert!(
                        c == [0.0; 4] || allowed.contains(&c),
                        "{name}: {c:?} is not a token"
                    );
                }
            }
        }
    }

    /// One channel from sRGB to linear light, as `Color::to_linear_array`.
    fn linear(c: Color) -> [f64; 4] {
        c.to_linear_array().map(f64::from)
    }

    /// `fg` composed over the opaque linear `under`, in linear light.
    fn over(fg: Color, under: [f64; 3]) -> [f64; 3] {
        let [r, g, b, a] = linear(fg);
        [
            r * a + under[0] * (1.0 - a),
            g * a + under[1] * (1.0 - a),
            b * a + under[2] * (1.0 - a),
        ]
    }

    /// WCAG 2.2 contrast ratio of two linear colors, 1 to 21.
    fn ratio(a: [f64; 3], b: [f64; 3]) -> f64 {
        let lum = |[r, g, b]: [f64; 3]| 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let (la, lb) = (lum(a), lum(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    /// The contrast of each kind's text against the row it sits on,
    /// measured as task-75 measures (`engine::theme::tests_contrast`): the
    /// block's field composed over the page (`colors.bg`) or a card
    /// (`colors.surface`), the kind's wash over that, the ink over the
    /// wash, in linear light. Every one reads at AA (4.5) in the five
    /// themes. `--nocapture` prints the table.
    #[test]
    fn every_kind_reads_at_aa_over_its_row() {
        let mut low = f64::MAX;
        for (name, theme) in themes() {
            let page = over(theme.colors.bg, [0.0; 3]);
            let card = over(theme.colors.surface, page);
            for (under, base) in [("colors.bg", page), ("colors.surface", card)] {
                let block = over(theme.glass.field, base);
                let mut line = format!("{name:>10} over {under:<14}");
                for kind in LineKind::ALL {
                    let row = kind.wash(&theme).map_or(block, |w| over(Color(w), block));
                    let ink = over(Color(kind.ink(&theme)), row);
                    let r = ratio(ink, row);
                    low = low.min(r);
                    line.push_str(&format!(" {kind:?} {r:.2}"));
                    assert!(r >= 4.5, "{name}, {kind:?} over {under}: {r:.2}");
                }
                println!("{line}");
            }
        }
        println!("lowest: {low:.2}");
    }

    /// Rows past the area are not drawn; scrolled, the first row shown is
    /// the one the offset reaches, never past the last.
    #[test]
    fn rows_scroll_inside_the_area_and_stop_at_the_last() {
        let theme = Theme::hoff();
        let rows: Vec<CodeLine> = (0..30)
            .map(|i| CodeLine::new(format!("line {i}"), LineKind::Context))
            .collect();
        let mut block = CodeBlock::lines(rows);
        let row_h = CodeBlock::row_h(&theme);
        let pad = theme.spacing.md;
        let bounds = Rect::new(0.0, 0.0, 300.0, pad * 2.0 + row_h * 5.0);
        let area = block.rows_rect(bounds, &theme);
        assert_eq!(area.h, row_h * 5.0);
        assert_eq!(block.max_scroll(bounds, &theme), row_h * 25.0);
        let drawn = nodes(&block, bounds, &theme);
        assert!(matches!(
            drawn[2],
            SceneNode::PushClip { x, y, w, h } if [x, y, w, h] == [area.x, area.y, area.w, area.h]
        ));
        assert!(matches!(drawn.last(), Some(SceneNode::PopClip)));
        let shown: Vec<String> = texts(&drawn).into_iter().map(|t| t.0).collect();
        assert_eq!(shown, ["line 0", "line 1", "line 2", "line 3", "line 4"]);
        block.scroll = row_h * 3.0;
        let shown = texts(&nodes(&block, bounds, &theme));
        assert_eq!(shown[0].0, "line 3");
        assert_eq!(shown[0].1, area.y, "the row the offset reaches, at the top");
        block.scroll = 10_000.0;
        let shown = texts(&nodes(&block, bounds, &theme));
        assert_eq!(
            shown.last().map(|t| t.0.as_str()),
            Some("line 29"),
            "no further than the last"
        );
        assert_eq!(shown.len(), 5);
    }

    /// A row longer than the block is cut at the inner width, at a phone
    /// width and a desktop one, in every theme: one row stays one line.
    #[test]
    fn a_long_row_is_cut_at_the_inner_width() {
        let long = format!(
            "+{}",
            "let value = compute(first, second, third);".repeat(4)
        );
        for (_, theme) in themes() {
            for width in [390.0, 1440.0] {
                let block = CodeBlock::lines(vec![CodeLine::new(&long, LineKind::Added)]);
                let bounds = Rect::new(0.0, 0.0, width, block.height_for(width, &theme));
                let inner = width - theme.spacing.md * 2.0;
                for (text, _, _) in texts(&nodes(&block, bounds, &theme)) {
                    let style = CodeBlock::text_style(&theme);
                    let (w, _) = TextMeasurer::measure_styled(&text, &style, None);
                    assert!(w <= inner + 0.5, "{width}: {w} > {inner}");
                    assert!(text.ends_with('\u{2026}') || text == long);
                }
            }
        }
    }
}
