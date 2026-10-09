//! What a tick asks of the loop.
//!
//! Under render on demand a frame is drawn when something visible
//! changed; motion that changes on its own (a spring, a caret) has to say
//! so from its tick. A `bool` ("keep the frames coming") can only ask for
//! every frame, so a caret that flips twice a second cost sixty frames a
//! second (task-73, scenario g). A [`Wake`] says two things instead:
//! whether this tick changed something visible (`now`), and how long
//! until the next tick is needed (`next`). A shell draws on `now`, sleeps
//! until `next` and sleeps for good when nothing is pending; input wakes
//! it as before.
//!
//! [`Blink`] is the caret clock the text widgets share: the phase inside
//! the current half-period, answering `advance` with the [`Wake`] of the
//! flip.

use std::time::Duration;

/// What a tick asks of the loop: a frame now, and when the next tick is
/// due. Composes with [`Wake::merge`]: any `now` wins, the earliest
/// `next` wins.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Wake {
    /// This tick changed something visible: a frame is needed now.
    pub now: bool,
    /// Time until the next tick is needed: the caret's next flip, the
    /// next step of a running animation. `Some(Duration::ZERO)` is
    /// continuous motion (the very next frame); `None` is nothing
    /// pending.
    pub next: Option<Duration>,
}

impl Wake {
    /// Nothing changed, nothing pending.
    pub const fn idle() -> Self {
        Self {
            now: false,
            next: None,
        }
    }

    /// Something visible changed; nothing more is scheduled.
    pub const fn now() -> Self {
        Self {
            now: true,
            next: None,
        }
    }

    /// Nothing changed yet; the next tick is due in `d`.
    pub const fn after(d: Duration) -> Self {
        Self {
            now: false,
            next: Some(d),
        }
    }

    /// Motion every frame (a spring, a spinner): changed now, and again
    /// on the very next frame.
    pub const fn continuous() -> Self {
        Self {
            now: true,
            next: Some(Duration::ZERO),
        }
    }

    /// The legacy `tick -> bool` lifted: `true` while moving is
    /// continuous motion, `false` is at rest.
    pub const fn moving(moving: bool) -> Self {
        if moving {
            Self::continuous()
        } else {
            Self::idle()
        }
    }

    /// Both asks in one: a frame now if either wants it, the earliest of
    /// the two deadlines.
    pub fn merge(self, other: Wake) -> Wake {
        let next = match (self.next, other.next) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, None) => a,
            (None, b) => b,
        };
        Wake {
            now: self.now || other.now,
            next,
        }
    }

    /// Anything pending: a frame now or a deadline ahead. What a shell
    /// without deadlines (`ControlFlow::Wait` and a `bool`) must keep the
    /// frames coming for.
    pub const fn pending(self) -> bool {
        self.now || self.next.is_some()
    }

    /// The next frame is wanted right away.
    pub fn is_continuous(self) -> bool {
        self.next == Some(Duration::ZERO)
    }
}

/// The caret clock: shown for one half-period, hidden for the next, from
/// the last reset. The widget owns the half-period (a platform timing
/// constant or its config) and passes it to [`Blink::advance`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blink {
    visible: bool,
    /// Seconds into the current half-period.
    phase: f32,
}

impl Default for Blink {
    fn default() -> Self {
        Self::new()
    }
}

impl Blink {
    /// A solid caret at the start of its half-period.
    pub const fn new() -> Self {
        Self {
            visible: true,
            phase: 0.0,
        }
    }

    /// On the visible half of the blink.
    pub const fn visible(&self) -> bool {
        self.visible
    }

    /// Solid caret, the half-period restarted (any edit or caret move
    /// calls this so the caret never blinks away mid-typing).
    pub fn reset(&mut self) {
        self.visible = true;
        self.phase = 0.0;
    }

    /// Advance by `dt` seconds with a half-period of `interval`. `now`
    /// when the visibility flipped; `next` is the time to the next flip,
    /// so between flips the ask is a deadline, not a frame. A `dt` long
    /// enough for an even number of flips lands on the same side and
    /// changes nothing visible. An `interval` that is not positive and
    /// finite means no blink: the caret stays solid and nothing is
    /// pending.
    pub fn advance(&mut self, dt: f32, interval: f32) -> Wake {
        if !(interval > 0.0 && interval.is_finite()) {
            self.visible = true;
            return Wake::idle();
        }
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        // in f64 so a long stall keeps the remainder exact
        let phase = f64::from(self.phase) + f64::from(dt);
        let interval = f64::from(interval);
        let rem = phase.rem_euclid(interval);
        // whole half-periods, from the same remainder so the two agree at
        // an exact multiple; the cast saturates for an absurd dt, and
        // parity is what matters
        let flips = ((phase - rem) / interval).round() as u64;
        let flipped = flips % 2 == 1;
        if flipped {
            self.visible = !self.visible;
        }
        self.phase = rem as f32;
        // rem < interval, so the remainder is a positive time; a flip
        // that rounds to zero is due on the very next frame
        let left = (interval - rem) as f32;
        Wake {
            now: flipped,
            next: Some(Duration::try_from_secs_f32(left).unwrap_or(Duration::ZERO)),
        }
    }
}
