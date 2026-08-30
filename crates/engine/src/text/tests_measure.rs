use super::backend::{CosmicTextBackend, StyleRun, TextBackend, TextStyle};
use super::measure::TextMeasurer;

fn inter(font_size: f32) -> TextStyle {
    TextStyle::new(font_size).with_family("Inter")
}

fn mono(font_size: f32) -> TextStyle {
    TextStyle::new(font_size).with_family("JetBrains Mono")
}

// -- measure --

#[test]
fn empty_text_measures_zero() {
    assert_eq!(TextMeasurer::measure("", 16.0, None), (0.0, 0.0));
}

#[test]
fn proportional_font_iiii_narrower_than_wwww() {
    let (narrow, _) = TextMeasurer::measure_styled("iiii", &inter(16.0), None);
    let (wide, _) = TextMeasurer::measure_styled("WWWW", &inter(16.0), None);
    assert!(narrow > 0.0);
    assert!(
        narrow < wide,
        "Inter is proportional: 'iiii' ({narrow}) must be narrower than 'WWWW' ({wide})"
    );
}

#[test]
fn monospace_font_iiii_same_width_as_wwww() {
    let (narrow, _) = TextMeasurer::measure_styled("iiii", &mono(16.0), None);
    let (wide, _) = TextMeasurer::measure_styled("WWWW", &mono(16.0), None);
    assert!(narrow > 0.0);
    assert!(
        (narrow - wide).abs() < 0.5,
        "JetBrains Mono is monospace: 'iiii' ({narrow}) must equal 'WWWW' ({wide})"
    );
}

#[test]
fn width_scales_with_font_size() {
    let (small, _) = TextMeasurer::measure_styled("Hello", &inter(12.0), None);
    let (large, _) = TextMeasurer::measure_styled("Hello", &inter(24.0), None);
    assert!(large > small * 1.5);
}

#[test]
fn single_line_height_matches_line_height() {
    let style = inter(16.0);
    let (_, h) = TextMeasurer::measure_styled("Hello", &style, None);
    assert!((h - style.line_height).abs() < 0.01);
}

#[test]
fn wrapping_small_max_width_grows_height() {
    let style = inter(16.0);
    let text = "the quick brown fox jumps over the lazy dog";
    let (unwrapped_w, unwrapped_h) = TextMeasurer::measure_styled(text, &style, None);
    let (wrapped_w, wrapped_h) = TextMeasurer::measure_styled(text, &style, Some(80.0));
    assert!(wrapped_w <= 80.0 + 0.5);
    assert!(wrapped_w < unwrapped_w);
    assert!(
        wrapped_h >= unwrapped_h * 2.0,
        "wrapping at 80px must produce multiple lines ({wrapped_h} vs {unwrapped_h})"
    );
}

#[test]
fn newlines_add_lines() {
    let style = inter(16.0);
    let (_, one) = TextMeasurer::measure_styled("a", &style, None);
    let (_, three) = TextMeasurer::measure_styled("a\nb\nc", &style, None);
    assert!((three - one * 3.0).abs() < 0.01);
}

#[test]
fn measure_is_cached() {
    // Two identical calls must return identical results (second from cache).
    let a = TextMeasurer::measure("cached text", 16.0, Some(200.0));
    let b = TextMeasurer::measure("cached text", 16.0, Some(200.0));
    assert_eq!(a, b);
}

#[test]
fn bold_weight_measures_at_least_regular() {
    let regular = mono(16.0);
    let bold = mono(16.0).with_weight(700);
    let (rw, _) = TextMeasurer::measure_styled("Hello", &regular, None);
    let (bw, _) = TextMeasurer::measure_styled("Hello", &bold, None);
    assert!(
        bw >= rw * 0.9,
        "bold ({bw}) should not collapse vs regular ({rw})"
    );
}

// -- weight -> face resolution (HOFF regression: weights 500/600/700) --
//
// cosmic-text keeps the requested family only on an *exact* weight match
// (`FontFallbackIter::default_font_match_key` filters `font_weight_diff == 0`).
// When only Inter-Regular was embedded, any text at weight 500/600/700 skipped
// Inter entirely and resolved per-word through the platform fallback lists —
// on macOS: Apple SD Gothic Neo + Apple Color Emoji at 500/600 (+35% advance),
// Menlo at 700 ("1,632" headlines, 600-weight labels).

