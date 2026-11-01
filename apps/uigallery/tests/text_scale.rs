//! The layout stays valid with larger text (the mantra: "content, font
//! scaling, zoom and densities"; ADR 0086): every page, at the person's
//! text size 1.0, 1.5 and 2.0, at a phone's width and a desktop's, drawn
//! whole (a frame as tall as the page). On each frame:
//!
//! - no text is cut: it fits the width it was given, and no clip or
//!   window edge cuts it across (a text wholly outside its clip is
//!   scrolled away, not cut);
//! - a control holds its label: a text that starts inside an actionable
//!   node's rect ends inside it;
//! - no two actionable nodes overlap, unless one holds the other (a row
//!   and its button);
//! - every actionable node is at least 24 px on each side (WCAG 2.5.8).
//! - a group's title is never cut with an ellipsis: the showcase mold
//!   wraps it whole and pushes its cards down.
//!
//! The same sweep runs with every text pseudo-localized, a translation
//! 1.4 times as long (`--pseudo-locale`), as designed and at twice the
//! size; and in a low window (400 px high), each trigger opened in turn
//! (`sweep::low`).
//!
//! The shell's sidebar (front H's, peeking out of the window by default)
//! is left out, as in `hit_targets`.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

mod sweep;

use sweep::{Case, report};

#[test]
fn every_page_holds_text_as_designed() {
    report(&Case::scales(1.0));
}

#[test]
fn every_page_holds_text_half_again_as_large() {
    report(&Case::scales(1.5));
}

#[test]
fn every_page_holds_text_twice_as_large() {
    report(&Case::scales(2.0));
}

#[test]
fn every_page_holds_a_longer_translation() {
    report(&Case::scales(1.0).pseudo());
}

#[test]
fn every_page_holds_a_longer_translation_twice_as_large() {
    report(&Case::scales(2.0).pseudo());
}

#[test]
fn what_a_page_opens_fits_a_low_window() {
    sweep::low::report();
}
