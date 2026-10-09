//! Awake, covered, asleep: the states a window and an app are in besides
//! "running", and what each does to the frames and to the work beside
//! them (task-76 item 7).
//!
//! A window is **covered** when the platform says it is not visible at
//! all (`WindowEvent::Occluded(true)`: minimized, hidden or fully behind
//! others on macOS, in the background on iOS; nowhere else today). An app
//! is **asleep** when its shell says so ([`Windows::sleep`]), or when the
//! platform took its windows ([`Windows::suspend`], android); the desktop
//! platforms give winit no word of the system's sleep, so there an app
//! sleeps only by its shell's hand. A window is **awake** when neither.
//!
//! The rule for frames does not change render on demand, it adds a
//! reason not to draw: a window that is not awake asks the platform for
//! nothing (its `request_redraw` is withheld, its `needs_redraw` is
//! false, its `deadline` is `None`, so a shell's `request_pending_redraws`
//! and `earliest_deadline` pass it by), and a continuous animation on it
//! becomes "waiting to show". The state moves on meanwhile: the shell's
//! catch-up ticks with the real time, so a spring that would have ended
//! has ended when the window shows. As the window shows again, or the
//! app wakes, every such window is invalidated and asks for one frame:
//! the first frame after a cover or a sleep is built from the state as it
//! is then, never the one drawn before. The device may not survive a
//! sleep either; that is the recovery before the acquire, per context,
//! and it needs nothing from here.
//!
//! [`Background`] is the same fact seen by the work that runs beside the
//! frames (a child process answering off the loop, a decode, a worker):
//! while the app sleeps a result that arrives stays where the worker put
//! it (its channel) and is applied on the first wake after the app wakes,
//! since the shell neither settles nor draws meanwhile; and a job asked
//! while asleep does not start, or starts as the app wakes, as each app
//! decides. One app runs per process and its pages run on the loop's
//! thread, so the flag is thread-local: a test's app is its thread's.

use std::cell::Cell;

use winit::window::WindowId;

use super::host::WindowHost;
use super::windows::Windows;

thread_local! {
    static ASLEEP: Cell<bool> = const { Cell::new(false) };
}

/// The app's sleep as the work beside the frames sees it: set by the
/// shell's `Windows` as the app sleeps and wakes, read by whoever starts
/// a job or applies a result.
pub struct Background;

impl Background {
    /// The app fell asleep (`true`) or woke (`false`). `Windows::sleep`
    /// and `wake` say it; a shell without a `Windows` says it itself.
    pub fn sleeping(asleep: bool) {
        ASLEEP.set(asleep);
    }

    /// Whether the app is asleep now: a job asked now waits for the wake
    /// (or is refused), and a result that arrives now is applied then.
    pub fn asleep() -> bool {
        ASLEEP.get()
    }
}

impl WindowHost {
    /// Neither covered nor asleep: the window shows and may draw.
    pub fn awake(&self) -> bool {
        !self.occluded && !self.sleeping
    }

    /// The platform says the window is not visible at all.
    pub fn occluded(&self) -> bool {
        self.occluded
    }

    /// The app of this window sleeps.
    pub fn sleeping(&self) -> bool {
        self.sleeping
    }

    /// The platform's word on the window's visibility
    /// (`WindowEvent::Occluded`). Covered, the pending request is
    /// withdrawn: nothing is asked until the window shows. Shown again,
    /// the scene is invalidated and one frame asked for, so the first
    /// frame is built from the state as it is now.
    pub fn set_occluded(&mut self, covered: bool) {
        if self.occluded == covered {
            return;
        }
        self.occluded = covered;
        self.visibility_changed();
    }

    /// The app sleeps or wakes, for this host (`Windows::sleep`, `wake`).
    pub(crate) fn set_sleeping(&mut self, asleep: bool) {
        if self.sleeping == asleep {
            return;
        }
        self.sleeping = asleep;
        self.visibility_changed();
    }

    fn visibility_changed(&mut self) {
        if self.awake() {
            log::debug!("window shows again: a new frame");
            self.compositor.invalidate();
            self.request_redraw();
        } else {
            log::debug!(
                "window paused (covered {}, asleep {})",
                self.occluded,
                self.sleeping
            );
            self.redraw_asked = false;
        }
    }
}

impl Windows {
    /// The app is asleep: no window draws until `wake`.
    pub fn asleep(&self) -> bool {
        self.asleep
    }

    /// Window `id` shows and may draw: it is open, not covered, and the
    /// app is not asleep. `false` for an unknown id.
    pub fn awake(&self, id: WindowId) -> bool {
        self.get(id).is_some_and(WindowHost::awake)
    }

    /// The app falls asleep (the system's sleep, where a shell hears of
    /// it; the background, where the platform says so): every window
    /// stops asking for frames and loses its deadline, and the work
    /// beside the frames hears it through [`Background`]. Nothing is
    /// drawn until `wake`.
    pub fn sleep(&mut self) {
        if self.asleep {
            return;
        }
        self.asleep = true;
        Background::sleeping(true);
        for (_, host) in self.iter_mut() {
            host.set_sleeping(true);
        }
        log::info!("the app sleeps: no frames until it wakes");
    }

    /// The app wakes: every window is invalidated and asks for one frame,
    /// built from the state as it is now (never the frame from before the
    /// sleep), and the work beside the frames may start and settle again.
    pub fn wake(&mut self) {
        if !self.asleep {
            return;
        }
        self.asleep = false;
        Background::sleeping(false);
        for (_, host) in self.iter_mut() {
            host.set_sleeping(false);
        }
        log::info!("the app wakes: every window draws anew");
    }
}
