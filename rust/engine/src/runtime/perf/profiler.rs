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
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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

/// Profilers a frame can take, one per frame in flight: a profiler is
/// busy from its frame's submit until the GPU has finished that frame.
const POOL: usize = 4;

thread_local! {
    static PROFILERS: RefCell<Option<Pool>> = const { RefCell::new(None) };
}

/// The profilers of one device.
struct Pool {
    device: wgpu::Device,
    /// Free to time a frame.
    idle: Vec<GpuProfiler>,
    /// A frame submitted, its queries closed and not resolved yet: Metal
    /// writes a pass's end only once the pass's fragments are done, and a
    /// resolve before that (in the frame's command buffer, or the next
    /// one) reads zero. They are resolved once the GPU says the frame is
    /// done, as `perf::gpu_time` does with the engine's own timestamps.
    waiting: Vec<(GpuProfiler, Arc<AtomicBool>)>,
    made: usize,
}

impl Pool {
    /// The frames the GPU finished are resolved and ended, and every
    /// frame whose timings came back goes out: Tracy takes it inside
    /// wgpu-profiler, Puffin here while its scopes are on.
    fn pump(&mut self, queue: &wgpu::Queue) {
        let (done, busy): (Vec<_>, Vec<_>) = std::mem::take(&mut self.waiting)
            .into_iter()
            .partition(|(_, done)| done.load(Ordering::Acquire));
        self.waiting = busy;
        for (mut profiler, _) in done {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("profiler_resolve"),
                });
            profiler.resolve_queries(&mut encoder);
            queue.submit(std::iter::once(encoder.finish()));
            if let Err(e) = profiler.end_frame() {
                log::warn!("profiler: {e}");
            }
            self.idle.push(profiler);
        }
        let period = queue.get_timestamp_period();
        for profiler in &mut self.idle {
            while let Some(results) = profiler.process_finished_frame(period) {
                if puffin::are_scopes_on() {
                    let mut puffin = puffin::GlobalProfiler::lock();
                    wgpu_profiler::puffin::output_frame_to_puffin(&mut puffin, &results);
                }
            }
        }
    }
}

/// One frame's passes timed by wgpu-profiler, on a profiler of the pool
/// that [`FramePasses::submitted`] hands back to it.
pub(crate) struct FramePasses {
    profiler: GpuProfiler,
    queries: Vec<GpuProfilerQuery>,
    layers: u32,
}

impl FramePasses {
    /// A free profiler of `gpu`'s device, made while the pool has room;
    /// `None` when every one is busy (the frame goes untimed) or
    /// wgpu-profiler refuses the settings.
    pub(crate) fn begin(gpu: &GpuContext) -> Option<Self> {
        let mut pool = PROFILERS
            .take()
            .filter(|pool| pool.device == gpu.device)
            .unwrap_or_else(|| Pool {
                device: gpu.device.clone(),
                idle: Vec::new(),
                waiting: Vec::new(),
                made: 0,
            });
        pool.pump(&gpu.queue);
        let profiler = match pool.idle.pop() {
            Some(profiler) => Some(profiler),
            None if pool.made < POOL => {
                pool.made += 1;
                new_profiler(gpu)
            }
            None => None,
        };
        PROFILERS.set(Some(pool));
        Some(Self {
            profiler: profiler?,
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

    /// Before the encoder finishes: every pass's query closed.
    pub(crate) fn close(&mut self, encoder: &mut wgpu::CommandEncoder) {
        for query in self.queries.drain(..) {
            self.profiler.end_query(encoder, query);
        }
    }

    /// After the frame's submit: the profiler waits in the pool until the
    /// GPU has done the work `queue` has so far.
    pub(crate) fn submitted(self, queue: &wgpu::Queue) {
        let done = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&done);
        queue.on_submitted_work_done(move || flag.store(true, Ordering::Release));
        PROFILERS.with_borrow_mut(|pool| {
            if let Some(pool) = pool {
                pool.waiting.push((self.profiler, done));
            }
        });
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
