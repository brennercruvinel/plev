//! Pixel-level regression tests for the glyph rasterizer.
//!
//! The invariant they all serve: **a string renders to the same pixels no
//! matter what else is on screen**. Glyph caching, atlas packing and atlas
//! growth are per-frame global state, and every defect in
//! `docs/adr/glyph-raster-identity-and-atlas-isolation.md` was a leak of
//! that state into one string's pixels.
//!
//! These need a GPU adapter. Where there is none they skip rather than fail,
//! so the suite still runs on adapterless CI.

use engine::text::TextStyle;
use engine::text::probe::{Rendered, Specimen, render};

const W: u32 = 900;
const SCALE: f32 = 2.0;
/// Rows the first specimen occupies, in physical pixels.
const BAND_Y: u32 = 0;
const BAND_H: u32 = 80;

fn scene(extra: &[(String, TextStyle)]) -> Vec<Specimen> {
    let ramp = engine::theme::TypographyScale::hoff();
    let mut out = vec![Specimen::new("Expense Tracker", ramp.title(), 20.0, 20.0)];
    for (i, (text, style)) in extra.iter().enumerate() {
        out.push(Specimen::new(
            text.clone(),
            style.clone(),
            20.0,
            80.0 + i as f32 * 40.0,
        ));
    }
    out
}

fn render_scene(extra: &[(String, TextStyle)]) -> Option<Rendered> {
    let s = scene(extra);
    let height = 80 + 40 * extra.len() as u32 + 40;
    render(&s, W, height, SCALE)
}

/// The rasterizer draws something at all — a blank frame would make every
/// other comparison in this file vacuously true.
#[test]
fn renders_ink() {
    let Some(img) = render_scene(&[]) else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    assert!(
        img.ink() > 500,
        "expected a drawn string, got {} inked pixels",
        img.ink()
    );
}

/// The regression that motivated all of this: with enough distinct glyphs on
/// screen the atlas doubles partway through the frame. Quads emitted before
/// the grow must still sample their own glyphs.
///
/// This fails loudly against the pre-fix engine, which baked normalized UVs
/// against the atlas size at emit time: after the grow every earlier quad
/// sampled the wrong region and the text became fragments of other glyphs.
#[test]
fn a_string_renders_identically_whether_or_not_the_atlas_grew() {
    let Some(alone) = render_scene(&[]) else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let filling = engine::text::probe::atlas_filling_specimens();
    let Some(crowded) = render_scene(&filling) else {
        return;
    };

    assert!(
        crowded.ink() > alone.ink(),
        "the filling specimens must actually have drawn, forcing the atlas \
         to grow; otherwise this test proves nothing"
    );
    assert_eq!(
        alone.band(BAND_Y, BAND_H),
        crowded.band(BAND_Y, BAND_H),
        "'Expense Tracker' rendered differently once the atlas grew: quads \
         emitted before the grow are sampling the wrong atlas region"
    );
}

/// Glyph identity must not depend on what was rasterized before it. Drawing
/// the same string twice in one frame, at different subpixel phases, must
/// not disturb the first one.
#[test]
fn repeating_a_string_does_not_change_the_first_copy() {
    let ramp = engine::theme::TypographyScale::hoff();
    let solo = vec![Specimen::new("Expense Tracker", ramp.title(), 20.0, 20.0)];
    let Some(a) = render(&solo, W, 160, SCALE) else {
        eprintln!("no GPU adapter; skipping");
        return;
    };

    // Same string again, offset by a third of a pixel: a different subpixel
    // bin, so a distinct set of glyph bitmaps.
    let mut repeated = solo;
    repeated.push(Specimen::new(
        "Expense Tracker",
        ramp.title(),
        20.333,
        100.0,
    ));
    let Some(b) = render(&repeated, W, 160, SCALE) else {
        return;
    };

    assert_eq!(
        a.band(BAND_Y, BAND_H),
        b.band(BAND_Y, BAND_H),
        "a second copy at another subpixel phase changed the first copy's \
         pixels: the glyph cache is aliasing across subpixel bins"
    );
}

/// Raster scale is part of glyph identity. Rendering at 1x must not poison
/// the 2x render of the same text.
#[test]
fn raster_scale_does_not_leak_between_renders() {
    let ramp = engine::theme::TypographyScale::hoff();
    let s = vec![Specimen::new("Expense Tracker", ramp.title(), 20.0, 20.0)];
    let Some(first) = render(&s, W, 160, 2.0) else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let Some(_at_1x) = render(&s, W, 160, 1.0) else {
        return;
    };
    let Some(again) = render(&s, W, 160, 2.0) else {
        return;
    };
    assert_eq!(
        first.rgba, again.rgba,
        "a 1x render between two 2x renders changed the 2x pixels"
    );
}

/// Dragging a window between monitors: the same scene, rendered after the
/// raster scale has changed, must look like the scene rendered at that scale
/// from the start.
///
/// Pre-fix, a scale change reset the glyph cache and the atlas allocator
/// while layers whose scene had not changed kept their vertices, so they
/// went on sampling texels that had been repacked with other glyphs.
#[test]
fn a_scale_change_does_not_leave_stale_glyphs_behind() {
    let ramp = engine::theme::TypographyScale::hoff();
    let mut s = vec![Specimen::new("Expense Tracker", ramp.title(), 20.0, 20.0)];
    for (i, (text, style)) in engine::text::probe::atlas_filling_specimens()
        .into_iter()
        .enumerate()
    {
        s.push(Specimen::new(text, style, 20.0, 80.0 + i as f32 * 40.0));
    }
    let height = 80 + 40 * 32 + 40;

    // Laptop panel, then external display, then back.
    let Some(first_at_2x) = render(&s, W, height, 2.0) else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let Some(_at_1x) = render(&s, W, height, 1.0) else {
        return;
    };
    let Some(again_at_2x) = render(&s, W, height, 2.0) else {
        return;
    };

    assert_eq!(
        first_at_2x.rgba, again_at_2x.rgba,
        "the scene rendered differently after a round trip through another \
         raster scale: glyph bitmaps or atlas slots survived the change"
    );
}

/// Ink is glyph coverage, not canvas. The threshold has to sit above the
/// cleared background or every comparison built on `ink()` is vacuous —
/// `crowded.ink() > alone.ink()` would then just be comparing image areas.
#[test]
fn ink_counts_glyph_coverage_not_background() {
    let ramp = engine::theme::TypographyScale::hoff();
    let s = vec![Specimen::new("Expense Tracker", ramp.title(), 20.0, 20.0)];
    let Some(img) = render(&s, W, 200, SCALE) else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let total = (img.width * img.height) as usize;
    assert!(img.ink() > 0, "the string drew nothing");
    assert!(
        img.ink() < total / 4,
        "ink() counted {} of {total} pixels — it is counting the background",
        img.ink()
    );
}
