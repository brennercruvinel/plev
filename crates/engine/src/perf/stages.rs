//! What each stage of a frame costs: how many times it ran, how much it
//! handled and how long it took, per frame. Off by default; [`enable`]
//! turns it on for the calling thread.
//!
//! A probe is one call at the site it measures: [`span`] times the scope
//! it lives in, [`count`] adds without a clock. With stages off a probe is
//! one relaxed atomic load and a branch: no clock read, no thread-local
//! access, no allocation. With stages on it reads the clock and adds into
//! this thread's frame; [`take`] hands that frame over and starts the
//! next one. The owner of the frame loop calls it once per frame: it is
//! the only frame boundary the counters know, so everything between two
//! takes (the events, the tick, the scene build, the draw) is one frame.
//!
//! Stages nest: a span's time includes every stage timed inside it (the
//! scene build includes the text it measured, a draw includes its
//! resolve). [`Stage`] says what each stage's items are.
//!
//! Records are per thread. The engine runs a frame on one thread, and a
//! test that turns stages on never sees another test's records.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicUsize, Ordering};

use web_time::Instant;

/// Threads with stages on. While it is zero every probe is one relaxed
/// load: the thread-local flag is only read once some thread turned
/// stages on.
static THREADS_ON: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static ON: Cell<bool> = const { Cell::new(false) };
    static GPU_SYNC: Cell<bool> = const { Cell::new(false) };
    static FRAME: RefCell<StageFrame> = const { RefCell::new(StageFrame::EMPTY) };
}

/// A stage of the frame, in the order a frame runs them. The ones marked
/// (app) are recorded by the app's frame loop, the rest by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    /// The whole frame as the app's loop sees it: events, tick, scene
    /// build and draw. (app)
    Frame,
    /// The app's event handlers; items: events delivered. (app)
    Input,
    /// A semantic snapshot built and finished where an app asks for one.
    /// (app)
    Projection,
    /// The animation tick before the scene is built. (app)
    Tick,
    /// Building the scene; items: scene nodes in every layer. (app)
    SceneBuild,
    /// `LayoutEngine::compute`; items: layout items.
    Layout,
    /// `TextMeasurer::measure_styled` answered by its cache.
    MeasureHit,
    /// `TextMeasurer::measure_styled` that had to shape; the time is the
    /// whole miss, its shaping included.
    MeasureMiss,
    /// Every shaping the measurer runs: its cache misses and the hit
    /// tests and caret queries it never caches; items: bytes shaped.
    Shape,
    /// An id bound to its rect (`Compositor::bind_to_layer`, the first
    /// bind of an id in a frame).
    Bind,
    /// `Compositor::hit`; items: bindings scanned.
    HitTest,
    /// The frame tail, `window::draw_frame`.
    Draw,
    /// `Compositor::resolve`: layer hashes, geometry of the dirty layers,
    /// their buffer uploads.
    Resolve,
    /// A layer's scene hash, one call per layer hashed; items: nodes.
    LayerHash,
    /// A dirty layer's geometry rebuilt; items: the layer's nodes.
    Geometry,
    /// Text of the dirty layers shaped, rasterized and emitted; items:
    /// glyphs in the dirty layers.
    TextResolve,
    /// Bytes written into GPU buffers (vertices and indices of shapes and
    /// text), one call per write; items: bytes.
    Upload,
    /// Layer passes, composite pass and submit; items: draw calls.
    Encode,
    /// Waiting for the GPU to run what the frame submitted, only with
    /// [`set_gpu_sync`]: the GPU time of the frame plus the latency of
    /// seeing it done, an upper bound.
    GpuWait,
}

impl Stage {
    /// How many stages there are.
    pub const COUNT: usize = 19;

    /// Every stage, in frame order (`ALL[s as usize] == s`).
    pub const ALL: [Stage; Stage::COUNT] = [
        Stage::Frame,
        Stage::Input,
        Stage::Projection,
        Stage::Tick,
        Stage::SceneBuild,
        Stage::Layout,
        Stage::MeasureHit,
        Stage::MeasureMiss,
        Stage::Shape,
        Stage::Bind,
        Stage::HitTest,
        Stage::Draw,
        Stage::Resolve,
        Stage::LayerHash,
        Stage::Geometry,
        Stage::TextResolve,
        Stage::Upload,
        Stage::Encode,
        Stage::GpuWait,
    ];

