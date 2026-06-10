use super::*;

#[test]
fn lerp_f32() {
    assert!((0.0_f32.lerp(&1.0, 0.5) - 0.5).abs() < 0.001);
    assert!((0.0_f32.lerp(&10.0, 0.0)).abs() < 0.001);
    assert!((0.0_f32.lerp(&10.0, 1.0) - 10.0).abs() < 0.001);
}

#[test]
fn lerp_array2() {
    let a = [0.0, 0.0];
    let b = [10.0, 20.0];
    let r = a.lerp(&b, 0.5);
    assert!((r[0] - 5.0).abs() < 0.001);
    assert!((r[1] - 10.0).abs() < 0.001);
}

#[test]
fn lerp_array4() {
    let a = [0.0, 0.0, 0.0, 0.0];
    let b = [1.0, 1.0, 1.0, 1.0];
    let r = a.lerp(&b, 0.25);
    for v in r {
        assert!((v - 0.25).abs() < 0.001);
    }
}

#[test]
fn lerp_array5() {
    let a = [0.0, 0.0, 0.0, 0.0, 0.0];
    let b = [10.0, 20.0, 30.0, 40.0, 50.0];
    let r = a.lerp(&b, 0.5);
    assert!((r[0] - 5.0).abs() < 0.001);
    assert!((r[4] - 25.0).abs() < 0.001);
}

#[test]
fn tween_idle_by_default() {
    let tw = Tween::new(0.0_f32, 1.0, Easing::Linear);
    assert_eq!(tw.state(), TweenState::Idle);
    assert!(!tw.is_animating());
    assert!((tw.get() - 0.0).abs() < 0.001);
}

#[test]
fn tween_set_target_starts_animation() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear);
    tw.set_target(100.0);
    assert!(tw.is_animating());
    assert_eq!(tw.state(), TweenState::Running);
}

#[test]
fn tween_tick_progresses() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear);
    tw.set_target(100.0);
    tw.tick(0.5);
    let v = tw.get();
    assert!((v - 50.0).abs() < 1.0, "Expected ~50 at t=0.5, got {}", v);
}

#[test]
fn tween_completes() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear);
    tw.set_target(100.0);
    tw.tick(1.5);
    assert_eq!(tw.state(), TweenState::Completed);
    assert!((tw.get() - 100.0).abs() < 0.001);
}

#[test]
fn tween_eased() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::EaseIn);
    tw.set_target(100.0);
    tw.tick(0.5);
    let v = tw.get();
    assert!(
        (v - 25.0).abs() < 1.0,
        "EaseIn at half should be ~25, got {}",
        v
    );
}

#[test]
fn tween_retarget() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear);
    tw.set_target(100.0);
    tw.tick(0.5);
    tw.set_target(200.0);
    tw.tick(0.5);
    let v = tw.get();
    assert!(
        (v - 125.0).abs() < 5.0,
        "Retarget midpoint expected ~125, got {}",
        v
    );
}

#[test]
fn tween_color() {
    let mut tw = Tween::new([0.0, 0.0, 0.0, 1.0_f32], 1.0, Easing::Linear);
    tw.set_target([1.0, 1.0, 1.0, 1.0]);
    tw.tick(0.5);
    let c = tw.get();
    assert!((c[0] - 0.5).abs() < 0.01);
    assert!((c[3] - 1.0).abs() < 0.01);
}

#[test]
fn tween_reset() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear);
    tw.set_target(100.0);
    tw.tick(0.5);
    tw.reset(50.0);
    assert_eq!(tw.state(), TweenState::Idle);
    assert!((tw.get() - 50.0).abs() < 0.001);
}

#[test]
fn tween_idle_tick_does_nothing() {
    let mut tw = Tween::new(42.0_f32, 1.0, Easing::Linear);
    tw.tick(1.0);
    assert_eq!(tw.state(), TweenState::Idle);
    assert!((tw.get() - 42.0).abs() < 0.001);
}

