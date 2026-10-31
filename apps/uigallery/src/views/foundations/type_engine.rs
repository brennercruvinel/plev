//! The Typography page's last group, "Engine": what the text engine does
//! with the theme's face, beyond the kit's tables, each in a card of the
//! showcase mold. The weight ladder with each sample's measured advance,
//! the unicode a face covers and the scripts the embedded Noto faces add
//! (adr-0071; the web build fetches a script's face the first time this
//! page shows it), and a paragraph that wraps to its card. Every height and
//! advance comes from the `TextMeasurer`, with the style that draws it.

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::Theme;
use engine::widgets::prelude::Rect;

use super::super::components::showcase::{self, CARD_MIN_W, Grid};

/// The cards, in order, named by their chips.
pub(super) const CARDS: [&str; 3] = ["Weights", "Unicode", "Wrapping"];
/// Cards to a row at full width.
const SPAN: usize = 3;
const WEIGHTS: [u16; 4] = [400, 500, 600, 700];
const WEIGHT_SAMPLE: &str = "Sphinx of black quartz";
/// Only glyphs the embedded faces cover: anything else rasterizes from a
/// system font (the engine warns about non-embedded faces at raster time).
const LATIN: [&str; 3] = [
    "Café naïve résumé façade über",
    "¿Qué? ¡Sí! «guillemets» „quotes“ — dash …",
    "€ £ ¥ © ® ™ · • ° àéîõü",
];
/// One line per embedded script: arabic, hebrew, devanagari, simplified
/// chinese, japanese, korean, emoji.
const SCRIPTS: [&str; 7] = [
    "مرحبا بالعالم",
    "שלום עולם",
    "नमस्ते दुनिया",
    "你好，世界",
    "こんにちは、世界",
    "안녕하세요 세계",
    "🙂 🎉 🚀 👍",
];
const WRAPPING_TEXT: &str = "Every weight of the theme's face is embedded in the engine, so \
    shaping never depends on host fonts. This paragraph wraps against its card and re-wraps \
    as the window resizes, measured with the same TextStyle that draws it.";

/// The engine cards' styles and insets, from the theme.
struct Styles {
    /// Inset of the content from the card's sides and bottom.
    pad: f32,
    /// Gap between two lines.
    gap: f32,
    weights: Vec<TextStyle>,
    /// The latin lines and the paragraph: the theme's body.
    body: TextStyle,
    /// The scripts: the body's size in the engine's default sans, so the
    /// Noto faces fill what it lacks.
    script: TextStyle,
    /// The measured readouts.
    readout: TextStyle,
}

impl Styles {
    fn of(theme: &Theme) -> Self {
        let t = &theme.typography;
        let line = |size: f32| t.line_height(size);
        Self {
            pad: theme.spacing.lg,
            gap: theme.spacing.xs,
            weights: WEIGHTS
                .iter()
                .map(|w| t.exact(t.body, line(t.body), *w))
                .collect(),
            body: t.body(),
            script: TextStyle::new(t.body).with_line_height(line(t.body)),
            readout: t.caption_r(),
        }
    }

    /// The content's top in a card: under its chip.
    fn top(&self, card: Rect, theme: &Theme) -> f32 {
        let chip = showcase::chip_rect(card.x, card.y, CARDS[0], theme);
        card.y + theme.spacing.sm + chip.h + theme.spacing.md
    }

    /// The lines a card holds, top to bottom, with their style.
    fn lines(&self, card: usize) -> Vec<(&'static str, &TextStyle)> {
        match card {
            0 => self.weights.iter().map(|s| (WEIGHT_SAMPLE, s)).collect(),
            1 => LATIN
                .iter()
                .map(|t| (*t, &self.body))
                .chain(SCRIPTS.iter().map(|t| (*t, &self.script)))
                .collect(),
            _ => vec![(WRAPPING_TEXT, &self.body)],
        }
    }

