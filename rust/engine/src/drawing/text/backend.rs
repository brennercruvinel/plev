//! `TextBackend` — trait boundary that isolates the engine from the text
//! shaping library (cosmic-text today, Parley as a possible future impl).
//!
//! Signatures are adapted from the technical plan (§3.1) to the project's
//! types: sizes are `(f32, f32)` tuples, cursor positions are byte offsets
//! (matching `TextBuffer`), and rects are `layout::ComputedBounds`.

use std::ops::Range;

use crate::compose::layout::ComputedBounds;

use super::measure::{ShapedText, TextMeasurer};

/// Default line-height multiplier used across the engine (see `builder::text`).
pub const DEFAULT_LINE_HEIGHT_FACTOR: f32 = 1.3;

/// Where a line may break inside a word (CSS `overflow-wrap`). Lines
/// always break between words (UAX#14: at a space, after a hyphen or a
/// slash, between CJK ideographs); this says what happens to a word wider
/// than the line on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverflowWrap {
    /// CSS `normal`: the word stays whole and runs past the line (the
    /// recipe clips or truncates it: `TextMeasurer::truncate_to_width`,
    /// `clamp_to_lines`). Its min-content width is the longest word.
    #[default]
    Normal,
    /// CSS `anywhere` (and `break-word` as drawn): a word wider than the
    /// line breaks between its graphemes. Its min-content width is the
    /// widest grapheme, so a flex item may shrink to one; what a recipe
    /// asks for text that must never run out (an editable area, a URL).
    Anywhere,
}

impl OverflowWrap {
    /// The shaper's wrap mode for this rule on `text`. The line breaker
    /// (UAX#14 without a dictionary) finds no word bounds in the scripts
    /// written without spaces between words (Thai, Lao, Khmer, Myanmar,
    /// the Tai scripts: class SA), so a paragraph of them would be one
    /// word; there a run wider than the line breaks between graphemes
    /// under either rule, the nearest the engine gets to a browser's
    /// dictionary breaks.
    pub(crate) fn cosmic(self, text: &str) -> cosmic_text::Wrap {
        match self {
            OverflowWrap::Normal if !text.chars().any(no_word_bounds) => cosmic_text::Wrap::Word,
            _ => cosmic_text::Wrap::WordOrGlyph,
        }
    }
}

/// A letter of a script whose words the line breaker cannot find (UAX#14
/// class SA): Thai, Lao, Myanmar, Khmer, Tai Le, New Tai Lue, Tai Tham,
/// Tai Viet, Myanmar Extended A and B.
fn no_word_bounds(c: char) -> bool {
    matches!(
        c,
        '\u{0E00}'..='\u{0EFF}'
            | '\u{1000}'..='\u{109F}'
            | '\u{1780}'..='\u{17FF}'
            | '\u{1950}'..='\u{19DF}'
            | '\u{1A20}'..='\u{1AAF}'
            | '\u{AA60}'..='\u{AADF}'
            | '\u{A9E0}'..='\u{A9FF}'
    )
}

/// Resolved style for a run of text. `font_family: None` means the engine
/// default family (sans-serif), matching `TextNodeKey`.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    /// Extra advance per glyph in px (CSS `letter-spacing`). The HOFF
    /// ramp tracks 0 since the Inclusive Sans swap (2026-08); the knob
    /// stays for call sites that need explicit tracking.
    pub letter_spacing: f32,
    pub font_family: Option<String>,
    /// OpenType features the run turns on beyond the face's defaults.
    pub features: TextFeatures,
    /// What a word wider than the line does; [`OverflowWrap::Normal`]
    /// unless the recipe asks.
    pub overflow_wrap: OverflowWrap,
}

/// The OpenType features a run turns on beyond its face's defaults; the
/// shaper applies them, and they are part of the shaping key.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TextFeatures {
    /// `tnum`: every figure as wide as every other, so numbers line up
    /// in a column and a counting value does not shift its neighbours.
    pub tabular_figures: bool,
}

impl TextFeatures {
    pub const NONE: Self = Self {
        tabular_figures: false,
    };
    pub const TABULAR: Self = Self {
        tabular_figures: true,
    };

