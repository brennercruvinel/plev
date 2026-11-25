//! The process's heap, counted: how many blocks code asks for and gives
//! back, and how many bytes, frame by frame.
//!
//! [`Counting`] wraps an allocator (the system's by default) and an app
//! installs it as its own:
//!
//! ```ignore
//! #[global_allocator]
//! static HEAP: Counting = Counting::system();
//! ```
//!
//! Off (the default) it adds one relaxed atomic load to a call. [`enable`]
//! turns the counters on for the process: two relaxed adds per call. A
//! block resized counts as one given back and one asked for. [`totals`]
//! reads the counters, and a `Recorder` keeps what moved between two
//! frames as a [`HeapFrame`].
//!
//! The counters start when they are turned on, not with the process: a
//! block older than that is counted when it is given back and was never
//! counted when it was asked for, so [`HeapFrame::live_bytes`] is the heap's
//! change since then, and can be below zero. An app that does not install
//! [`Counting`] reads zeros, and [`HeapFrame::counted`] says so.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static ON: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicU64 = AtomicU64::new(0);
static FREES: AtomicU64 = AtomicU64::new(0);
static BYTES_IN: AtomicU64 = AtomicU64::new(0);
static BYTES_OUT: AtomicU64 = AtomicU64::new(0);

/// An allocator that counts what goes through it while [`enable`]d.
pub struct Counting<A = System>(A);

impl Counting<System> {
    /// The system's allocator, counted.
    pub const fn system() -> Self {
        Self(System)
    }
}

impl<A> Counting<A> {
    /// `inner`, counted.
    pub const fn new(inner: A) -> Self {
        Self(inner)
    }
}

#[inline]
fn asked(bytes: usize) {
    if ON.load(Ordering::Relaxed) {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES_IN.fetch_add(bytes as u64, Ordering::Relaxed);
    }
}

#[inline]
fn given_back(bytes: usize) {
    if ON.load(Ordering::Relaxed) {
        FREES.fetch_add(1, Ordering::Relaxed);
        BYTES_OUT.fetch_add(bytes as u64, Ordering::Relaxed);
    }
}

// SAFETY: every call is handed to the wrapped allocator unchanged, with
// the caller's own pointer and layout; the counters are atomics, touched
// without allocating, so nothing here calls back into the allocator.
unsafe impl<A: GlobalAlloc> GlobalAlloc for Counting<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's contract for `alloc`, passed on as it came.
        let block = unsafe { self.0.alloc(layout) };
        if !block.is_null() {
            asked(layout.size());
        }
        block
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's contract for `alloc_zeroed`, passed on.
        let block = unsafe { self.0.alloc_zeroed(layout) };
        if !block.is_null() {
            asked(layout.size());
        }
        block
    }

    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        given_back(layout.size());
        // SAFETY: `block` came from this allocator with `layout`, which is
        // the wrapped one's own block and layout.
        unsafe { self.0.dealloc(block, layout) }
    }

    unsafe fn realloc(&self, block: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller's contract for `realloc`, passed on as it came.
        let moved = unsafe { self.0.realloc(block, layout, new_size) };
        if !moved.is_null() {
            given_back(layout.size());
            asked(new_size);
        }
        moved
    }
}

/// Turn the counters on or off for the process. They keep what they
/// counted.
pub fn enable(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

/// The counters are on.
pub fn is_enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Everything counted so far.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeapTotals {
    /// Blocks asked for.
    pub allocs: u64,
    /// Blocks given back.
    pub frees: u64,
    /// Bytes asked for.
    pub bytes_in: u64,
    /// Bytes given back.
    pub bytes_out: u64,
}

/// The counters now. Each is read on its own: a call running on another
/// thread can sit between two of them.
pub fn totals() -> HeapTotals {
    HeapTotals {
        allocs: ALLOCS.load(Ordering::Relaxed),
        frees: FREES.load(Ordering::Relaxed),
        bytes_in: BYTES_IN.load(Ordering::Relaxed),
        bytes_out: BYTES_OUT.load(Ordering::Relaxed),
    }
}

impl HeapTotals {
    /// Bytes asked for and not given back since the counters turned on.
    pub fn live_bytes(&self) -> i64 {
        self.bytes_in.wrapping_sub(self.bytes_out) as i64
    }

    /// What moved from `earlier` to here.
    pub fn since(&self, earlier: &HeapTotals) -> HeapFrame {
        HeapFrame {
            counted: self.allocs > 0 || self.frees > 0,
            allocs: self.allocs.saturating_sub(earlier.allocs),
            frees: self.frees.saturating_sub(earlier.frees),
            bytes_in: self.bytes_in.saturating_sub(earlier.bytes_in),
            bytes_out: self.bytes_out.saturating_sub(earlier.bytes_out),
            live_bytes: self.live_bytes(),
        }
    }
}

/// The heap over one frame: from the end of the frame before to its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeapFrame {
    /// The app counts its heap: [`Counting`] is its allocator and the
    /// counters are on. False leaves every other field at zero.
    pub counted: bool,
    /// Blocks asked for.
    pub allocs: u64,
    /// Blocks given back.
    pub frees: u64,
    /// Bytes asked for.
    pub bytes_in: u64,
    /// Bytes given back.
    pub bytes_out: u64,
    /// Bytes asked for and not given back since the counters turned on,
    /// as of the frame's end.
    pub live_bytes: i64,
}

impl HeapFrame {
    /// Bytes the frame asked for less the bytes it gave back.
    pub fn net_bytes(&self) -> i64 {
        self.bytes_in.wrapping_sub(self.bytes_out) as i64
    }
}
