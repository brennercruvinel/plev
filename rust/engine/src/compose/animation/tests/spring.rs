use super::super::*;

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
        "Under-damped spring should overshoot, max was {max_val}"
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
        "30fps vs 60fps should match: {v30} vs {v60}"
    );
    assert!(
        (v60 - v120).abs() < 0.5,
        "60fps vs 120fps should match: {v60} vs {v120}"
    );
}

#[test]
fn spring_high_stiffness_stable() {
    let mut s: Spring<f32> = Spring::new(0.0).with_config(10000.0, 100.0, 1.0);
    s.set_target(100.0);
    for _ in 0..600 {
        s.tick(1.0 / 60.0);
        let v = s.get();
        assert!(v.is_finite(), "Spring value should be finite, got {v}");
        assert!(
            v.abs() < 10000.0,
            "Spring value should not diverge, got {v}"
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
        "Expected critical damping (ratio=1.0), got {ratio}"
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
        "Critically damped should not significantly overshoot, max was {max_val}"
    );
}

#[test]
fn spring_with_motion_configures() {
    use crate::styling::theme::MotionPhysics;
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
    use crate::styling::theme::Theme;
    let theme = Theme::dark();
    let s: Spring<f32> = Spring::new(0.0).with_motion(&theme.motion);
    assert_eq!(s.stiffness, theme.motion.stiffness);
    assert_eq!(s.damping, theme.motion.damping);
    assert_eq!(s.mass, theme.motion.mass);
}

#[test]
fn spring_with_intent_motion_destructive_faster() {
    use crate::styling::theme::{Intent, Theme};
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
    use crate::styling::theme::Theme;
    let theme = Theme::dark();
    let mut s: Spring<f32> = Spring::new(0.0).with_motion(&theme.motion);
    s.set_target(100.0);
    for _ in 0..300 {
        s.tick(1.0 / 60.0);
    }
    assert!(!s.is_animating());
    assert!((s.get() - 100.0).abs() < 0.1);
}

/// Ticks a comparison runs at most: 25 s at 120 Hz, past the slowest
/// config's rest.
const LIMIT: usize = 3000;

/// The stepping the spring had before it kept its displacement (task-104
/// front I): the absolute value, rounded to the target's scale each tick.
/// The reference the uses of today are held to.
fn absolute_steps(from: f32, to: f32, (k, d, m): (f32, f32, f32), dt: f32) -> Vec<f32> {
    let (mut value, mut velocity) = (from, 0.0_f32);
    let mut out = Vec::new();
    for _ in 0..LIMIT {
        let omega_0_sq = k / m;
        let gamma = d / (2.0 * m);
        let disc = gamma * gamma - omega_0_sq;
        let x = value - to;
        let (pp, pv, vp, vv) = if disc < -1e-6 {
            let wd = (-disc).sqrt();
            let e = (-gamma * dt).exp();
            let (c, s) = ((wd * dt).cos(), (wd * dt).sin());
            (
                e * (c + gamma / wd * s),
                e * s / wd,
                -e * omega_0_sq * s / wd,
                e * (c - gamma / wd * s),
            )
        } else if disc > 1e-6 {
            let sq = disc.sqrt();
            let (r1, r2) = (-gamma + sq, -gamma - sq);
            let (e1, e2) = ((r1 * dt).exp(), (r2 * dt).exp());
            let inv = 1.0 / (r1 - r2);
            (
                (r1 * e2 - r2 * e1) * inv,
                (e1 - e2) * inv,
                r1 * r2 * (e2 - e1) * inv,
                (r1 * e1 - r2 * e2) * inv,
            )
        } else {
            let e = (-gamma * dt).exp();
            (
                e * (1.0 + gamma * dt),
                e * dt,
                -e * gamma * gamma * dt,
                e * (1.0 - gamma * dt),
            )
        };
        let nd = x * pp + velocity * pv;
        velocity = x * vp + velocity * vv;
        value = to + nd;
        if nd * nd < 1e-6 && velocity * velocity < 1e-6 {
            out.push(to);
            return out;
        }
        out.push(value);
    }
    out
}

/// The uses of today (positions and sizes up to a thousand, the theme's
/// motion and the widgets' own configs, at 60 and 120 Hz) move as they
/// did, within a thousandth, tick by tick; where they come to rest moves
/// only within a hundredth of a pixel of the target, where the old
/// stepping rounded on (and where it never came to rest, now it does).
#[test]
fn the_uses_of_today_move_as_they_did() {
    let configs = [
        (170.0, 26.0, 1.0),
        (100.0, 10.0, 1.0),
        (200.0, 2.0, 1.0),
        (300.0, 30.0, 1.0),
        (400.0, 40.0, 2.0),
    ];
    for cfg in configs {
        for (from, to) in [(0.0, 1.0), (0.0, 100.0), (240.0, 12.0), (0.0, 999.0)] {
            for dt in [1.0 / 60.0, 1.0 / 120.0] {
                let before = absolute_steps(from, to, cfg, dt);
                let mut s: Spring<f32> = Spring::new(from).with_config(cfg.0, cfg.1, cfg.2);
                s.set_target(to);
                let mut now = Vec::new();
                while s.is_animating() && now.len() < LIMIT {
                    s.tick(dt);
                    now.push(s.get());
                }
                let at = |v: &[f32]| v.len();
                assert!(now.len() < LIMIT, "{cfg:?} {from}->{to} at {dt}: rests now");
                // the rest tick moves only within a hundredth of a pixel
                // of the target, where the old stepping rounded on (or
                // never rested at all: 0 to 999 at 120 Hz with the theme's
                // motion, the bug this fixes)
                let first = at(&before).min(at(&now));
                for (i, v) in before.iter().chain(&now).enumerate() {
                    let tick = if i < at(&before) { i } else { i - at(&before) };
                    assert!(
                        tick < first || (v - to).abs() < 0.01,
                        "{cfg:?} {from}->{to} at {dt} tick {tick}: {v} still moving \
                         ({} ticks then, {} now)",
                        at(&before),
                        at(&now)
                    );
                }
                for (i, (a, b)) in before.iter().zip(&now).enumerate() {
                    assert!(
                        (a - b).abs() < 1e-3,
                        "{cfg:?} {from}->{to} tick {i}: {a} vs {b}"
                    );
                }
            }
        }
    }
}

/// Thousands and millions of pixels down (a long page, a long document),
/// the spring still comes to rest, on its target.
#[test]
fn a_spring_far_from_zero_comes_to_rest_on_its_target() {
    for (from, to) in [(167.0, 2349.0), (5_000.0, 9_000.0), (2.0e6, 2.000_5e6)] {
        let mut s: Spring<f32> = Spring::new(from).with_config(170.0, 26.0, 1.0);
        s.set_target(to);
        let mut ticks = 0;
        while s.is_animating() {
            s.tick(1.0 / 60.0);
            ticks += 1;
            assert!(ticks < 180, "{from}->{to}: still moving after 3 s");
        }
        assert_eq!(s.get(), to, "{from}->{to}");
    }
}