/// Advance of `text` at `weight` must stay close to the regular advance:
/// faces of the same family differ a few percent, a family fallback does not.
fn assert_advance_close_to_regular(text: &str, style: &TextStyle, weight: u16) {
    let (rw, _) = TextMeasurer::measure_styled(text, style, None);
    let (ww, _) = TextMeasurer::measure_styled(text, &style.clone().with_weight(weight), None);
    assert!(rw > 0.0);
    let ratio = (ww - rw).abs() / rw;
    // Sibling faces of one family differ by up to ~10% (Rubik Bold runs ~8.5%
    // wider than Rubik Regular); a broken family fallback drifts ~35%+. The
    // threshold catches the fallback bug while admitting real per-face width.
    assert!(
        ratio < 0.13,
        "{text:?} at weight {weight} ({ww}px) drifted {:.1}% from regular ({rw}px): \
         weight resolved to a fallback family instead of a sibling face",
        ratio * 100.0
    );
}

#[test]
fn semibold_digits_advance_close_to_regular() {
    // StatCard headline from the showcase.
    assert_advance_close_to_regular("1,632", &inter(28.0), 600);
}

#[test]
fn medium_multiword_advance_close_to_regular() {
    assert_advance_close_to_regular("Expense Tracker", &inter(18.0), 500);
}

#[test]
fn default_family_weights_advance_close_to_regular() {
    // Widgets pass `font_family: None` (engine default family).
    let style = TextStyle::new(16.0);
    for weight in [500, 600, 700] {
        assert_advance_close_to_regular("Expense Tracker 1,632", &style, weight);
    }
}

#[test]
fn inter_family_resolves_inter_faces_for_all_ui_weights() {
    for weight in [400u16, 500, 600, 700] {
        let style = inter(16.0).with_weight(weight);
        let faces = TextMeasurer::resolved_faces("Expense Tracker 1,632", &style);
        assert!(!faces.is_empty());
        for (family, face_weight) in &faces {
            assert_eq!(
                family, "Inter",
                "weight {weight} fell back to family {family:?} (weight {face_weight})"
            );
            assert_eq!(
                *face_weight, weight,
                "weight {weight} resolved to Inter face of weight {face_weight}"
            );
        }
    }
}

#[test]
fn default_family_resolves_rubik_faces_for_all_ui_weights() {
    // `font_family: None` maps to Family::SansSerif; the engine pins that to
    // the embedded Rubik (the HOFF reference typeface) so weights resolve
    // deterministically on any system.
    for weight in [400u16, 500, 600, 700] {
        let style = TextStyle::new(16.0).with_weight(weight);
        let faces = TextMeasurer::resolved_faces("Expense Tracker 1,632", &style);
        assert!(!faces.is_empty());
        for (family, face_weight) in &faces {
            assert_eq!(
                family, "Rubik",
                "default family at weight {weight} fell back to {family:?} (weight {face_weight})"
            );
            assert_eq!(*face_weight, weight);
        }
    }
}

// -- cursor_x / hit_test round-trip --

fn char_boundaries(text: &str) -> Vec<usize> {
    let mut cursors: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    cursors.push(text.len());
    cursors
}

fn assert_round_trip(text: &str, style: &TextStyle) {
    let y = style.line_height / 2.0;
    for cursor in char_boundaries(text) {
        let x = TextMeasurer::cursor_x_styled(text, style, None, cursor);
        let hit = TextMeasurer::hit_test_styled(text, style, None, x, y);
        assert_eq!(
            hit, cursor,
            "round-trip failed for {text:?} cursor {cursor} (x={x}, got {hit})"
        );
    }
}

#[test]
fn round_trip_proportional() {
    assert_round_trip("Hello, World!", &inter(16.0));
}

#[test]
fn round_trip_monospace() {
    assert_round_trip("Hello, World!", &mono(16.0));
}