    /// Short name for a table row.
    pub fn name(self) -> &'static str {
        match self {
            Stage::Frame => "frame",
            Stage::Input => "input",
            Stage::Projection => "semantic projection",
            Stage::Tick => "tick",
            Stage::SceneBuild => "scene build",
            Stage::Layout => "layout (LayoutEngine)",
            Stage::MeasureHit => "text measure, cache hit",
            Stage::MeasureMiss => "text measure, cache miss",
            Stage::Shape => "text shaping (measurer)",
            Stage::Bind => "semantic bind",
            Stage::HitTest => "hit test (Compositor::hit)",
            Stage::Draw => "draw (frame tail)",
            Stage::Resolve => "compositor resolve",
            Stage::LayerHash => "layer hash",
            Stage::Geometry => "geometry rebuild",
            Stage::TextResolve => "text resolve",
            Stage::Upload => "gpu buffer upload",
            Stage::Encode => "encode and submit",
            Stage::GpuWait => "gpu wait",
        }
    }

    /// What the stage's items count, or `None` when it counts none.
    pub fn items(self) -> Option<&'static str> {
        match self {
            Stage::Input => Some("events"),
            Stage::SceneBuild | Stage::LayerHash | Stage::Geometry => Some("nodes"),
            Stage::Layout => Some("layout items"),
            Stage::Shape | Stage::Upload => Some("bytes"),
            Stage::HitTest => Some("bindings"),
            Stage::TextResolve => Some("glyphs"),
            Stage::Encode => Some("draw calls"),
            _ => None,
        }
    }
}

/// One stage's cost in one frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StageCost {
    /// Times the stage ran.
    pub calls: u64,
    /// What it handled, in the unit of [`Stage::items`].
    pub items: u64,
    /// Time spent in it; zero for a stage recorded by [`count`].
    pub nanos: u64,
}

impl StageCost {
    const ZERO: StageCost = StageCost {
        calls: 0,
        items: 0,
        nanos: 0,
    };

    /// The time in microseconds.
    pub fn micros(&self) -> f64 {
        self.nanos as f64 / 1_000.0
    }
}

/// Every stage's cost in one frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageFrame {
    costs: [StageCost; Stage::COUNT],
}

impl Default for StageFrame {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl StageFrame {
    const EMPTY: StageFrame = StageFrame {
        costs: [StageCost::ZERO; Stage::COUNT],
    };

    pub fn get(&self, stage: Stage) -> StageCost {
        self.costs[stage as usize]
    }

    /// Nothing was recorded.
    pub fn is_empty(&self) -> bool {
        self.costs.iter().all(|c| c.calls == 0)
    }

