//! GPU-free text measurement, hit-testing, and cursor geometry.
//!
//! The render-side `TextSystem` couples its `FontSystem` to the glyph atlas
//! (needs device/queue). Layout and input handling have to shape text before
//! any GPU resource exists, so this module keeps its own `FontSystem` in a
//! thread-local: the engine is single-threaded and a `FontSystem` is too
//! expensive to build per call.
//!
//! The font set mirrors `TextSystem::new` (system fonts on desktop plus the
//! embedded faces from `super::fonts`: Inter 400/500/600/700, JetBrains Mono,
//! Codicons) so measurements match what is rasterized.

use std::cell::RefCell;
use std::num::NonZeroUsize;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight};
use lru::LruCache;

use crate::layout::ComputedBounds;

use super::backend::TextStyle;

const MEASURE_CACHE_CAPACITY: usize = 2048;

thread_local! {
    static MEASURE_CTX: RefCell<MeasureContext> = RefCell::new(MeasureContext::new());
}

struct MeasureContext {
    font_system: FontSystem,
    // Scratch buffer reused across measure/hit-test queries.
    scratch: Buffer,
    // Measurement cache keyed by (text, style, width bucket).
    cache: LruCache<MeasureKey, (f32, f32)>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct MeasureKey {
    text: String,
    font_size_bits: u32,
    line_height_bits: u32,
    font_weight: u16,
    font_family: Option<String>,
    // Wrap width rounded to whole pixels (plan: "largura-bucket").
    width_bucket: Option<i32>,
}

impl MeasureKey {
    fn new(text: &str, style: &TextStyle, max_width: Option<f32>) -> Self {
        Self {
            text: text.to_string(),
            font_size_bits: style.font_size.to_bits(),
            line_height_bits: style.line_height.to_bits(),
            font_weight: style.font_weight,
            font_family: style.font_family.clone(),
            width_bucket: max_width.map(|w| w.round() as i32),
        }
    }
}

impl MeasureContext {
    fn new() -> Self {
        let mut font_system = new_font_system();
        let scratch = Buffer::new(&mut font_system, Metrics::new(16.0, 20.8));
        Self {
            font_system,
            scratch,
            cache: LruCache::new(NonZeroUsize::new(MEASURE_CACHE_CAPACITY).unwrap()),
        }
    }

    /// Shape `text` into the scratch buffer with the given style and wrap width.
    fn prepare(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) {
        let fs = &mut self.font_system;
        self.scratch
            .set_metrics(fs, Metrics::new(style.font_size, style.line_height));
        self.scratch.set_size(fs, max_width, None);
        self.scratch
            .set_text(fs, text, &attrs_for(style), Shaping::Advanced, None);
        self.scratch.shape_until_scroll(fs, false);
    }
}

fn new_font_system() -> FontSystem {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        not(target_os = "ios")
    ))]
    let mut font_system = FontSystem::new();

    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    let mut font_system = {
        let db = cosmic_text::fontdb::Database::new();
        FontSystem::new_with_locale_and_db("en-US".to_string(), db)
    };

    // Exactly the faces TextSystem::new registers, so measurements match
    // what is rasterized regardless of installed system fonts.
    super::fonts::register_embedded_fonts(font_system.db_mut());

    font_system
}

fn attrs_for(style: &TextStyle) -> Attrs<'_> {
    let attrs = Attrs::new().weight(Weight(style.font_weight));
    match style.font_family {
        Some(ref family) => attrs.family(Family::Name(family)),
        None => attrs,
    }
}

/// Byte offset of the start of each buffer line, splitting on the same line
/// endings cosmic-text's `LineIter` recognizes (LF, CRLF, CR, LFCR).
fn line_starts(text: &str) -> Vec<usize> {
    let bytes = text.as_bytes();
    let mut starts = vec![0];
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                i += if bytes.get(i + 1) == Some(&b'\r') {
                    2
                } else {
                    1
                };
                starts.push(i);
            }
            b'\r' => {
                i += if bytes.get(i + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
                starts.push(i);
            }
            _ => i += 1,
        }
    }
    starts
}

fn measure_runs(buffer: &Buffer) -> (f32, f32) {
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    for run in buffer.layout_runs() {
        width = width.max(run.line_w);
        height = run.line_top + run.line_height;
    }
    (width, height)
}

fn hit_to_byte(buffer: &Buffer, starts: &[usize], x: f32, y: f32) -> usize {
    match buffer.hit(x, y) {
        Some(cursor) => starts.get(cursor.line).copied().unwrap_or(0) + cursor.index,
        None => 0,
    }
}

/// Caret rect for a byte offset: `(x, line_top, 0, line_height)`.
fn cursor_rect_in(buffer: &Buffer, starts: &[usize], cursor_byte: usize) -> ComputedBounds {
    let line_i = starts
        .partition_point(|&s| s <= cursor_byte)
        .saturating_sub(1);
    let local = cursor_byte - starts.get(line_i).copied().unwrap_or(0);

    let mut rect = ComputedBounds::default();
    let mut found = false;
    for run in buffer.layout_runs() {
        if !found {
            // Track the first run as fallback (empty text, offset 0).
            rect.height = run.line_height;
            found = true;
        }
        if run.line_i != line_i {
            continue;
        }
        rect.y = run.line_top;
        rect.height = run.line_height;
        for glyph in run.glyphs.iter() {
            if local >= glyph.start && local < glyph.end {
                rect.x = glyph.x;
                return rect;
            }
        }
        if let Some(last) = run.glyphs.last()
            && local >= last.end
        {
            // Caret past this run; keep end-of-run as candidate (covers both
            // end-of-line and positions on later wrapped runs of this line).
            rect.x = last.x + last.w;
        }
    }
    rect
}