#[test]
fn round_trip_narrow_chars_proportional() {
    // 'i' and 'l' are very narrow in Inter — the old 0.6 ratio failed here.
    assert_round_trip("illiilli", &inter(16.0));
}

#[test]
fn cursor_x_monotonic_in_cursor() {
    let style = inter(16.0);
    let text = "WiWiWi";
    let mut last = -1.0;
    for cursor in char_boundaries(text) {
        let x = TextMeasurer::cursor_x_styled(text, &style, None, cursor);
        assert!(
            x > last,
            "cursor_x must be strictly increasing ({x} after {last})"
        );
        last = x;
    }
}

#[test]
fn cursor_x_end_matches_width() {
    let style = inter(16.0);
    let text = "Hello";
    let (w, _) = TextMeasurer::measure_styled(text, &style, None);
    let x = TextMeasurer::cursor_x_styled(text, &style, None, text.len());
    assert!(
        (x - w).abs() < 0.5,
        "caret at end ({x}) should sit at text width ({w})"
    );
}

#[test]
fn hit_test_second_line_after_wrap() {
    let style = inter(16.0);
    let text = "aaaa bbbb cccc dddd";
    let (_, h) = TextMeasurer::measure_styled(text, &style, Some(60.0));
    assert!(h > style.line_height); // it wrapped
    // A click on the second line must land past the first line's bytes.
    let hit = TextMeasurer::hit_test_styled(text, &style, Some(60.0), 1.0, style.line_height * 1.5);
    assert!(hit > 0);
}

// -- TextBackend / CosmicTextBackend --

fn single_run(text: &str, style: TextStyle) -> Vec<StyleRun> {
    vec![StyleRun {
        range: 0..text.len(),
        style,
    }]
}

#[test]
fn backend_measure_matches_measurer() {
    let mut backend = CosmicTextBackend::new();
    let runs = single_run("Hello", inter(16.0));
    let from_backend = backend.measure("Hello", &runs, None);
    let from_measurer = TextMeasurer::measure_styled("Hello", &inter(16.0), None);
    assert_eq!(from_backend, from_measurer);
}

#[test]
fn backend_shape_hit_test_round_trip() {
    let mut backend = CosmicTextBackend::new();
    let style = inter(16.0);
    let text = "Hello";
    let shaped = backend.shape(text, &single_run(text, style.clone()), None);
    for cursor in char_boundaries(text) {
        let rect = backend.cursor_geometry(&shaped, cursor);
        let hit = backend.hit_test(&shaped, rect.x, rect.y + rect.height / 2.0);
        assert_eq!(hit, cursor);
    }
}

#[test]
fn backend_cursor_geometry_has_line_height() {
    let mut backend = CosmicTextBackend::new();
    let style = inter(16.0);
    let shaped = backend.shape("Hi", &single_run("Hi", style.clone()), None);
    let rect = backend.cursor_geometry(&shaped, 0);
    assert!((rect.height - style.line_height).abs() < 0.01);
    assert_eq!(backend.line_height(&style), style.line_height);
}

#[test]
fn backend_empty_runs_uses_default_style() {
    let mut backend = CosmicTextBackend::new();
    let (w, h) = backend.measure("Hello", &[], None);
    assert!(w > 0.0);
    assert!((h - TextStyle::default().line_height).abs() < 0.01);
}

#[test]
fn shaped_text_size_matches_measure() {
    let style = inter(16.0);
    let shaped = TextMeasurer::shape("Hello wrap test", &style, Some(60.0));
    let size = shaped.size();
    let measured = TextMeasurer::measure_styled("Hello wrap test", &style, Some(60.0));
    assert_eq!(size, measured);
}

// -- letter-spacing (HOFF body family: 0.025em) --

