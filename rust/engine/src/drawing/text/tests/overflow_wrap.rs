//! `OverflowWrap`: lines break between words; a word wider than the line
//! stays whole (CSS `overflow-wrap: normal`) unless the style asks for
//! `Anywhere` or `BreakWord`. A word broken letter by letter (the Collapsible's label,
//! the sidebar's at 220) came from the shaper's old default, which broke
//! any word that did not fit.

use super::super::backend::{OverflowWrap, TextStyle};
use super::super::measure::TextMeasurer;

fn style() -> TextStyle {
    TextStyle::new(14.0).with_line_height(20.0)
}

fn lines_of(text: &str, style: &TextStyle, w: f32) -> usize {
    let (_, h) = TextMeasurer::measure_styled(text, style, Some(w));
    (h / style.line_height).round() as usize
}

#[test]
fn the_default_is_normal() {
    assert_eq!(TextStyle::new(14.0).overflow_wrap, OverflowWrap::Normal);
    assert_eq!(TextStyle::default().overflow_wrap, OverflowWrap::Normal);
}

#[test]
fn a_word_wider_than_the_line_stays_whole_and_runs_past_it() {
    let s = style();
    let (word_w, _) = TextMeasurer::measure_styled("Collapsible", &s, None);
    for w in [0.0, 1.0, 20.0, word_w * 0.5, word_w - 1.0] {
        let (mw, h) = TextMeasurer::measure_styled("Collapsible", &s, Some(w));
        assert_eq!((h / s.line_height).round(), 1.0, "one line at {w}");
        assert!(
            (mw - word_w).abs() < 0.5,
            "the word's own width at {w}: {mw} vs {word_w}"
        );
    }
}

#[test]
fn anywhere_breaks_a_word_wider_than_the_line_between_its_letters() {
    let s = style().with_overflow_wrap(OverflowWrap::Anywhere);
    let (word_w, _) = TextMeasurer::measure_styled("Collapsible", &s, None);
    let w = word_w * 0.5;
    let (mw, _) = TextMeasurer::measure_styled("Collapsible", &s, Some(w));
    assert!(lines_of("Collapsible", &s, w) >= 2);
    assert!(mw <= w + 0.5, "{mw} fits {w}");
}

#[test]
fn a_narrow_line_breaks_between_words_never_inside_one() {
    let s = style();
    let text = "Materials imported from the shared library";
    let lines = TextMeasurer::lines(text, &s, 1.0);
    let words: Vec<String> = text.split(' ').map(str::to_owned).collect();
    let got: Vec<String> = lines.into_iter().map(|(l, _)| l).collect();
    assert_eq!(got, words, "one word a line, each whole");
    // a width that holds the longest word holds every word whole too
    let longest = words
        .iter()
        .map(|w| TextMeasurer::measure_styled(w, &s, None).0)
        .fold(0.0, f32::max);
    for (line, lw) in TextMeasurer::lines(text, &s, longest + 1.0) {
        assert!(lw <= longest + 1.5, "{line:?} is {lw}");
        assert!(words.iter().any(|w| line.contains(w.as_str())), "{line:?}");
    }
}

/// `BreakWord` draws as `Anywhere`: a word wider than the line breaks
/// between its letters, a narrow line still breaks between words; only
/// its min-content differs (the layout's, `compose::layout`).
#[test]
fn break_word_draws_as_anywhere() {
    let any = style().with_overflow_wrap(OverflowWrap::Anywhere);
    let brk = style().with_overflow_wrap(OverflowWrap::BreakWord);
    for (text, w) in [
        ("Collapsible", 30.0),
        ("Materials imported from the shared library", 80.0),
        ("Materials imported from the shared library", 400.0),
    ] {
        assert_eq!(
            TextMeasurer::lines(text, &brk, w),
            TextMeasurer::lines(text, &any, w),
            "{text} at {w}"
        );
    }
    assert_eq!(OverflowWrap::BreakWord.min_content(), OverflowWrap::Normal);
    assert_eq!(OverflowWrap::Anywhere.min_content(), OverflowWrap::Anywhere);
    assert_eq!(OverflowWrap::Normal.min_content(), OverflowWrap::Normal);
}

#[test]
fn the_cache_tells_the_two_rules_apart() {
    let normal = style();
    let anywhere = style().with_overflow_wrap(OverflowWrap::Anywhere);
    // measured in either order, each keeps its own answer
    assert_eq!(lines_of("Supercalifragilistic", &normal, 30.0), 1);
    assert!(lines_of("Supercalifragilistic", &anywhere, 30.0) > 1);
    assert_eq!(lines_of("Supercalifragilistic", &normal, 30.0), 1);
}

#[test]
fn a_clamp_truncates_a_word_that_runs_past_the_line() {
    let s = style();
    let out = TextMeasurer::clamp_to_lines("Supercalifragilistic", &s, 60.0, 1);
    assert!(out.ends_with('\u{2026}'), "{out:?}");
    let (w, _) = TextMeasurer::measure_styled(&out, &s, Some(60.0));
    assert!(w <= 60.5, "{out:?} is {w} wide");
    // text that fits stays as it is
    assert_eq!(
        TextMeasurer::clamp_to_lines("two words", &s, 400.0, 1),
        "two words"
    );
}

#[test]
fn the_text_key_carries_the_rule() {
    use crate::drawing::compositor::TextNodeKey;
    let anywhere = style().with_overflow_wrap(OverflowWrap::Anywhere);
    let a = TextNodeKey::from_style("x", &anywhere, Some(10.0));
    let n = TextNodeKey::from_style("x", &style(), Some(10.0));
    assert_eq!(a.overflow_wrap, OverflowWrap::Anywhere);
    assert_eq!(n.overflow_wrap, OverflowWrap::Normal);
    assert_ne!(a, n, "two shapings, two cache entries");
    assert_eq!(
        TextNodeKey::new("x", 14.0, 20.0, None).overflow_wrap,
        OverflowWrap::Normal
    );
}

/// Thai has no spaces between words and the line breaker no dictionary
/// for it: a run wider than the line still breaks, between graphemes.
#[test]
fn a_script_without_word_bounds_still_wraps() {
    let s = style();
    let thai = "ภาษาไทยไม่มีการเว้นวรรคระหว่างคำแต่ต้องตัดบรรทัดได้";
    assert!(lines_of(thai, &s, 90.0) > 1);
    assert_eq!(
        lines_of("Supercalifragilistic", &s, 30.0),
        1,
        "Latin keeps its word"
    );
}
