//! Headless editor tests: no GPU, no OS clipboard, real text shaping.

use std::cell::RefCell;
use std::rc::Rc;

use editor_core::{Document, Selection, SelectionSet};
use winit::event::Ime;
use winit::keyboard::{Key, ModifiersState, NamedKey};

use crate::compositor::{Compositor, SceneNode};
use crate::layout::ComputedBounds;

use super::view::visible_line_range;
use super::{ClipboardProvider, EditorTheme, EditorView, LocalClipboard, MouseEvent};

const VIEW_W: f32 = 800.0;
const VIEW_H: f32 = 420.0; // 20 lines at the default 21px line height

fn bounds() -> ComputedBounds {
    ComputedBounds {
        x: 0.0,
        y: 0.0,
        width: VIEW_W,
        height: VIEW_H,
    }
}

/// Editor over `text` with an in-memory clipboard and laid-out bounds.
fn editor(text: &str) -> EditorView {
    let mut ed =
        EditorView::new(Document::load(text)).with_clipboard(Box::new(LocalClipboard::new()));
    ed.set_bounds(bounds());
    ed
}

fn press(ed: &mut EditorView, key: Key, mods: ModifiersState) -> bool {
    ed.handle_key(&key, mods)
}

fn type_str(ed: &mut EditorView, text: &str) {
    for ch in text.chars() {
        press(
            ed,
            Key::Character(ch.to_string().into()),
            ModifiersState::empty(),
        );
    }
}

fn primary(ed: &EditorView) -> Selection {
    ed.document.selections().primary()
}

/// Clipboard whose storage the test can inspect.
#[derive(Clone, Default)]
struct SharedClipboard(Rc<RefCell<Option<String>>>);

impl ClipboardProvider for SharedClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.0.borrow().clone()
    }

    fn set_text(&mut self, text: &str) {
        *self.0.borrow_mut() = Some(text.to_string());
    }
}

// ---------------------------------------------------------------------------
// Virtualization
// ---------------------------------------------------------------------------

#[test]
fn visible_range_for_scroll_and_viewport() {
    // 100k lines, 20px lines, 600px viewport, scrolled to 40_000px.
    let r = visible_line_range(40_000.0, 600.0, 20.0, 100_000, 8);
    assert_eq!(r, (2000 - 8)..(2030 + 8));
}

#[test]
fn visible_range_clamps_at_document_edges() {
    assert_eq!(visible_line_range(0.0, 100.0, 20.0, 3, 4), 0..3);
    assert_eq!(visible_line_range(0.0, 100.0, 20.0, 1000, 0), 0..5);
    // Scrolled to the bottom of a 1000-line document.
    let r = visible_line_range(1000.0 * 20.0 - 100.0, 100.0, 20.0, 1000, 4);
    assert_eq!(r, (995 - 4)..1000);
    assert_eq!(visible_line_range(0.0, 100.0, 20.0, 0, 4), 0..0);
}

#[test]
fn render_emits_only_visible_lines_of_100k_doc() {
    let text: String = (0..100_000).map(|i| format!("line {i}\n")).collect();
    let mut ed = editor(&text);
    ed.scroll.scroll_to(50_000.0 * ed.config.line_height);

    let mut comp = Compositor::new();
    comp.begin_frame();
    ed.render(&mut comp, bounds(), &EditorTheme::default());

    let texts: Vec<&str> = comp.layers()[0]
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.as_str()),
            _ => None,
        })
        .collect();

    // 20 visible + 2*8 overscan lines, each emitting content + line number.
    let visible = ed.visible_lines();
    assert_eq!(visible.len(), 20 + 2 * ed.config.overscan_lines);
    assert_eq!(texts.len(), 2 * visible.len());
    assert!(texts.contains(&"line 50000"));
    assert!(!texts.contains(&"line 0"));
    assert!(!texts.contains(&"line 99999"));
}

// ---------------------------------------------------------------------------
// Mouse: hit-testing, clicks, drag, wheel
// ---------------------------------------------------------------------------

