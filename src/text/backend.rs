//! `TextBackend` — trait boundary that isolates the engine from the text
//! shaping library (cosmic-text today, Parley as a possible future impl).
//!
//! Signatures are adapted from the technical plan (§3.1) to the project's
//! types: sizes are `(f32, f32)` tuples, cursor positions are byte offsets
//! (matching `TextBuffer`), and rects are `layout::ComputedBounds`.

use std::ops::Range;

use crate::layout::ComputedBounds;

use super::measure::{ShapedText, TextMeasurer};

/// Default line-height multiplier used across the engine (see `builder::text`).
pub const DEFAULT_LINE_HEIGHT_FACTOR: f32 = 1.3;

/// Resolved style for a run of text. `font_family: None` means the engine
/// default family (sans-serif), matching `TextNodeKey`.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub font_family: Option<String>,
}

impl TextStyle {
    pub fn new(font_size: f32) -> Self {
        Self {
            font_size,
            line_height: font_size * DEFAULT_LINE_HEIGHT_FACTOR,
            font_weight: 400,
            font_family: None,
        }
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
