use super::*;

fn assert_endpoints(easing: Easing) {
    let v0 = ease(0.0, easing);
    let v1 = ease(1.0, easing);
    assert!(
        (v0 - 0.0).abs() < 0.01,
        "{easing:?}: ease(0) = {v0} (expected ~0)"
    );
    assert!(
        (v1 - 1.0).abs() < 0.01,
        "{easing:?}: ease(1) = {v1} (expected ~1)"
    );
}

#[test]
fn easing_linear() {
    assert_endpoints(Easing::Linear);
    assert!((ease(0.5, Easing::Linear) - 0.5).abs() < 0.001);
}

#[test]
fn easing_quad_endpoints() {
    assert_endpoints(Easing::EaseIn);
    assert_endpoints(Easing::EaseOut);
    assert_endpoints(Easing::EaseInOut);
}

#[test]
fn easing_cubic_endpoints() {
    assert_endpoints(Easing::EaseInCubic);
    assert_endpoints(Easing::EaseOutCubic);
    assert_endpoints(Easing::EaseInOutCubic);
}

#[test]
fn easing_quart_endpoints() {
    assert_endpoints(Easing::EaseInQuart);
    assert_endpoints(Easing::EaseOutQuart);
    assert_endpoints(Easing::EaseInOutQuart);
}

#[test]
fn easing_quint_endpoints() {
    assert_endpoints(Easing::EaseInQuint);
    assert_endpoints(Easing::EaseOutQuint);
    assert_endpoints(Easing::EaseInOutQuint);
}

#[test]
fn easing_sine_endpoints() {
    assert_endpoints(Easing::EaseInSine);
    assert_endpoints(Easing::EaseOutSine);
    assert_endpoints(Easing::EaseInOutSine);
}

#[test]
fn easing_expo_endpoints() {
    assert_endpoints(Easing::EaseInExpo);
    assert_endpoints(Easing::EaseOutExpo);
    assert_endpoints(Easing::EaseInOutExpo);
}

#[test]
fn easing_circ_endpoints() {
    assert_endpoints(Easing::EaseInCirc);
    assert_endpoints(Easing::EaseOutCirc);
    assert_endpoints(Easing::EaseInOutCirc);
}

#[test]
fn easing_back_endpoints() {
    assert_endpoints(Easing::EaseInBack);
    assert_endpoints(Easing::EaseOutBack);
    assert_endpoints(Easing::EaseInOutBack);
}

#[test]
fn easing_elastic_endpoints() {
    assert_endpoints(Easing::EaseInElastic);
    assert_endpoints(Easing::EaseOutElastic);
    assert_endpoints(Easing::EaseInOutElastic);
}

#[test]
fn easing_bounce_endpoints() {
    assert_endpoints(Easing::EaseInBounce);
    assert_endpoints(Easing::EaseOutBounce);
    assert_endpoints(Easing::EaseInOutBounce);
}

#[test]
fn easing_cubic_bezier_linear() {
    let cb = Easing::CubicBezier(0.0, 0.0, 1.0, 1.0);
    assert_endpoints(cb);
    assert!((ease(0.5, cb) - 0.5).abs() < 0.02);
}

#[test]
fn easing_clamps_input() {
    assert_eq!(ease(-1.0, Easing::Linear), 0.0);
    assert_eq!(ease(2.0, Easing::Linear), 1.0);
}

#[test]
fn ease_in_slower_at_start() {
    let v = ease(0.25, Easing::EaseIn);
    assert!(v < 0.25, "EaseIn at 0.25 should be < 0.25, got {v}");
}

#[test]
fn ease_out_faster_at_start() {
    let v = ease(0.25, Easing::EaseOut);
    assert!(v > 0.25, "EaseOut at 0.25 should be > 0.25, got {v}");
}

#[test]
fn easing_step() {
    assert!((ease(0.0, Easing::Step) - 0.0).abs() < 0.01);
    assert!((ease(0.49, Easing::Step) - 0.0).abs() < 0.01);
    assert!((ease(0.5, Easing::Step) - 1.0).abs() < 0.01);
    assert!((ease(1.0, Easing::Step) - 1.0).abs() < 0.01);
}

#[test]
fn easing_hold() {
    assert!((ease(0.0, Easing::Hold)).abs() < 0.01);
    assert!((ease(0.5, Easing::Hold)).abs() < 0.01);
    assert!((ease(0.99, Easing::Hold)).abs() < 0.01);
    assert!((ease(1.0, Easing::Hold) - 1.0).abs() < 0.01);
}
