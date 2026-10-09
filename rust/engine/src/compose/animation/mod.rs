mod easing;
mod keyframe;
mod spring;
mod tween;
#[cfg(test)]
mod tests;

pub use easing::{Easing, ease};
pub use keyframe::{Keyframe, KeyframeSequence};
pub use spring::{Spring, SpringInterpolate};
pub use tween::{Interpolate, Repeat, Tween, TweenState};

use std::cell::Cell;

use web_time::Instant;

thread_local! {
    /// Whether this thread's motion is reduced (see [`set_reduced_motion`]).
    static REDUCED: Cell<bool> = const { Cell::new(false) };
}

/// The system's "reduce motion" for every transition this thread runs:
/// with it on, a [`Spring`] and a finite [`Tween`] reach their end on
/// their next tick, whatever the time step, so a transition ends on the
/// first frame and nothing asks for more (task-75 phase 1). A tween that
/// repeats forever has no end to reach and keeps going, as the ambient
/// loops that count their own time do (a spinner, a skeleton's pulse).
///
/// The preference is the thread's: the one that ticks the widgets, the
/// event loop's, where `accessibility::Bridge` sets it from the system
/// (`accessibility::motion`). A thread starts with it off, so a test, a
/// headless frame and the regression baselines never see a person's
/// setting.
pub fn set_reduced_motion(on: bool) {
    REDUCED.with(|r| r.set(on));
}

/// Whether this thread's transitions end at once (see
/// [`set_reduced_motion`]).
pub fn reduced_motion() -> bool {
    REDUCED.with(Cell::get)
}

#[derive(Clone, Copy, Debug)]
pub struct AnimationTick {
    /// Seconds to animate by: the time since the last tick, clamped to
    /// 0.1 s so a stall does not jump a motion to its end.
    pub dt: f32,
    /// Seconds since the clock started.
    pub elapsed: f32,
    /// Seconds since the last tick, real and unclamped, for deadlines: a
    /// loop that slept until a caret's next flip hands the widget the
    /// whole sleep (task-82). 0.0 on the first tick.
    pub since_last: f32,
}

pub struct FrameClock {
    start: Instant,
    last: Instant,
    ticked: bool,
}

impl Default for FrameClock {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameClock {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last: now,
            ticked: false,
        }
    }

    pub fn tick(&mut self) -> AnimationTick {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        let elapsed = now.duration_since(self.start).as_secs_f32();
        let since_last = if self.ticked { dt } else { 0.0 };
        self.last = now;
        self.ticked = true;
        AnimationTick {
            dt: dt.min(0.1),
            elapsed,
            since_last,
        }
    }
}

