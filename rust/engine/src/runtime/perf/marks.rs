//! What code says about a frame besides how long it took: a value under a
//! name ([`plot!`](crate::plot)) and a line of text at a moment
//! ([`message!`](crate::message)).
//!
//! Both are off by default and turn on with the recorder that reads them
//! ([`enable`]). Off, a call is one relaxed atomic load and a branch: the
//! value is not kept and the message's text is never built. On, a plot
//! keeps the last value given under its name since the last [`take`], and
//! a message keeps its text, its thread and its moment on the zones'
//! clock, so it lines up with them. [`take`], once per frame, hands both
//! over; what a frame says over [`MAX_PLOTS_PER_FRAME`] names or
//! [`MAX_MESSAGES_PER_FRAME`] lines is counted in [`MarkFrame::dropped`].

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use super::zones;

/// Plot names one frame keeps.
pub const MAX_PLOTS_PER_FRAME: usize = 64;

/// Messages one frame keeps.
pub const MAX_MESSAGES_PER_FRAME: usize = 256;

/// Marks are on in the process. The only state the off path reads.
static ON: AtomicBool = AtomicBool::new(false);

static MARKS: Mutex<MarkFrame> = Mutex::new(MarkFrame {
    plots: Vec::new(),
    messages: Vec::new(),
    dropped: 0,
});

/// A value under a name, as the frame left it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotPoint {
    pub name: &'static str,
    pub value: f64,
}

/// A line of text at a moment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// Nanoseconds from the zones' epoch to the moment it was said.
    pub at_ns: u64,
    /// The thread that said it.
    pub thread: String,
    pub text: String,
}

/// What was said between two takes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarkFrame {
    /// One point per name, in the order the names first came.
    pub plots: Vec<PlotPoint>,
    /// In the order they were said.
    pub messages: Vec<Message>,
    /// Points and messages over the frame's limits, not kept.
    pub dropped: u64,
}

impl MarkFrame {
    pub fn is_empty(&self) -> bool {
        self.plots.is_empty() && self.messages.is_empty() && self.dropped == 0
    }
}

/// Marks are on in the process.
#[inline]
pub fn is_enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Turn marks on or off for every thread. Turning them off drops what was
/// said since the last take.
pub fn enable(on: bool) {
    if ON.swap(on, Ordering::Relaxed) && !on {
        take();
    }
}

/// `value` under `name` for this frame; a later value under the same name
/// replaces it. Inert when marks are off.
#[inline]
pub fn plot(name: &'static str, value: f64) {
    if is_enabled() {
        keep_plot(name, value);
    }
}

fn keep_plot(name: &'static str, value: f64) {
    let mut marks = zones::lock(&MARKS);
    if let Some(point) = marks.plots.iter_mut().find(|p| p.name == name) {
        point.value = value;
    } else if marks.plots.len() < MAX_PLOTS_PER_FRAME {
        marks.plots.push(PlotPoint { name, value });
    } else {
        marks.dropped += 1;
    }
}

/// The line `text` builds, said now. Inert when marks are off: `text` is
/// not called.
#[inline]
pub fn message(text: impl FnOnce() -> String) {
    if is_enabled() {
        keep_message(text());
    }
}

fn keep_message(text: String) {
    let at_ns = zones::now_ns();
    let thread = std::thread::current()
        .name()
        .map_or_else(|| "thread".to_owned(), str::to_owned);
    let mut marks = zones::lock(&MARKS);
    if marks.messages.len() < MAX_MESSAGES_PER_FRAME {
        marks.messages.push(Message {
            at_ns,
            thread,
            text,
        });
    } else {
        marks.dropped += 1;
    }
}

/// What was said since the last take; the next take starts empty.
pub fn take() -> MarkFrame {
    std::mem::take(&mut *zones::lock(&MARKS))
}

/// Keep `$value` (a number) under `$name` (a `&'static str`) for this
/// frame. See `engine::runtime::perf::marks`.
#[macro_export]
macro_rules! plot {
    ($name:expr, $value:expr) => {{
        #[allow(clippy::unnecessary_cast)]
        let value = ($value) as f64;
        $crate::runtime::perf::marks::plot($name, value)
    }};
}

/// Say a line about this moment, written as `format!` takes it. The line
/// is built only while marks are on. See `engine::runtime::perf::marks`.
#[macro_export]
macro_rules! message {
    ($($arg:tt)*) => {
        $crate::runtime::perf::marks::message(|| ::std::format!($($arg)*))
    };
}
