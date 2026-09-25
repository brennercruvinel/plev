//! Logical movement in mixed-direction text. The rope moves in text
//! order whatever the script's direction: `move_right` is the next
//! grapheme, `move_left` the previous one, and a selection is one
//! logical range. Visual order is the widget's, from the layout.

use ropey::Rope;

use crate::movement::{line_end, line_start, move_left, move_right, move_word_right};
use crate::selection::Selection;

const MIXED_HEBREW: &str = "abc אבג def";
const MIXED_ARABIC: &str = "hello مرحبا 123 world";
const HEBREW: &str = "שלום עולם";

/// Every caret stop `move_right` visits from the start, then the stops
/// `move_left` visits back from the end.
fn walks(text: &str) -> (Vec<usize>, Vec<usize>) {
    let r = Rope::from_str(text);
    let mut forward = vec![0];
    let mut pos = 0;
    loop {
        let next = move_right(&r, Selection::caret(pos), false).head;
        if next == pos {
            break;
        }
        forward.push(next);
        pos = next;
    }
    let mut back = vec![pos];
    loop {
        let prev = move_left(&r, Selection::caret(pos), false).head;
        if prev == pos {
            break;
        }
        back.push(prev);
        pos = prev;
    }
    (forward, back)
}

fn assert_walks(text: &str, expected: &[usize]) {
    let (forward, mut back) = walks(text);
    assert_eq!(forward, expected, "move_right through {text:?}");
    back.reverse();
    assert_eq!(back, expected, "move_left back through {text:?}");
}

#[test]
fn hebrew_inside_latin_moves_in_text_order() {
    // Two-byte letters inside the RTL run, one step each; the step order
    // is the byte order, not the screen order.
    assert_walks(MIXED_HEBREW, &[0, 1, 2, 3, 4, 6, 8, 10, 11, 12, 13, 14]);
}

#[test]
fn arabic_with_digits_inside_latin_moves_in_text_order() {
    // The digits sit at bidi level 2 inside the RTL run and still come
    // right after the Arabic word in text order.
    assert_walks(
        MIXED_ARABIC,
        &[
            0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 14, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        ],
    );
}

#[test]
fn right_to_left_paragraph_moves_in_text_order() {
    assert_walks(HEBREW, &[0, 2, 4, 6, 8, 9, 11, 13, 15, 17]);
}

#[test]
fn marks_ride_with_their_letter() {
    // Hebrew niqqud and Arabic harakat are combining marks: the letter
    // and its marks are one grapheme, one step.
    let niqqud = "שָׁלוֹם"; // shin+qamats+shin dot, lamed, vav+holam, final mem
    assert_walks(niqqud, &[0, 6, 8, 12, 14]);
    let harakat = "مَرْحَبًا"; // mim+fatha, reh+sukun, hah+fatha, beh+fathatan, alef
    assert_walks(harakat, &[0, 4, 8, 12, 16, 18]);
}

#[test]
fn a_selection_across_directions_is_one_logical_range() {
    let r = Rope::from_str(MIXED_HEBREW);
    let mut sel = Selection::caret(3);
    for _ in 0..3 {
        sel = move_right(&r, sel, true);
    }
    assert_eq!(sel, Selection::new(3, 8));
    assert_eq!(r.byte_slice(sel.min()..sel.max()).to_string(), " אב");
    // Back over the boundary, the anchor holds and the range shrinks.
    sel = move_left(&r, sel, true);
    sel = move_left(&r, sel, true);
    assert_eq!(sel, Selection::new(3, 4));
    // Collapsing goes to the logical ends.
    let wide = Selection::new(2, 12);
    assert_eq!(move_left(&r, wide, false), Selection::caret(2));
    assert_eq!(move_right(&r, wide, false), Selection::caret(12));
}

#[test]
fn line_ends_and_words_are_logical() {
    let r = Rope::from_str(MIXED_ARABIC);
    let mid = Selection::caret(10);
    assert_eq!(line_start(&r, mid, false), Selection::caret(0));
    assert_eq!(line_end(&r, mid, false), Selection::caret(26));
    let mut stops = Vec::new();
    let mut sel = Selection::caret(0);
    loop {
        let next = move_word_right(&r, sel, false);
        if next == sel {
            break;
        }
        stops.push(next.head);
        sel = next;
    }
    assert_eq!(
        stops,
        vec![5, 16, 20, 26],
        "hello, the Arabic word, 123, world"
    );
}
