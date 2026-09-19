use crate::text_input::*;

#[test]
fn text_input_new() {
    let ti = TextInput::new();
    assert!(!ti.focused);
    assert!(ti.buffer.is_empty());
}

#[test]
fn text_input_focus_unfocus() {
    let mut ti = TextInput::new();
    ti.focus();
    assert!(ti.focused);
    ti.unfocus();
    assert!(!ti.focused);
}

#[test]
fn text_input_typing() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.handle_char('h');
    ti.handle_char('i');
    assert_eq!(ti.buffer.text(), "hi");
}

#[test]
fn text_input_typing_unfocused_ignored() {
    let mut ti = TextInput::new();
    ti.handle_char('x');
    assert!(ti.buffer.is_empty());
}

#[test]
fn text_input_backspace() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.handle_char('a');
    ti.handle_char('b');
    ti.handle_backspace();
    assert_eq!(ti.buffer.text(), "a");
}

#[test]
fn text_input_blink() {
    let mut ti = TextInput::new();
    ti.focus();
    assert!(ti.cursor_visible);
    ti.tick(0.54);
    assert!(!ti.cursor_visible);
    ti.tick(0.54);
    assert!(ti.cursor_visible);
}

#[test]
fn text_input_reset_blink_on_type() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.tick(0.54); // blink off
    assert!(!ti.cursor_visible);
    ti.handle_char('a'); // resets blink
    assert!(ti.cursor_visible);
}

#[test]
fn text_input_click_positions_cursor() {
    let mut ti = TextInput::new();
    ti.buffer.set_text("hello");
    // Click exactly where the real caret for byte 3 sits.
    let x = crate::text::TextMeasurer::cursor_x("hello", 16.0, 3);
    ti.handle_click(x);
    assert!(ti.focused);
    assert_eq!(ti.buffer.cursor(), 3);
}

#[test]
fn text_input_click_proportional_narrow_chars() {
    // Narrow glyphs ('i') in a proportional font: the old fixed-ratio
    // (0.6 * font_size) mapping landed on the wrong char here.
    let mut ti = TextInput::new();
    let text = "iiiiiiiiii";
    ti.buffer.set_text(text);

    let x = crate::text::TextMeasurer::cursor_x(text, 16.0, 7);
    ti.handle_click(x);
    assert_eq!(ti.buffer.cursor(), 7);

    // The same x through the old heuristic would have missed.
    let heuristic_cursor = (x / (16.0 * 0.6)).round() as usize;
    assert_ne!(
        heuristic_cursor, 7,
        "narrow proportional glyphs must defeat the fixed-ratio mapping"
    );
}

#[test]
fn text_input_click_middle_of_glyph_rounds_to_nearest_boundary() {
    let mut ti = TextInput::new();
    ti.buffer.set_text("hello");
    let b2 = crate::text::TextMeasurer::cursor_x("hello", 16.0, 2);
    let b3 = crate::text::TextMeasurer::cursor_x("hello", 16.0, 3);
    // Click slightly left of the midpoint of the third glyph -> cursor 2.
    ti.handle_click(b2 + (b3 - b2) * 0.25);
    assert_eq!(ti.buffer.cursor(), 2);
    // Click slightly right of the midpoint -> cursor 3.
    ti.handle_click(b2 + (b3 - b2) * 0.75);
    assert_eq!(ti.buffer.cursor(), 3);
}

#[test]
fn text_input_cursor_x_round_trip_all_positions() {
    let mut ti = TextInput::new();
    let text = "Wide and iiii";
    ti.buffer.set_text(text);
    let mut cursors: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    cursors.push(text.len());
    for cursor in cursors {
        let x = crate::text::TextMeasurer::cursor_x(text, 16.0, cursor);
        ti.handle_click(x);
        assert_eq!(
            ti.buffer.cursor(),
            cursor,
            "click round-trip at byte {cursor}"
        );
    }
}

#[test]
fn text_input_build_scene_empty_unfocused() {
    let ti = TextInput::new().with_placeholder("Type here...");
    let nodes = ti.build_scene(0.0, 0.0, 200.0);
    // Should have: bg rect + placeholder text
    assert!(nodes.len() >= 2);
    assert!(matches!(
        &nodes[0],
        crate::compositor::SceneNode::Rect { .. }
    ));
}

#[test]
fn text_input_build_scene_focused_with_text() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.handle_char('h');
    ti.handle_char('i');
    let nodes = ti.build_scene(0.0, 0.0, 200.0);
    // bg + 4 borders + text + cursor = 7
    assert!(nodes.len() >= 5);
}

#[test]
fn text_input_handle_ime() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.handle_ime("hello", "");
    assert_eq!(ti.buffer.text(), "hello");
    ti.handle_ime(" world", "preedit");
    assert_eq!(ti.buffer.text(), "hello world");
}

#[test]
fn text_input_handle_ime_unfocused() {
    let mut ti = TextInput::new();
    ti.handle_ime("ignored", "");
    assert!(ti.buffer.is_empty());
}

#[test]
fn text_input_build_scene_selection() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.buffer.set_text("hello");
    ti.buffer.select_all();
    let nodes = ti.build_scene(0.0, 0.0, 200.0);
    // Should include selection rect
    assert!(nodes.len() >= 6);
}
