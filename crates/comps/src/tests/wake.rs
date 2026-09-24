//! The tick contract: how asks compose, and the caret clock's flips with
//! an injected `dt`.

use std::time::Duration;

use crate::wake::{Blink, Wake};

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
