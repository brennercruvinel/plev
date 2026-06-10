use crate::text_input::*;

// -- TextBuffer tests --

#[test]
fn new_buffer_empty() {
    let buf = TextBuffer::new();
    assert!(buf.is_empty());
    assert_eq!(buf.cursor(), 0);
    assert_eq!(buf.text(), "");
}

#[test]
fn with_text() {
    let buf = TextBuffer::with_text("hello");
    assert_eq!(buf.text(), "hello");
    assert_eq!(buf.cursor(), 5);
}

#[test]
fn insert_char_basic() {
    let mut buf = TextBuffer::new();
    buf.insert_char('a');
    assert_eq!(buf.text(), "a");
    assert_eq!(buf.cursor(), 1);
    buf.insert_char('b');
    assert_eq!(buf.text(), "ab");
    assert_eq!(buf.cursor(), 2);
}

#[test]
fn insert_str() {
    let mut buf = TextBuffer::new();
    buf.insert_str("hello");
    assert_eq!(buf.text(), "hello");
    assert_eq!(buf.cursor(), 5);
}

#[test]
fn insert_at_middle() {
    let mut buf = TextBuffer::with_text("ac");
    buf.cursor = 1;
    buf.insert_char('b');
    assert_eq!(buf.text(), "abc");
    assert_eq!(buf.cursor(), 2);
}

#[test]
fn delete_back_basic() {
    let mut buf = TextBuffer::with_text("abc");
    buf.delete_back();
    assert_eq!(buf.text(), "ab");
    assert_eq!(buf.cursor(), 2);
}

#[test]
fn delete_back_at_start() {
    let mut buf = TextBuffer::with_text("abc");
    buf.cursor = 0;
    buf.delete_back();
    assert_eq!(buf.text(), "abc");
    assert_eq!(buf.cursor(), 0);
}

#[test]
fn delete_forward_basic() {
    let mut buf = TextBuffer::with_text("abc");
    buf.cursor = 0;
    buf.delete_forward();
    assert_eq!(buf.text(), "bc");
    assert_eq!(buf.cursor(), 0);
}

#[test]
fn delete_forward_at_end() {
    let mut buf = TextBuffer::with_text("abc");
    buf.delete_forward();
    assert_eq!(buf.text(), "abc");
}

#[test]
fn move_left_right() {
    let mut buf = TextBuffer::with_text("abc");
    buf.cursor = 2;
    buf.move_left();
    assert_eq!(buf.cursor(), 1);
    buf.move_right();
    assert_eq!(buf.cursor(), 2);
}

#[test]
fn move_left_at_start() {
    let mut buf = TextBuffer::with_text("abc");
    buf.cursor = 0;
    buf.move_left();
    assert_eq!(buf.cursor(), 0);
}

#[test]
fn move_right_at_end() {
    let mut buf = TextBuffer::with_text("abc");
    buf.move_right();
    assert_eq!(buf.cursor(), 3);
}

#[test]
fn move_home_end() {
    let mut buf = TextBuffer::with_text("hello world");
    buf.cursor = 5;
    buf.move_home();
    assert_eq!(buf.cursor(), 0);
    buf.move_end();
    assert_eq!(buf.cursor(), 11);
}

#[test]
fn select_all_and_get() {
    let mut buf = TextBuffer::with_text("hello");
    buf.select_all();
    assert_eq!(buf.selection(), Some((0, 5)));
    assert_eq!(buf.selected_text(), Some("hello"));
}

#[test]
fn select_all_empty() {
    let mut buf = TextBuffer::new();
    buf.select_all();
    assert_eq!(buf.selection(), None);
}

#[test]
fn delete_selection() {
    let mut buf = TextBuffer::with_text("hello world");
    buf.selection = Some((5, 11));
    buf.delete_selection();
    assert_eq!(buf.text(), "hello");
    assert_eq!(buf.cursor(), 5);
}

#[test]
fn insert_replaces_selection() {
    let mut buf = TextBuffer::with_text("hello");
    buf.select_all();
    buf.insert_char('x');
    assert_eq!(buf.text(), "x");
    assert_eq!(buf.cursor(), 1);
}

#[test]
fn backspace_deletes_selection() {
    let mut buf = TextBuffer::with_text("hello world");
    buf.selection = Some((0, 5));
    buf.delete_back();
    assert_eq!(buf.text(), " world");
    assert_eq!(buf.cursor(), 0);
}

// Multi-byte char tests
#[test]
fn multibyte_insert() {
    let mut buf = TextBuffer::new();
    buf.insert_char('\u{00e9}'); // e-acute
    assert_eq!(buf.text(), "\u{00e9}");
    assert_eq!(buf.cursor(), 2); // 2-byte UTF-8
}

#[test]
fn multibyte_delete_back() {
    let mut buf = TextBuffer::with_text("caf\u{00e9}");
    buf.delete_back();
    assert_eq!(buf.text(), "caf");
}

#[test]
fn multibyte_move() {
    let mut buf = TextBuffer::with_text("a\u{00e9}b");
    buf.cursor = 3; // after e-acute
    buf.move_left();
    assert_eq!(buf.cursor(), 1); // before e-acute
    buf.move_right();
    assert_eq!(buf.cursor(), 3); // after e-acute
}

#[test]
fn emoji_handling() {
    let mut buf = TextBuffer::new();
    buf.insert_str("hi ");
    buf.insert_char('\u{1F600}'); // grinning face, 4 bytes
    assert_eq!(buf.text(), "hi \u{1F600}");
    buf.delete_back();
    assert_eq!(buf.text(), "hi ");
}

#[test]
fn set_text_resets() {
    let mut buf = TextBuffer::with_text("old");
    buf.cursor = 1;
    buf.selection = Some((0, 2));
    buf.set_text("new");
    assert_eq!(buf.text(), "new");
    assert_eq!(buf.cursor(), 3);
    assert_eq!(buf.selection(), None);
}

// -- Cursor-pixel mapping tests --

#[test]
fn cursor_to_x_start() {
    assert!((cursor_to_x("hello", 0, 16.0) - 0.0).abs() < 0.01);
}

#[test]
fn cursor_to_x_end() {
    let x = cursor_to_x("hello", 5, 16.0);
    assert!((x - 48.0).abs() < 0.01); // 5 chars * 9.6 = 48.0
}

#[test]
fn x_to_cursor_start() {
    assert_eq!(x_to_cursor("hello", 0.0, 16.0), 0);
}

#[test]
fn x_to_cursor_end() {
    let pos = x_to_cursor("hello", 100.0, 16.0);
    assert_eq!(pos, 5); // capped at end
}

#[test]
fn x_to_cursor_middle() {
    let pos = x_to_cursor("hello", 24.0, 16.0);
    // ~24 / 9.6 = 2.5, rounds to 3
    assert!(pos >= 2 && pos <= 3);
}
