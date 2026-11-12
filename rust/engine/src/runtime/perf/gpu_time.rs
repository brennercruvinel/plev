//! What the GPU itself spends on a frame, from timestamp queries.
//!
//! The layer passes and the composite pass of `window::draw_frame` each
//! write a timestamp where they start and where they end, into a query set
//! of their own frame. The queries are resolved only once the GPU has
//! finished that frame: Metal writes an end-of-pass sample when the pass's
//! fragment work completes, and a resolve in the same command buffer can
//! run before that (nothing orders a blit after a sample; the composite's
//! end read zero in the first trials). So a later frame's encoder resolves
//! the finished frame's queries and copies them into a readback buffer
//! mapped on submit; [`GpuTimestamps::collect`], after that frame's submit
//! and GPU wait, reads what the GPU has filled and records it as
//! [`Stage::GpuSpan`], [`Stage::GpuLayers`] and [`Stage::GpuComposite`].
//!
//! Nothing waits for the timestamps. With the GPU sync on the reading of a
//! frame lands in the next drawn frame (the sync's wait sees the frame
//! done, the next frame resolves and its wait sees the copy mapped);
//! without it a few frames later. A frame that finds every slot in flight
//! writes no timestamps.
//!
//! Only a context built with `RenderConfig::gpu_timestamps` on an adapter
//! with `TIMESTAMP_QUERY` has a [`GpuTimestamps`], and only a frame drawn
//! with stages on writes them; anywhere else nothing here runs and no pass
//! carries a timestamp write. The writes are per pass
//! (`RenderPassDescriptor::timestamp_writes`): writes inside a pass need
//! `TIMESTAMP_QUERY_INSIDE_PASSES`, which tile-based GPUs (Apple's) do not
//! offer, so a pass is the finest grain.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use super::stages::{self, Stage};

/// Layer passes one frame can time; a frame that encodes more leaves the
/// rest untimed (their time still falls inside the span).
const MAX_LAYER_PASSES: u32 = 63;

/// The composite pass writes queries 0 and 1, the layer passes 2 onward.
const FIRST_LAYER_QUERY: u32 = 2;

/// A query set's size.
const QUERIES: u32 = FIRST_LAYER_QUERY + 2 * MAX_LAYER_PASSES;

/// Frames whose timestamps can be in flight at once: written, waiting for
/// the GPU, resolving or mapped. Without the GPU sync a frame's reading
/// comes back three frames later, so four keep every frame timed.
const SLOTS: usize = 4;

/// A slot's state, moved forward by the frames and the GPU callbacks.
const FREE: u8 = 0;
/// A frame wrote its passes' timestamps; the GPU has not finished it.
const SUBMITTED: u8 = 1;
/// The GPU finished the frame: every sample is written.
const DONE: u8 = 2;
/// A later frame resolves and copies the queries, mapped on its submit.
const RESOLVING: u8 = 3;
const MAPPED: u8 = 4;
const FAILED: u8 = 5;

/// The device features to ask for: `TIMESTAMP_QUERY` when `wanted` and
/// the adapter has it, nothing otherwise.
pub(crate) fn device_features(adapter: wgpu::Features, wanted: bool) -> wgpu::Features {
    if !wanted {
        return wgpu::Features::empty();
    }
    if adapter.contains(wgpu::Features::TIMESTAMP_QUERY) {
        wgpu::Features::TIMESTAMP_QUERY
    } else {
        log::warn!("gpu timestamps: the adapter has no TIMESTAMP_QUERY; the gpu stages stay empty");
        wgpu::Features::empty()
    }
}

/// The query sets and buffers of a context with timestamps, one slot per
/// frame in flight.
pub(crate) struct GpuTimestamps {
    slots: Vec<Slot>,
    /// Nanoseconds per timestamp tick (`Queue::get_timestamp_period`).
    period: f32,
}

struct Slot {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    state: Arc<AtomicU8>,
    /// Queries the frame in this slot wrote: the composite's pair, then
    /// one pair per layer pass.
    queries: AtomicU32,
}

