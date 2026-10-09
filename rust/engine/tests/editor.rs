//! Large-document benchmark of the editor (task-74, phase 7). A generated
//! document of 10 thousand, 100 thousand and one million lines is opened,
//! scrolled and edited in the middle, and the numbers are printed as the
//! rows of the table in the task file. Ignored by default: the numbers
//! only mean something in release.
//!
//! ```text
//! cargo test --offline --release -p comps --test editor_large_bench -- --ignored --nocapture
//! ```
//!
//! A frame is the editor's scene pushed into a `Compositor` and resolved:
//! `scene` stops at `Compositor::resolve_scene` (no GPU, what the editor
//! controls); `frame` goes through `engine::headless::Renderer::draw`,
//! which shapes the text nodes, rasterizes new glyphs and submits the
//! frame, as a window does. `frame` is skipped when no GPU adapter answers.
//! Memory is the heap the document and the editor keep after the open
//! (live bytes, counted by the allocator below), and the peak the open
//! reached above what was live before it.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use engine::widgets::editor::{EditorTheme, EditorView};
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::{Renderer, Viewport};
use engine::compose::layout::ComputedBounds;
use editor::{Document, Selection, SelectionSet};
use winit::keyboard::{Key, ModifiersState, NamedKey};

// -- the allocator that counts ------------------------------------------------

struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn grew(bytes: usize) {
    let now = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK.fetch_max(now, Ordering::Relaxed);
}

// SAFETY: every call forwards to `System` with the caller's own pointer and
// layout, so the allocator contract is `System`'s; the counters only read
// the sizes.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: same layout the caller passed, which it guarantees valid.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            grew(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from `alloc`/`realloc` above with this layout.
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: `ptr` and `layout` are the caller's, from this allocator.
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
            grew(new_size);
        }
        new
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

// -- the document and the frame -----------------------------------------------

const VIEW_W: f32 = 1200.0;
const VIEW_H: f32 = 800.0;

/// Code-like lines of 44 to 65 bytes, none equal to another (the line
/// number is in the text), so no two lines share a shaping.
fn document_text(lines: usize) -> String {
    let mut text = String::with_capacity(lines * 72);
    for i in 0..lines {
        let body = match i % 4 {
            0 => format!("fn line_{i}(x: u32) -> u32 {{ x.wrapping_mul({i}) ^ 0x5f }}"),
            1 => format!("    let value_{i} = table[{i} % len].checked_add(offset)?;"),
            2 => format!("    // the quick brown fox jumps over line {i}, again"),
            _ => format!("    assert_eq!(value_{i}, expected, \"row {i}\");"),
        };
        text.push_str(&body);
        text.push('\n');
    }
    text
}

fn bounds() -> ComputedBounds {
    ComputedBounds {
        x: 0.0,
        y: 0.0,
        width: VIEW_W,
        height: VIEW_H,
    }
}

/// Where a frame ends: the resolved scene, or the headless GPU frame.
struct Frames {
    compositor: Compositor,
    renderer: Option<Renderer>,
    theme: EditorTheme,
}

impl Frames {
    fn new(gpu: bool) -> Option<Self> {
        let renderer = if gpu {
            Some(Renderer::new(Viewport::new(VIEW_W, VIEW_H, 2.0), RenderConfig::default()).ok()?)
        } else {
            None
        };
        Some(Self {
            compositor: Compositor::new(),
            renderer,
            theme: EditorTheme::default(),
        })
    }

    /// One frame of `ed`, timed from the scene's start to the frame built.
    fn frame(&mut self, ed: &mut EditorView) -> Duration {
        let start = Instant::now();
        self.compositor.begin_frame();
        ed.render(&mut self.compositor, bounds(), &self.theme);
        match self.renderer {
            Some(ref mut r) => {
                r.draw(&mut self.compositor, [0.0, 0.0, 0.0, 1.0]);
            }
            None => self.compositor.resolve_scene((VIEW_W, VIEW_H)),
        }
        start.elapsed()
    }
}

// -- the numbers ---------------------------------------------------------------

struct Stats {
    mean_ms: f64,
    p99_ms: f64,
}