#[test]
fn click_positions_cursor_with_scroll() {
    let line = "0123456789";
    let text = format!("{line}\n").repeat(100);
    let mut ed = editor(&text);
    let lh = ed.config.line_height;
    ed.scroll.scroll_to(50.0 * lh); // line 50 is the first visible line

    // Click in the vertical middle of the first visible row, at the caret
    // x of byte 3 (nudged right so the hit lands inside char 3).
    let x = ed.text_origin_x() + ed.caret_x(line, 3) + 1.0;
    assert!(ed.handle_mouse(MouseEvent::Down {
        x,
        y: lh * 0.5,
        alt: false,
        shift: false,
    }));
    let expected = ed.document.rope().line_to_byte(50) + 3;
    assert_eq!(primary(&ed), Selection::caret(expected));
}

#[test]
fn click_past_line_end_clamps_to_content_end() {
    let mut ed = editor("ab\ncdef");
    let y = ed.config.line_height * 0.5;
    ed.handle_mouse(MouseEvent::Down {
        x: VIEW_W - 10.0,
        y,
        alt: false,
        shift: false,
    });
    assert_eq!(primary(&ed), Selection::caret(2));
}

#[test]
fn drag_extends_selection() {
    let line = "hello world";
    let mut ed = editor(line);
    let y = ed.config.line_height * 0.5;
    let x0 = ed.text_origin_x() + ed.caret_x(line, 0);
    let x5 = ed.text_origin_x() + ed.caret_x(line, 5) + 1.0;
    ed.handle_mouse(MouseEvent::Down {
        x: x0,
        y,
        alt: false,
        shift: false,
    });
    ed.handle_mouse(MouseEvent::Drag { x: x5, y });
    assert_eq!(primary(&ed), Selection::new(0, 5));
    assert!(ed.handle_mouse(MouseEvent::Up));
    // Further motion after release does nothing.
    assert!(!ed.handle_mouse(MouseEvent::Drag { x: x0, y }));
}

#[test]
fn double_click_selects_word() {
    let line = "hello world foo";
    let mut ed = editor(line);
    let y = ed.config.line_height * 0.5;
    // Inside "world" (bytes 6..11).
    let x = ed.text_origin_x() + ed.caret_x(line, 8) + 1.0;
    ed.handle_mouse(MouseEvent::Down {
        x,
        y,
        alt: false,
        shift: false,
    });
    ed.handle_mouse(MouseEvent::Up);
    ed.handle_mouse(MouseEvent::Down {
        x,
        y,
        alt: false,
        shift: false,
    });
    assert_eq!(primary(&ed), Selection::new(6, 11));
}

#[test]
fn double_click_on_accented_word() {
    let line = "ção mundo";
    let mut ed = editor(line);
    let y = ed.config.line_height * 0.5;
    let x = ed.text_origin_x() + ed.caret_x(line, 2) + 1.0;
    ed.handle_mouse(MouseEvent::Down {
        x,
        y,
        alt: false,
        shift: false,
    });
    ed.handle_mouse(MouseEvent::Up);
    ed.handle_mouse(MouseEvent::Down {
        x,
        y,
        alt: false,
        shift: false,
    });
    assert_eq!(primary(&ed), Selection::new(0, 5)); // "ção" is 5 bytes
}

#[test]
fn triple_click_selects_line_including_newline() {
    let mut ed = editor("first\nsecond\nthird");
    let y = ed.config.line_height * 1.5; // middle of line 1
    let x = ed.text_origin_x() + 1.0;
    for _ in 0..3 {
        ed.handle_mouse(MouseEvent::Down {
            x,
            y,
            alt: false,
            shift: false,
        });
        ed.handle_mouse(MouseEvent::Up);
    }
    assert_eq!(primary(&ed), Selection::new(6, 13)); // "second\n"
}

