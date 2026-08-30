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
    pub dt: f32,
    pub elapsed: f32,
}

pub struct FrameClock {
    start: Instant,
    last: Instant,
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
        }
    }

    pub fn tick(&mut self) -> AnimationTick {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        let elapsed = now.duration_since(self.start).as_secs_f32();
        self.last = now;
        AnimationTick {
            dt: dt.min(0.1),
            elapsed,
        }
    }
}

#[cfg(test)]
mod tests_easing;
#[cfg(test)]
mod tests_keyframe;
#[cfg(test)]
mod tests_spring;
#[cfg(test)]
mod tests_tween;
