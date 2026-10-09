//! Reduced motion (task-75 phase 1): with the thread's preference on,
//! every transition ends on its first tick; with it off, nothing changes.

use super::super::*;

/// The preference on for one test, off again when it ends (a test runs
/// on a thread of its own; the guard keeps it so even if one did not).
struct Reduced;

impl Reduced {
    fn on() -> Self {
        set_reduced_motion(true);
        Reduced
    }
}

impl Drop for Reduced {
    fn drop(&mut self) {
        set_reduced_motion(false);
    }
}

#[test]
fn a_thread_starts_with_motion() {
    assert!(!reduced_motion());
    let other = std::thread::spawn(|| {
        set_reduced_motion(true);
        reduced_motion()
    });
    assert_eq!(other.join().ok(), Some(true));
    assert!(!reduced_motion(), "another thread's preference is its own");
}

#[test]
fn a_spring_lands_on_its_target_on_the_first_frame() {
    let _reduced = Reduced::on();
    let mut s: Spring<f32> = Spring::new(0.0).with_config(170.0, 26.0, 1.0);
    s.set_target(1.0);
    assert!(s.is_animating(), "asked to move");
    s.tick(1.0 / 60.0);
    assert!(!s.is_animating(), "at rest after one frame");
    assert_eq!(s.get(), 1.0);
    // a step of no time lands it too: the first tick of a headless frame
    s.set_target(0.0);
    s.tick(0.0);
    assert_eq!(s.get(), 0.0);
    assert!(!s.is_animating());
}

#[test]
fn with_motion_the_same_spring_is_still_moving_after_one_frame() {
    let mut s: Spring<f32> = Spring::new(0.0).with_config(170.0, 26.0, 1.0);
    s.set_target(1.0);
    s.tick(1.0 / 60.0);
    assert!(s.is_animating());
    assert!(s.get() > 0.0 && s.get() < 1.0, "{}", s.get());
}

#[test]
fn a_tween_ends_on_the_first_frame_and_a_loop_runs_on() {
    let _reduced = Reduced::on();
    let mut fade = Tween::new(0.0_f32, 0.3, Easing::EaseInOut).with_delay(0.5);
    fade.set_target(1.0);
    fade.tick(1.0 / 60.0);
    assert_eq!(fade.state(), TweenState::Completed);
    assert_eq!(fade.get(), 1.0);
    assert!(!fade.is_animating());

    let mut twice = Tween::new(0.0_f32, 0.3, Easing::Linear)
        .with_repeat(Repeat::Times(2))
        .with_reverse(true);
    twice.set_target(1.0);
    twice.tick(1.0 / 60.0);
    assert_eq!(twice.state(), TweenState::Completed, "a count has an end");
    assert_eq!(twice.get(), 0.0, "a reversing tween ends where it began");

    let mut pulse = Tween::new(0.0_f32, 1.0, Easing::Linear).with_repeat(Repeat::Infinite);
    pulse.set_target(1.0);
    pulse.tick(0.25);
    assert!(pulse.is_animating(), "no end to reach");
    assert!((pulse.get() - 0.25).abs() < 1e-6);
}

#[test]
fn with_motion_the_same_tween_is_still_running_after_one_frame() {
    let mut fade = Tween::new(0.0_f32, 0.3, Easing::EaseInOut);
    fade.set_target(1.0);
    fade.tick(1.0 / 60.0);
    assert_eq!(fade.state(), TweenState::Running);
    assert!(fade.get() < 1.0);
}
