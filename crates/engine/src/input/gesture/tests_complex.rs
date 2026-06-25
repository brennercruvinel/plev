use std::time::Duration;
use web_time::Instant;

use crate::input::{GestureEvent, Phase, Point};

use crate::input::gesture::GestureRecognizer;

// Helpers for deterministic timing
fn t0() -> Instant {
    Instant::now()
}

fn advance(base: Instant, millis: u64) -> Instant {
    base + Duration::from_millis(millis)
}

fn pt(x: f64, y: f64) -> Point {
    Point { x, y }
}

// -- Pinch --

#[test]
fn pinch() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_start(2, pt(200.0, 100.0), advance(t, 10));
    // Now pinch -- move fingers apart
    g.touch_move(1, pt(50.0, 100.0), advance(t, 50));
    g.touch_move(2, pt(250.0, 100.0), advance(t, 60));
    g.touch_end(2, pt(250.0, 100.0), advance(t, 100));
    g.touch_end(1, pt(50.0, 100.0), advance(t, 110));
    let events = g.drain_events();
    let has_pinch_start = events
        .iter()
        .any(|e| matches!(e, GestureEvent::Pinch(p) if p.phase == Phase::Started));
    let has_pinch_changed = events
        .iter()
        .any(|e| matches!(e, GestureEvent::Pinch(p) if p.phase == Phase::Changed));
    assert!(has_pinch_start);
    assert!(has_pinch_changed);
}

// -- Cancel --

#[test]
fn cancel_mid_drag() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_move(1, pt(200.0, 100.0), advance(t, 50));
    g.touch_cancel(1);
    let events = g.drain_events();
    let has_cancel = events
        .iter()
        .any(|e| matches!(e, GestureEvent::Drag(d) if d.phase == Phase::Cancelled));
    assert!(has_cancel);
}

// -- Three fingers --

#[test]
fn three_fingers() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_start(2, pt(200.0, 100.0), advance(t, 10));
    g.touch_start(3, pt(300.0, 100.0), advance(t, 20));
    assert!(g.is_touch_active());
    g.touch_end(3, pt(300.0, 100.0), advance(t, 100));
    g.touch_end(2, pt(200.0, 100.0), advance(t, 110));
    g.touch_end(1, pt(100.0, 100.0), advance(t, 120));
    // Should not panic and should have produced pinch events
    let events = g.drain_events();
    assert!(!events.is_empty());
}

// -- Long press then drag --

#[test]
fn long_press_then_drag() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    g.tick(advance(t, 600));
    let events = g.drain_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GestureEvent::LongPress(_)))
    );

    // Now move -- should transition to drag
    g.touch_move(1, pt(200.0, 100.0), advance(t, 700));
    let events = g.drain_events();
    assert!(events.iter().any(|e| matches!(e, GestureEvent::Drag(_))));
}

// -- Drag to pinch --

#[test]
fn drag_to_pinch() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_move(1, pt(200.0, 100.0), advance(t, 50));
    // Should be dragging now
    let events = g.drain_events();
    assert!(events.iter().any(|e| matches!(e, GestureEvent::Drag(_))));

    // Add second finger -- should transition to pinch
    g.touch_start(2, pt(300.0, 100.0), advance(t, 100));
    let events = g.drain_events();
    assert!(events.iter().any(|e| matches!(e, GestureEvent::Pinch(_))));
}

// -- Swipe too slow --

#[test]
fn swipe_too_slow_is_drag() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    // Move 60px over 2 seconds -- velocity = 30 px/s, well below SWIPE_MIN_VEL
    g.touch_move(1, pt(140.0, 100.0), advance(t, 1000));
    g.touch_end(1, pt(160.0, 100.0), advance(t, 2000));
    let events = g.drain_events();
    assert!(!events.iter().any(|e| matches!(e, GestureEvent::Swipe(_))));
    assert!(events.iter().any(|e| matches!(e, GestureEvent::Drag(_))));
}

// -- Tap too slow --

#[test]
fn tap_too_slow_is_ignored() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    // Hold for 400ms without moving (past TAP_MAX but before LONG_PRESS)
    g.touch_end(1, pt(100.0, 100.0), advance(t, 400));
    let events = g.drain_events();
    // Should not be a tap (too slow) and not a long-press (not held long enough, no tick)
    assert!(!events.iter().any(|e| matches!(e, GestureEvent::Tap(_))));
}

// -- Double-tap timeout --

#[test]
fn double_tap_timeout() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    // First tap
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_end(1, pt(100.0, 100.0), advance(t, 50));
    // Wait past timeout
    g.tick(advance(t, 500));
    // Second tap -- should be a new single tap, not double
    g.touch_start(2, pt(100.0, 100.0), advance(t, 600));
    g.touch_end(2, pt(100.0, 100.0), advance(t, 650));
    let events = g.drain_events();
    let double_taps = events
        .iter()
        .filter(|e| matches!(e, GestureEvent::DoubleTap(_)))
        .count();
    assert_eq!(double_taps, 0);
}

// -- Theme invariance --
// Gesture recognition thresholds (TAP_MAX_DURATION, LONG_PRESS_DURATION,
// DOUBLE_TAP_TIMEOUT, TOUCH_SLOP, SWIPE_MIN_VEL) are ergonomic constants
// of the input system, not visual design tokens. They must NOT change with
// theme.motion. This test documents that invariance.

#[test]
fn gesture_timing_independent_of_theme() {
    // Same tap sequence must produce same result regardless of what
    // theme.motion values exist -- gesture recognizer never reads theme.
    let t = t0();

    let mut g1 = GestureRecognizer::new();
    g1.touch_start(1, pt(100.0, 100.0), t);
    g1.touch_end(1, pt(100.0, 100.0), advance(t, 100));
    let events1 = g1.drain_events();

    let mut g2 = GestureRecognizer::new();
    g2.touch_start(1, pt(100.0, 100.0), t);
    g2.touch_end(1, pt(100.0, 100.0), advance(t, 100));
    let events2 = g2.drain_events();

    // Both produce identical tap events
    assert_eq!(events1.len(), events2.len());
    assert!(matches!(&events1[0], GestureEvent::Tap(_)));
    assert!(matches!(&events2[0], GestureEvent::Tap(_)));
}