    /// The features as the shaper takes them; `None` when there are none.
    pub(crate) fn cosmic(self) -> Option<cosmic_text::FontFeatures> {
        if self == Self::NONE {
            return None;
        }
        let mut f = cosmic_text::FontFeatures::new();
        if self.tabular_figures {
            f.enable(cosmic_text::FeatureTag::new(b"tnum"));
        }
        Some(f)
    }
}

impl TextStyle {
    pub fn new(font_size: f32) -> Self {
        Self {
            font_size,
            line_height: font_size * DEFAULT_LINE_HEIGHT_FACTOR,
            font_weight: 400,
            letter_spacing: 0.0,
            font_family: None,
            features: TextFeatures::NONE,
            overflow_wrap: OverflowWrap::Normal,
        }
    }

    /// The rule for a word wider than the line (CSS `overflow-wrap`).
    pub fn with_overflow_wrap(mut self, overflow_wrap: OverflowWrap) -> Self {
        self.overflow_wrap = overflow_wrap;
        self
    }

    pub fn with_family(mut self, family: &str) -> Self {
        self.font_family = Some(family.to_string());
        self
    }

    pub fn with_weight(mut self, weight: u16) -> Self {
        self.font_weight = weight;
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }

    /// Letter spacing in px (convert em values at the call site:
    /// `em * font_size`).
    pub fn with_letter_spacing(mut self, letter_spacing: f32) -> Self {
        self.letter_spacing = letter_spacing;
        self
    }

    pub fn with_features(mut self, features: TextFeatures) -> Self {
        self.features = features;
        self
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new(16.0)
    }
}

/// A styled span of text — the unit of rich text (prerequisite for syntax
/// highlighting). The current `CosmicTextBackend` only honors a single style
/// per text (the first run); per-span attrs land with rich-text (WS-A.3).
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRun {
    pub range: Range<usize>,
    pub style: TextStyle,
}

/// Abstraction over text shaping/measuring. Implementations must not require
/// GPU resources: layout and input handling run before any surface exists.
pub trait TextBackend {
    /// A shaped line/paragraph, cacheable by callers.
    type Shaped;

    /// Shape `text` with the given style runs, wrapping at `max_width`.
    fn shape(&mut self, text: &str, runs: &[StyleRun], max_width: Option<f32>) -> Self::Shaped;

    /// Measure `text`: returns `(width, height)` after wrapping at `max_width`.
    fn measure(&mut self, text: &str, runs: &[StyleRun], max_width: Option<f32>) -> (f32, f32);

    /// Visual position -> byte offset into the original text.
    fn hit_test(&self, shaped: &Self::Shaped, x: f32, y: f32) -> usize;

    /// Byte offset -> caret rect (x, y, width=0, height=line height).
    fn cursor_geometry(&self, shaped: &Self::Shaped, cursor_byte: usize) -> ComputedBounds;

    /// Line height for a given style.
    fn line_height(&self, style: &TextStyle) -> f32;
}

/// `TextBackend` implementation backed by cosmic-text, delegating to the
/// GPU-free `TextMeasurer`.
#[derive(Debug, Default)]
pub struct CosmicTextBackend;

impl CosmicTextBackend {
    pub fn new() -> Self {
        Self
    }

    fn run_style(runs: &[StyleRun]) -> TextStyle {
        runs.first()
            .map(|run| run.style.clone())
            .unwrap_or_default()
    }
}

impl TextBackend for CosmicTextBackend {
    type Shaped = ShapedText;

    fn shape(&mut self, text: &str, runs: &[StyleRun], max_width: Option<f32>) -> Self::Shaped {
        TextMeasurer::shape(text, &Self::run_style(runs), max_width)
    }

    fn measure(&mut self, text: &str, runs: &[StyleRun], max_width: Option<f32>) -> (f32, f32) {
        TextMeasurer::measure_styled(text, &Self::run_style(runs), max_width)
    }

    fn hit_test(&self, shaped: &Self::Shaped, x: f32, y: f32) -> usize {
        shaped.hit_test(x, y)
    }

    fn cursor_geometry(&self, shaped: &Self::Shaped, cursor_byte: usize) -> ComputedBounds {
        shaped.cursor_rect(cursor_byte)
    }

    fn line_height(&self, style: &TextStyle) -> f32 {
        style.line_height
    }
}
