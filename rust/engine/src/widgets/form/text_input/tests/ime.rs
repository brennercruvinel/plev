//! The input method in the single-line field (task-74 phase 4): the
//! preedit is kept apart from the buffer and drawn inline, underlined,
//! with the caret at the input method's; a commit is typing, over the
//! selection; a blur or the input method going away drops the preedit;
//! the projection never holds it. Japanese on the desktop shapes with the
//! system's fonts, so the geometry is checked against the same measurer
//! with the same style, never against numbers.

use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::text::TextMeasurer;
use engine::theme::Theme;
use sem::{SemId, SnapshotBuilder, StateRef, Value};

use crate::core::Rect;
use crate::form::TextField;
use crate::form::text_input::*;
use crate::wake::Wake;

/// `にほんご`: four kana of three bytes each.
const NIHONGO: &str = "にほんご";

fn field(text: &str) -> TextField {
    let mut f = TextField::new("Full name").with_text(text);
    f.focus();
    f
}

fn bounds(theme: &Theme) -> Rect {
    Rect::new(16.0, 16.0, 320.0, TextField::height(theme))
}

fn drawn(f: &TextField, theme: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    f.render(&mut c, bounds(theme), theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

#[test]
fn the_preedit_is_kept_apart_from_the_buffer_at_the_caret() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.buffer.set_text("abcd");
    ti.buffer.set_cursor(2);
    ti.handle_ime_at("", NIHONGO, Some((3, 6)));
    assert_eq!(ti.buffer.text(), "abcd", "the buffer holds no preedit");
    assert_eq!(ti.buffer.cursor(), 2);
    assert_eq!(ti.preedit(), NIHONGO);
    let (composed, span) = ti.composed().expect("composing");
    assert_eq!(composed, "abにほんごcd");
    assert_eq!(span, 2..14);

    // The next step replaces it; an empty one ends the composition.
    ti.handle_ime_at("", "に", None);
    assert_eq!(ti.composed().unwrap().0, "abにcd");
    ti.handle_ime_at("", "", None);
    assert_eq!(ti.preedit(), "");
    assert!(ti.composed().is_none());
    assert_eq!(ti.buffer.text(), "abcd");

    // Unfocused, the input method reaches nothing.
    let mut blurred = TextInput::new();
    blurred.handle_ime_at("x", NIHONGO, None);
    assert!(blurred.buffer.is_empty() && blurred.preedit().is_empty());
}

#[test]
fn a_commit_is_typing_over_the_selection_and_ends_the_preedit() {
    let theme = Theme::hoff();
    let style = TextField::text_style(&theme);
    let mut f = field("hello world");
    f.input.buffer.set_cursor(6);
    f.input.buffer.select_to(11);
    assert!(f.compose("", "世界", None));
    let (composed, span) = f.input.composed().unwrap();
    assert_eq!(
        composed, "hello 世界",
        "the preedit draws over the selection"
    );
    assert_eq!(span, 6..12);
    assert!(
        f.input.selection_boxes(&style, None).is_empty(),
        "no wash while composing"
    );
    assert_eq!(f.text(), "hello world");

    // winit clears the preedit, then commits.
    assert!(f.compose("", "", None));
    assert!(f.compose("世界", "", None));
    assert_eq!(f.text(), "hello 世界");
    assert_eq!(f.input.buffer.cursor(), 12);
    assert_eq!(f.input.buffer.selection(), None);
    assert_eq!(f.input.preedit(), "");

    // The commit is what typing the same text leaves.
    let mut typed = field("hello world");
    typed.input.buffer.set_cursor(6);
    typed.input.buffer.select_to(11);
    assert!(typed.insert("世界"));
    assert_eq!(typed.text(), f.text());
    assert_eq!(typed.input.buffer.cursor(), f.input.buffer.cursor());

    // A commit and the next preedit in one step (Korean does this).
    assert!(f.compose("!", "に", None));
    assert_eq!(f.text(), "hello 世界!");
    assert_eq!(f.input.preedit(), "に");
}

#[test]
fn a_blur_or_the_input_method_going_away_drops_the_preedit() {
    let mut f = field("ab");
    assert!(f.compose("", NIHONGO, None));
    f.unfocus();
    assert_eq!(f.input.preedit(), "", "the blur drops it");
    assert_eq!(f.text(), "ab", "and keeps the text");
    assert!(!f.compose("", NIHONGO, None), "blurred: nothing composes");

    f.focus();
    assert!(f.compose("", NIHONGO, None));
    assert!(f.end_compose(), "Ime::Disabled drops it");
    assert_eq!((f.text(), f.input.preedit()), ("ab", ""));
    assert!(!f.end_compose(), "nothing left to drop");

    let mut off = field("ab").disabled(true);
    assert!(
        !off.compose("x", NIHONGO, None),
        "a disabled field takes none"
    );
    assert_eq!((off.text(), off.input.preedit()), ("ab", ""));
}

/// A caret that does not fall on a character, or past the preedit, is
/// put at its end rather than panic.
#[test]
fn an_input_method_caret_off_a_character_goes_to_the_end() {
    let mut ti = TextInput::new();
    ti.focus();
    let style = TextField::text_style(&Theme::hoff());
    for cursor in [Some((1, 2)), Some((99, 99)), None] {
        ti.handle_ime_at("", NIHONGO, cursor);
        let end = TextMeasurer::cursor_x_styled(NIHONGO, &style, None, NIHONGO.len());
        assert_eq!(ti.caret_x(&style, None), end, "{cursor:?}");
    }
}

#[test]
fn the_field_draws_the_preedit_underlined_with_the_caret_at_the_input_methods() {
    let theme = Theme::hoff();
    let style = TextField::text_style(&theme);
    let b = bounds(&theme);
    let pad = theme.spacing.md;
    let text_w = b.w - pad * 2.0;
    let origin = b.x + pad;
    let mut f = field("ab");
    assert!(f.compose("", NIHONGO, Some((3, 6))));
    // The caret stays solid while composing, on the blink's hidden half
    // too, and asks for no blink frames.
    assert_eq!(f.wake(0.6), Wake::idle());
    assert!(f.input.cursor_visible());

    let nodes = drawn(&f, &theme);
    let texts: Vec<&str> = nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, .. } => Some(key.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, vec!["abにほんご"]);

    let x_at =
        |pos| origin + TextMeasurer::cursor_x_styled("abにほんご", &style, Some(text_w), pos);
    let underline = theme.control.edge_width;
    let lines: Vec<(f32, f32, f32, f32)> = nodes
        .iter()
        .filter_map(|n| match *n {
            SceneNode::Rect { x, y, w, h, color } if color == theme.glass.text_active.0 => {
                Some((x, y, w, h))
            }
            _ => None,
        })
        .collect();
    let text_y = b.y + TextMeasurer::vertical_center(&style, b.h);
    assert_eq!(lines.len(), 1, "one underline: {lines:?}");
    let (x, y, w, h) = lines[0];
    assert!((x - x_at(2)).abs() < 0.01, "from the preedit's start");
    assert!((x + w - x_at(14)).abs() < 0.01, "to its end");
    assert_eq!((y, h), (text_y + style.line_height - underline, underline));

    let carets: Vec<f32> = nodes
        .iter()
        .filter_map(|n| match *n {
            SceneNode::Rect { x, color, .. } if color == theme.colors.accent.0 => Some(x),
            _ => None,
        })
        .collect();
    assert_eq!(carets.len(), 1);
    assert!(
        (carets[0] - x_at(5)).abs() < 0.01,
        "the caret after ほ's start, where the input method put it"
    );

    // Composing into an empty field shows the preedit, not the placeholder.
    let mut empty = field("");
    empty.compose("", NIHONGO, None);
    let nodes = drawn(&empty, &theme);
    assert!(nodes.iter().any(|n| matches!(n,
        SceneNode::Text { key, color, .. } if key.text == NIHONGO && color == &theme.colors.text.0)));
    assert!(
        !nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Text { key, .. } if key.text == "Full name"))
    );
}

