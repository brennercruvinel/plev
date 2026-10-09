use std::time::Duration;
use web_time::Instant;

use crate::input::{GestureEvent, Phase, Point, SwipeDirection};

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

// -- Tap --

#[test]
fn tap() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_end(1, pt(100.0, 100.0), advance(t, 100));
    let events = g.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], GestureEvent::Tap(_)));
    if let GestureEvent::Tap(ref e) = events[0] {
        assert!((e.position.x - 100.0).abs() < 0.001);
    }
}

#[test]
fn tap_rejected_moved() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    // Move well past slop
    g.touch_move(1, pt(200.0, 100.0), advance(t, 50));
    g.touch_end(1, pt(200.0, 100.0), advance(t, 100));
    let events = g.drain_events();
    // Should get drag events, not tap
    assert!(events.iter().all(|e| !matches!(e, GestureEvent::Tap(_))));
    assert!(events.iter().any(|e| matches!(e, GestureEvent::Drag(_))));
}

// -- Double Tap --

#[test]
fn double_tap() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    // First tap
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_end(1, pt(100.0, 100.0), advance(t, 50));
    // Second tap within timeout and slop
    g.touch_start(2, pt(105.0, 105.0), advance(t, 150));
    g.touch_end(2, pt(105.0, 105.0), advance(t, 200));
    let events = g.drain_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GestureEvent::DoubleTap(_)))
    );
}

#[test]
fn double_tap_rejected_far() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    // First tap
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_end(1, pt(100.0, 100.0), advance(t, 50));
    // Second tap too far away
    g.touch_start(2, pt(300.0, 300.0), advance(t, 150));
    g.touch_end(2, pt(300.0, 300.0), advance(t, 200));
    let events = g.drain_events();
    // Should get two taps, not a double-tap
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GestureEvent::DoubleTap(_)))
    );
    let tap_count = events
        .iter()
        .filter(|e| matches!(e, GestureEvent::Tap(_)))
        .count();
    assert_eq!(tap_count, 2);
}

// -- Long Press --

#[test]
fn long_press() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    // Tick past long-press threshold
    g.tick(advance(t, 600));
    let events = g.drain_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GestureEvent::LongPress(_)))
    );
}

#[test]
fn long_press_rejected_moved() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    // Move past slop before long-press triggers
    g.touch_move(1, pt(200.0, 100.0), advance(t, 100));
    g.tick(advance(t, 600));
    let events = g.drain_events();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GestureEvent::LongPress(_)))
    );
}

// -- Swipe --

#[test]
fn swipe_right() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    g.touch_move(1, pt(200.0, 105.0), advance(t, 50));
    g.touch_end(1, pt(300.0, 110.0), advance(t, 100));
    let events = g.drain_events();
    let swipe = events.iter().find(|e| matches!(e, GestureEvent::Swipe(_)));
    assert!(swipe.is_some());
    if let Some(GestureEvent::Swipe(s)) = swipe {
        assert_eq!(s.direction, SwipeDirection::Right);
    }
}

#[test]
fn swipe_up() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 300.0), t);
    g.touch_move(1, pt(105.0, 200.0), advance(t, 50));
    g.touch_end(1, pt(110.0, 50.0), advance(t, 100));
    let events = g.drain_events();
    let swipe = events.iter().find(|e| matches!(e, GestureEvent::Swipe(_)));
    assert!(swipe.is_some());
    if let Some(GestureEvent::Swipe(s)) = swipe {
        assert_eq!(s.direction, SwipeDirection::Up);
    }
}

// -- Drag --

#[test]
fn drag_lifecycle() {
    let mut g = GestureRecognizer::new();
    let t = t0();
    g.touch_start(1, pt(100.0, 100.0), t);
    // Move past slop slowly (not fast enough for swipe)
    g.touch_move(1, pt(115.0, 100.0), advance(t, 100));
    g.touch_move(1, pt(130.0, 100.0), advance(t, 500));
    g.touch_end(1, pt(145.0, 100.0), advance(t, 1000));
    let events = g.drain_events();
    let drag_started = events
        .iter()
        .any(|e| matches!(e, GestureEvent::Drag(d) if d.phase == Phase::Started));
    let drag_changed = events
        .iter()
        .any(|e| matches!(e, GestureEvent::Drag(d) if d.phase == Phase::Changed));
    let drag_ended = events
        .iter()
        .any(|e| matches!(e, GestureEvent::Drag(d) if d.phase == Phase::Ended));
    assert!(drag_started);
    assert!(drag_changed);
    assert!(drag_ended);
}
