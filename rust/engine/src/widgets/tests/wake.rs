//! The tick contract: how asks compose, the caret clock's flips and the
//! toasts' lifetimes, with an injected `dt`.

use std::time::Duration;

use crate::styling::theme::{Intent, Theme};

use crate::widgets::feedback::ToastManager;
use crate::widgets::wake::{Blink, Wake};
use crate::widgets::widget::WidgetEvent;

const HALF: f32 = 0.53;

fn secs(d: Duration) -> f32 {
    d.as_secs_f32()
}

#[test]
fn idle_is_the_default_and_asks_nothing() {
    assert_eq!(Wake::default(), Wake::idle());
    assert!(!Wake::idle().pending());
    assert!(!Wake::idle().is_continuous());
}

#[test]
fn continuous_is_a_frame_now_and_the_very_next() {
    let w = Wake::continuous();
    assert!(w.now && w.is_continuous() && w.pending());
    assert_eq!(Wake::moving(true), Wake::continuous());
    assert_eq!(Wake::moving(false), Wake::idle());
}

#[test]
fn merge_keeps_any_now_and_the_earliest_deadline() {
    let soon = Wake::after(Duration::from_millis(200));
    let later = Wake::after(Duration::from_millis(500));
    assert_eq!(soon.merge(later), soon);
    assert_eq!(later.merge(soon), soon);
    assert_eq!(Wake::idle().merge(later), later);
    assert_eq!(later.merge(Wake::idle()), later);
    let merged = Wake::now().merge(later);
    assert!(merged.now);
    assert_eq!(merged.next, later.next);
    // continuous motion beats every deadline
    assert!(later.merge(Wake::continuous()).is_continuous());
}

#[test]
fn a_deadline_is_pending_but_not_a_frame_now() {
    let w = Wake::after(Duration::from_millis(300));
    assert!(w.pending());
    assert!(!w.now);
    assert!(!w.is_continuous());
}

#[test]
fn blink_flips_at_the_half_period_and_asks_for_the_remainder_between() {
    let mut b = Blink::new();
    assert!(b.visible());
    let w = b.advance(0.2, HALF);
    assert!(!w.now, "no flip yet");
    assert!(b.visible());
    assert!((secs(w.next.unwrap()) - 0.33).abs() < 1e-3, "{:?}", w.next);
    let w = b.advance(0.2, HALF);
    assert!(!w.now);
    assert!((secs(w.next.unwrap()) - 0.13).abs() < 1e-3);
    let w = b.advance(0.2, HALF);
    assert!(w.now, "0.6 s: flipped off");
    assert!(!b.visible());
    // the phase carries over: 0.07 s into the hidden half
    assert!((secs(w.next.unwrap()) - 0.46).abs() < 1e-3, "{:?}", w.next);
}

#[test]
fn blink_flips_back_and_never_asks_for_every_frame() {
    let mut b = Blink::new();
    let mut flips = 0;
    let mut t = 0.0;
    while t < 5.0 {
        let w = b.advance(1.0 / 60.0, HALF);
        t += 1.0 / 60.0;
        assert!(!w.is_continuous(), "a caret is a deadline, not a frame");
        if w.now {
            flips += 1;
        }
    }
    // 5 s of 0.53 s half-periods: nine flips, ending hidden
    assert_eq!(flips, 9);
    assert!(!b.visible());
}

#[test]
fn blink_reset_makes_the_caret_solid_and_restarts_the_half_period() {
    let mut b = Blink::new();
    b.advance(0.6, HALF);
    assert!(!b.visible());
    b.reset();
    assert!(b.visible());
    let w = b.advance(0.0, HALF);
    assert!(!w.now);
    assert!((secs(w.next.unwrap()) - HALF).abs() < 1e-6);
}

#[test]
fn blink_an_even_number_of_flips_changes_nothing_visible() {
    let mut b = Blink::new();
    let w = b.advance(HALF * 2.0 + 0.01, HALF);
    assert!(!w.now, "two flips land on the same side");
    assert!(b.visible());
    assert!((secs(w.next.unwrap()) - (HALF - 0.01)).abs() < 1e-3);
    let w = b.advance(HALF * 3.0, HALF);
    assert!(w.now, "three flips: hidden");
    assert!(!b.visible());
}

#[test]
fn blink_without_an_interval_stays_solid_and_idle() {
    for interval in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let mut b = Blink::new();
        b.advance(0.6, HALF);
        assert!(!b.visible());
        assert_eq!(b.advance(1.0, interval), Wake::idle());
        assert!(b.visible(), "no blink means a solid caret");
    }
}

#[test]
fn blink_ignores_a_dt_that_is_not_time() {
    let mut b = Blink::new();
    for dt in [0.0, -0.5, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let w = b.advance(dt, HALF);
        assert!(!w.now);
        assert!((secs(w.next.unwrap()) - HALF).abs() < 1e-6, "dt {dt}");
    }
}

#[test]
fn blink_survives_a_very_long_stall() {
    let mut b = Blink::new();
    let w = b.advance(1.0e9, HALF);
    assert!(w.next.is_some());
    assert!(secs(w.next.unwrap()) <= HALF);
    // and keeps ticking sanely afterwards
    let w = b.advance(0.1, HALF);
    assert!(w.next.is_some());
}

// -- toasts: a deadline between two springs ---------------------------------

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