    /// The content height of a card whose content is `w` wide.
    fn content_h(&self, card: usize, w: f32) -> f32 {
        let lines = self.lines(card);
        let gaps = self.gap * lines.len().saturating_sub(1) as f32;
        let wrap = (card == 2).then_some(w);
        gaps + lines
            .iter()
            .map(|(text, style)| TextMeasurer::measure_styled(text, style, wrap).1)
            .sum::<f32>()
    }
}

/// The group: three cards, each as tall as its chip and content.
pub(super) fn grid(content: Rect, theme: &Theme) -> Grid {
    let s = Styles::of(theme);
    let w = showcase::card_width_at(SPAN, CARD_MIN_W, content, theme);
    let probe = Rect::new(0.0, 0.0, w, 0.0);
    let top = s.top(probe, theme);
    let heights = (0..CARDS.len())
        .map(|i| top + s.content_h(i, w - s.pad * 2.0) + s.pad)
        .collect();
    Grid {
        span: SPAN,
        min_w: CARD_MIN_W,
        heights,
    }
}

/// The cards, drawn in `cards` (the group's layout).
pub(super) fn draw(c: &mut Compositor, cards: &[Rect], theme: &Theme) {
    let s = Styles::of(theme);
    for (i, card) in cards.iter().enumerate().take(CARDS.len()) {
        showcase::card(c, *card, CARDS[i], theme);
        let x = card.x + s.pad;
        let room = card.w - s.pad * 2.0;
        let mut y = s.top(*card, theme);
        for (text, style) in s.lines(i) {
            let wrap = (i == 2).then_some(room);
            let shown = if wrap.is_some() {
                text.to_string()
            } else {
                TextMeasurer::truncate_to_width(text, style, room)
            };
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&shown, style, wrap),
                x,
                y,
                color: theme.colors.text.0,
            });
            let (advance, h) = TextMeasurer::measure_styled(&shown, style, wrap);
            if i == 0 {
                readout(c, style, advance, x, y, room, &s, theme);
            }
            y += h + s.gap;
        }
    }
}

/// A weight's readout after its sample: the weight and the advance the
/// measurer gave it, where the card leaves room.
#[allow(clippy::too_many_arguments)]
fn readout(
    c: &mut Compositor,
    style: &TextStyle,
    advance: f32,
    x: f32,
    y: f32,
    room: f32,
    s: &Styles,
    theme: &Theme,
) {
    let text = format!("{} · {advance:.1}px", style.font_weight);
    let at = x + advance + s.pad;
    let left = x + room - at;
    if left <= 0.0 {
        return;
    }
    let text = TextMeasurer::truncate_to_width(&text, &s.readout, left);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&text, &s.readout, None),
        x: at,
        y: y + (style.line_height - s.readout.line_height).max(0.0) / 2.0,
        color: theme.colors.text_mid.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ladder's samples are the four weights in the theme's face, and
    /// heavier never measures narrower.
    #[test]
    fn the_weight_ladder_measures_in_the_theme_face() {
        let theme = Theme::hoff_core();
        let s = Styles::of(&theme);
        let advances: Vec<f32> = s
            .weights
            .iter()
            .map(|w| TextMeasurer::measure_styled(WEIGHT_SAMPLE, w, None).0)
            .collect();
        assert!(
            s.weights
                .iter()
                .all(|w| w.font_family.as_deref() == Some("Inter"))
        );
        assert!(
            advances.windows(2).all(|a| a[1] >= a[0] - 0.5),
            "{advances:?}"
        );
    }

    /// The paragraph's card grows as it narrows: it wraps to the card.
    #[test]
    fn the_paragraph_wraps_to_its_card() {
        let theme = Theme::hoff_core();
        let narrow = grid(Rect::new(0.0, 0.0, 320.0, 800.0), &theme);
        let wide = grid(Rect::new(0.0, 0.0, 1400.0, 800.0), &theme);
        assert!(narrow.heights[2] > wide.heights[2]);
    }
}
