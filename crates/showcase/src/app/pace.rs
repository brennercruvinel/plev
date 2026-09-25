//! The loop's pacing: time catches up before what woke the loop lands,
//! the frame's tick says what it asks of the loop, the frame is drawn
//! for the window that asked (given to the screen reader, and recorded
//! when the session is), and the answer sets each window's deadline and
//! the loop's wait (see the shell's module doc).

use comps::prelude::Wake;
use web_time::Instant;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::App;

impl App {
    /// The frame of window `id`, once its GPU is ready: the tick, the
    /// scene drawn from it (the whole view in the main window, the page
    /// alone in its own), the tree of it for a screen reader and the
    /// recording's copy (main window), and the pace each window asks for.
    pub(super) fn redraw(&mut self, event_loop: &ActiveEventLoop, id: WindowId) {
        if !self.windows.get(id).is_some_and(|host| host.has_gpu()) {
            return;
        }
        // A covered window, or a sleeping app: no frame (the one it shows
        // with is asked as it shows again; `sleep.rs`).
        if !self.windows.awake(id) {
            return;
        }
        // The real time since the last tick: a deadline the loop
        // slept through arrives whole (`dt` is clamped for motion).
        let since_last = self.clock.tick().since_last;
        let wake = self.frame_wake(since_last);
        let page_wake = self.view.detached_wake();
        log::debug!(
            "frame: {:.3} s since the last tick, wake now {} next {:?}, page {:?}",
            since_last,
            wake.now,
            wake.next,
            page_wake.next
        );
        let clear = self.view.theme.colors.bg.to_linear_array();
        if Some(id) == self.main {
            let Some(host) = self.windows.get_mut(id) else {
                return;
            };
            host.begin_frame();
            // Build the scene (includes compositor.begin_frame()), then
            // the shared frame tail.
            self.view.render(host.compositor_mut());
            host.present(clear, "showcase_frame");
            // The tree of this frame, placed by the bindings it just drew;
            // what a reader is shown (the page, the toasts and the overlay
            // over it, what only a reader hears) is projected only while a
            // reader listens.
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            if let (Some(bridge), Some(window)) = (self.access.bridge.as_mut(), host.window()) {
                let view = &self.view;
                bridge.update(window, host.compositor(), || view.reader_semantics());
            }
            // The frame just drawn, where each bound control shows in it,
            // and the state it was drawn from.
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            self.record_frame();
            // The frame to whoever watches this window's scene (task-79).
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            self.offer_scene(id);
            // The bar and the tray follow what this frame was drawn from.
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            {
                self.sync_menu();
                self.sync_tray();
            }
        } else if self.is_page_window(id) {
            let Some(host) = self.windows.get_mut(id) else {
                return;
            };
            host.begin_frame();
            self.view.render_detached(host.compositor_mut());
            host.present(clear, "showcase_page_window");
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            self.offer_scene(id);
        }
        // `wake.now` is carried by this frame: the scene was built after
        // the tick, so a flipped caret is already in it.
        self.pace(event_loop, wake, page_wake);
    }

    /// Advances what animates to now, before anything from outside the
    /// frame lands (input, a worker result, a reader's or an agent's
    /// request). The tick gets the real time since the last one, so a
    /// change applied between two frames would otherwise take the whole
    /// gap on the next: a switch flipped after three idle seconds would
    /// jump to its end, a toast pushed then would start three seconds
    /// old, a caret focused then would start in any phase. This tick
    /// draws nothing; if it changed something visible, it asks for the
    /// frame. Asleep, nothing: the clock does not tick either, so the
    /// whole time asleep lands on the first tick after the wake.
    pub(super) fn catch_up(&mut self) {
        if self.windows.asleep() {
            return;
        }
        let since_last = self.clock.tick().since_last;
        self.advance(since_last);
    }

    /// [`App::catch_up`] with the elapsed time given. What the tick
    /// settled (a job that ended) and what the pages asked to tell the
    /// person are notified right after, when no window has the focus.
    /// Nothing while the app sleeps.
    pub(super) fn advance(&mut self, since_last: f32) {
        if self.windows.asleep() {
            return;
        }
        if self.view.wake(since_last).now {
            self.caught_up = true;
            self.invalidate();
        }
        if self.view.detached_wake().now
            && let Some(id) = self.page_window_id()
        {
            self.invalidate_in(id);
        }
        // A print job that answered, or one held for the wake: the shell
        // projects it and the bar follows the next frame (task-76 item 9).
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if self.view.settle_print() {
            self.caught_up = true;
            self.invalidate();
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        self.notify_pending();
    }

    /// The frame's tick: what animates, advanced by the time since the
    /// last tick (a catch-up or a frame), and what that asks of the loop.
    /// `now` also carries a visible change a catch-up made since the last
    /// frame; a print thread that is out asks for a look at its channel.
    pub(super) fn frame_wake(&mut self, since_last: f32) -> Wake {
        let mut wake = self.view.wake(since_last).merge(self.view.print_wake());
        wake.now |= std::mem::take(&mut self.caught_up);
        wake
    }

    /// What the frame's [`Wake`]s ask of the loop, after the frame that
    /// carries their `now` was drawn: `wake` is the main window's (the
    /// chrome, the overlays and, when it is there, the page), `page_wake`
    /// the page's while it is in its own window. Continuous motion draws
    /// that window's next frame; a deadline is that window's, and the
    /// loop waits until the earliest of all; nothing pending sleeps until
    /// input. The wait ends early on any event, so the git worker, the
    /// screen reader and the agent port still get their frame through
    /// `invalidate` whenever they ask.
    pub(super) fn pace(&mut self, event_loop: &ActiveEventLoop, wake: Wake, page_wake: Wake) {
        let main = self.main;
        self.pace_window(main, wake);
        let page = self.page_window_id().filter(|id| Some(*id) != main);
        self.pace_window(page, page_wake);
        match self.windows.earliest_deadline() {
            Some(at) => event_loop.set_control_flow(ControlFlow::WaitUntil(at)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    /// One window's share of the pace.
    fn pace_window(&mut self, id: Option<WindowId>, wake: Wake) {
        let Some(host) = id.and_then(|id| self.windows.get_mut(id)) else {
            return;
        };
        match wake.next {
            Some(next) if next.is_zero() => {
                // Springs still moving: keep the frames coming.
                host.compositor_mut().invalidate();
                host.request_redraw();
                host.set_deadline(None);
            }
            Some(next) => host.set_deadline(Some(Instant::now() + next)),
            None => host.set_deadline(None),
        }
    }

    /// A deadline came due: the windows whose deadline it was ask for
    /// their frame; the others keep theirs.
    pub(super) fn deadlines_due(&mut self) {
        let now = Instant::now();
        for (_, host) in self.windows.iter_mut() {
            if host.deadline().is_some_and(|at| at <= now) {
                host.set_deadline(None);
                host.compositor_mut().invalidate();
                host.request_redraw();
            }
        }
    }
}

#[cfg(test)]
mod tests;
