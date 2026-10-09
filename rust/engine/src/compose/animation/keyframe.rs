use super::easing::{Easing, ease};
use super::tween::{Interpolate, TweenState};

#[derive(Clone, Debug)]
pub struct Keyframe<T> {
    pub value: T,
    pub time: f32,
    pub easing: Easing,
}

#[derive(Clone, Debug)]
pub struct KeyframeSequence<T: Interpolate> {
    keyframes: Vec<Keyframe<T>>,
    duration: f32,
    elapsed: f32,
    state: TweenState,
}

impl<T: Interpolate> KeyframeSequence<T> {
    pub fn new(duration: f32) -> Self {
        Self {
            keyframes: Vec::new(),
            duration: duration.max(0.001),
            elapsed: 0.0,
            state: TweenState::Idle,
        }
    }

    pub fn keyframe(mut self, value: T, time: f32, easing: Easing) -> Self {
        self.keyframes.push(Keyframe {
            value,
            time: time.clamp(0.0, 1.0),
            easing,
        });
        self
    }

    pub fn start(mut self) -> Self {
        self.keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self.state = TweenState::Running;
        self.elapsed = 0.0;
        self
    }

    pub fn advance_by(&mut self, dt: f32) {
        if self.state != TweenState::Running {
            return;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.state = TweenState::Completed;
        }
    }

    pub fn advance_and_wrap(&mut self, dt: f32) {
        self.elapsed += dt;
        if self.duration > 0.0 {
            self.elapsed %= self.duration;
        }
    }

    pub fn advance_and_reverse(&mut self, dt: f32) {
        self.elapsed += dt;
        if self.duration > 0.0 {
            let full = self.duration * 2.0;
            self.elapsed %= full;
            if self.elapsed > self.duration {
                self.elapsed = full - self.elapsed;
            }
        }
    }

    pub fn now(&self) -> T {
        assert!(
            !self.keyframes.is_empty(),
            "KeyframeSequence has no keyframes"
        );
        if self.keyframes.len() == 1 {
            return self.keyframes[0].value.clone();
        }

        let t = if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let mut prev_idx = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= t {
                prev_idx = i;
            } else {
                break;
            }
        }

        let next_idx = (prev_idx + 1).min(self.keyframes.len() - 1);
        if prev_idx == next_idx {
            return self.keyframes[prev_idx].value.clone();
        }

        let prev = &self.keyframes[prev_idx];
        let next = &self.keyframes[next_idx];
        let seg_range = next.time - prev.time;
        if seg_range <= 0.0 {
            return next.value.clone();
        }
        let seg_t = ((t - prev.time) / seg_range).clamp(0.0, 1.0);
        let eased_t = ease(seg_t, prev.easing);
        prev.value.lerp(&next.value, eased_t)
    }

    pub fn is_animating(&self) -> bool {
        self.state == TweenState::Running
    }

    pub fn state(&self) -> TweenState {
        self.state
    }
}