fn stats(mut samples: Vec<Duration>) -> Stats {
    samples.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1000.0;
    let mean = samples.iter().map(|d| ms(*d)).sum::<f64>() / samples.len() as f64;
    let at = ((samples.len() as f64 * 0.99).ceil() as usize).clamp(1, samples.len()) - 1;
    Stats {
        mean_ms: mean,
        p99_ms: ms(samples[at]),
    }
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn type_key(ed: &mut EditorView, key: Key) {
    ed.handle_key(&key, ModifiersState::empty());
}

/// Open, scroll and edit a document of `lines` lines; one table row.
fn run(lines: usize, gpu: bool) -> Option<String> {
    let text = document_text(lines);
    let mut frames = Frames::new(gpu)?;
    // Warm the fonts and the glyph atlas on a small document, so the open
    // measures the document, not the first use of the text stack.
    let mut warm = EditorView::new(Document::load(&document_text(100)));
    frames.frame(&mut warm);
    drop(warm);

    let live_before = LIVE.load(Ordering::Relaxed);
    PEAK.store(live_before, Ordering::Relaxed);
    let start = Instant::now();
    let mut ed = EditorView::new(Document::load(&text));
    frames.frame(&mut ed);
    let open = start.elapsed();
    let kept = LIVE.load(Ordering::Relaxed).saturating_sub(live_before);
    let peak = PEAK.load(Ordering::Relaxed).saturating_sub(live_before);
    drop(text);

    // A long wheel scroll: three lines a frame from the top.
    let lh = ed.config.line_height;
    let mut wheel = Vec::with_capacity(2000);
    for _ in 0..2000 {
        ed.scroll.scroll_by(3.0 * lh);
        wheel.push(frames.frame(&mut ed));
    }

    // A scrollbar drag from the top to the end: every frame a new viewport.
    let mut drag = Vec::with_capacity(300);
    for k in 0..300 {
        let max = ed.scroll.max_offset();
        ed.scroll.scroll_to(max * k as f32 / 299.0);
        drag.push(frames.frame(&mut ed));
    }

    // The same wheel with the whole document selected: every line on screen
    // draws selection boxes, laid out from its shaped clusters.
    ed.select_all();
    ed.scroll.scroll_to(0.0);
    let mut selected = Vec::with_capacity(1000);
    for _ in 0..1000 {
        ed.scroll.scroll_by(3.0 * lh);
        selected.push(frames.frame(&mut ed));
    }

    // Insert and delete one character on the middle line, to the frame after.
    let rope = ed.document.rope();
    let at = rope.line_to_byte(lines / 2) + 4;
    ed.document
        .set_selections(SelectionSet::single(Selection::caret(at)));
    ed.scroll_to_cursor();
    frames.frame(&mut ed);
    let mut edit = Vec::with_capacity(400);
    for _ in 0..200 {
        let start = Instant::now();
        type_key(&mut ed, Key::Character("x".into()));
        frames.frame(&mut ed);
        edit.push(start.elapsed());
        let start = Instant::now();
        type_key(&mut ed, Key::Named(NamedKey::Backspace));
        frames.frame(&mut ed);
        edit.push(start.elapsed());
    }
    assert_eq!(ed.document.len_lines(), lines + 1, "the edits cancel out");

    let (wheel, drag, edit) = (stats(wheel), stats(drag), stats(edit));
    let selected = stats(selected);
    Some(format!(
        "| {lines} | {} | {:.1} | {:.3} / {:.3} | {:.3} / {:.3} | {:.3} / {:.3} \
         | {:.3} / {:.3} | {:.1} | {:.1} |",
        if gpu { "frame" } else { "scene" },
        open.as_secs_f64() * 1000.0,
        wheel.mean_ms,
        wheel.p99_ms,
        drag.mean_ms,
        drag.p99_ms,
        selected.mean_ms,
        selected.p99_ms,
        edit.mean_ms,
        edit.p99_ms,
        mib(kept),
        mib(peak),
    ))
}

#[test]
#[ignore = "benchmark: run in release with --ignored --nocapture"]
fn editor_large_document_bench() {
    println!(
        "| lines | until | open (ms) | wheel mean / p99 (ms) | drag mean / p99 (ms) \
         | wheel, all selected (ms) | edit mean / p99 (ms) | kept (MiB) | open peak (MiB) |"
    );
    println!("|---|---|---|---|---|---|---|---|---|");
    for gpu in [false, true] {
        for lines in [10_000, 100_000, 1_000_000] {
            match run(lines, gpu) {
                Some(row) => println!("{row}"),
                None => println!("| {lines} | frame | no GPU adapter |"),
            }
        }
    }
}