    fn add(&mut self, stage: Stage, items: u64, nanos: u64) {
        let cost = &mut self.costs[stage as usize];
        cost.calls += 1;
        cost.items += items;
        cost.nanos += nanos;
    }
}

/// Stages on this thread are on.
#[inline]
pub fn is_enabled() -> bool {
    THREADS_ON.load(Ordering::Relaxed) != 0 && ON.with(Cell::get)
}

/// Turn stages on or off for this thread. Turning them off drops what the
/// current frame recorded and the GPU sync with it.
pub fn enable(on: bool) {
    ON.with(|flag| {
        if flag.replace(on) != on {
            if on {
                THREADS_ON.fetch_add(1, Ordering::Relaxed);
            } else {
                THREADS_ON.fetch_sub(1, Ordering::Relaxed);
            }
        }
    });
    if !on {
        GPU_SYNC.with(|sync| sync.set(false));
        take();
    }
}

/// With stages on, make the frame tail wait for the GPU after submit and
/// record the wait as [`Stage::GpuWait`] (see [`wait_for_gpu`]). It
/// serializes the CPU and the GPU and spins a core while it waits, so it
/// is for measuring only; it never changes a pixel.
pub fn set_gpu_sync(on: bool) {
    GPU_SYNC.with(|sync| sync.set(on));
}

/// What this thread recorded since the last take; the next frame starts
/// empty.
pub fn take() -> StageFrame {
    FRAME.with(|frame| {
        frame
            .try_borrow_mut()
            .map(|mut f| std::mem::take(&mut *f))
            .unwrap_or_default()
    })
}

/// One run of `stage` that handled `items`, with no time.
#[inline]
pub fn count(stage: Stage, items: u64) {
    if is_enabled() {
        record(stage, items, 0);
    }
}

/// Time the scope the returned span lives in as one run of `stage`.
#[inline]
pub fn span(stage: Stage) -> Span {
    Span {
        stage,
        items: 0,
        start: if is_enabled() {
            Some(Instant::now())
        } else {
            None
        },
    }
}

/// Run `wait` (a blocking wait for the GPU) and time it as
/// [`Stage::GpuWait`], only when stages and the GPU sync are both on.
#[inline]
pub fn gpu_wait(wait: impl FnOnce()) {
    if is_enabled() && GPU_SYNC.with(Cell::get) {
        let _wait = span(Stage::GpuWait);
        wait();
    }
}

/// With the GPU sync on, wait until `device` has run everything submitted
/// and record the wait as [`Stage::GpuWait`]. The wait spins on
/// `PollType::Poll`: wgpu's blocking wait sleeps a millisecond between
/// checks on metal, which would round every reading up to that step.
#[inline]
pub fn wait_for_gpu(device: &wgpu::Device) {
    gpu_wait(|| spin_until_idle(device));
}

/// How long the GPU wait spins before it gives up on a queue that does not
/// drain (a lost device).
#[cfg(not(target_arch = "wasm32"))]
const GPU_WAIT_LIMIT: std::time::Duration = std::time::Duration::from_secs(2);

#[cfg(not(target_arch = "wasm32"))]
fn spin_until_idle(device: &wgpu::Device) {
    let start = Instant::now();
    loop {
        match device.poll(wgpu::PollType::Poll) {
            Ok(status) if status.is_queue_empty() => return,
            Ok(_) if start.elapsed() < GPU_WAIT_LIMIT => std::hint::spin_loop(),
            Ok(_) => {
                log::warn!("gpu wait: the queue did not drain in {GPU_WAIT_LIMIT:?}");
                return;
            }
            Err(e) => {
                log::warn!("gpu wait: {e}");
                return;
            }
        }
    }
}

/// WebGPU runs its callbacks from the browser's loop: nothing to wait on.
#[cfg(target_arch = "wasm32")]
fn spin_until_idle(_device: &wgpu::Device) {}

fn record(stage: Stage, items: u64, nanos: u64) {
    FRAME.with(|frame| {
        if let Ok(mut f) = frame.try_borrow_mut() {
            f.add(stage, items, nanos);
        }
    });
}

/// A timed run of one stage, recorded when it drops. Inert when stages
/// were off at its start.
#[must_use = "a span times the scope it lives in"]
pub struct Span {
    stage: Stage,
    items: u64,
    start: Option<Instant>,
}

impl Span {
    /// What this run handled, in the unit of [`Stage::items`].
    #[inline]
    pub fn items(mut self, items: u64) -> Self {
        self.items = items;
        self
    }

