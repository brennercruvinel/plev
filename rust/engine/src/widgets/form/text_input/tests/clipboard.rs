//! Copy, cut, paste and select all in the single-line field (task-74,
//! the person's request), on a `LocalClipboard`: no test here touches
//! the system's clipboard.

use crate::runtime::platform::clipboard::{ClipboardProvider, LocalClipboard};

use crate::widgets::form::text_input::*;
use crate::widgets::form::{ClipKey, TextField};

fn field(text: &str) -> TextField {
    let mut f = TextField::new("Full name").with_text(text);
    f.focus();
    f
}

fn holding(text: &str) -> LocalClipboard {
    let mut cb = LocalClipboard::new();
    cb.set_text(text);
    cb
}

#[test]
fn copy_takes_the_selection_and_leaves_the_text() {
    let mut cb = holding("before");
    let mut f = field("hello world");
    assert!(!f.has_selection());
    assert!(!f.copy(&mut cb), "nothing selected: nothing copied");
    assert_eq!(cb.get_text().as_deref(), Some("before"));

    f.input.buffer.set_cursor(6);
    f.input.buffer.select_to(11);
    assert!(f.has_selection());
    assert!(f.copy(&mut cb));
    assert_eq!(cb.get_text().as_deref(), Some("world"));
    assert_eq!(f.text(), "hello world");

    assert!(f.select_all());
    assert!(f.copy(&mut cb));
    assert_eq!(cb.get_text().as_deref(), Some("hello world"));
}

#[test]
fn cut_takes_the_selection_out() {
    let mut cb = holding("before");
    let mut f = field("hello world");
    assert!(!f.cut(&mut cb));
    assert_eq!(
        (f.text(), cb.get_text().as_deref()),
        ("hello world", Some("before"))
    );
    f.input.buffer.set_cursor(11);
    f.input.buffer.select_to(5);
    assert!(f.cut(&mut cb));
    assert_eq!(cb.get_text().as_deref(), Some(" world"));
    assert_eq!(f.text(), "hello");
    assert_eq!(f.input.buffer.cursor(), 5);
    assert!(!f.has_selection());
}

#[test]
fn paste_types_over_the_selection_on_one_line() {
    let mut cb = holding("big\r\nwide\nworld\n");
    let mut f = field("hello there");
    f.input.buffer.set_cursor(6);
    f.input.buffer.select_to(11);
    assert!(f.paste(&mut cb));
    assert_eq!(f.text(), "hello big wide world");
    assert_eq!(f.input.buffer.cursor(), f.text().len());

    // What typing the same text leaves.
    let mut typed = field("hello there");
    typed.input.buffer.set_cursor(6);
    typed.input.buffer.select_to(11);
    assert!(typed.insert("big wide world"));
    assert_eq!(typed.text(), f.text());

    let mut empty = LocalClipboard::new();
    assert!(!f.paste(&mut empty), "an empty clipboard pastes nothing");
    assert!(!f.paste(&mut holding("\n\r\n")), "nor only line breaks");
    assert_eq!(f.text(), "hello big wide world");
}

#[test]
fn a_single_line_keeps_the_words_and_drops_the_breaks() {
    assert_eq!(single_line("a\r\nb\rc\nd"), "a b c d");
    assert_eq!(single_line("\n\ntwo\n\nlines\n"), "two  lines");
    assert_eq!(single_line("tab\there"), "tab here");
    assert_eq!(single_line("bell\u{7}less"), "bellless");
    assert_eq!(single_line("日本語 مرحبا"), "日本語 مرحبا");
}

#[test]
fn an_unfocused_or_disabled_field_leaves_the_clipboard_alone() {
    let mut cb = holding("clip");
    let mut blurred = TextField::new("").with_text("abc");
    blurred.input.buffer.select_all();
    assert!(!blurred.copy(&mut cb));
    assert!(!blurred.cut(&mut cb));
    assert!(!blurred.paste(&mut cb));
    assert!(!blurred.select_all());
    assert_eq!(
        (blurred.text(), cb.get_text().as_deref()),
        ("abc", Some("clip"))
    );

    let mut off = field("abc").disabled(true);
    off.input.focus();
    off.input.buffer.select_all();
    assert!(!off.copy(&mut cb) && !off.cut(&mut cb) && !off.paste(&mut cb));
    assert_eq!(
        (off.text(), cb.get_text().as_deref()),
        ("abc", Some("clip"))
    );
}

/// The keys an app maps: cmd (ctrl) with c, x, v, a, either case.
#[test]
fn the_clipboard_keys_are_c_x_v_and_a() {
    assert_eq!(ClipKey::of("c"), Some(ClipKey::Copy));
    assert_eq!(ClipKey::of("X"), Some(ClipKey::Cut));
    assert_eq!(ClipKey::of("v"), Some(ClipKey::Paste));
    assert_eq!(ClipKey::of("a"), Some(ClipKey::SelectAll));
    assert_eq!(ClipKey::of("t"), None);
    assert_eq!(ClipKey::of("cv"), None);
    let mut cb = holding("pasted");
    let mut f = field("typed");
    assert!(f.clip(ClipKey::SelectAll, &mut cb));
    assert!(f.clip(ClipKey::Paste, &mut cb));
    assert_eq!(f.text(), "pasted");
    assert!(f.clip(ClipKey::SelectAll, &mut cb));
    assert!(f.clip(ClipKey::Cut, &mut cb));
    assert_eq!((f.text(), cb.get_text().as_deref()), ("", Some("pasted")));
    assert!(!f.clip(ClipKey::Copy, &mut cb), "nothing left to copy");
}

#[test]
fn the_core_pastes_and_cuts_as_the_field_does() {
    let mut cb = holding("x\ny");
    let mut ti = TextInput::new();
    assert!(!ti.handle_paste(&mut cb), "unfocused");
    ti.focus();
    assert!(ti.handle_paste(&mut cb));
    assert_eq!(ti.buffer.text(), "x y");
    ti.handle_select_all();
    assert!(ti.handle_cut(&mut cb));
    assert_eq!(
        (ti.buffer.text(), cb.get_text().as_deref()),
        ("", Some("x y"))
    );
}
