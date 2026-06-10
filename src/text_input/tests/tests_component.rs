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
    ti.handle_click(24.0); // should position cursor around char 2-3
    assert!(ti.focused);
    assert!(ti.buffer.cursor() >= 2 && ti.buffer.cursor() <= 3);
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