impl GpuTimestamps {
    /// The query resources when `device` has `TIMESTAMP_QUERY`; `None`
    /// otherwise.
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let buffer = |label, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: bytes(QUERIES),
                usage,
                mapped_at_creation: false,
            })
        };
        let slots = (0..SLOTS)
            .map(|_| Slot {
                set: device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("gpu_timestamps"),
                    ty: wgpu::QueryType::Timestamp,
                    count: QUERIES,
                }),
                resolve: buffer(
                    "gpu_timestamps_resolve",
                    wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                ),
                readback: buffer(
                    "gpu_timestamps_readback",
                    wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                ),
                state: Arc::new(AtomicU8::new(FREE)),
                queries: AtomicU32::new(0),
            })
            .collect();
        Some(Self {
            slots,
            period: queue.get_timestamp_period(),
        })
    }

    /// Encode, into this frame's `encoder`, the resolve of every earlier
    /// frame the GPU has finished and its copy into the slot's readback,
    /// mapped once this frame's work is done.
    pub(crate) fn resolve_finished(&self, encoder: &mut wgpu::CommandEncoder) {
        for slot in &self.slots {
            if slot.state.load(Ordering::Acquire) != DONE {
                continue;
            }
            let queries = slot.queries.load(Ordering::Relaxed);
            let size = bytes(queries);
            encoder.resolve_query_set(&slot.set, 0..queries, &slot.resolve, 0);
            encoder.copy_buffer_to_buffer(&slot.resolve, 0, &slot.readback, 0, size);
            slot.state.store(RESOLVING, Ordering::Release);
            let state = Arc::clone(&slot.state);
            encoder.map_buffer_on_submit(&slot.readback, wgpu::MapMode::Read, ..size, move |r| {
                state.store(if r.is_ok() { MAPPED } else { FAILED }, Ordering::Release);
            });
        }
    }

    /// Read every readback the GPU has filled, record its frame's GPU time
    /// (when stages are on) and free the slot. Never waits: a slot still
    /// in flight is left for a later frame.
    pub(crate) fn collect(&self) {
        for slot in &self.slots {
            match slot.state.load(Ordering::Acquire) {
                MAPPED => {
                    let ticks = slot.ticks();
                    slot.readback.unmap();
                    slot.state.store(FREE, Ordering::Release);
                    GpuReading::from_ticks(&ticks, self.period).record();
                }
                FAILED => {
                    log::warn!("gpu timestamps: a readback failed to map; that frame is untimed");
                    slot.state.store(FREE, Ordering::Release);
                }
                _ => {}
            }
        }
    }
}

impl Slot {
    /// The resolved ticks, while the readback is mapped.
    fn ticks(&self) -> Vec<u64> {
        let queries = self.queries.load(Ordering::Relaxed);
        let view = self.readback.slice(..bytes(queries)).get_mapped_range();
        view.chunks_exact(8)
            .map(|q| q.try_into().map(u64::from_ne_bytes).unwrap_or(0))
            .collect()
    }
}

/// Bytes of `queries` resolved timestamps.
fn bytes(queries: u32) -> u64 {
    u64::from(queries) * u64::from(wgpu::QUERY_SIZE)
}

/// The timestamp writes of one frame's passes, handed out in the order
/// the passes are encoded.
pub(crate) struct FrameStamps<'a> {
    slot: &'a Slot,
    layer_passes: u32,
}

impl<'a> FrameStamps<'a> {
    /// This frame's timestamps: stages are on, the context has timestamps
    /// and a slot is free. `None` otherwise, and no pass of the frame
    /// writes one.
    pub(crate) fn begin(timestamps: Option<&'a GpuTimestamps>) -> Option<Self> {
        let timestamps = timestamps.filter(|_| stages::is_enabled())?;
        let slot = timestamps
            .slots
            .iter()
            .find(|s| s.state.load(Ordering::Acquire) == FREE)?;
        Some(Self {
            slot,
            layer_passes: 0,
        })
    }

    /// The writes of the next layer pass (a resumed pass is a pass of its
    /// own), or `None` once [`MAX_LAYER_PASSES`] are handed out.
    pub(crate) fn layer_pass(&mut self) -> Option<wgpu::RenderPassTimestampWrites<'a>> {
        if self.layer_passes == MAX_LAYER_PASSES {
            return None;
        }
        let first = FIRST_LAYER_QUERY + 2 * self.layer_passes;
        self.layer_passes += 1;
        Some(self.writes(first))
    }

    /// The writes of the composite pass.
    pub(crate) fn composite_pass(&self) -> wgpu::RenderPassTimestampWrites<'a> {
        self.writes(0)
    }

    fn writes(&self, first: u32) -> wgpu::RenderPassTimestampWrites<'a> {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.slot.set,
            beginning_of_pass_write_index: Some(first),
            end_of_pass_write_index: Some(first + 1),
        }
    }

    /// After the frame's submit: the slot waits for the GPU to finish the
    /// work `queue` has so far, then a later frame resolves it.
    pub(crate) fn submitted(self, queue: &wgpu::Queue) {
        let queries = FIRST_LAYER_QUERY + 2 * self.layer_passes;
        self.slot.queries.store(queries, Ordering::Relaxed);
        self.slot.state.store(SUBMITTED, Ordering::Release);
        let state = Arc::clone(&self.slot.state);
        queue.on_submitted_work_done(move || state.store(DONE, Ordering::Release));
    }
}