    /// End the run now; `items` is only called when the run is recorded.
    #[inline]
    pub fn end(mut self, items: impl FnOnce() -> u64) {
        if self.start.is_some() {
            self.items = items();
        }
    }
}

impl Drop for Span {
    #[inline]
    fn drop(&mut self) {
        if let Some(start) = self.start {
            let nanos = u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX);
            record(self.stage, self.items, nanos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Stages on for one test, off again when it ends (even on a failed
    /// assert), so a reused test thread starts clean.
    struct On;

    impl On {
        fn new() -> Self {
            enable(false);
            enable(true);
            On
        }
    }

    impl Drop for On {
        fn drop(&mut self) {
            enable(false);
        }
    }

    #[test]
    fn off_by_default_a_probe_records_nothing() {
        enable(false);
        assert!(!is_enabled());
        count(Stage::Bind, 1);
        drop(span(Stage::Layout).items(3));
        span(Stage::Encode).end(|| panic!("items are not computed while off"));
        gpu_wait(|| panic!("the gpu wait does not run while off"));
        assert!(take().is_empty());
    }

    #[test]
    fn on_records_counts_times_and_items() {
        let _on = On::new();
        count(Stage::Bind, 1);
        count(Stage::Bind, 1);
        {
            let _layout = span(Stage::Layout).items(5);
            std::thread::sleep(Duration::from_millis(1));
        }
        let frame = take();
        assert_eq!(
            frame.get(Stage::Bind),
            StageCost {
                calls: 2,
                items: 2,
                nanos: 0
            }
        );
        let layout = frame.get(Stage::Layout);
        assert_eq!((layout.calls, layout.items), (1, 5));
        assert!(layout.nanos >= 1_000_000, "{layout:?}");
        assert!(layout.micros() >= 1_000.0);
        assert_eq!(frame.get(Stage::Encode), StageCost::default());
    }

    #[test]
    fn take_hands_the_frame_over_and_starts_the_next_empty() {
        let _on = On::new();
        count(Stage::Upload, 64);
        assert_eq!(take().get(Stage::Upload).items, 64);
        assert!(take().is_empty(), "the frame after a take starts empty");
        count(Stage::Upload, 8);
        let next = take().get(Stage::Upload);
        assert_eq!((next.calls, next.items), (1, 8));
    }

    #[test]
    fn turning_off_drops_the_frame_and_stops_recording() {
        let _on = On::new();
        count(Stage::HitTest, 1);
        enable(false);
        assert!(!is_enabled());
        count(Stage::HitTest, 1);
        assert!(take().is_empty());
    }

    #[test]
    fn a_span_started_while_off_stays_inert_when_stages_turn_on() {
        enable(false);
        let early = span(Stage::Resolve);
        let _on = On::new();
        drop(early);
        assert!(take().is_empty());
    }

    #[test]
    fn nested_spans_record_both_and_the_outer_includes_the_inner() {
        let _on = On::new();
        {
            let _draw = span(Stage::Draw);
            {
                let _resolve = span(Stage::Resolve);
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        let frame = take();
        let (draw, resolve) = (frame.get(Stage::Draw), frame.get(Stage::Resolve));
        assert_eq!((draw.calls, resolve.calls), (1, 1));
        assert!(draw.nanos >= resolve.nanos, "{draw:?} {resolve:?}");
    }

    #[test]
    fn end_computes_the_items_when_it_records() {
        let _on = On::new();
        span(Stage::Encode).end(|| 7);
        let encode = take().get(Stage::Encode);
        assert_eq!((encode.calls, encode.items), (1, 7));
    }

    #[test]
    fn the_gpu_wait_runs_only_with_the_sync_on() {
        let _on = On::new();
        gpu_wait(|| panic!("no sync asked"));
        set_gpu_sync(true);
        let mut waited = false;
        gpu_wait(|| waited = true);
        assert!(waited);
        assert_eq!(take().get(Stage::GpuWait).calls, 1);
        enable(false);
        enable(true);
        gpu_wait(|| panic!("turning stages off drops the sync"));
    }

    #[test]
    fn records_stay_on_the_thread_that_made_them() {
        let _on = On::new();
        count(Stage::Bind, 1);
        let other = std::thread::spawn(|| {
            // stages are on in the process, not on this thread
            count(Stage::Bind, 1);
            take()
        })
        .join();
        match other {
            Ok(frame) => assert!(frame.is_empty(), "{frame:?}"),
            Err(_) => panic!("the other thread panicked"),
        }
        assert_eq!(take().get(Stage::Bind).calls, 1);
    }

    #[test]
    fn every_stage_has_its_slot_and_a_distinct_name() {
        let mut names = std::collections::HashSet::new();
        for (i, stage) in Stage::ALL.iter().enumerate() {
            assert_eq!(*stage as usize, i, "{stage:?}");
            assert!(names.insert(stage.name()), "{} twice", stage.name());
        }
    }
}
