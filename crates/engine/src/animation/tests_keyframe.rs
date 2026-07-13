use super::*;

#[test]
fn frame_clock_ticks() {
    let mut clock = FrameClock::new();
    let tick = clock.tick();
    assert!(tick.dt >= 0.0);
    assert!(tick.elapsed >= 0.0);
    assert!(tick.dt <= 0.1);
}

#[test]
fn frame_clock_dt_clamped() {
    let mut clock = FrameClock::new();
    let tick = clock.tick();
    assert!(tick.dt <= 0.1);
}

#[test]
fn keyframe_sequence_basic() {
    let mut seq = KeyframeSequence::new(2.0)
        .keyframe(0.0_f32, 0.0, Easing::Linear)
        .keyframe(50.0, 0.5, Easing::Linear)
        .keyframe(100.0, 1.0, Easing::Linear)
        .start();

    assert!(seq.is_animating());
    assert!((seq.now() - 0.0).abs() < 0.01);

    seq.advance_by(0.5);
    let v = seq.now();
    assert!((v - 25.0).abs() < 1.0, "At t=0.25, expected ~25, got {v}");

    seq.advance_by(0.5);
    let v = seq.now();
    assert!((v - 50.0).abs() < 1.0, "At t=0.5, expected ~50, got {v}");

    seq.advance_by(1.0);
    let v = seq.now();
    assert!((v - 100.0).abs() < 0.01, "At t=1.0, expected 100, got {v}");
    assert_eq!(seq.state(), TweenState::Completed);
}

#[test]
fn keyframe_sequence_per_segment_easing() {
    let mut seq = KeyframeSequence::new(1.0)
        .keyframe(0.0_f32, 0.0, Easing::EaseIn)
        .keyframe(50.0, 0.5, Easing::Linear)
        .keyframe(100.0, 1.0, Easing::Linear)
        .start();

    seq.advance_by(0.25);
    let v = seq.now();
    assert!(v < 25.0, "EaseIn should be slower at start, got {v}");
}

#[test]
fn keyframe_sequence_wrap() {
    let mut seq = KeyframeSequence::new(1.0)
        .keyframe(0.0_f32, 0.0, Easing::Linear)
        .keyframe(100.0, 1.0, Easing::Linear)
        .start();

    seq.advance_and_wrap(1.5);
    let v = seq.now();
    assert!((v - 50.0).abs() < 1.0, "After wrap, expected ~50, got {v}");
}

#[test]
fn keyframe_sequence_reverse() {
    let mut seq = KeyframeSequence::new(1.0)
        .keyframe(0.0_f32, 0.0, Easing::Linear)
        .keyframe(100.0, 1.0, Easing::Linear)
        .start();

    seq.advance_and_reverse(1.5);
    let v = seq.now();
    assert!(
        (v - 50.0).abs() < 1.0,
        "After reverse, expected ~50, got {v}"
    );
}
