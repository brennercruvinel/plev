mod easing;
mod keyframe;
mod spring;
mod tween;

pub use easing::{Easing, ease};
pub use keyframe::{Keyframe, KeyframeSequence};
pub use spring::{Spring, SpringInterpolate};
pub use tween::{Interpolate, Repeat, Tween, TweenState};

use web_time::Instant;

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

#[cfg(test)]
mod tests_clock;
#[cfg(test)]
mod tests_easing;
#[cfg(test)]
mod tests_keyframe;
#[cfg(test)]
mod tests_spring;
#[cfg(test)]
mod tests_tween;
