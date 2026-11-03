//! The frame as Puffin and Tracy see it (task-105, a POC behind the
//! `profiler` feature).
//!
//! wgpu-profiler times each pass `window::draw_frame` encodes, through
//! the pass's own timestamp writes ([`PassStamps`]): Apple's GPUs and
//! WebGPU write timestamps only where a pass starts and ends, so a pass
//! is the finest grain here too. A finished frame's timings go to Puffin
//! as a thread named "GPU" while Puffin's scopes are on, and to Tracy's
//! GPU context when the app started Tracy's client (`profiler-tracy`).
//! The CPU side is [`cpu_scope!`](crate::cpu_scope), in both.
//!
//! The profiler lives with the thread that draws, one per device: a
//! device recovered in place gets a new one.

use std::cell::RefCell;

#[doc(hidden)]
pub use puffin as __puffin;
#[cfg(all(feature = "profiler-tracy", not(target_arch = "wasm32")))]
#[doc(hidden)]
pub use tracy_client as __tracy;

/// A CPU scope named `$name`, open until the end of the block: Puffin's
/// (recorded while its scopes are on) and, with `profiler-tracy`, a
/// Tracy zone while its client runs.
#[macro_export]
macro_rules! cpu_scope {
    ($name:literal) => {
        $crate::runtime::perf::profiler::__puffin::profile_scope!($name);
        $crate::__tracy_zone!($name);
    };
}

#[cfg(all(feature = "profiler-tracy", not(target_arch = "wasm32")))]
#[doc(hidden)]
#[macro_export]
macro_rules! __tracy_zone {
    ($name:literal) => {
        let _zone = $crate::runtime::perf::profiler::__tracy::Client::running()
            .map(|c| c.span_alloc(Some($name), "", file!(), line!(), 0));
    };
}

#[cfg(not(all(feature = "profiler-tracy", not(target_arch = "wasm32"))))]
#[doc(hidden)]
#[macro_export]
macro_rules! __tracy_zone {
    ($name:literal) => {};
}

use wgpu_profiler::{GpuProfiler, GpuProfilerQuery, GpuProfilerSettings};

use super::gpu_time::PassStamps;
use crate::drawing::gpu::GpuContext;

thread_local! {
    static PROFILER: RefCell<Option<(wgpu::Device, GpuProfiler)>> = const { RefCell::new(None) };
}

/// One frame's passes timed by wgpu-profiler: taken out of the thread's
/// slot by [`FramePasses::begin`], back in it after [`FramePasses::submitted`].
pub(crate) struct FramePasses {
    device: wgpu::Device,
    profiler: GpuProfiler,
    queries: Vec<GpuProfilerQuery>,
    layers: u32,
}

impl FramePasses {
    /// The profiler of `gpu`'s device, made on its first frame; `None`
    /// when wgpu-profiler refuses the settings.
    pub(crate) fn begin(gpu: &GpuContext) -> Option<Self> {
        let kept = PROFILER
            .with_borrow_mut(Option::take)
            .filter(|(device, _)| *device == gpu.device);
        let (device, profiler) = match kept {
            Some(kept) => kept,
            None => (gpu.device.clone(), new_profiler(gpu)?),
        };
        Some(Self {
            device,
            profiler,
            queries: Vec::new(),
            layers: 0,
        })
    }

    fn pass(
        &mut self,
        label: String,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        let query = self.profiler.begin_pass_query(label, encoder);
        self.queries.push(query);
        self.queries.last()?.render_pass_timestamp_writes()
    }

    /// Before the encoder finishes: every pass closed, the queries
    /// resolved into the frame's command buffer.
    pub(crate) fn close(&mut self, encoder: &mut wgpu::CommandEncoder) {
        for query in self.queries.drain(..) {
            self.profiler.end_query(encoder, query);
        }
        self.profiler.resolve_queries(encoder);
    }

    /// After the submit: the GPU's frame ends, and the oldest finished one
    /// goes out (Tracy takes it inside wgpu-profiler; Puffin here).
    pub(crate) fn submitted(mut self, queue: &wgpu::Queue) {
        if let Err(e) = self.profiler.end_frame() {
            log::warn!("profiler: {e}");
        }
        let finished = self
            .profiler
            .process_finished_frame(queue.get_timestamp_period());
        if let Some(results) = finished.filter(|_| puffin::are_scopes_on()) {
            wgpu_profiler::puffin::output_frame_to_puffin(
                &mut puffin::GlobalProfiler::lock(),
                &results,
            );
        }
        PROFILER.with_borrow_mut(|slot| *slot = Some((self.device, self.profiler)));
    }
}

impl PassStamps for FramePasses {
    fn layer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.layers += 1;
        let label = format!("layer pass {}", self.layers);
        self.pass(label, encoder)
    }

    fn composite(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.pass("composite pass".to_owned(), encoder)
    }
}

/// The CPU's frame ends, in Puffin and Tracy: after the present
/// (`WindowHost::present`), so a frame holds what the app built for it,
/// its present included, and not the wait for the next one.
pub(crate) fn end_frame() {
    puffin::GlobalProfiler::lock().new_frame();
    #[cfg(all(feature = "profiler-tracy", not(target_arch = "wasm32")))]
    if let Some(client) = tracy_client::Client::running() {
        client.frame_mark();
    }
}

/// A profiler on `gpu`'s device: with Tracy's GPU context when Tracy's
/// client runs and the device writes timestamps on an encoder (the
/// context's clock sync does), plain otherwise.
fn new_profiler(gpu: &GpuContext) -> Option<GpuProfiler> {
    let settings = GpuProfilerSettings::default();
    #[cfg(all(feature = "profiler-tracy", not(target_arch = "wasm32")))]
    if tracy_client::Client::is_running() {
        let synced = gpu
            .device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);
        if !synced {
            log::warn!("profiler: no TIMESTAMP_QUERY_INSIDE_ENCODERS; Tracy sees the CPU only");
            return GpuProfiler::new(&gpu.device, settings).ok();
        }
        let backend = gpu.adapter().get_info().backend;
        match GpuProfiler::new_with_tracy_client(settings.clone(), backend, &gpu.device, &gpu.queue)
        {
            Ok(profiler) => return Some(profiler),
            Err(e) => log::warn!("profiler: no Tracy GPU context ({e}); Puffin only"),
        }
    }
    GpuProfiler::new(&gpu.device, settings)
        .inspect_err(|e| log::warn!("profiler: {e}"))
        .ok()
}