#[test]
fn alt_click_adds_cursor_and_typing_edits_all() {
    let mut ed = editor("aa\nbb");
    let y0 = ed.config.line_height * 0.5;
    let y1 = ed.config.line_height * 1.5;
    let x = ed.text_origin_x();
    ed.handle_mouse(MouseEvent::Down {
        x,
        y: y0,
        alt: false,
        shift: false,
    });
    ed.handle_mouse(MouseEvent::Up);
    ed.handle_mouse(MouseEvent::Down {
        x,
        y: y1,
        alt: true,
        shift: false,
    });
    assert_eq!(ed.document.selections().len(), 2);

    type_str(&mut ed, "x");
    assert_eq!(ed.document.to_string(), "xaa\nxbb");
}

#[test]
fn shift_click_extends_from_existing_caret() {
    let line = "hello world";
    let mut ed = editor(line);
    ed.document.set_selections(SelectionSet::caret(2));
    let y = ed.config.line_height * 0.5;
    let x = ed.text_origin_x() + ed.caret_x(line, 7) + 1.0;
    ed.handle_mouse(MouseEvent::Down {
        x,
        y,
        alt: false,
        shift: true,
    });
    assert_eq!(primary(&ed), Selection::new(2, 7));
}

#[test]
fn wheel_scrolls_and_clamps() {
    let text = "x\n".repeat(100);
    let mut ed = editor(&text);
    assert!(ed.handle_mouse(MouseEvent::Wheel { dy: 30.0 }));
    assert_eq!(ed.scroll.offset(), 30.0);
    // Scrolling above the top is a no-op.
    assert!(ed.handle_mouse(MouseEvent::Wheel { dy: -100.0 }));
    assert!(!ed.handle_mouse(MouseEvent::Wheel { dy: -10.0 }));
    assert_eq!(ed.scroll.offset(), 0.0);
}

// ---------------------------------------------------------------------------
// Keyboard editing
// ---------------------------------------------------------------------------

#[test]
fn typing_inserts_at_cursor() {
    let mut ed = editor("");
    type_str(&mut ed, "héllo");
    assert_eq!(ed.document.to_string(), "héllo");
    assert_eq!(primary(&ed), Selection::caret("héllo".len()));
}

#[test]
fn enter_tab_backspace_delete() {
    let mut ed = editor("");
    type_str(&mut ed, "fn");
    press(
        &mut ed,
        Key::Named(NamedKey::Enter),
        ModifiersState::empty(),
    );
    press(&mut ed, Key::Named(NamedKey::Tab), ModifiersState::empty());
    assert_eq!(ed.document.to_string(), "fn\n    ");

    press(
        &mut ed,
        Key::Named(NamedKey::Backspace),
        ModifiersState::empty(),
    );
    assert_eq!(ed.document.to_string(), "fn\n   ");

    ed.document.set_selections(SelectionSet::caret(0));
    press(
        &mut ed,
        Key::Named(NamedKey::Delete),
        ModifiersState::empty(),
    );
    assert_eq!(ed.document.to_string(), "n\n   ");
}

#[test]
fn typing_replaces_multi_cursor_selections() {
    let mut ed = editor("foo bar foo");
    let mut sels = SelectionSet::caret(0);
    assert!(sels.select_all_matches(ed.document.rope(), "foo"));
    ed.document.set_selections(sels);
    type_str(&mut ed, "qux");
    assert_eq!(ed.document.to_string(), "qux bar qux");
}

#[test]
fn arrows_move_and_shift_extends() {
    let mut ed = editor("ab cd");
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowRight),
        ModifiersState::empty(),
    );
    assert_eq!(primary(&ed), Selection::caret(1));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowRight),
        ModifiersState::SHIFT,
    );
    assert_eq!(primary(&ed), Selection::new(1, 2));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowLeft),
        ModifiersState::empty(),
    );
    assert_eq!(primary(&ed), Selection::caret(1));
}

#[test]
fn alt_arrows_move_by_word() {
    let mut ed = editor("foo bar_baz qux");
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowRight),
        ModifiersState::ALT,
    );
    assert_eq!(primary(&ed), Selection::caret(3));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowRight),
        ModifiersState::ALT,
    );
    assert_eq!(primary(&ed), Selection::caret(11));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowLeft),
        ModifiersState::ALT,
    );
    assert_eq!(primary(&ed), Selection::caret(4));
}