/// Who hands a frame's passes their timestamp writes, in the order the
/// passes are encoded: the engine's own [`FrameStamps`], or wgpu-profiler
/// with the `profiler` feature (`perf::profiler`).
pub(crate) trait PassStamps {
    fn layer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>>;
    fn composite(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>>;
}

impl PassStamps for FrameStamps<'_> {
    fn layer(
        &mut self,
        _: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.layer_pass()
    }

    fn composite(
        &mut self,
        _: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        Some(self.composite_pass())
    }
}

/// What one frame's timestamps say, in nanoseconds, cut along the GPU's
/// timeline rather than per pass: a tile-based GPU runs the composite's
/// vertex stage alongside the layer passes and holds its fragments until
/// their textures are done, so the composite's own start-to-end would
/// count the layers' time twice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuReading {
    /// The earliest start written to the latest end written; `None` when
    /// none was.
    pub span: Option<u64>,
    /// The first layer pass's start to the last one's end; `None` without
    /// a layer pass.
    pub layers: Option<u64>,
    /// Layer passes with both timestamps.
    pub layer_passes: u64,
    /// The composite's end minus the later of its start and the last
    /// layer pass's end: its time once the layers are done.
    pub composite: Option<u64>,
    /// Passes without a usable pair: a timestamp never written (zero, or
    /// all ones), or an end before its start. On Metal a pass that draws
    /// nothing (a dirty layer with no node: a clear) samples its start
    /// and never its end.
    pub incomplete: u64,
}

/// The earliest start and the latest end seen.
#[derive(Clone, Copy)]
struct Bounds {
    first: u64,
    last: u64,
}

impl Bounds {
    const NONE: Bounds = Bounds {
        first: u64::MAX,
        last: 0,
    };

    fn add(&mut self, start: Option<u64>, end: Option<u64>) {
        if let Some(t) = start {
            self.first = self.first.min(t);
        }
        if let Some(t) = end {
            self.last = self.last.max(t);
        }
    }

    fn end(self) -> Option<u64> {
        (self.last > 0).then_some(self.last)
    }

    fn ticks(self) -> Option<u64> {
        (self.first <= self.last).then(|| self.last - self.first)
    }
}

impl GpuReading {
    /// `ticks` are the composite's pair, then one pair per layer pass;
    /// `period` is nanoseconds per tick.
    pub(crate) fn from_ticks(ticks: &[u64], period: f32) -> Self {
        let nanos = |t: u64| (t as f64 * f64::from(period)).round() as u64;
        let written = |t: u64| (t != 0 && t != u64::MAX).then_some(t);
        let mut reading = Self::default();
        let (mut all, mut layers) = (Bounds::NONE, Bounds::NONE);
        let mut composite = (None, None);
        for (i, pair) in ticks.chunks_exact(2).enumerate() {
            let &[begin, end] = pair else {
                continue;
            };
            // a backwards pair bounds nothing; one side written still does
            let (start, stop) = match (written(begin), written(end)) {
                (Some(b), Some(e)) if e < b => (None, None),
                sides => sides,
            };
            let full = start.is_some() && stop.is_some();
            reading.incomplete += u64::from(!full);
            all.add(start, stop);
            if i == 0 {
                composite = (start, stop);
            } else {
                layers.add(start, stop);
                reading.layer_passes += u64::from(full);
            }
        }
        reading.span = all.ticks().map(nanos);
        reading.layers = layers.ticks().map(nanos);
        reading.composite = match composite {
            (start, Some(stop)) => start
                .max(layers.end())
                .map(|from| nanos(stop.saturating_sub(from))),
            _ => None,
        };
        reading
    }

