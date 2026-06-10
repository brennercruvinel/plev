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
    assert!(
        ratio < 0.08,
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
fn default_family_resolves_inter_faces_for_all_ui_weights() {
    // `font_family: None` maps to Family::SansSerif; the engine pins that to
    // the embedded Inter so weights resolve deterministically on any system.
    for weight in [400u16, 500, 600, 700] {
        let style = TextStyle::new(16.0).with_weight(weight);
        let faces = TextMeasurer::resolved_faces("Expense Tracker 1,632", &style);
        assert!(!faces.is_empty());
        for (family, face_weight) in &faces {
            assert_eq!(
                family, "Inter",
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