/// A shell over `turns` display turns, as the showcase and ide shells
/// pace: turn 0 is the frame of the event that pushed; afterwards a turn
/// is drawn when the last ask was continuous or its deadline has come
/// due, and the drawn turn's tick gets the time since the last drawn turn
/// (`since_last`). Returns each drawn turn with its ask.
fn paced(turns: usize, mut wake: impl FnMut(f32) -> Wake) -> Vec<(usize, Wake)> {
    let mut drawn = vec![(0, wake(0.0))];
    for turn in 1..turns {
        let (last, ask) = *drawn.last().expect("turn 0 is drawn");
        let since = (turn - last) as f32 * TURN;
        let due = match ask.next {
            Some(d) => since >= d.as_secs_f32(),
            None => false,
        };
        if due {
            drawn.push((turn, wake(since)));
        }
    }
    drawn
}

#[test]
fn a_toast_springs_in_rests_on_a_deadline_and_springs_out() {
    let theme = Theme::dark();
    let mut toasts = ToastManager::new();
    toasts.push("saved", Intent::Constructive, &theme);
    // six seconds at 60 Hz: 360 turns
    let drawn = paced(360, |dt| toasts.wake(dt));
    assert!(toasts.is_empty(), "gone within the six seconds");
    assert_eq!(drawn.last().map(|(_, w)| w.next), Some(None), "idle after");

    // the entry: consecutive turns, each asking for the next
    let rest = drawn
        .iter()
        .position(|(_, w)| !w.is_continuous())
        .expect("the entry settles");
    let (rest_turn, rest_ask) = drawn[rest];
    assert_eq!(rest_turn, rest, "the entry draws every turn");
    // at rest the ask is the dismissal: 4 s after the first tick, which
    // counted no time (the push came with the event)
    let left = rest_ask.next.expect("a deadline, not idle").as_secs_f32();
    let due = rest_turn as f32 * TURN + left;
    assert!((due - toasts.duration).abs() < TURN, "due at {due}");
    // nothing drawn while it rests; the next frame is the dismissal, on
    // the first turn at or after the deadline
    let (dismiss_turn, dismiss_ask) = drawn[rest + 1];
    let at = dismiss_turn as f32 * TURN - toasts.duration;
    assert!((-1e-3..TURN + 1e-3).contains(&at), "dismissed {at} s late");
    assert!(dismiss_ask.is_continuous(), "the exit spring moves");
    // and the exit draws every turn until the toast is dropped
    let exit = &drawn[rest + 1..];
    assert!(exit.windows(2).all(|p| p[1].0 == p[0].0 + 1));
    // the springs draw every turn; the rest between them draws none
    let resting = dismiss_turn - rest_turn - 1;
    let last = drawn.last().map_or(0, |(turn, _)| *turn);
    assert_eq!(drawn.len() + resting, last + 1, "every turn but the rest");
    assert!(
        resting as f32 * TURN > 2.0,
        "rested {resting} turns undrawn"
    );
}

/// A tick counts all the time it is given: a toast pushed and then
/// ticked with ten idle seconds is already leaving. That is why the
/// shells catch up to the instant of an event before they apply it, and
/// why the toast needs no notion of when it was pushed.
#[test]
fn a_toast_counts_all_the_time_its_tick_is_given() {
    let theme = Theme::dark();
    let mut late = ToastManager::new();
    late.push("refused", Intent::Destructive, &theme);
    late.wake(10.0);
    assert!(late.visible().next().is_none_or(|t| t.is_closing()));

    // caught up first, the same push lives its whole duration
    let mut caught = ToastManager::new();
    assert_eq!(caught.wake(10.0), Wake::idle(), "the catch-up");
    caught.push("refused", Intent::Destructive, &theme);
    assert!(caught.wake(TURN).is_continuous(), "springing in");
    let t = caught.visible().next().expect("on screen");
    assert!(!t.is_closing());
    assert!(t.progress() < 0.1, "the entry starts on this frame");
}

#[test]
fn a_click_dismiss_springs_the_toast_out() {
    let theme = Theme::dark();
    let mut toasts = ToastManager::new();
    toasts.push("clickme", Intent::Neutral, &theme);
    for _ in 0..60 {
        toasts.wake(TURN);
    }
    let rect = toasts.visible_rects(&theme, 800.0, 600.0)[0];
    let (x, y) = rect.center();
    let click = WidgetEvent::MouseDown { x, y };
    assert!(toasts.handle_event(&click, &theme, 800.0, 600.0).clicked);
    let w = toasts.wake(TURN);
    assert!(w.is_continuous(), "the exit spring moves");
    assert!(toasts.visible().next().expect("still leaving").is_closing());
}

#[test]
fn toasts_that_never_dismiss_or_are_gone_ask_for_nothing() {
    let theme = Theme::dark();
    let mut toasts = ToastManager::new();
    assert_eq!(toasts.wake(TURN), Wake::idle(), "no toast");
    toasts.duration = f32::INFINITY;
    toasts.push("sticky", Intent::Neutral, &theme);
    for _ in 0..60 {
        toasts.wake(TURN);
    }
    assert_eq!(toasts.wake(TURN), Wake::idle(), "at rest with no deadline");
    for dt in [-1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(toasts.wake(dt), Wake::idle(), "dt {dt} is no time");
    }
    assert_eq!(toasts.len(), 1);
}
