//! The measurer's own font system and caches, one per thread (see
//! `super::measure`), and the one door to them: [`with_ctx`] brings the
//! faces a target fetches late (`super::on_demand`) into the font system
//! before any query reads it, dropping every measurement made without them.

use std::cell::RefCell;
use std::num::NonZeroUsize;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight};
use lru::LruCache;

use crate::runtime::perf::stages::{self, Stage};

use super::backend::TextStyle;
use super::cluster_cache::{CLUSTER_CACHE_BUDGET, ClusterCache};
use super::measure::LineMetrics;

const MEASURE_CACHE_CAPACITY: usize = 2048;
const VMETRICS_CACHE_CAPACITY: usize = 256;

thread_local! {
    static MEASURE_CTX: RefCell<MeasureContext> = RefCell::new(MeasureContext::new());
}

pub(super) struct MeasureContext {
    pub(super) font_system: FontSystem,
    // Scratch buffer reused across measure/hit-test queries.
    pub(super) scratch: Buffer,
    // Measurement cache keyed by (text, style, width bucket).
    pub(super) cache: LruCache<MeasureKey, (f32, f32)>,
    // Vertical metrics per style (small: one entry per distinct style).
    pub(super) vmetrics_cache: LruCache<VMetricsKey, LineMetrics>,
    // Shaped clusters per line of text (see `super::cluster_cache`).
    pub(super) clusters: ClusterCache,
    // The late faces this font system holds (`super::on_demand`) and the
    // generation they are of.
    late: Vec<cosmic_text::fontdb::ID>,
    late_generation: u64,
}

/// Runs `f` on this thread's measure context, the late faces brought in
/// first: every query goes through here, so none reads a measurement
/// made before a face it needed arrived.
pub(super) fn with_ctx<R>(f: impl FnOnce(&mut MeasureContext) -> R) -> R {
    MEASURE_CTX.with(|ctx| {
        let ctx = &mut *ctx.borrow_mut();
        ctx.sync_late();
        f(ctx)
    })
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct MeasureKey {
    text: String,
    font_size_bits: u32,
    line_height_bits: u32,
    font_weight: u16,
    letter_spacing_bits: u32,
    font_family: Option<String>,
    // Wrap width rounded to whole pixels (plan: "largura-bucket").
    width_bucket: Option<i32>,
}

impl MeasureKey {
    pub(super) fn new(text: &str, style: &TextStyle, max_width: Option<f32>) -> Self {
        Self {
            text: text.to_string(),
            font_size_bits: style.font_size.to_bits(),
            line_height_bits: style.line_height.to_bits(),
            font_weight: style.font_weight,
            letter_spacing_bits: style.letter_spacing.to_bits(),
            font_family: style.font_family.clone(),
            width_bucket: max_width.map(|w| w.round() as i32),
        }
    }
}

/// Cache key for [`LineMetrics`]: the vertical metrics of a style do not
/// depend on the text or on letter spacing.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct VMetricsKey {
    font_size_bits: u32,
    line_height_bits: u32,
    font_weight: u16,
    font_family: Option<String>,
}

impl VMetricsKey {
    pub(super) fn new(style: &TextStyle) -> Self {
        Self {
            font_size_bits: style.font_size.to_bits(),
            line_height_bits: style.line_height.to_bits(),
            font_weight: style.font_weight,
            font_family: style.font_family.clone(),
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
            vmetrics_cache: LruCache::new(NonZeroUsize::new(VMETRICS_CACHE_CAPACITY).unwrap()),
            clusters: ClusterCache::new(CLUSTER_CACHE_BUDGET),
            late: Vec::new(),
            late_generation: 0,
        }
    }

    /// Loads the late faces again when more arrived, and drops what was
    /// measured or shaped without them.
    fn sync_late(&mut self) {
        let generation = super::on_demand::generation();
        if generation == self.late_generation {
            return;
        }
        self.late = super::on_demand::load(self.font_system.db_mut(), &self.late);
        self.late_generation = generation;
        self.cache.clear();
        self.vmetrics_cache.clear();
        self.clusters.clear();
    }

    /// Shape `text` into the scratch buffer with the given style and wrap width.
    pub(super) fn prepare(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) {
        let _probe = stages::span(Stage::Shape).items(text.len() as u64);
        super::on_demand::want(text);
        let fs = &mut self.font_system;
        self.scratch
            .set_metrics(fs, Metrics::new(style.font_size, style.line_height));
        self.scratch.set_size(fs, max_width, None);
        self.scratch
            .set_text(fs, text, &attrs_for(style), Shaping::Advanced, None);
        self.scratch.shape_until_scroll(fs, false);
    }
}

/// Lines in this thread's cluster cache (tests read the cache through it).
#[cfg(test)]
pub(super) fn cached_cluster_lines() -> usize {
    with_ctx(|ctx| ctx.clusters.len())
}

fn new_font_system() -> FontSystem {
    // Exactly the faces TextSystem::new registers, so measurements match
    // what is rasterized regardless of installed system fonts.
    super::fonts::font_system(super::fonts::FaceSet::Target).0
}

pub(super) fn attrs_for(style: &TextStyle) -> Attrs<'_> {
    let mut attrs = Attrs::new().weight(Weight(style.font_weight));
    if style.letter_spacing != 0.0 && style.font_size > 0.0 {
        // cosmic-text tracking is in EM; the engine API is px.
        attrs = attrs.letter_spacing(style.letter_spacing / style.font_size);
    }
    match style.font_family {
        Some(ref family) => attrs.family(Family::Name(family)),
        None => attrs,
    }
}
