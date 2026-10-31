//! `TextMeasurer::lines`: the shaper's own line breaks (UAX#14), for a
//! widget that aligns each line on its own.

use super::super::backend::TextStyle;
use super::super::measure::TextMeasurer;

fn sans() -> TextStyle {
    TextStyle::new(14.0)
}

/// Every line fits the width, give or take `drift` (a line shaped on its
/// own can come out a little wider than inside its paragraph), the
/// lines put back together are the text's words in order, and there are
/// as many as the wrapped measure is tall.
fn holds(text: &str, w: f32, drift: f32) -> Vec<(String, f32)> {
    let style = sans();
    let lines = TextMeasurer::lines(text, &style, w);
    assert!(!lines.is_empty(), "{text:?}");
    for (line, lw) in &lines {
        assert!(*lw <= w + drift, "{line:?} is {lw} wide in {w}: {lines:?}");
    }
    let joined: String = lines.iter().map(|(l, _)| l.as_str()).collect();
    let squash = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    assert_eq!(squash(&joined), squash(text), "nothing lost or doubled");
    let (_, h) = TextMeasurer::measure_styled(text, &style, Some(w));
    let n = (h / style.line_height).round() as usize;
    assert_eq!(lines.len(), n, "{text:?}: the lines a wrapped draw takes");
    lines
}

#[test]
fn latin_breaks_between_words() {
    let lines = holds(
        "The quick brown fox jumps over the lazy dog again and again",
        120.0,
        0.5,
    );
    assert!(lines.len() > 2);
    for (line, _) in &lines {
        assert_eq!(line.trim(), line, "no whitespace at the ends");
    }
}

#[test]
fn cjk_breaks_between_ideographs_without_a_space() {
    let text = "你好世界这是一个没有空格的很长的句子用来测试换行";
    let lines = holds(text, 90.0, 0.5);
    assert!(
        lines.len() > 1,
        "a run of ideographs still wraps: {lines:?}"
    );
}

#[test]
fn thai_breaks_at_word_bounds_without_a_space() {
    let text = "ภาษาไทยไม่มีการเว้นวรรคระหว่างคำแต่ต้องตัดบรรทัดได้";
    // Thai marks shaped on their own line take a few px more than in the
    // paragraph the shaper broke
    let lines = holds(text, 90.0, 4.0);
    assert!(lines.len() > 1, "Thai wraps with no space: {lines:?}");
}

#[test]
fn a_hard_break_and_an_empty_line_are_kept() {
    let lines = TextMeasurer::lines("one\n\ntwo", &sans(), 400.0);
    let words: Vec<&str> = lines.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(words, ["one", "", "two"]);
    assert_eq!(lines[1].1, 0.0);
    assert!(TextMeasurer::lines("", &sans(), 400.0).is_empty());
}
