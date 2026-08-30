use super::easing::{Easing, ease};

pub trait Interpolate: Clone {
    fn lerp(&self, target: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn lerp(&self, target: &Self, t: f32) -> Self {
        self + (target - self) * t
    }
}

impl<const N: usize> Interpolate for [f32; N] {
    fn lerp(&self, target: &Self, t: f32) -> Self {
        std::array::from_fn(|i| self[i] + (target[i] - self[i]) * t)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TweenState {
    Idle,
    Running,
    Completed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repeat {
    None,
    Times(u32),
    Infinite,
}

#[derive(Clone, Debug)]
pub struct Tween<T: Interpolate> {
    from: T,
    to: T,
    duration: f32,
    elapsed: f32,
    easing: Easing,
    state: TweenState,
    delay: f32,
    repeat: Repeat,
    reverse: bool,
}

impl<T: Interpolate> Tween<T> {
    pub fn new(initial: T, duration: f32, easing: Easing) -> Self {
        Self {
            from: initial.clone(),
            to: initial,
            duration: duration.max(0.001),
            elapsed: 0.0,
            easing,
            state: TweenState::Idle,
            delay: 0.0,
            repeat: Repeat::None,
            reverse: false,
        }
    }

    /// Create with duration derived from theme MotionPhysics settling time.
    pub fn from_motion(initial: T, motion: &crate::theme::MotionPhysics, easing: Easing) -> Self {
        let duration = motion.settling_time().clamp(0.1, 2.0);
        Self::new(initial, duration, easing)
    }

    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay.max(0.0);
        self
    }

    pub fn with_repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn with_reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    pub fn set_target(&mut self, target: T) {
        self.from = self.get();
        self.to = target;
        self.elapsed = 0.0;
        self.state = TweenState::Running;
    }

    fn cycle_duration(&self) -> f32 {
        if self.reverse {
            self.duration * 2.0
        } else {
            self.duration
        }
    }

    fn total_play_time(&self) -> Option<f32> {
        let cycle = self.cycle_duration();
        match self.repeat {
            Repeat::None => Some(cycle),
            Repeat::Times(n) => Some(cycle * (n as f32 + 1.0)),
            Repeat::Infinite => None,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.state != TweenState::Running {
            return;
        }
        self.elapsed += dt;

        let active = self.elapsed - self.delay;
        if active < 0.0 {
            return;
        }

        if let Some(total) = self.total_play_time()
            && active >= total
        {
            self.elapsed = self.delay + total;
            self.state = TweenState::Completed;
        }
    }

    pub fn get(&self) -> T {
        let active = (self.elapsed - self.delay).max(0.0);

        if self.state == TweenState::Idle || active <= 0.0 {
            return self.from.clone();
        }

        if self.state == TweenState::Completed {
            return if self.reverse {
                self.from.clone()
            } else {
                self.to.clone()
            };
        }

        let cycle_dur = self.cycle_duration();
        let cycle_pos = if cycle_dur > 0.0 {
            active % cycle_dur
        } else {
            0.0
        };

        let t = if self.reverse {
            let half = self.duration;
            if cycle_pos < half {
                cycle_pos / half
            } else {
                (cycle_dur - cycle_pos) / half
            }
        } else {
            if self.duration > 0.0 {
                (active % self.duration) / self.duration
            } else {
                1.0
            }
        };

        let eased = ease(t.clamp(0.0, 1.0), self.easing);
        self.from.lerp(&self.to, eased)
    }

    pub fn is_animating(&self) -> bool {
        self.state == TweenState::Running
    }

    pub fn state(&self) -> TweenState {
        self.state
    }

    pub fn reset(&mut self, value: T) {
        self.from = value.clone();
        self.to = value;
        self.elapsed = 0.0;
        self.state = TweenState::Idle;
    }
}