#[test]
fn cmd_arrows_jump_to_line_edges_and_document_edges() {
    let mut ed = editor("hello\nworld");
    ed.document.set_selections(SelectionSet::caret(8));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowLeft),
        ModifiersState::SUPER,
    );
    assert_eq!(primary(&ed), Selection::caret(6));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowRight),
        ModifiersState::SUPER,
    );
    assert_eq!(primary(&ed), Selection::caret(11));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowUp),
        ModifiersState::SUPER,
    );
    assert_eq!(primary(&ed), Selection::caret(0));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowDown),
        ModifiersState::SUPER,
    );
    assert_eq!(primary(&ed), Selection::caret(11));
}

#[test]
fn vertical_movement_keeps_goal_column() {
    let mut ed = editor("abcdef\nxy\nabcdef");
    ed.document.set_selections(SelectionSet::caret(4));
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowDown),
        ModifiersState::empty(),
    );
    assert_eq!(primary(&ed), Selection::caret(9)); // clamped to end of "xy"
    press(
        &mut ed,
        Key::Named(NamedKey::ArrowDown),
        ModifiersState::empty(),
    );
    assert_eq!(primary(&ed), Selection::caret(14)); // back to column 4
}

#[test]
fn smart_home_and_end() {
    let mut ed = editor("  indent");
    ed.document.set_selections(SelectionSet::caret(8));
    press(&mut ed, Key::Named(NamedKey::Home), ModifiersState::empty());
    assert_eq!(primary(&ed), Selection::caret(2)); // first non-whitespace
    press(&mut ed, Key::Named(NamedKey::Home), ModifiersState::empty());
    assert_eq!(primary(&ed), Selection::caret(0)); // toggles to column 0
    press(&mut ed, Key::Named(NamedKey::End), ModifiersState::empty());
    assert_eq!(primary(&ed), Selection::caret(8));
}

#[test]
fn page_down_moves_a_viewport_and_follows_cursor() {
    let text = "x\n".repeat(100);
    let mut ed = editor(&text);
    let page = (VIEW_H / ed.config.line_height).floor() as usize;
    press(
        &mut ed,
        Key::Named(NamedKey::PageDown),
        ModifiersState::empty(),
    );
    let line = ed.document.rope().byte_to_line(primary(&ed).head);
    assert_eq!(line, page);
    // The cursor was scrolled into view.
    let visible = ed.visible_lines();
    assert!(visible.contains(&line));
    press(
        &mut ed,
        Key::Named(NamedKey::PageUp),
        ModifiersState::empty(),
    );
    assert_eq!(primary(&ed), Selection::caret(0));
}

#[test]
fn cmd_a_selects_all() {
    let mut ed = editor("hello\nworld");
    press(&mut ed, Key::Character("a".into()), ModifiersState::SUPER);
    assert_eq!(primary(&ed), Selection::new(0, 11));
}

#[test]
fn cmd_z_undoes_and_cmd_shift_z_redoes() {
    let mut ed = editor("base");
    ed.document.set_selections(SelectionSet::caret(4));
    type_str(&mut ed, "hi"); // word-like: coalesces into one undo group
    assert_eq!(ed.document.to_string(), "basehi");

    press(&mut ed, Key::Character("z".into()), ModifiersState::SUPER);
    assert_eq!(ed.document.to_string(), "base");
    assert_eq!(primary(&ed), Selection::caret(4));

    press(
        &mut ed,
        Key::Character("z".into()),
        ModifiersState::SUPER | ModifiersState::SHIFT,
    );
    assert_eq!(ed.document.to_string(), "basehi");
}

#[test]
fn unbound_cmd_keys_are_not_consumed() {
    let mut ed = editor("x");
    assert!(!press(
        &mut ed,
        Key::Character("s".into()),
        ModifiersState::SUPER
    ));
    assert_eq!(ed.document.to_string(), "x");
}

// ---------------------------------------------------------------------------
// Clipboard
// ---------------------------------------------------------------------------

