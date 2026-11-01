//! Code block tests: a block without rows as it always drew, each row
//! kind's tokens and their contrast, the rows' scroll and cut, and a
//! diff's marker column.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::TypeRole;

use super::{CodeBlock, CodeLine, LineKind};
use crate::widgets::widget::Rect;

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
    let caption = theme.typography.run(TypeRole::Caption.strong()).line_height + theme.spacing.xs;
    let line = theme.typography.role(TypeRole::Code).line_height;
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
        let r = &theme.navigation.code;
        let (added, removed) = (r.added.0, r.removed.0);
        // a row's runs (its marker, its code) joined by line
        let mut rows: Vec<(String, f32, [f32; 4])> = Vec::new();
        for (text, y, ink) in texts(&drawn).into_iter().skip(1) {
            match rows.last_mut() {
                Some(last) if last.1 == y => last.0.push_str(&text),
                _ => rows.push((text, y, ink)),
            }
        }
        assert_eq!(rows.len(), 4, "{name}: every row shows");
        for (row, (text, _, ink)) in diff().iter().zip(&rows) {
            assert_eq!(*text, row.text, "{name}");
            assert_eq!(*ink, r.row_ink.0, "{name}: {text}");
        }
        let area = block.rows_rect(bounds, &theme);
        let row_h = CodeBlock::row_h(&theme);
        let at = |i: usize| area.y + i as f32 * row_h;
        assert_eq!(
            washes(&drawn),
            [(at(0), r.header.0), (at(2), removed), (at(3), added)],
            "{name}: a wash behind the header, the removed and the added, none behind context"
        );
        let allowed = [
            r.fill.0,
            r.border.0,
            r.caption_color.0,
            r.header.0,
            r.row_ink.0,
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
/// measured as task-75 measures (`engine::styling::theme::tests::contrast`): the
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
            let block = over(theme.navigation.code.fill, base);
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
            let inner = width - theme.navigation.code.pad * 2.0;
            let style = CodeBlock::text_style(&theme);
            for (text, _, _) in texts(&nodes(&block, bounds, &theme)) {
                let (w, _) = TextMeasurer::measure_styled(&text, &style, None);
                let lead =
                    LineKind::marker_w(&style) - TextMeasurer::measure_styled("+", &style, None).0;
                assert!(lead + w <= inner + 0.5, "{width}: {w} > {inner}");
                assert!(text.ends_with('\u{2026}') || text == long);
            }
        }
    }
}

/// The code after a diff's marker starts at one x on every row, the
/// face being proportional: each row shifts by what its sign is
/// narrower than the marker column. Code with no diff row stays put.
#[test]
fn a_diff_s_code_starts_at_one_x_after_its_marker() {
    let theme = Theme::hoff_core();
    let style = CodeBlock::text_style(&theme);
    let block = CodeBlock::lines(diff());
    let bounds = Rect::new(0.0, 0.0, 420.0, block.height_for(420.0, &theme));
    let drawn = nodes(&block, bounds, &theme);
    let starts: Vec<f32> = drawn
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, x, .. } => {
                let (sign, _) = LineKind::ALL.iter().find_map(|k| {
                    k.marker(&key.text)
                        .filter(|_| *k != LineKind::Context || key.text.starts_with(' '))
                })?;
                Some(x + TextMeasurer::measure_styled(sign, &style, None).0)
            }
            _ => None,
        })
        .collect();
    assert_eq!(starts.len(), 3, "{starts:?}");
    assert!(
        starts.windows(2).all(|w| (w[0] - w[1]).abs() < 0.01),
        "{starts:?}"
    );
    let code = CodeBlock::lines(vec![CodeLine::new("    indented()", LineKind::Context)]);
    let at = texts(&nodes(&code, bounds, &theme))[0].0.clone();
    assert_eq!(at, "    indented()", "no diff, no marker");
    assert_eq!(LineKind::Header.marker("src/lib.rs"), None);
    assert_eq!(LineKind::Added.marker("+x"), Some(("+", "x")));
    assert_eq!(LineKind::Added.marker("x"), None);
}
