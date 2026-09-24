//! The loop's pacing: time catches up before what woke the loop lands,
//! the frame's tick says what it asks of the loop, and the answer sets
//! the wait (see the shell's module doc).

use comps::prelude::Wake;
use web_time::Instant;
use winit::event_loop::{ActiveEventLoop, ControlFlow};

use super::App;

impl App {
    /// Advances what animates to now, before anything from outside the
    /// frame lands (input, a worker result, a reader's or an agent's
    /// request). The tick gets the real time since the last one, so a
    /// change applied between two frames would otherwise take the whole
    /// gap on the next: a switch flipped after three idle seconds would
    /// jump to its end, a toast pushed then would start three seconds
    /// old, a caret focused then would start in any phase. This tick
    /// draws nothing; if it changed something visible, it asks for the
    /// frame.
    pub(super) fn catch_up(&mut self) {
        let since_last = self.clock.tick().since_last;
        self.advance(since_last);
    }

    /// [`App::catch_up`] with the elapsed time given.
    pub(super) fn advance(&mut self, since_last: f32) {
        if self.view.wake(since_last).now {
            self.caught_up = true;
            self.invalidate();
        }
    }

    /// The frame's tick: what animates, advanced by the time since the
    /// last tick (a catch-up or a frame), and what that asks of the loop.
    /// `now` also carries a visible change a catch-up made since the last
    /// frame.
    pub(super) fn frame_wake(&mut self, since_last: f32) -> Wake {
        let mut wake = self.view.wake(since_last);
        wake.now |= std::mem::take(&mut self.caught_up);
        wake
    }

    /// What the frame's [`Wake`] asks of the loop, after the frame that
    /// carries its `now` was drawn: continuous motion draws the next
    /// frame; a deadline sets the wait to that instant; nothing pending
    /// sleeps until input. The wait ends early on any event, so the git
    /// worker, the screen reader and the agent port still get their frame
    /// through `invalidate` whenever they ask.
    pub(super) fn pace(&mut self, event_loop: &ActiveEventLoop, wake: Wake) {
        match wake.next {
            Some(next) if next.is_zero() => {
                // Springs still moving: keep the frames coming.
                self.compositor.invalidate();
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
                event_loop.set_control_flow(ControlFlow::Wait);
            }
            Some(next) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + next))
            }
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }
}

#[cfg(test)]
mod tests;
