//! The person's text size (ADR 0086): the curve keeps every size at the
//! identity, grows each with the factor and never shrinks one; the
//! adapted theme is the base one at the default preferences, and its
//! controls grow by the line their label grew.

use crate::drawing::text::TextMeasurer;
use crate::styling::theme::{ControlSize, Prefs, TextScale, Theme, TypeRole, TypographyScale};
use crate::widgets::buttons::{Button, ButtonSize};

fn themes() -> Vec<(&'static str, Theme)> {
    let mut out = vec![
        ("core", Theme::hoff_core()),
        ("hoff", Theme::hoff()),
        ("dark", Theme::dark()),
        ("light", Theme::light()),
    ];
    for name in ["nord", "dracula", "moonlight"] {
        out.push((name, Theme::named(name).expect("built in")));
    }
    out
}

const SIZES: [f32; 12] = [
    6.0, 10.0, 11.0, 12.0, 13.0, 15.0, 18.0, 20.0, 32.0, 48.0, 56.0, 120.0,
];

#[test]
fn the_identity_draws_every_size_as_designed() {
    let s = TextScale::IDENTITY;
    for size in SIZES {
        assert_eq!(s.size(size), size);
        assert_eq!(s.line(size, size * 1.4), size * 1.4);
        assert_eq!(s.metric(16.0, size), 16.0);
    }
    assert_eq!(TextScale::default(), TextScale::IDENTITY);
}

#[test]
fn every_factor_grows_sizes_in_order_and_never_shrinks_one() {
    for f in [1.1, 1.15, 1.25, 1.5, 1.7, 2.0] {
        for (curve, s) in [
            ("tables", TextScale::new(f)),
            ("linear", TextScale::linear(f)),
        ] {
            let mut last = 0.0;
            for size in SIZES {
                let out = s.size(size);
                assert!(out >= size, "{curve} {f}: {size} -> {out}");
                assert!(
                    out <= size * TextScale::CEILING + 1e-3,
                    "{curve} {f}: {size} -> {out}"
                );
                assert!(out > last, "{curve} {f}: {size} -> {out} after {last}");
                last = out;
            }
        }
    }
}

#[test]
fn a_sampled_curve_is_held_under_the_ceiling_and_in_order() {
    let s = TextScale::from_points(&[[10.0, 30.0], [8.0, 9.0], [20.0, 30.0], [40.0, 40.0]]);
    assert_eq!(s.points(), &[[10.0, 20.0], [20.0, 30.0], [40.0, 40.0]]);
    assert_eq!(s.size(15.0), 25.0);
    assert_eq!(s.size(80.0), 80.0, "past the last point, its ratio");
    assert!(TextScale::from_points(&[]).is_identity());
    assert!(TextScale::from_points(&[[12.0, 12.0], [20.0, 20.0]]).is_identity());
    assert_eq!(TextScale::linear(1.5).points().len(), 2);
}

#[test]
fn the_tables_grow_small_text_by_the_factor_and_titles_by_less() {
    let s = TextScale::new(2.0);
    assert_eq!(s.size(12.0), 24.0, "Android 14 at 200%: 12 -> 24");
    assert_eq!(s.size(30.0), 38.0, "30 -> 38");
    assert_eq!(s.size(100.0), 100.0);
    let half = TextScale::new(1.5);
    assert_eq!(half.size(14.0), 22.0);
    // between two tables, the straight line between them
    let between = TextScale::new(1.4).size(10.0);
    assert!((between - 14.0).abs() < 1e-4, "{between}");
    assert_eq!(TextScale::linear(2.0).size(48.0), 96.0);
}

#[test]
fn the_factor_is_held_between_one_and_the_ceiling() {
    assert_eq!(TextScale::new(3.0).factor(), TextScale::CEILING);
    assert_eq!(TextScale::new(0.8).factor(), 1.0);
    assert_eq!(TextScale::new(f32::NAN).factor(), 1.0);
    assert!(TextScale::new(0.5).is_identity());
}

#[test]
fn the_line_follows_the_size_by_the_same_ratio() {
    let s = TextScale::new(1.5);
    for (size, lh) in [(12.0, 16.0), (14.0, 20.0), (48.0, 56.0)] {
        let ratio = s.size(size) / size;
        assert!((s.line(size, lh) - lh * ratio).abs() < 1e-4);
    }
}

