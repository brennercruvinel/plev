//! The drawn glyphs must span the width the measurer promised, at every
//! raster scale.
//!
//! This is the ADR invariant "one TextStyle per run, shared by measurement
//! and drawing" checked in pixels rather than in advances: layout sizes a
//! shape from `measure_styled`, and if the rasterizer paints wider or
//! narrower than that, labels overflow their pills and centred text sits
//! off-centre.
//!
//! Scale is what changes between monitors: a Retina panel reports 2.0, an
//! external display commonly 1.0, with fractional factors in between.

use engine::text::probe::{Specimen, render};
use engine::text::{TextMeasurer, TextStyle};

const LOGICAL_W: u32 = 1400;
const LOGICAL_H: u32 = 120;
const ORIGIN_X: f32 = 20.0;

fn style(size: f32, weight: u16) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(size * 1.4)
        .with_weight(weight)
}

/// Drawn extent (in logical px) of `text`, or `None` without a GPU.
fn drawn_width(text: &str, st: &TextStyle, scale: f32) -> Option<f32> {
    let scene = vec![Specimen::new(text, st.clone(), ORIGIN_X, 20.0)];
    let img = render(&scene, LOGICAL_W, LOGICAL_H, scale)?;
    let (first, last) = img.ink_extent(0, img.height)?;
    Some((last + 1 - first) as f32 / scale)
}

/// The painted run is as wide as the shaped run.
///
/// Tolerance is one logical pixel plus the side bearings the outermost
/// glyphs contribute: an advance includes bearings, ink does not, so ink is
/// legitimately a hair narrower. It can never be *wider*, and it can never
/// drift by the tens of percent that extra tracking or a family fallback
/// produce — the "c a r d s" symptom in
/// docs/adr/embed-every-font-weight-in-use.md.
#[test]
fn drawn_extent_matches_the_measured_advance_at_every_scale() {
    const TEXT: &str = "1ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut checked = 0;
    for scale in [1.0f32, 1.25, 1.5, 2.0, 3.0] {
        for weight in [400u16, 500, 600, 700] {
            for size in [11.0f32, 15.0, 19.0] {
                let st = style(size, weight);
                let expected = TextMeasurer::measure_styled(TEXT, &st, None).0;
                let Some(drawn) = drawn_width(TEXT, &st, scale) else {
                    eprintln!("no GPU adapter; skipping");
                    return;
                };
                assert!(
                    drawn <= expected + 1.0,
                    "scale {scale}, weight {weight}, {size}px: painted {drawn:.1}px \
                     but the measurer promised {expected:.1}px — layout will size \
                     every shape around this text too small"
                );
                assert!(
                    drawn >= expected - size,
                    "scale {scale}, weight {weight}, {size}px: painted only \
                     {drawn:.1}px of a {expected:.1}px run — glyphs are missing \
                     or collapsed"
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 60);
}

/// Widths scale linearly with the raster scale. If a scale change altered
/// the shaped result (rather than only the bitmap resolution), text would
/// reflow when the window moved between monitors.
#[test]
fn raster_scale_changes_resolution_not_layout() {
    const TEXT: &str = "Expense Tracker \u{b7} 1,632";
    let st = style(19.0, 500);
    let Some(at_1x) = drawn_width(TEXT, &st, 1.0) else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    for scale in [1.25f32, 1.5, 2.0, 3.0] {
        let Some(w) = drawn_width(TEXT, &st, scale) else {
            return;
        };
        assert!(
            (w - at_1x).abs() <= 2.0,
            "the same run occupies {w:.1} logical px at scale {scale} but \
             {at_1x:.1} at 1x: raster scale is leaking into layout"
        );
    }
}

/// Fractional scale factors are real (macOS "More Space" modes, many
/// external displays) and must not smear or drop glyphs.
#[test]
fn fractional_scales_draw_every_glyph() {
    const TEXT: &str = "Builder";
    let st = style(20.0, 500);
    let expected = TextMeasurer::measure_styled(TEXT, &st, None).0;
    for scale in [1.1f32, 1.25, 1.3333334, 1.5, 1.7777778, 2.5] {
        let Some(drawn) = drawn_width(TEXT, &st, scale) else {
            eprintln!("no GPU adapter; skipping");
            return;
        };
        assert!(
            drawn <= expected + 1.0 && drawn >= expected - 20.0,
            "scale {scale}: painted {drawn:.1}px against a measured \
             {expected:.1}px"
        );
    }
}
