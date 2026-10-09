//! The loop's pacing: time catches up before what woke the loop lands,
//! the frame's tick says what it asks of the loop, and the answer sets
//! the wait (see the shell's module doc).

use std::time::Instant;

use comps::prelude::Wake;
use winit::event_loop::{ActiveEventLoop, ControlFlow};

use crate::App;

impl App {
    /// Advances what animates to now, before anything from outside the
    /// frame lands (a key, a click, a git result, a file change). The
    /// tick gets the real time since the last one, so a change applied
    /// between two frames would otherwise take the whole gap on the next:
    /// a refusal toast pushed after ten idle seconds would start ten
    /// seconds old and leave at once, a caret focused then would start in
    /// any phase. This tick draws nothing; if it changed something
    /// visible, it asks for the frame.
    pub(crate) fn catch_up(&mut self) {
        let since_last = self.clock.tick().since_last;
        self.advance(since_last);
    }

    /// [`App::catch_up`] with the elapsed time given.
    pub(crate) fn advance(&mut self, since_last: f32) {
        if self.bench.wake(since_last).now {
            self.caught_up = true;
            self.invalidate();
        }
    }

    /// The frame's tick: what animates, advanced by the time since the
    /// last tick (a catch-up or a frame), and what that asks of the loop.
    /// `now` also carries a visible change a catch-up made since the last
    /// frame.
    pub(crate) fn frame_wake(&mut self, since_last: f32) -> Wake {
        let mut wake = self.bench.wake(since_last);
        wake.now |= std::mem::take(&mut self.caught_up);
        wake
    }

    /// What the frame's [`Wake`] asks of the loop, after the frame that
    /// carries its `now` was drawn: continuous motion draws the next
    /// frame; a deadline sets the wait to that instant; nothing pending
    /// sleeps until an event. Any event ends a wait early.
    pub(crate) fn pace(&mut self, event_loop: &ActiveEventLoop, wake: Wake) {
        match wake.next {
            Some(next) if next.is_zero() => {
                // Something still moves: keep the frames coming.
                self.invalidate();
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
mod tests {
    //! The shell's pacing with an injected clock: a change from outside
    //! the frame lands after the loop caught up (`App::advance`, what
    //! `catch_up` runs with the real elapsed time), so it counts its time
    //! from the change, however long the window sat idle before it.

    use comps::prelude::Wake;
    use ide::workbench::Workbench;

    use crate::App;

    /// One turn of a 60 Hz display.
    const TURN: f32 = 1.0 / 60.0;

    /// How long the window sat idle before the event.
    const IDLE: f32 = 10.0;

    /// How long after the event its frame is drawn.
    const LATENCY: f32 = 0.004;

    /// The shell around a workbench with no repository, settled.
    fn app() -> App {
        let mut app = App::new(Workbench::new(1280.0, 800.0), None);
        for _ in 0..600 {
            if !app.frame_wake(TURN).is_continuous() {
                break;
            }
        }
        app
    }

    /// The frames over `turns` display turns after an event: turn 0 is
    /// the event's frame; afterwards a turn is drawn when the last ask was
    /// continuous or its deadline has come due, and its tick gets the time
    /// since the last drawn turn.
    fn paced(app: &mut App, turns: usize) -> Vec<(usize, Wake)> {
        let mut drawn = vec![(0, app.frame_wake(LATENCY))];
        for turn in 1..turns {
            let (last, ask) = *drawn.last().expect("turn 0 is drawn");
            let since = (turn - last) as f32 * TURN;
            if ask.next.is_some_and(|d| since >= d.as_secs_f32()) {
                drawn.push((turn, app.frame_wake(since)));
            }
        }
        drawn
    }

    /// The toasts the workbench projects.
    fn notices(app: &App) -> usize {
        let snap = app.bench.snapshot().expect("snapshot");
        let id: sem::SemId = "notices".parse().expect("id");
        snap.children(&id).count()
    }

    #[test]
    fn a_refusal_toast_after_an_idle_gap_lives_its_whole_duration() {
        let mut app = app();
        app.advance(IDLE);
        // Enter on an empty message: refused, and the refusal is a toast
        assert!(app.bench.toggle_commit_form());
        assert!(app.bench.submit_commit());
        assert!(app.bench.cancel_commit_form());
        assert_eq!(notices(&app), 1);
        let drawn = paced(&mut app, 360);
        let rest = drawn
            .iter()
            .position(|(_, ask)| !ask.is_continuous())
            .expect("the entry settles");
        let (rest_turn, rest_ask) = drawn[rest];
        let left = rest_ask.next.expect("a deadline while it rests");
        let due = rest_turn as f32 * TURN + LATENCY + left.as_secs_f32();
        assert!((due - 4.0).abs() < TURN, "leaves {due} s after the refusal");
        assert_eq!(notices(&app), 0, "gone within six seconds");
    }

    #[test]
    fn a_caret_focused_after_an_idle_gap_starts_solid_and_flips_on_time() {
        let mut app = app();
        app.advance(IDLE);
        assert!(
            app.bench.toggle_commit_form(),
            "`c` opens the form, focused"
        );
        let drawn = paced(&mut app, 360);
        let (_, first) = drawn[0];
        assert!(!first.now, "no flip on the key's frame");
        // a whole half-period from the key, none of the idle before it
        let next = first.next.expect("a flip is pending").as_secs_f32();
        assert!(
            (0.53 - LATENCY - 1e-3..0.53 + 1e-3).contains(&next),
            "{next}"
        );
        let flips: Vec<usize> = drawn[1..].iter().map(|(turn, _)| *turn).collect();
        assert!(drawn[1..].iter().all(|(_, ask)| ask.now), "{drawn:?}");
        assert_eq!(flips.len(), 11, "{drawn:?}");
        assert_eq!(flips[0], 32, "the first flip 0.53 s after the key");
    }
}
