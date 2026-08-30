//! Typography section: the HOFF type scale set in the embedded UI typeface
//! (Inclusive Sans 400/500/600/700; JetBrains Mono for code). Absorbs the
//! `text` engine example's patterns into the design system: every specimen
//! is drawn with the same `TextStyle` that measures it and every readout
//! (advances, wrapped heights) is a live `TextMeasurer` value, never an
//! eyeballed constant.

use engine::compositor::{Compositor, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{Theme, TypographyScale};
use engine::ui::widgets::{EventResult, Rect, WidgetEvent};

use super::{group_label, panel};

const GAP: f32 = 12.0;
const LABEL_H: f32 = 30.0;
const ROW_PAD: f32 = 6.0;
/// Panel inner padding (wrapping demo).
const INNER_PAD: f32 = 16.0;
/// Width at which the section splits into two columns (ramp left, rest right).
const TWO_COL_MIN: f32 = 900.0;

pub struct TypographySection;

/// One drawable line of the section, with its absolute y resolved.
enum Line {
    /// Uppercase group label.
    Group(&'static str),
    /// HOFF mixin specimen: dim readout line, then the sample at its style.
    Specimen {
        name: &'static str,
        style: TextStyle,
        sample: &'static str,
    },
    /// Weight ladder row: sample at `weight` plus the measured advance.
    Weight {
        style: TextStyle,
        weight: u16,
        advance: f32,
    },
    /// Plain line at a given style (families, unicode coverage).
    Plain { style: TextStyle, text: String },
    /// Wrapped paragraph inside a panel.
    Para {
        style: TextStyle,
        text: &'static str,
        rect: Rect,
    },
}

/// The HOFF ramp, one entry per mixin of `TypographyScale` (the measured
/// tokens — docs/adr/measured-design-tokens-over-eyeballed-values).
fn ramp() -> [(&'static str, TextStyle, &'static str); 12] {
    let t = TypographyScale::hoff();
    [
        ("=h4", t.h4(), "The quick brown fox"),
        ("=title", t.title(), "The quick brown fox"),
        ("=base-m", t.base_m(), "Jumps over the lazy dog"),
        ("=base-r", t.base_r(), "Jumps over the lazy dog"),
        ("=base-2m", t.base_2m(), "Body text for paragraphs"),
        ("=base-2sm", t.base_2sm(), "Body text for paragraphs"),
        ("=body", t.body(), "Secondary information and labels"),
        ("=body-2r", t.body_2r(), "Secondary information and labels"),
        ("=caption-r", t.caption_r(), "Fine print and timestamps"),
        ("=caption-sm", t.caption_sm(), "Fine print and timestamps"),
        ("=small-r", t.small_r(), "Auxiliary details"),
        ("=hairline", t.hairline(), "Auxiliary details"),
    ]
}

fn weight_ladder() -> [(TextStyle, u16); 4] {
    [400u16, 500, 600, 700].map(|w| {
        (
            TextStyle::new(18.0)
                .with_line_height(18.0 * 1.4)
                .with_weight(w),
            w,
        )
    })
}

const WEIGHT_SAMPLE: &str = "Sphinx of black quartz";

fn mono_style(size: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(size * 1.4)
        .with_family("JetBrains Mono")
}

fn readout_style() -> TextStyle {
    TextStyle::new(11.0).with_line_height(15.0)
}

impl Line {
    fn height(&self) -> f32 {
        match self {
            Line::Group(_) => LABEL_H,
            Line::Specimen { style, .. } => 15.0 + style.line_height + ROW_PAD,
            Line::Weight { style, .. } | Line::Plain { style, .. } => style.line_height + ROW_PAD,
            Line::Para { rect, .. } => rect.h + GAP,
        }
    }
}

impl TypographySection {
    pub fn new() -> Self {
        Self
    }

    /// All lines with absolute y, plus the total content height. Wide
    /// layouts put the ramp on the left and the rest on the right; narrow
    /// stacks everything. The paragraph panel is always full width, below.
    fn layout(content: Rect) -> (Vec<(f32, Line)>, f32) {
        let two_col = content.w >= TWO_COL_MIN;

        let mut left: Vec<Line> = vec![Line::Group("HOFF TYPE RAMP — INCLUSIVE SANS")];
        left.extend(
            ramp()
                .into_iter()
                .map(|(name, style, sample)| Line::Specimen {
                    name,
                    style,
                    sample,
                }),
        );

        let mut right: Vec<Line> = vec![Line::Group("WEIGHT LADDER — 18PX, MEASURED ADVANCE")];
        right.extend(weight_ladder().into_iter().map(|(style, weight)| {
            let (advance, _) = TextMeasurer::measure_styled(WEIGHT_SAMPLE, &style, None);
            Line::Weight {
                style,
                weight,
                advance,
            }
        }));
        right.push(Line::Group("FAMILIES"));
        right.push(Line::Plain {
            style: TextStyle::new(14.0).with_line_height(19.6),
            text: "Inclusive Sans — the embedded UI sans".to_string(),
        });
        right.push(Line::Plain {
            style: mono_style(13.0),
            text: "JetBrains Mono — code, readouts, data".to_string(),
        });
        right.push(Line::Group("UNICODE"));
        // Only glyphs the embedded faces actually cover: anything else
        // silently rasterizes from a system font (the engine warns about
        // non-embedded faces at raster time). Inclusive Sans is Latin-only;
        // CJK/math/Greek demos would be that fallback, not our typography.
        for text in [
            "Café naïve résumé façade über",
            "¿Qué? ¡Sí! «guillemets» „quotes“ — dash …",
            "€ £ ¥ © ® ™ · • ° àéîõü",
        ] {
            right.push(Line::Plain {
                style: TextStyle::new(14.0).with_line_height(20.0),
                text: text.to_string(),
            });
        }

        let flow = |lines: Vec<Line>, mut y: f32| -> (Vec<(f32, Line)>, f32) {
            let placed: Vec<(f32, Line)> = lines
                .into_iter()
                .map(|line| {
                    let at = y;
                    y += line.height();
                    (at, line)
                })
                .collect();
            (placed, y)
        };

        let (mut placed, left_bottom) = flow(left, content.y);
        // Column x is resolved at render time; y positions are per column.
        let total_h = if two_col {
            let (right_placed, right_bottom) = flow(right, content.y);
            placed.extend(right_placed);
            left_bottom.max(right_bottom) - content.y
        } else {
            let (right_placed, bottom) = flow(right, left_bottom + GAP);
            placed.extend(right_placed);
            bottom - content.y
        };

        // Wrapping demo: full-width panel under the columns, height measured
        // with the SAME style that draws it.
        let style = TextStyle::new(14.0).with_line_height(21.0);
        let inner_w = content.w - INNER_PAD * 2.0;
        let (_, text_h) = TextMeasurer::measure_styled(WRAPPING_TEXT, &style, Some(inner_w));
        let panel = Rect::new(
            content.x,
            content.y + total_h + LABEL_H,
            content.w,
            text_h + INNER_PAD * 2.0,
        );
        placed.push((
            content.y + total_h,
            Line::Group("WRAPPING — REAL LINE BREAKS"),
        ));
        placed.push((
            panel.y,
            Line::Para {
                style,
                text: WRAPPING_TEXT,
                rect: panel,
            },
        ));
        (placed, panel.y + panel.h - content.y)
    }

    /// Column x for a line: ramp lines sit in the left column, the rest in
    /// the right column on wide layouts (all left-aligned when narrow).
    fn line_x(content: Rect, line: &Line) -> f32 {
        let two_col = content.w >= TWO_COL_MIN;
        if !two_col {
            return content.x;
        }
        match line {
            Line::Group("HOFF TYPE RAMP — INCLUSIVE SANS") | Line::Specimen { .. } => content.x,
            Line::Group("WRAPPING — REAL LINE BREAKS") | Line::Para { .. } => content.x,
            _ => content.x + (content.w * 0.55 - GAP).floor() + GAP * 2.0,
        }
    }

    pub fn content_height(&self, content: Rect) -> f32 {
        Self::layout(content).1 + GAP
    }

    /// Static gallery page: no widgets, nothing to hit-test.
    pub fn handle_event(&mut self, _event: &WidgetEvent, _content: Rect) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (lines, _) = Self::layout(content);
        let dim = theme.colors.text_dim.0;
        let fg = theme.colors.text.0;
        for (y, line) in &lines {
            let x = Self::line_x(content, line);
            match line {
                Line::Group(label) => group_label(c, label, x, *y, theme),
                Line::Specimen {
                    name,
                    style,
                    sample,
                } => {
                    let readout = format!(
                        "{name} — {}px/{} · lh {:.1}",
                        style.font_size, style.font_weight, style.line_height
                    );
                    c.draw_text(
                        TextNodeKey::from_style(&readout, &readout_style(), None),
                        x,
                        *y,
                        theme.glass.text_placeholder.0,
                    );
                    c.draw_text(
                        TextNodeKey::from_style(sample, style, None),
                        x,
                        *y + 15.0,
                        fg,
                    );
                }
                Line::Weight {
                    style,
                    weight,
                    advance,
                } => {
                    c.draw_text(
                        TextNodeKey::from_style(WEIGHT_SAMPLE, style, None),
                        x,
                        *y,
                        fg,
                    );
                    let readout = format!("{weight} · {advance:.1}px");
                    c.draw_text(
                        TextNodeKey::from_style(&readout, &mono_style(12.0), None),
                        x + advance + 16.0,
                        *y + 3.0,
                        dim,
                    );
                }
                Line::Plain { style, text } => {
                    c.draw_text(TextNodeKey::from_style(text, style, None), x, *y, fg);
                }
                Line::Para { style, text, rect } => {
                    panel(c, *rect, theme);
                    c.draw_text(
                        TextNodeKey::from_style(text, style, Some(rect.w - INNER_PAD * 2.0)),
                        rect.x + INNER_PAD,
                        rect.y + INNER_PAD,
                        dim,
                    );
                }
            }
        }
    }
}

const WRAPPING_TEXT: &str = "Inclusive Sans ships in every UI weight (400/500/600/700) as static \
    faces embedded in the engine, so shaping never depends on host fonts. cosmic-text only keeps \
    the requested family on an exact weight match; this paragraph wraps against the panel width \
    and re-wraps as the window resizes, measured with the same TextStyle that draws it.";

#[cfg(test)]
mod tests {
    use super::*;
    use engine::compositor::SceneNode;

    fn narrow() -> Rect {
        Rect::new(288.0, 80.0, 472.0, 440.0)
    }

    fn wide() -> Rect {
        Rect::new(288.0, 80.0, 1272.0, 840.0)
    }

    #[test]
    fn renders_at_narrow_and_wide_without_overflow() {
        let theme = Theme::hoff();
        let section = TypographySection::new();
        for content in [narrow(), wide()] {
            let mut c = Compositor::new();
            section.render(&mut c, content, &theme);
            let nodes = c
                .layer(engine::compositor::LayerId::DEFAULT)
                .unwrap()
                .nodes();
            assert!(nodes.len() > 30, "thin scene: {} nodes", nodes.len());
            // No text node starts past the content's right edge.
            for n in nodes {
                if let SceneNode::Text { x, .. } = n {
                    assert!(*x <= content.x + content.w, "text starts past the edge");
                }
            }
            assert!(section.content_height(content) > 0.0);
        }
    }

    #[test]
    fn narrow_stacks_taller_than_wide_two_columns() {
        // Wide splits ramp/rest into two columns; narrow stacks them, so the
        // page must grow (responsiveness is content-driven, not a constant).
        let narrow_h = TypographySection::layout(narrow()).1;
        let wide_h = TypographySection::layout(wide()).1;
        assert!(
            narrow_h > wide_h,
            "narrow ({narrow_h}) must stack taller than wide ({wide_h})"
        );
    }

    #[test]
    fn weight_ladder_advances_are_measured_and_ordered() {
        // Inclusive Sans ships all four UI weights; heavier faces advance
        // wider (a fallback family would jump by ~35%, siblings by a few %).
        let widths: Vec<f32> = weight_ladder()
            .iter()
            .map(|(style, _)| TextMeasurer::measure_styled(WEIGHT_SAMPLE, style, None).0)
            .collect();
        for pair in widths.windows(2) {
            assert!(
                pair[1] >= pair[0],
                "advances must not shrink with weight: {widths:?}"
            );
        }
        assert!(
            widths[3] > widths[0],
            "bold must advance wider than regular: {widths:?}"
        );
    }

    #[test]
    fn wrapped_paragraph_height_is_measured_not_constant() {
        let style = TextStyle::new(14.0).with_line_height(21.0);
        let (_, h_narrow) = TextMeasurer::measure_styled(WRAPPING_TEXT, &style, Some(440.0));
        let (_, h_wide) = TextMeasurer::measure_styled(WRAPPING_TEXT, &style, Some(1240.0));
        assert!(
            h_narrow > h_wide,
            "narrower panel must wrap taller ({h_narrow} vs {h_wide})"
        );
    }

    #[test]
    fn every_ramp_style_uses_an_embedded_weight() {
        for (name, style, _) in ramp() {
            assert!(
                [400, 500, 600, 700].contains(&style.font_weight),
                "{name} requests weight {}, which has no embedded face",
                style.font_weight
            );
        }
    }
}