#[test]
fn copy_cut_paste_round_trip() {
    let clipboard = SharedClipboard::default();
    let mut ed =
        EditorView::new(Document::load("hello world")).with_clipboard(Box::new(clipboard.clone()));
    ed.set_bounds(bounds());

    ed.document
        .set_selections(SelectionSet::single(Selection::new(0, 5)));
    assert!(press(
        &mut ed,
        Key::Character("c".into()),
        ModifiersState::SUPER
    ));
    assert_eq!(clipboard.0.borrow().as_deref(), Some("hello"));

    ed.document.set_selections(SelectionSet::caret(11));
    assert!(press(
        &mut ed,
        Key::Character("v".into()),
        ModifiersState::SUPER
    ));
    assert_eq!(ed.document.to_string(), "hello worldhello");

    ed.document
        .set_selections(SelectionSet::single(Selection::new(5, 11)));
    assert!(press(
        &mut ed,
        Key::Character("x".into()),
        ModifiersState::SUPER
    ));
    assert_eq!(clipboard.0.borrow().as_deref(), Some(" world"));
    assert_eq!(ed.document.to_string(), "hellohello");
}

#[test]
fn copy_with_caret_only_is_noop() {
    let clipboard = SharedClipboard::default();
    let mut ed = EditorView::new(Document::load("abc")).with_clipboard(Box::new(clipboard.clone()));
    ed.set_bounds(bounds());
    assert!(!press(
        &mut ed,
        Key::Character("c".into()),
        ModifiersState::SUPER
    ));
    assert!(clipboard.0.borrow().is_none());
}

#[test]
fn multi_cursor_copy_paste_distributes_pieces() {
    let clipboard = SharedClipboard::default();
    let mut ed =
        EditorView::new(Document::load("one\ntwo")).with_clipboard(Box::new(clipboard.clone()));
    ed.set_bounds(bounds());

    // Select "one" and "two" with two cursors, copy both.
    ed.document.set_selections(SelectionSet::new(
        vec![Selection::new(0, 3), Selection::new(4, 7)],
        0,
    ));
    press(&mut ed, Key::Character("c".into()), ModifiersState::SUPER);
    assert_eq!(clipboard.0.borrow().as_deref(), Some("one\ntwo"));

    // Two carets at line ends: each receives its own piece.
    ed.document.set_selections(SelectionSet::new(
        vec![Selection::caret(3), Selection::caret(7)],
        0,
    ));
    press(&mut ed, Key::Character("v".into()), ModifiersState::SUPER);
    assert_eq!(ed.document.to_string(), "oneone\ntwotwo");
}

#[test]
fn paste_with_mismatched_counts_inserts_whole_text() {
    let clipboard = SharedClipboard::default();
    clipboard.0.borrow_mut().replace("a\nb\nc".to_string());
    let mut ed =
        EditorView::new(Document::load("x\ny")).with_clipboard(Box::new(clipboard.clone()));
    ed.set_bounds(bounds());

    ed.document.set_selections(SelectionSet::new(
        vec![Selection::caret(1), Selection::caret(3)],
        0,
    ));
    press(&mut ed, Key::Character("v".into()), ModifiersState::SUPER);
    assert_eq!(ed.document.to_string(), "xa\nb\nc\nya\nb\nc");
}

// ---------------------------------------------------------------------------
// Undo via mouse + keyboard interplay
// ---------------------------------------------------------------------------

#[test]
fn undo_restores_text_and_selection_after_mouse_edit() {
    let line = "hello world";
    let mut ed = editor(line);
    let y = ed.config.line_height * 0.5;
    let x = ed.text_origin_x() + ed.caret_x(line, 8) + 1.0;
    // Double-click "world", overtype it.
    ed.handle_mouse(MouseEvent::Down {
        x,
        y,
        alt: false,
        shift: false,
    });
    ed.handle_mouse(MouseEvent::Up);
    ed.handle_mouse(MouseEvent::Down {
        x,
        y,
        alt: false,
        shift: false,
    });
    type_str(&mut ed, "rust");
    assert_eq!(ed.document.to_string(), "hello rust");

    press(&mut ed, Key::Character("z".into()), ModifiersState::SUPER);
    assert_eq!(ed.document.to_string(), "hello world");
    assert_eq!(primary(&ed), Selection::new(6, 11));
}

