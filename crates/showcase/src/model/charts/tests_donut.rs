//! tests for the donut geometry: full-turn closure, measured center label,
//! legend layout, percent-label fitting and the slice tessellation helper.

use std::f32::consts::{FRAC_PI_2, TAU};

use plev::text::TextStyle;
use plev::ui::widgets::Rect;

use super::{donut, rects_overlap, slice_polygon};

fn center_style() -> TextStyle {
    TextStyle::new(20.0).with_weight(600)
}

fn label_style() -> TextStyle {
    TextStyle::new(12.0).with_weight(500)
}

fn demo_items() -> Vec<(&'static str, f32)> {
    vec![
        ("alpha", 35.0),
        ("beta", 25.0),
        ("gamma", 20.0),
        ("delta", 12.0),
        ("epsilon", 8.0),
    ]
}

#[test]
fn slices_close_exactly_to_a_full_turn() {
    let d = donut(
        &demo_items(),
        Rect::new(0.0, 0.0, 600.0, 300.0),
        "100",
        center_style(),
        label_style(),
    );
    assert_eq!(d.slices.len(), 5);
    let sweep_sum: f32 = d.slices.iter().map(|s| s.sweep).sum();
    assert!(
        (sweep_sum - TAU).abs() < 1e-5,
        "donut must close 360 degrees"
    );
    let last = d.slices.last().unwrap();
    assert!(
        (last.start + last.sweep - (-FRAC_PI_2 + TAU)).abs() < 1e-5,
        "last slice must land back on the start angle"
    );
    let fraction_sum: f32 = d.slices.iter().map(|s| s.fraction).sum();
    assert!((fraction_sum - 1.0).abs() < 1e-5);
    // slices are contiguous: each starts where the previous ended.
    for pair in d.slices.windows(2) {
        assert!((pair[0].start + pair[0].sweep - pair[1].start).abs() < 1e-5);
    }
}

#[test]
fn center_label_is_measured_centered_and_keeps_its_weight() {
    let d = donut(
        &demo_items(),
        Rect::new(20.0, 10.0, 500.0, 280.0),
        "total 100",
        center_style(),
        label_style(),
    );
    let b = d.center_label.bounds();
    assert!((b.x + b.w / 2.0 - d.center.0).abs() < 0.01);
    assert!((b.y + b.h / 2.0 - d.center.1).abs() < 0.01);
    assert!(b.w > 0.0, "real measured width, not zero");
    // the style that measured the text is the one that will draw it.
    assert_eq!(d.center_label.style.font_weight, 600);
    assert!(d.inner_r > 0.0 && d.inner_r < d.outer_r);
}

#[test]
fn legend_sits_right_of_the_ring_with_non_overlapping_rows() {
    let rect = Rect::new(0.0, 0.0, 600.0, 300.0);
    let d = donut(&demo_items(), rect, "100", center_style(), label_style());
    assert_eq!(d.legend.len(), 5);
    for item in &d.legend {
        assert!(
            item.swatch.x >= d.center.0 + d.outer_r,
            "legend must clear the ring"
        );
        assert!(item.label.x >= item.swatch.x + item.swatch.w);
        assert!(item.label.x + item.label.w <= rect.x + rect.w + 0.01);
    }
    for (i, a) in d.legend.iter().enumerate() {
        for b in d.legend.iter().skip(i + 1) {
            assert!(
                !rects_overlap(&a.label.bounds(), &b.label.bounds(), 0.0),
                "legend rows must not overlap"
            );
        }
    }
}

#[test]
fn percent_labels_fit_in_roomy_rects_and_drop_in_tight_ones() {
    let roomy = donut(
        &demo_items(),
        Rect::new(0.0, 0.0, 600.0, 300.0),
        "100",
        center_style(),
        label_style(),
    );
    assert!(
        roomy.slices.iter().all(|s| s.percent.is_some()),
        "every share fits a 300px-tall ring"
    );
    let kept: Vec<_> = roomy
        .slices
        .iter()
        .filter_map(|s| s.percent.as_ref())
        .collect();
    for (i, a) in kept.iter().enumerate() {
        for b in kept.iter().skip(i + 1) {
            assert!(
                !rects_overlap(&a.bounds(), &b.bounds(), 0.0),
                "percent labels '{}' and '{}' overlap",
                a.text,
                b.text
            );
        }
    }

    let tight = donut(
        &demo_items(),
        Rect::new(0.0, 0.0, 140.0, 80.0),
        "100",
        center_style(),
        label_style(),
    );
    assert!(
        tight.slices.iter().any(|s| s.percent.is_none()),
        "a narrow rect must drop labels that cannot fit"
    );
    let sweep_sum: f32 = tight.slices.iter().map(|s| s.sweep).sum();
    assert!((sweep_sum - TAU).abs() < 1e-5, "closure holds at any size");
}

#[test]
fn zero_negative_and_nan_values_get_no_slice_but_keep_their_legend_row() {
    let items = vec![
        ("ok", 10.0),
        ("zero", 0.0),
        ("neg", -5.0),
        ("nan", f32::NAN),
    ];
    let d = donut(
        &items,
        Rect::new(0.0, 0.0, 400.0, 200.0),
        "10",
        center_style(),
        label_style(),
    );
    assert_eq!(d.slices.len(), 1, "only the positive value earns an angle");
    assert_eq!(d.slices[0].index, 0);
    assert!((d.slices[0].sweep - TAU).abs() < 1e-5);
    assert_eq!(d.legend.len(), 4);

    let nothing = donut(
        &[("a", 0.0), ("b", 0.0)],
        Rect::new(0.0, 0.0, 400.0, 200.0),
        "0",
        center_style(),
        label_style(),
    );
    assert!(nothing.slices.is_empty(), "zero total, zero slices");
    assert_eq!(nothing.legend.len(), 2);
}

#[test]
fn slice_polygon_tessellates_between_the_radii() {
    let center = (50.0, 50.0);
    let poly = slice_polygon(center, 30.0, 60.0, 0.0, FRAC_PI_2, 8);
    assert_eq!(poly.len(), 18, "outer and inner arcs, 9 points each");
    for &(x, y) in &poly {
        let r = ((x - center.0).powi(2) + (y - center.1).powi(2)).sqrt();
        assert!(
            (29.99..=60.01).contains(&r),
            "vertex radius {r} out of ring"
        );
    }
    // a pie wedge (inner radius zero) closes through the center.
    let wedge = slice_polygon(center, 0.0, 60.0, 0.0, FRAC_PI_2, 8);
    assert_eq!(*wedge.last().unwrap(), center);
}
