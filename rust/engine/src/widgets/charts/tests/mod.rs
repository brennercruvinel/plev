//! Unit tests of the charts.

#[cfg(test)]
pub(super) mod donut;

use super::*;

fn style() -> TextStyle {
    TextStyle::new(12.0).with_weight(500)
}

#[test]
fn measured_label_has_real_extent_and_carries_its_style() {
    let label = Label::measured("1.000", style());
    assert!(label.w > 0.0 && label.h > 0.0);
    // the exact style used to measure is the one the draw site gets.
    assert_eq!(label.style, style());
    // wider text measures wider (real shaping, not a heuristic).
    let wider = Label::measured("1.000.000", style());
    assert!(wider.w > label.w);
}

#[test]
fn centered_and_right_aligned_anchor_the_measured_box() {
    let c = Label::measured("42", style()).centered_on(100.0, 50.0);
    let b = c.bounds();
    assert!((b.x + b.w / 2.0 - 100.0).abs() < 0.01);
    assert!((b.y + b.h / 2.0 - 50.0).abs() < 0.01);

    let r = Label::measured("42", style()).right_aligned(80.0, 20.0);
    assert!((r.x + r.w - 80.0).abs() < 0.01);
}

#[test]
fn drop_colliding_keeps_first_and_leaves_no_overlap() {
    // labels stacked nearly on top of each other: only spaced survivors.
    let labels: Vec<Label> = (0..10)
        .map(|i| Label::measured("ovrlap", style()).at(0.0, i as f32 * 4.0))
        .collect();
    let kept = drop_colliding(labels, 2.0);
    assert!(!kept.is_empty(), "first label always survives");
    assert!(kept.len() < 10, "dense stack must be decimated");
    for (i, a) in kept.iter().enumerate() {
        for b in kept.iter().skip(i + 1) {
            assert!(
                !rects_overlap(&a.bounds(), &b.bounds(), 2.0),
                "kept labels must not collide"
            );
        }
    }
}

#[test]
fn drop_colliding_keeps_everything_when_spaced() {
    let labels: Vec<Label> = (0..5)
        .map(|i| Label::measured("ok", style()).at(0.0, i as f32 * 40.0))
        .collect();
    assert_eq!(drop_colliding(labels, 2.0).len(), 5);
}
