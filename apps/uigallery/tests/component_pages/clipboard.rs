//! Copy, cut, paste and select all in the gallery's text fields (task-74,
//! the person's request), as the shell forwards a key with cmd or ctrl
//! held (`ShowcaseView::handle_chord`, winit-free): the focused field's
//! clipboard, never a letter typed; with no field focused, the key goes
//! where it went before. Every gallery here holds a `LocalClipboard`: no
//! test touches the system's clipboard.

use engine::runtime::platform::clipboard::{ClipboardProvider, LocalClipboard};
use accord::Value;

use super::harness::{Gallery, gallery};

const NAME: &str = "input_page.field[0]";
const EMAIL: &str = "input_page.field[1]";

/// The gallery on `page`, with a clipboard of its own holding `held`.
fn with_clipboard(page: &str, held: Option<&str>) -> Gallery {
    let mut g = gallery(page);
    let mut board = LocalClipboard::new();
    if let Some(text) = held {
        board.set_text(text);
    }
    g.view.set_clipboard(Box::new(board));
    g
}

fn text(g: &Gallery, field: &str) -> Option<Value> {
    g.value(&format!("{field}:text"))
}

/// Whether any node of the projection, its label or its state, holds
/// `text`.
fn projected(g: &Gallery, text: &str) -> bool {
    let snapshot = g.view.semantics().unwrap_or_default();
    snapshot.nodes().values().any(|n| {
        n.label.contains(text)
            || n.state
                .values()
                .any(|v| matches!(v, Value::Text(t) if t.contains(text)))
    })
}

#[test]
fn a_field_copies_cuts_and_pastes_through_the_clipboard_keys() {
    let mut g = with_clipboard("Input", None);
    g.click(NAME);
    g.type_in("Ada Lovelace");
    assert!(g.view.handle_chord("a", true), "select all");
    assert!(g.view.handle_chord("c", true), "copy");
    assert_eq!(text(&g, NAME), Some(Value::from("Ada Lovelace")));

    g.click(EMAIL);
    assert!(g.view.handle_chord("v", true), "paste");
    g.frame();
    assert_eq!(text(&g, EMAIL), Some(Value::from("Ada Lovelace")));

    g.click(NAME);
    assert!(g.view.handle_chord("A", true));
    assert!(g.view.handle_chord("x", true), "cut");
    g.frame();
    assert_eq!(text(&g, NAME), Some(Value::from("")));
    assert!(g.view.handle_chord("v", true));
    assert!(g.view.handle_chord("v", true));
    assert_eq!(
        text(&g, NAME),
        Some(Value::from("Ada LovelaceAda Lovelace"))
    );
}

/// Line breaks become spaces in a single-line field.
#[test]
fn a_paste_is_one_line() {
    let mut g = with_clipboard("Input", Some("first\r\nsecond\n"));
    g.click(NAME);
    assert!(g.view.handle_chord("v", true));
    assert_eq!(text(&g, NAME), Some(Value::from("first second")));
}

/// A letter with cmd or ctrl is never typed: not the clipboard keys with
/// the other modifier (ctrl on macOS), not any other letter, and the
/// shortcuts a bare letter has stay off while the field has the keys.
#[test]
fn a_key_with_cmd_or_ctrl_never_types_into_a_field() {
    let mut g = with_clipboard("Input", Some("clip"));
    g.click(NAME);
    let theme = g.view.theme_name.clone();
    assert!(!g.view.handle_chord("v", false), "not the primary modifier");
    assert!(!g.view.handle_chord("t", true));
    assert!(!g.view.handle_chord("1", true));
    assert!(!g.view.handle_chord("c", true), "nothing selected to copy");
    assert_eq!(text(&g, NAME), Some(Value::from("")));
    assert_eq!(g.view.theme_name, theme);
    assert_eq!(g.view.page_title(), "Input");
}

/// With no field focused the key goes where it went before (`t` still
/// turns the theme), and the clipboard is neither read nor written.
#[test]
fn with_no_field_focused_nothing_changes() {
    let mut g = with_clipboard("Input", Some("secret"));
    let before = g.state();
    assert!(!g.view.handle_chord("v", true));
    assert!(!g.view.handle_chord("c", true));
    assert_eq!(g.state(), before);
    let theme = g.view.theme_name.clone();
    assert!(g.view.handle_chord("t", true), "as handle_key(\"t\")");
    assert_ne!(g.view.theme_name, theme);
}

/// The clipboard is the person's: the projection an agent reads never
/// holds what it holds, and a field offers no action on it.
#[test]
fn the_clipboard_never_shows_in_the_projection() {
    let mut g = with_clipboard("Input", Some("secret"));
    g.click(NAME);
    g.type_in("abc");
    assert!(g.view.handle_chord("a", true));
    assert!(g.view.handle_chord("c", true));
    assert!(!projected(&g, "secret"));
    let snapshot = g.view.semantics().unwrap();
    for node in snapshot.nodes().values() {
        for action in &node.actions {
            let name = action.name.as_str();
            assert!(
                !["copy", "cut", "paste"].contains(&name),
                "{}: {name}",
                node.id
            );
        }
    }
}

/// The app pages' fields take the same keys: the todo's add field.
#[test]
fn the_todo_field_takes_the_clipboard_keys() {
    let mut g = with_clipboard("Todo", Some("buy milk"));
    g.click("todo.input");
    assert!(g.view.handle_chord("v", true));
    assert_eq!(text(&g, "todo.input"), Some(Value::from("buy milk")));
    assert!(!g.view.handle_chord("t", true), "never typed");
    assert_eq!(text(&g, "todo.input"), Some(Value::from("buy milk")));
}
