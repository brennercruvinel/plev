//! The loop's pacing: time catches up before what woke the loop lands,
//! the frame's tick says what it asks of the loop, the frame is drawn
//! (given to the screen reader, and recorded when the session is), and
//! the answer sets the wait (see the shell's module doc).

use comps::prelude::Wake;
use web_time::Instant;
use winit::event_loop::{ActiveEventLoop, ControlFlow};

use super::{App, GpuState};
use crate::renderer;

impl App {
    /// The frame, once the GPU is ready: the tick, the scene drawn from
    /// it, the tree of it for a screen reader, the recording's copy, and
    /// the pace it asks for.
    pub(super) fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, GpuState::Ready { .. }) {
            return;
        }
        // The real time since the last tick: a deadline the loop
        // slept through arrives whole (`dt` is clamped for motion).
        let since_last = self.clock.tick().since_last;
        let wake = self.frame_wake(since_last);
        log::debug!(
            "frame: {:.3} s since the last tick, wake now {} next {:?}",
            since_last,
            wake.now,
            wake.next
        );
        let GpuState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool,
        } = &mut self.state
        else {
            return;
        };
        renderer::render_frame(
            gpu,
            text_system,
            effects,
            texture_pool,
            &mut self.compositor,
            &mut self.view,
        );
        // The tree of this frame, placed by the bindings it just drew;
        // the page is projected only while a reader listens.
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let (Some(bridge), Some(window)) = (self.access.bridge.as_mut(), self.window.as_ref()) {
            let view = &self.view;
            bridge.update(window, &self.compositor, || view.semantics());
        }
        // The frame just drawn, where each bound control shows in it, and
        // the state it was drawn from.
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        self.record_frame();
        // `wake.now` is carried by this frame: the scene was built after
        // the tick, so a flipped caret is already in it.
        self.pace(event_loop, wake);
    }

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