#[test]
fn letter_spacing_increases_advance_per_glyph() {
    // 0.025em at 14px = 0.35px of tracking, the HOFF =body-2r value.
    let spacing = 0.35;
    let base = inter(14.0);
    let spaced = base.clone().with_letter_spacing(spacing);
    let text = "Research Social";
    let n = text.chars().count() as f32;
    let (w0, _) = TextMeasurer::measure_styled(text, &base, None);
    let (w1, _) = TextMeasurer::measure_styled(text, &spaced, None);
    let delta = w1 - w0;
    assert!(
        delta > 0.0,
        "tracking must widen the advance ({w0} -> {w1})"
    );
    assert!(
        (delta - spacing * (n - 1.0)).abs() <= 1.0,
        "delta {delta} should be ~spacing*(n_glyphs-1) = {}",
        spacing * (n - 1.0)
    );
}

#[test]
fn letter_spacing_distinguishes_measure_cache() {
    // Same text/style, different spacing: the cached entries must differ,
    // otherwise pills and tabs measure without their tracking.
    let text = "Follow";
    let (w0, _) = TextMeasurer::measure_styled(text, &inter(14.0), None);
    let (w1, _) = TextMeasurer::measure_styled(text, &inter(14.0).with_letter_spacing(0.35), None);
    assert!(w1 > w0);
}

#[test]
fn text_node_key_distinguishes_letter_spacing() {
    use crate::compositor::TextNodeKey;
    use std::hash::{Hash, Hasher};

    let style = inter(14.0);
    let a = TextNodeKey::from_style("Follow", &style, None);
    let b = TextNodeKey::from_style("Follow", &style.clone().with_letter_spacing(0.35), None);
    assert_ne!(a, b, "TextSystem shaping-cache keys must differ");

    let hash = |key: &TextNodeKey| {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut h);
        h.finish()
    };
    assert_ne!(hash(&a), hash(&b));
    assert_eq!(b.letter_spacing_bits, 0.35_f32.to_bits());
}

#[test]
fn text_node_key_from_style_carries_every_field() {
    use crate::compositor::TextNodeKey;

    let style = TextStyle::new(14.0)
        .with_line_height(23.8)
        .with_weight(500)
        .with_letter_spacing(0.35)
        .with_family("Inter");
    let key = TextNodeKey::from_style("post body", &style, Some(320.0));
    assert_eq!(key.font_size_bits, 14.0_f32.to_bits());
    assert_eq!(key.line_height_bits, 23.8_f32.to_bits());
    assert_eq!(key.font_weight, 500);
    assert_eq!(key.letter_spacing_bits, 0.35_f32.to_bits());
    assert_eq!(key.font_family.as_deref(), Some("Inter"));
    assert_eq!(key.max_width_bits, Some(320.0_f32.to_bits()));
}

// -- vertical centering by real metrics --

#[test]
fn line_metrics_come_from_real_faces() {
    let style = inter(14.0);
    let m = TextMeasurer::line_metrics(&style);
    assert!(m.ascent > 0.0 && m.descent > 0.0);
    assert!(m.ascent > m.descent, "Latin faces are top-heavy");
    assert!(
        m.glyph_height() < m.line_height,
        "Inter ascent+descent (~1.21em) fits inside the 1.3 line box"
    );
    // The baseline sits inside the line box, below its midpoint.
    assert!(m.baseline > m.line_height / 2.0 && m.baseline < m.line_height);
}

#[test]
fn vertical_center_centers_glyph_box_in_44px_pill() {
    // base-2sm inside the canonical 44px HOFF button.
    let style = crate::theme::TypographyScale::hoff().base_2sm();
    let container_h = 44.0;
    let y = TextMeasurer::vertical_center(&style, container_h);
    let m = TextMeasurer::line_metrics(&style);
    let top_gap = y + m.glyph_top();
    let bottom_gap = container_h - (y + m.glyph_top() + m.glyph_height());
    assert!(
        (top_gap - bottom_gap).abs() < 0.01,
        "glyph box must be optically centered (top {top_gap} vs bottom {bottom_gap})"
    );
    assert!(top_gap > 0.0 && top_gap < container_h / 2.0);
}

#[test]
fn vertical_center_consistent_across_ramp() {
    // Every HOFF style stays strictly inside a 44px control.
    let ramp = crate::theme::TypographyScale::hoff();
    for style in [
        ramp.caption_sm(),
        ramp.base_2sm(),
        ramp.base_2m(),
        ramp.base_m(),
    ] {
        let y = TextMeasurer::vertical_center(&style, 44.0);
        let m = TextMeasurer::line_metrics(&style);
        assert!(y + m.glyph_top() > 0.0);
        assert!(y + m.glyph_top() + m.glyph_height() < 44.0);
    }
}

