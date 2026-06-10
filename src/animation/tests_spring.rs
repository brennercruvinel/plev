use super::*;

#[test]
fn spring_at_rest_by_default() {
    let s: Spring<f32> = Spring::new(0.0);
    assert!(!s.is_animating());
    assert!((s.get() - 0.0).abs() < 0.001);
}

#[test]
fn spring_set_target_starts_animation() {
    let mut s: Spring<f32> = Spring::new(0.0);
    s.set_target(100.0);
    assert!(s.is_animating());
}

#[test]
fn spring_converges() {
    let mut s: Spring<f32> = Spring::new(0.0);
    s.stiffness = 200.0;
    s.damping = 20.0;
    s.set_target(100.0);
    for _ in 0..300 {
        s.tick(1.0 / 60.0);
    }
    assert!(!s.is_animating(), "Spring should be at rest after 5s");
    assert!(
        (s.get() - 100.0).abs() < 0.1,
        "Spring should converge to target, got {}",
        s.get()
    );
}

#[test]
fn spring_with_config() {
    let s: Spring<f32> = Spring::new(0.0).with_config(200.0, 15.0, 1.0);
    assert_eq!(s.stiffness, 200.0);
    assert_eq!(s.damping, 15.0);
    assert_eq!(s.mass, 1.0);
}

#[test]
fn spring_overshoots_with_low_damping() {
    let mut s: Spring<f32> = Spring::new(0.0).with_config(200.0, 2.0, 1.0);
    s.set_target(100.0);
    let mut max_val = 0.0_f32;
    for _ in 0..300 {
        s.tick(1.0 / 60.0);
        max_val = max_val.max(s.get());
    }
    assert!(
        max_val > 100.0,
        "Under-damped spring should overshoot, max was {}",
        max_val
    );
}

#[test]
fn spring_2d() {
    let mut s: Spring<[f32; 2]> = Spring::new([0.0, 0.0]);
    s.set_target([100.0, 200.0]);
    for _ in 0..300 {
        s.tick(1.0 / 60.0);
    }
    let v = s.get();
    assert!((v[0] - 100.0).abs() < 0.5);
    assert!((v[1] - 200.0).abs() < 0.5);
}

#[test]
fn spring_frame_rate_independence() {
    let stiffness = 200.0;
    let damping = 20.0;
    let mass = 1.0;
    let target = 100.0;

    let mut s30: Spring<f32> = Spring::new(0.0).with_config(stiffness, damping, mass);
    s30.set_target(target);
    for _ in 0..150 {
        s30.tick(1.0 / 30.0);
    }

    let mut s60: Spring<f32> = Spring::new(0.0).with_config(stiffness, damping, mass);
    s60.set_target(target);
    for _ in 0..300 {
        s60.tick(1.0 / 60.0);
    }

    let mut s120: Spring<f32> = Spring::new(0.0).with_config(stiffness, damping, mass);
    s120.set_target(target);
    for _ in 0..600 {
        s120.tick(1.0 / 120.0);
    }

    let v30 = s30.get();
    let v60 = s60.get();
    let v120 = s120.get();
    assert!(
        (v30 - v60).abs() < 0.5,
        "30fps vs 60fps should match: {} vs {}",
        v30,
        v60
    );
    assert!(
        (v60 - v120).abs() < 0.5,
        "60fps vs 120fps should match: {} vs {}",
        v60,
        v120
    );
}

#[test]
fn spring_high_stiffness_stable() {
    let mut s: Spring<f32> = Spring::new(0.0).with_config(10000.0, 100.0, 1.0);
    s.set_target(100.0);
    for _ in 0..600 {
        s.tick(1.0 / 60.0);
        let v = s.get();
        assert!(v.is_finite(), "Spring value should be finite, got {}", v);
        assert!(
            v.abs() < 10000.0,
            "Spring value should not diverge, got {}",
            v
        );
    }
    assert!(
        (s.get() - 100.0).abs() < 1.0,
        "High stiffness should converge to target"
    );
}

#[test]
fn spring_damping_ratio() {
    let s: Spring<f32> = Spring::new(0.0).with_config(100.0, 20.0, 1.0);
    let ratio = s.damping_ratio();
    assert!(
        (ratio - 1.0).abs() < 0.01,
        "Expected critical damping (ratio=1.0), got {}",
        ratio
    );

    let s2: Spring<f32> = Spring::new(0.0).with_config(100.0, 10.0, 1.0);
    assert!(s2.damping_ratio() < 1.0, "Under-damped expected");

    let s3: Spring<f32> = Spring::new(0.0).with_config(100.0, 30.0, 1.0);
    assert!(s3.damping_ratio() > 1.0, "Over-damped expected");
}

#[test]
fn spring_critically_damped_no_overshoot() {
    let mut s: Spring<f32> = Spring::new(0.0).with_config(100.0, 20.0, 1.0);
    s.set_target(100.0);
    let mut max_val = 0.0_f32;
    for _ in 0..300 {
        s.tick(1.0 / 60.0);
        max_val = max_val.max(s.get());
    }
    assert!(
        max_val <= 100.5,
        "Critically damped should not significantly overshoot, max was {}",
        max_val
    );
}

#[test]
fn spring_with_motion_configures() {
    use crate::theme::MotionPhysics;
    let motion = MotionPhysics {
        mass: 1.5,
        stiffness: 200.0,
        damping: 25.0,
    };
    let s: Spring<f32> = Spring::new(0.0).with_motion(&motion);
    assert_eq!(s.stiffness, 200.0);
    assert_eq!(s.damping, 25.0);
    assert_eq!(s.mass, 1.5);
}

#[test]
fn spring_with_motion_from_theme() {
    use crate::theme::Theme;
    let theme = Theme::dark();
    let s: Spring<f32> = Spring::new(0.0).with_motion(&theme.motion);
    assert_eq!(s.stiffness, theme.motion.stiffness);
    assert_eq!(s.damping, theme.motion.damping);
    assert_eq!(s.mass, theme.motion.mass);
}

#[test]
fn spring_with_intent_motion_destructive_faster() {
    use crate::theme::{Intent, Theme};
    let theme = Theme::dark();
    let neutral: Spring<f32> = Spring::new(0.0).with_motion(&theme.motion);
    let destructive: Spring<f32> =
        Spring::new(0.0).with_motion(&theme.intent_motion(Intent::Destructive));
    // Destructive has higher stiffness = faster response
    assert!(destructive.stiffness > neutral.stiffness);
    assert!(destructive.mass < neutral.mass);
}

#[test]
fn spring_with_motion_converges() {
    use crate::theme::Theme;
    let theme = Theme::dark();
    let mut s: Spring<f32> = Spring::new(0.0).with_motion(&theme.motion);
    s.set_target(100.0);
    for _ in 0..300 {
        s.tick(1.0 / 60.0);
    }
    assert!(!s.is_animating());
    assert!((s.get() - 100.0).abs() < 0.1);
}
