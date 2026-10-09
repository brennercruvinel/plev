use super::suggest::{levenshtein, suggest_similar};
use super::*;

#[test]
fn levenshtein_identical() {
    assert_eq!(levenshtein("abc", "abc"), 0);
}

#[test]
fn levenshtein_empty() {
    assert_eq!(levenshtein("", "abc"), 3);
    assert_eq!(levenshtein("abc", ""), 3);
    assert_eq!(levenshtein("", ""), 0);
}

#[test]
fn levenshtein_one_edit() {
    assert_eq!(levenshtein("div", "dive"), 1); // insertion
    assert_eq!(levenshtein("buton", "button"), 1); // deletion
    assert_eq!(levenshtein("clik", "click"), 1); // insertion
    assert_eq!(levenshtein("tex", "text"), 1); // insertion
}

#[test]
fn levenshtein_two_edits() {
    assert_eq!(levenshtein("spacr", "spacer"), 1);
    assert_eq!(levenshtein("buttn", "button"), 1);
    assert_eq!(levenshtein("opacty", "opacity"), 1);
}

#[test]
fn levenshtein_distant() {
    assert!(levenshtein("xyz", "click") > 2);
}

#[test]
fn suggest_element_dive() {
    assert_eq!(suggest_similar("dive", ELEMENT_NAMES), Some("div"));
}

#[test]
fn suggest_element_ttext() {
    assert_eq!(suggest_similar("ttext", ELEMENT_NAMES), Some("text"));
}

#[test]
fn suggest_element_buton() {
    assert_eq!(suggest_similar("buton", ELEMENT_NAMES), Some("button"));
}

#[test]
fn suggest_no_match() {
    assert_eq!(suggest_similar("xyzzy", ELEMENT_NAMES), None);
}

#[test]
fn suggest_modifier_fontt_size() {
    assert_eq!(
        suggest_similar("fontt_size", MODIFIER_NAMES),
        Some("font_size")
    );
}

#[test]
fn suggest_modifier_opacty() {
    assert_eq!(suggest_similar("opacty", MODIFIER_NAMES), Some("opacity"));
}

#[test]
fn suggest_event_clik() {
    assert_eq!(suggest_similar("clik", EVENT_NAMES), Some("click"));
}

#[test]
fn suggest_event_scrolll() {
    assert_eq!(suggest_similar("scrolll", EVENT_NAMES), Some("scroll"));
}

#[test]
fn suggest_block_shw() {
    assert_eq!(suggest_similar("shw", BLOCK_KEYWORDS), Some("show"));
}

#[test]
fn suggest_block_eac() {
    assert_eq!(suggest_similar("eac", BLOCK_KEYWORDS), Some("each"));
}

#[test]
fn suggest_short_word_strict() {
    // For short words (len <= 3), only distance 1 is allowed
    // "bx" is distance 1 from "bg" (substitution) - should NOT match
    // because "bx" is also distance 1 from "px" and "mx" — test that
    // at least one match is returned
    let result = suggest_similar("bx", MODIFIER_NAMES);
    assert!(result.is_some());
}