#[test]
fn the_default_preferences_adapt_nothing() {
    for (name, base) in themes() {
        let t = base.adapted(&Prefs::default());
        assert_eq!(t.typography, base.typography, "{name}");
        assert_eq!(t.control, base.control, "{name}");
        assert_eq!(t.button, base.button, "{name}");
        assert_eq!(t.form, base.form, "{name}");
        assert_eq!(t.overlay, base.overlay, "{name}");
        assert_eq!(t.selection, base.selection, "{name}");
        assert_eq!(t.navigation, base.navigation, "{name}");
        assert_eq!(t.disclosure, base.disclosure, "{name}");
        assert_eq!(t.message, base.message, "{name}");
        assert_eq!(t.header, base.header, "{name}");
    }
}

fn larger(theme: &Theme, f: f32) -> Theme {
    theme.adapted(&Prefs::default().with_text_scale(TextScale::new(f)))
}

#[test]
fn every_role_draws_larger_and_keeps_its_order() {
    for (name, base) in themes() {
        let big = larger(&base, 2.0).typography;
        let t = &base.typography;
        for role in TypeRole::ALL {
            let (a, b) = (t.role(role), big.role(role));
            assert!(b.font_size > a.font_size, "{name} {role:?}");
            assert!(b.line_height >= a.line_height, "{name} {role:?}");
            assert!(
                b.line_height >= b.font_size,
                "{name} {role:?}: the line holds the size"
            );
        }
        // after the fluid roles took their width
        let narrow = big.role_at(TypeRole::H1, 360.0);
        assert!(
            narrow.font_size < big.role(TypeRole::H1).font_size,
            "{name}"
        );
        assert!(
            narrow.font_size > t.role_at(TypeRole::H1, 360.0).font_size,
            "{name}"
        );
    }
}

#[test]
fn ladders_grow_by_the_line_and_keep_their_padding() {
    for (name, base) in themes() {
        let big = larger(&base, 2.0);
        for size in ControlSize::ALL {
            assert!(
                big.control.height(size) > base.control.height(size),
                "{name}"
            );
        }
        for (b, m) in big.button.sizes.iter().zip(base.button.sizes.iter()) {
            let grew =
                big.typography.run(m.text).line_height - base.typography.run(m.text).line_height;
            assert_eq!(b.height, (m.height + grew).ceil(), "{name}");
            assert_eq!(b.pad_x, m.pad_x, "{name}: padding never scales");
        }
        assert!(big.overlay.menu.item_h > base.overlay.menu.item_h, "{name}");
        assert!(big.form.select.panel.row_h > base.form.select.panel.row_h);
        assert!(big.disclosure.tree.row_h > base.disclosure.tree.row_h);
        assert!(big.navigation.sidebar_link.height > base.navigation.sidebar_link.height);
        assert_eq!(big.spacing, base.spacing, "{name}: gaps never scale");
    }
}

#[test]
fn a_button_at_twice_the_text_holds_its_label() {
    for (name, base) in themes() {
        let big = larger(&base, 2.0);
        for size in [
            ButtonSize::Xs,
            ButtonSize::Sm,
            ButtonSize::Md,
            ButtonSize::Lg,
        ] {
            let b = Button::new("Continue").size(size);
            let (w, h) = b.preferred_size(&big);
            assert!(h > size.height(&base), "{name} {size:?}");
            let style = size.text_style(&big);
            let (tw, th) = TextMeasurer::measure_styled("Continue", &style, None);
            let m = size.metrics(&big);
            assert!(th <= h, "{name} {size:?}: {th} in {h}");
            assert!(tw + m.pad_x * 2.0 <= w + 0.5, "{name} {size:?}");
        }
    }
}

#[test]
fn exact_styles_scale_too() {
    let mut t: TypographyScale = Theme::hoff_core().typography;
    let at_one = t.exact(13.0, 20.0, 400);
    t.text_scale = TextScale::linear(1.5);
    let big = t.exact(13.0, 20.0, 400);
    assert_eq!(big.font_size, at_one.font_size * 1.5);
    assert_eq!(big.line_height, 30.0);
    let mono = t.exact_mono(13.0, 20.0, 400);
    assert_eq!(mono.font_size, 19.5);
}