#[test]
fn tween_with_delay() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear).with_delay(0.5);
    tw.set_target(100.0);
    tw.tick(0.25);
    assert!(
        (tw.get() - 0.0).abs() < 0.01,
        "During delay, value should stay at from"
    );
    tw.tick(0.25);
    assert!(
        (tw.get() - 0.0).abs() < 0.01,
        "At delay boundary, animation hasn't started"
    );
    tw.tick(0.5);
    let v = tw.get();
    assert!(
        (v - 50.0).abs() < 2.0,
        "After delay + 0.5s, expected ~50, got {}",
        v
    );
}

#[test]
fn tween_with_repeat() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear).with_repeat(Repeat::Times(1));
    tw.set_target(100.0);
    tw.tick(0.5);
    assert!((tw.get() - 50.0).abs() < 2.0);
    tw.tick(0.7);
    assert!(tw.is_animating(), "Should still be in second cycle");
    tw.tick(1.0);
    assert_eq!(tw.state(), TweenState::Completed);
}

#[test]
fn tween_infinite_repeat() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear).with_repeat(Repeat::Infinite);
    tw.set_target(100.0);
    for _ in 0..1000 {
        tw.tick(0.016);
    }
    assert!(tw.is_animating(), "Infinite repeat should never complete");
}

#[test]
fn tween_with_reverse() {
    let mut tw = Tween::new(0.0_f32, 1.0, Easing::Linear).with_reverse(true);
    tw.set_target(100.0);
    tw.tick(0.5);
    let v1 = tw.get();
    assert!(
        (v1 - 50.0).abs() < 2.0,
        "Forward half: expected ~50, got {}",
        v1
    );
    tw.tick(0.5);
    let v2 = tw.get();
    assert!(
        (v2 - 100.0).abs() < 2.0,
        "At peak: expected ~100, got {}",
        v2
    );
    tw.tick(0.5);
    let v3 = tw.get();
    assert!(
        (v3 - 50.0).abs() < 2.0,
        "Reverse half: expected ~50, got {}",
        v3
    );
    tw.tick(0.5);
    assert_eq!(tw.state(), TweenState::Completed);
    let v4 = tw.get();
    assert!(
        (v4 - 0.0).abs() < 0.01,
        "Completed reverse: expected from (0), got {}",
        v4
    );
}

#[test]
fn tween_from_motion_uses_settling_time() {
    use crate::theme::Theme;
    let theme = Theme::dark();
    let mut tw = Tween::from_motion(0.0_f32, &theme.motion, Easing::EaseInOut);
    let expected = theme.motion.settling_time().clamp(0.1, 2.0);
    assert!(!tw.is_animating()); // starts idle
    tw.set_target(100.0);
    assert!(tw.is_animating());
    // Tick past expected duration -> should complete
    tw.tick(expected + 0.01);
    assert_eq!(tw.state(), TweenState::Completed);
}

#[test]
fn tween_from_motion_destructive_faster() {
    use crate::theme::{Intent, Theme};
    let theme = Theme::dark();
    let neutral = Tween::from_motion(0.0_f32, &theme.motion, Easing::EaseInOut);
    let destructive = Tween::from_motion(
        0.0_f32,
        &theme.intent_motion(Intent::Destructive),
        Easing::EaseInOut,
    );
    // Destructive intent settles faster -> shorter tween duration
    // We verify by ticking both to completion
    let mut n = neutral;
    n.set_target(100.0);
    let mut d = destructive;
    d.set_target(100.0);
    // Tick destructive for neutral's settling time - it should complete first
    let neutral_time = theme.motion.settling_time().clamp(0.1, 2.0);
    let dest_time = theme
        .intent_motion(Intent::Destructive)
        .settling_time()
        .clamp(0.1, 2.0);
    assert!(
        dest_time < neutral_time,
        "destructive={}s should be < neutral={}s",
        dest_time,
        neutral_time
    );
}
