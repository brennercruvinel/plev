//! A masked input (a password): a bullet per grapheme drawn, the editing
//! on the text, nothing copied out of it; and `LineFit`, the place of a
//! line in a box narrower than it.

use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::runtime::platform::clipboard::{ClipboardProvider, LocalClipboard};

use crate::widgets::form::text_input::view::{LineFit, Shown};
use crate::widgets::form::text_input::*;

fn style() -> TextStyle {
    TextStyle::new(16.0).with_family("Inclusive Sans")
}

fn masked(text: &str) -> TextInput {
    let mut ti = TextInput::new();
    ti.masked = true;
    ti.buffer.set_text(text);
    ti.focus();
    ti
}

/// `e` and a combining acute are one grapheme, three bytes: one bullet.
#[test]
fn a_mask_draws_a_bullet_per_grapheme_and_maps_offsets_both_ways() {
    let text = "ae\u{301}b";
    let shown = Shown::new(text, true);
    let bullet = MASK.len_utf8();
    assert_eq!(shown.text(), MASK.to_string().repeat(3));
    assert_eq!(
        [0, 1, 4, 5].map(|p| shown.shown_pos(p)),
        [0, bullet, 2 * bullet, 3 * bullet]
    );
    assert_eq!(
        [0, bullet, 2 * bullet, 3 * bullet, 99].map(|p| shown.source_pos(p)),
        [0, 1, 4, 5, 5]
    );
    let plain = Shown::new(text, false);
    assert_eq!(
        (plain.text(), plain.shown_pos(4), plain.source_pos(4)),
        (text, 4, 4)
    );
    assert_eq!(Shown::new("", true).text(), "");
}

/// Bullets run left to right, so in a masked right-to-left text Right
/// goes forward; the caret and a click sit on the bullets.
#[test]
fn a_masked_input_moves_and_hits_on_its_bullets_in_text_order() {
    let mut ti = masked("שלום");
    ti.buffer.set_cursor(0);
    ti.handle_right();
    assert_eq!(ti.buffer.cursor(), 2, "one grapheme forward");
    let style = style();
    let one = TextMeasurer::measure_styled(&MASK.to_string(), &style, None).0;
    assert!((ti.caret_x(&style, None) - one).abs() < 0.5);
    let two = TextMeasurer::measure_styled(&MASK.to_string().repeat(2), &style, None).0;
    ti.handle_click_in(two + 0.1, &style, None);
    assert_eq!(ti.buffer.cursor(), 4, "after the second grapheme");
    assert_eq!(ti.shown(), MASK.to_string().repeat(4));
    assert_eq!(ti.buffer.text(), "שלום");
}

/// A selection of a masked text draws over its bullets.
#[test]
fn a_masked_selection_covers_its_bullets() {
    let mut ti = masked("hunter2");
    ti.handle_select_all();
    let style = style();
    let all = TextMeasurer::measure_styled(&MASK.to_string().repeat(7), &style, None).0;
    let boxes = ti.selection_boxes(&style, None);
    assert_eq!(boxes.len(), 1);
    assert!(boxes[0].0.abs() < 0.5 && (boxes[0].1 - all).abs() < 0.5);
}

/// Nothing leaves a masked input by the clipboard; paste still types.
#[test]
fn nothing_leaves_a_masked_input_by_copy_or_cut() {
    let mut ti = masked("hunter2");
    ti.handle_select_all();
    let mut cb = LocalClipboard::new();
    cb.set_text("before");
    assert!(!ti.handle_copy(&mut cb));
    assert!(!ti.handle_cut(&mut cb));
    assert_eq!(cb.get_text().as_deref(), Some("before"));
    assert_eq!(ti.buffer.text(), "hunter2");
    assert!(ti.handle_paste(&mut cb));
    assert_eq!(ti.buffer.text(), "before");
    ti.masked = false;
    ti.handle_select_all();
    assert!(ti.handle_copy(&mut cb), "unmasked, it copies");
}

/// A line that fits lays out in its box; a wider one is unbounded and
/// slides only as far as keeps the caret in, and not at all blurred.
#[test]
fn a_line_slides_only_when_wider_than_its_box_and_the_caret_would_leave() {
    assert_eq!(
        LineFit::new(80.0, 100.0, Some((80.0, 2.0))),
        LineFit {
            width: Some(100.0),
            scroll: 0.0,
            overflows: false
        }
    );
    let end = LineFit::new(300.0, 100.0, Some((300.0, 2.0)));
    assert_eq!((end.width, end.overflows), (None, true));
    assert_eq!(end.scroll, 202.0, "the caret's trailing edge on the box's");
    assert_eq!(LineFit::new(300.0, 100.0, Some((10.0, 2.0))).scroll, 0.0);
    assert_eq!(LineFit::new(300.0, 100.0, None).scroll, 0.0);
}