#[test]
fn the_candidate_window_goes_to_the_caret_and_follows_the_preedit() {
    let theme = Theme::hoff();
    let style = TextField::text_style(&theme);
    let b = bounds(&theme);
    let pad = theme.spacing.md;
    let text_w = b.w - pad * 2.0;
    let mut f = field("ab");
    f.input.buffer.set_cursor(1);
    let area = f.ime_area(b, &theme);
    let text_y = b.y + TextMeasurer::vertical_center(&style, b.h);
    let at = |text: &str, pos| {
        b.x + pad + TextMeasurer::cursor_x_styled(text, &style, Some(text_w), pos)
    };
    assert!((area.x - at("ab", 1)).abs() < 0.01);
    assert_eq!((area.y, area.h), (text_y, style.line_height));
    assert_eq!(area.w, theme.control.focus_ring_width);

    f.compose("", NIHONGO, None);
    let area = f.ime_area(b, &theme);
    assert!(
        (area.x - at("aにほんごb", 13)).abs() < 0.01,
        "the preedit's caret"
    );
}

#[test]
fn the_projection_holds_the_committed_text_never_the_preedit() {
    let id = SemId::parse("form.name").unwrap();
    let at = StateRef::parse("form.name:text").unwrap();
    let text = |f: &TextField| {
        let mut b = SnapshotBuilder::new();
        f.project(&mut b, &id, "name", None);
        b.finish().unwrap().value(&at).cloned()
    };
    let mut f = field("ab").sem(id.clone());
    f.compose("", NIHONGO, None);
    assert_eq!(text(&f), Some(Value::Text("ab".into())));
    assert_eq!(f.text(), "ab");
    f.compose("日本語", "", None);
    assert_eq!(text(&f), Some(Value::Text("ab日本語".into())));
}