// -- truncate_to_width (single-line ellipsis via real shaping) -------------

#[test]
fn truncate_keeps_text_that_fits() {
    let s = "short enough";
    assert_eq!(TextMeasurer::truncate_to_width(s, &inter(14.0), 500.0), s);
}

#[test]
fn truncate_empty_string_stays_empty() {
    assert_eq!(TextMeasurer::truncate_to_width("", &inter(14.0), 100.0), "");
}

#[test]
fn truncate_ellipsizes_to_one_line_that_really_fits() {
    let style = inter(14.0);
    let long = "the quick brown fox jumps over the lazy dog, again and again and again";
    let out = TextMeasurer::truncate_to_width(long, &style, 120.0);
    assert!(out.ends_with('\u{2026}'));
    assert!(out.len() < long.len());
    // The ADR invariant: measured with the SAME style the caller draws.
    let (w, _) = TextMeasurer::measure_styled(&out, &style, None);
    assert!(w <= 120.0, "truncated width {w} exceeds max_width");
}

#[test]
fn truncate_maximizes_the_kept_prefix() {
    let style = inter(14.0);
    let long = "abcdefghij".repeat(20);
    let out = TextMeasurer::truncate_to_width(&long, &style, 200.0);
    let kept_graphemes = out.trim_end_matches('\u{2026}').chars().count();
    // The next candidate (one more grapheme + ellipsis) must overflow.
    let wider: String = long.chars().take(kept_graphemes + 1).collect();
    let (w, _) = TextMeasurer::measure_styled(&format!("{wider}\u{2026}"), &style, None);
    assert!(w > 200.0, "a longer prefix still fits: {w} <= 200");
}

#[test]
fn truncate_trims_trailing_whitespace_before_the_ellipsis() {
    let style = inter(14.0);
    // Spaces land right at the cut: "aaa aaa aaa …" must not keep a
    // dangling space before "…".
    let s = "aaaa aaaa aaaa aaaa aaaa aaaa aaaa aaaa";
    let out = TextMeasurer::truncate_to_width(s, &style, 100.0);
    assert!(out.ends_with('\u{2026}'));
    assert!(!out.ends_with(" \u{2026}"), "trailing space kept: {out:?}");
}

#[test]
fn truncate_never_splits_a_grapheme_cluster() {
    let style = inter(14.0);
    // Flag emoji are one grapheme made of two regional indicators; a
    // char-wise cut would split them into lone (invalid) indicators.
    let s = "🇧🇷🇧🇷🇧🇷🇧🇷🇧🇷🇧🇷🇧🇷🇧🇷 tail text here";
    let out = TextMeasurer::truncate_to_width(s, &style, 80.0);
    assert!(out.ends_with('\u{2026}'));
    // The kept part must be a grapheme-aligned prefix: a char-wise cut
    // can produce a prefix that still passes `starts_with` (a lone
    // regional indicator), so check alignment against the input's
    // grapheme boundaries directly.
    use unicode_segmentation::UnicodeSegmentation;
    let kept = out.trim_end_matches('\u{2026}');
    let aligned: Vec<String> = (0..=s.graphemes(true).count())
        .map(|k| s.graphemes(true).take(k).collect())
        .collect();
    assert!(
        aligned.iter().any(|p| p == kept),
        "output prefix {kept:?} is not a grapheme-aligned prefix of the input"
    );
}

#[test]
fn truncate_returns_the_ellipsis_when_nothing_fits() {
    let style = inter(14.0);
    // max_width smaller than the ellipsis itself: the minimal signal.
    let out = TextMeasurer::truncate_to_width("hello world", &style, 1.0);
    assert_eq!(out, "\u{2026}");
    // Zero and negative widths behave the same (no panic, no empty).
    assert_eq!(
        TextMeasurer::truncate_to_width("hello", &style, 0.0),
        "\u{2026}"
    );
}