    /// Record the reading into this thread's stage frame (nothing while
    /// stages are off or when no timestamp was written).
    fn record(&self) {
        if self.incomplete > 0 {
            log::debug!(
                "gpu timestamps: {} pass(es) without a pair",
                self.incomplete
            );
        }
        let Some(span) = self.span else {
            return;
        };
        stages::add(Stage::GpuSpan, 0, span);
        stages::add(
            Stage::GpuLayers,
            self.layer_passes,
            self.layers.unwrap_or(0),
        );
        if let Some(composite) = self.composite {
            stages::add(Stage::GpuComposite, 0, composite);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_feature_is_asked_only_when_wanted_and_offered() {
        let offered = wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::DEPTH_CLIP_CONTROL;
        assert_eq!(device_features(offered, false), wgpu::Features::empty());
        assert_eq!(
            device_features(offered, true),
            wgpu::Features::TIMESTAMP_QUERY,
            "only the timestamp feature, never the rest of what is offered"
        );
        assert_eq!(
            device_features(wgpu::Features::DEPTH_CLIP_CONTROL, true),
            wgpu::Features::empty()
        );
    }

    #[test]
    fn a_reading_cuts_the_timeline_into_layers_and_composite() {
        // composite 900..1000, layers 100..400 and 450..800
        let reading = GpuReading::from_ticks(&[900, 1000, 100, 400, 450, 800], 1.0);
        assert_eq!(
            reading,
            GpuReading {
                span: Some(900),
                layers: Some(700),
                layer_passes: 2,
                composite: Some(100),
                incomplete: 0,
            }
        );
    }

    #[test]
    fn a_composite_started_alongside_the_layers_counts_from_their_end() {
        // the composite's vertex stage starts at 150, its fragments wait
        // for the layer pass that ends at 600
        let reading = GpuReading::from_ticks(&[150, 1000, 100, 600], 1.0);
        assert_eq!(reading.layers, Some(500));
        assert_eq!(reading.composite, Some(400));
        assert_eq!(reading.span, Some(900));
    }

    #[test]
    fn a_composite_alone_is_its_own_time() {
        let reading = GpuReading::from_ticks(&[300, 700], 1.0);
        assert_eq!(reading.layers, None);
        assert_eq!(reading.layer_passes, 0);
        assert_eq!(reading.composite, Some(400));
        assert_eq!(reading.span, Some(400));
    }

    #[test]
    fn ticks_convert_by_the_period() {
        let reading = GpuReading::from_ticks(&[10, 22, 1, 4], 83.333);
        assert_eq!(reading.composite, Some(1000), "12 ticks of 83.333 ns");
        assert_eq!(reading.layers, Some(250));
        assert_eq!(reading.span, Some(1750), "ticks 1 to 22");
    }

    #[test]
    fn a_pass_without_both_timestamps_counts_as_incomplete() {
        // composite never written, one full layer pass, one backwards
        let reading = GpuReading::from_ticks(&[0, 0, 100, 200, 500, 400], 1.0);
        assert_eq!(reading.composite, None, "zero is a timestamp never written");
        assert_eq!((reading.layer_passes, reading.layers), (1, Some(100)));
        assert_eq!(reading.incomplete, 2);
        assert_eq!(reading.span, Some(100), "a backwards pair bounds nothing");
    }

    #[test]
    fn a_pass_that_sampled_one_side_still_bounds_the_timeline() {
        // a composite with its end only (all ones at its start), an empty
        // layer pass (start only), a full one
        let ticks = [u64::MAX, 900, 50, 0, 100, 200];
        let reading = GpuReading::from_ticks(&ticks, 1.0);
        assert_eq!(reading.layer_passes, 1);
        assert_eq!(reading.layers, Some(150), "from the empty pass's start");
        assert_eq!(reading.composite, Some(700), "from the last layer end");
        assert_eq!(reading.incomplete, 2);
        assert_eq!(reading.span, Some(850));
    }

    #[test]
    fn no_timestamp_written_reads_as_no_span_and_records_nothing() {
        for ticks in [&[][..], &[0, 0][..], &[7][..]] {
            let reading = GpuReading::from_ticks(ticks, 1.0);
            assert_eq!(reading.span, None, "{ticks:?}");
        }
        stages::enable(true);
        let _ = stages::take();
        GpuReading::from_ticks(&[0, 0, 5, 1], 1.0).record();
        let frame = stages::take();
        stages::enable(false);
        assert!(frame.is_empty(), "{frame:?}");
    }

    #[test]
    fn a_reading_records_the_three_gpu_stages_only_while_on() {
        let reading = GpuReading::from_ticks(&[900, 1000, 100, 400], 1.0);
        stages::enable(false);
        reading.record();
        assert!(stages::take().is_empty());

        stages::enable(true);
        let _ = stages::take();
        reading.record();
        let frame = stages::take();
        stages::enable(false);
        assert_eq!(frame.get(Stage::GpuSpan).nanos, 900);
        let layers = frame.get(Stage::GpuLayers);
        assert_eq!((layers.calls, layers.items, layers.nanos), (1, 1, 300));
        assert_eq!(frame.get(Stage::GpuComposite).nanos, 100);
        assert_eq!(frame.gpu_micros(), Some(1));
    }
}