// ---------------------------------------------------------------------------
// IME
// ---------------------------------------------------------------------------

#[test]
fn ime_preedit_renders_inline_and_commit_inserts() {
    let mut ed = editor("ab");
    ed.document.set_selections(SelectionSet::caret(1));

    assert!(ed.handle_ime(&Ime::Preedit("にほ".to_string(), Some((6, 6)))));
    assert_eq!(ed.preedit().unwrap().text, "にほ");
    // Composing keys belong to the IME, not the editor.
    assert!(!press(
        &mut ed,
        Key::Character("x".into()),
        ModifiersState::empty()
    ));

    // The preedit is spliced into the rendered line, document untouched.
    let mut comp = Compositor::new();
    comp.begin_frame();
    ed.render(&mut comp, bounds(), &EditorTheme::default());
    let texts: Vec<&str> = comp.layers()[0]
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.as_str()),
            _ => None,
        })
        .collect();
    assert!(texts.contains(&"aにほb"));
    assert_eq!(ed.document.to_string(), "ab");

    assert!(ed.handle_ime(&Ime::Commit("日本".to_string())));
    assert!(ed.preedit().is_none());
    assert_eq!(ed.document.to_string(), "a日本b");
    assert_eq!(primary(&ed), Selection::caret(1 + "日本".len()));
}

#[test]
fn ime_disabled_clears_preedit() {
    let mut ed = editor("ab");
    ed.handle_ime(&Ime::Preedit("x".to_string(), None));
    assert!(ed.handle_ime(&Ime::Disabled));
    assert!(ed.preedit().is_none());
    assert!(!ed.handle_ime(&Ime::Disabled));
}

#[test]
fn ime_cursor_rect_tracks_preedit_caret() {
    let line = "ab";
    let mut ed = editor(line);
    ed.document.set_selections(SelectionSet::caret(1));
    let base = ed.ime_cursor_rect();
    assert_eq!(base.x, ed.text_origin_x() + ed.caret_x(line, 1));
    assert_eq!(base.height, ed.config.line_height);

    ed.handle_ime(&Ime::Preedit("にほ".to_string(), Some((6, 6))));
    let composing = ed.ime_cursor_rect();
    assert!(composing.x > base.x); // caret sits after the preedit text
}

// ---------------------------------------------------------------------------
// Cursor blink
// ---------------------------------------------------------------------------

#[test]
fn cursor_blinks_at_configured_interval() {
    let mut ed = editor("x");
    ed.config.cursor_blink_interval = 0.5;
    assert!(!ed.tick(0.3));
    assert!(ed.tick(0.3)); // 0.6s elapsed: toggled off
    assert!(!ed.cursor_visible);
    assert!(ed.tick(0.5));
    assert!(ed.cursor_visible);

    ed.tick(0.6);
    ed.reset_blink();
    assert!(ed.cursor_visible);
}

#[test]
fn hidden_cursor_emits_no_primary_caret_rect() {
    let mut ed = editor("x");
    let theme = EditorTheme::default();

    let count_cursor_rects = |ed: &mut EditorView| {
        let mut comp = Compositor::new();
        comp.begin_frame();
        ed.render(&mut comp, bounds(), &theme);
        comp.layers()[0]
            .nodes()
            .iter()
            .filter(|n| matches!(n, SceneNode::Rect { color, w, .. } if *color == theme.cursor && *w == 2.0))
            .count()
    };

    assert_eq!(count_cursor_rects(&mut ed), 1);
    while !ed.tick(0.1) {}
    assert_eq!(count_cursor_rects(&mut ed), 0);
}

// ---------------------------------------------------------------------------
// Gutter
// ---------------------------------------------------------------------------

#[test]
fn gutter_width_adapts_to_line_count() {
    let small = editor("a\nb");
    let large = editor(&"x\n".repeat(99_999));
    assert!(large.gutter_width() > small.gutter_width());

    let mut no_gutter = editor("a");
    no_gutter.config.show_gutter = false;
    assert_eq!(no_gutter.gutter_width(), 0.0);
}