// ---------------------------------------------------------------------------
// TextMeasurer — public, GPU-free measurement API
// ---------------------------------------------------------------------------

/// Stateless facade over the thread-local shaping context.
pub struct TextMeasurer;

impl TextMeasurer {
    /// Measure `text` at `font_size` with the engine default style.
    /// Returns `(width, height)`; `max_width` enables wrapping.
    pub fn measure(text: &str, font_size: f32, max_width: Option<f32>) -> (f32, f32) {
        Self::measure_styled(text, &TextStyle::new(font_size), max_width)
    }

    /// Measure `text` with an explicit style. Results are cached by
    /// (text, style, width bucket).
    pub fn measure_styled(text: &str, style: &TextStyle, max_width: Option<f32>) -> (f32, f32) {
        if text.is_empty() {
            return (0.0, 0.0);
        }
        let key = MeasureKey::new(text, style, max_width);
        MEASURE_CTX.with(|ctx| {
            let ctx = &mut *ctx.borrow_mut();
            if let Some(&size) = ctx.cache.get(&key) {
                return size;
            }
            ctx.prepare(text, style, max_width);
            let size = measure_runs(&ctx.scratch);
            ctx.cache.put(key, size);
            size
        })
    }

    /// Visual position -> byte offset, engine default style.
    pub fn hit_test(text: &str, font_size: f32, max_width: Option<f32>, x: f32, y: f32) -> usize {
        Self::hit_test_styled(text, &TextStyle::new(font_size), max_width, x, y)
    }

    /// Visual position -> byte offset with an explicit style.
    pub fn hit_test_styled(
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
        x: f32,
        y: f32,
    ) -> usize {
        if text.is_empty() {
            return 0;
        }
        MEASURE_CTX.with(|ctx| {
            let ctx = &mut *ctx.borrow_mut();
            ctx.prepare(text, style, max_width);
            hit_to_byte(&ctx.scratch, &line_starts(text), x, y)
        })
    }

    /// Byte offset -> caret x position, engine default style (single line).
    pub fn cursor_x(text: &str, font_size: f32, cursor_byte: usize) -> f32 {
        Self::cursor_x_styled(text, &TextStyle::new(font_size), None, cursor_byte)
    }

    /// Byte offset -> caret x position with an explicit style.
    pub fn cursor_x_styled(
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
        cursor_byte: usize,
    ) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        MEASURE_CTX.with(|ctx| {
            let ctx = &mut *ctx.borrow_mut();
            ctx.prepare(text, style, max_width);
            cursor_rect_in(&ctx.scratch, &line_starts(text), cursor_byte).x
        })
    }

    /// Distinct `(family, weight)` of the faces used to shape `text` with
    /// `style`, in glyph order. Diagnostic API: guards against family
    /// fallback when a requested weight has no matching embedded face
    /// (cosmic-text only keeps the requested family on an *exact* weight
    /// match, otherwise it walks per-word fallback lists).
    pub fn resolved_faces(text: &str, style: &TextStyle) -> Vec<(String, u16)> {
        MEASURE_CTX.with(|ctx| {
            let ctx = &mut *ctx.borrow_mut();
            ctx.prepare(text, style, None);
            let mut faces: Vec<(String, u16)> = Vec::new();
            for run in ctx.scratch.layout_runs() {
                for glyph in run.glyphs.iter() {
                    let Some(face) = ctx.font_system.db().face(glyph.font_id) else {
                        continue;
                    };
                    let family = face
                        .families
                        .first()
                        .map(|(name, _)| name.clone())
                        .unwrap_or_default();
                    let entry = (family, face.weight.0);
                    if !faces.contains(&entry) {
                        faces.push(entry);
                    }
                }
            }
            faces
        })
    }

    /// Shape `text` into an owned, queryable `ShapedText` (for `TextBackend`).
    pub fn shape(text: &str, style: &TextStyle, max_width: Option<f32>) -> ShapedText {
        MEASURE_CTX.with(|ctx| {
            let ctx = &mut *ctx.borrow_mut();
            let fs = &mut ctx.font_system;
            let mut buffer = Buffer::new(fs, Metrics::new(style.font_size, style.line_height));
            buffer.set_size(fs, max_width, None);
            buffer.set_text(fs, text, &attrs_for(style), Shaping::Advanced, None);
            buffer.shape_until_scroll(fs, false);
            ShapedText {
                buffer,
                line_starts: line_starts(text),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// ShapedText — owned shaped paragraph, queryable without the FontSystem
// ---------------------------------------------------------------------------

pub struct ShapedText {
    buffer: Buffer,
    line_starts: Vec<usize>,
}

impl ShapedText {
    /// `(width, height)` of the shaped text.
    pub fn size(&self) -> (f32, f32) {
        measure_runs(&self.buffer)
    }

    /// Visual position -> byte offset into the original text.
    pub fn hit_test(&self, x: f32, y: f32) -> usize {
        hit_to_byte(&self.buffer, &self.line_starts, x, y)
    }

    /// Byte offset -> caret rect `(x, y, 0, line_height)`.
    pub fn cursor_rect(&self, cursor_byte: usize) -> ComputedBounds {
        cursor_rect_in(&self.buffer, &self.line_starts, cursor_byte)
    }

    /// Byte offset -> caret x position.
    pub fn cursor_x(&self, cursor_byte: usize) -> f32 {
        self.cursor_rect(cursor_byte).x
    }
}
