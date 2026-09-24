use crate::form::text_input::*;
use crate::wake::Wake;

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
    assert!(ti.cursor_visible());
    ti.wake(0.0); // the focus frame
    ti.wake(0.54);
    assert!(!ti.cursor_visible());
    ti.wake(0.54);
    assert!(ti.cursor_visible());
}

/// A shell passes the real time since the last frame, and the key that
/// focused can come after seconds of idle: the half-period starts at the
/// focus, so the caret shows solid and its first flip is a whole
/// half-period away.
#[test]
fn text_input_focus_after_an_idle_gap_starts_the_half_period_at_the_focus() {
    let mut ti = TextInput::new();
    ti.focus();
    let w = ti.wake(10.0);
    assert!(!w.now, "no flip on the focus frame");
    assert!(ti.cursor_visible());
    let next = w.next.expect("focused: a flip is pending").as_secs_f32();
    assert!((next - CURSOR_BLINK_INTERVAL).abs() < 1e-6, "{next}");
}

/// A key that lands late in a half-period restarts it at the key: the
/// caret stays solid for a whole half-period after typing, never the
/// remainder of the one before.
#[test]
fn text_input_a_keystroke_restarts_the_half_period_at_the_key() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.wake(0.0);
    ti.wake(0.5);
    assert!(ti.cursor_visible(), "0.5 s: not flipped yet");
    ti.handle_char('a');
    // the key's frame arrives 0.5 s after the last one
    let w = ti.wake(0.5);
    assert!(!w.now && ti.cursor_visible());
    let next = w.next.expect("a flip is pending").as_secs_f32();
    assert!((next - CURSOR_BLINK_INTERVAL).abs() < 1e-6, "{next}");
}

/// The blink is a deadline: with a 60 Hz `dt` the caret flips at the
/// half-period (0.53 s), and between flips the input asks for the time
/// left, never for a frame.
#[test]
fn text_input_blink_flips_on_time_and_asks_only_for_the_next_flip() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.wake(0.0); // the focus frame
    let dt = 1.0 / 60.0;
    let mut flips = Vec::new();
    for turn in 1..=120 {
        let w = ti.wake(dt);
        let elapsed = turn as f32 * dt;
        let next = w
            .next
            .expect("focused: the next flip is pending")
            .as_secs_f32();
        assert!(
            !w.is_continuous(),
            "turn {turn}: a caret never asks for every frame"
        );
        if w.now {
            flips.push(turn);
        } else {
            // the time left is the half-period minus the phase, within a turn
            let phase = elapsed % CURSOR_BLINK_INTERVAL;
            assert!(
                (next - (CURSOR_BLINK_INTERVAL - phase)).abs() < dt,
                "turn {turn}: next {next} for phase {phase}"
            );
        }
    }
    // 2 s at 60 Hz: the flips at 0.53 s, 1.06 s and 1.59 s, on the turn
    // that crosses each (32, 64, 96)
    assert_eq!(flips, vec![32, 64, 96]);
    assert!(!ti.cursor_visible(), "three flips: hidden");
}

#[test]
fn text_input_unfocused_has_no_caret_and_nothing_pending() {
    let mut ti = TextInput::new();
    assert_eq!(ti.wake(0.54), Wake::idle());
    ti.focus();
    ti.wake(0.54);
    ti.unfocus();
    assert_eq!(ti.wake(0.54), Wake::idle());
    assert!(!ti.cursor_visible());
}

#[test]
fn text_input_reset_blink_on_type() {
    let mut ti = TextInput::new();
    ti.focus();
    ti.wake(0.0); // the focus frame
    ti.wake(0.54); // blink off
    assert!(!ti.cursor_visible());
    ti.handle_char('a'); // resets blink
    assert!(ti.cursor_visible());
    // and the half-period restarts from the keystroke
    let w = ti.wake(0.0);
    assert!(!w.now);
    assert!((w.next.unwrap().as_secs_f32() - CURSOR_BLINK_INTERVAL).abs() < 1e-6);
}

#[test]
fn text_input_click_positions_cursor() {
    let mut ti = TextInput::new();
    ti.buffer.set_text("hello");
    // Click exactly where the real caret for byte 3 sits.
    let style = engine::text::TextStyle::new(16.0);
    let x = engine::text::TextMeasurer::cursor_x_styled("hello", &style, None, 3);
    ti.handle_click(x, &style);
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

    let style = engine::text::TextStyle::new(16.0);
    let x = engine::text::TextMeasurer::cursor_x_styled(text, &style, None, 7);
    ti.handle_click(x, &style);
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
    let style = engine::text::TextStyle::new(16.0);
    let b2 = engine::text::TextMeasurer::cursor_x_styled("hello", &style, None, 2);
    let b3 = engine::text::TextMeasurer::cursor_x_styled("hello", &style, None, 3);
    // Click slightly left of the midpoint of the third glyph -> cursor 2.
    ti.handle_click(b2 + (b3 - b2) * 0.25, &style);
    assert_eq!(ti.buffer.cursor(), 2);
    // Click slightly right of the midpoint -> cursor 3.
    ti.handle_click(b2 + (b3 - b2) * 0.75, &style);
    assert_eq!(ti.buffer.cursor(), 3);
}

#[test]
fn text_input_cursor_x_round_trip_all_positions() {
    let mut ti = TextInput::new();
    let text = "Wide and iiii";
    ti.buffer.set_text(text);
    let mut cursors: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    cursors.push(text.len());
    let style = engine::text::TextStyle::new(16.0);
    for cursor in cursors {
        let x = engine::text::TextMeasurer::cursor_x_styled(text, &style, None, cursor);
        ti.handle_click(x, &style);
        assert_eq!(
            ti.buffer.cursor(),
            cursor,
            "click round-trip at byte {cursor}"
        );
    }
}

#[test]
fn text_field_empty_unfocused_draws_field_and_placeholder() {
    use crate::form::TextField;
    use engine::compositor::{Compositor, LayerId, SceneNode};
    let theme = engine::theme::Theme::hoff();
    let field = TextField::new("Type here...");
    let mut c = Compositor::new();
    c.begin_frame();
    field.render(
        &mut c,
        crate::core::Rect::new(0.0, 0.0, 200.0, TextField::height(&theme)),
        &theme,
    );
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    // Field surface + rim + placeholder text; no caret, no focus ring.
    assert!(matches!(nodes[0], SceneNode::RoundedRect { .. }));
    assert_eq!(
        nodes
            .iter()
            .filter(|n| matches!(n, SceneNode::Text { .. }))
            .count(),
        1
    );
    assert!(!nodes.iter().any(|n| matches!(n, SceneNode::Rect { .. })));
}

#[test]
fn text_field_focused_with_text_draws_ring_text_and_caret() {
    use crate::form::TextField;
    use engine::compositor::{Compositor, LayerId, SceneNode};
    let theme = engine::theme::Theme::hoff();
    let mut field = TextField::new("");
    field.focus();
    field.insert("hi");
    let mut c = Compositor::new();
    c.begin_frame();
    field.render(
        &mut c,
        crate::core::Rect::new(0.0, 0.0, 200.0, TextField::height(&theme)),
        &theme,
    );
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    let rounded = nodes
        .iter()
        .filter(|n| matches!(n, SceneNode::RoundedRect { .. }))
        .count();
    assert_eq!(rounded, 3, "focus ring + surface + rim");
    assert!(nodes.iter().any(|n| matches!(n, SceneNode::Text { .. })));
    let caret = nodes.iter().any(|n| {
        matches!(n, SceneNode::Rect { color, .. }
        if *color == theme.colors.accent.0)
    });
    assert!(caret, "the caret is the accent");
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
fn text_field_selection_draws_the_accent_wash() {
    use crate::form::{EditKey, TextField};
    use engine::compositor::{Compositor, LayerId, SceneNode};
    let theme = engine::theme::Theme::hoff();
    let mut field = TextField::new("");
    field.focus();
    field.insert("hello");
    assert!(field.edit(EditKey::SelectAll));
    let mut c = Compositor::new();
    c.begin_frame();
    field.render(
        &mut c,
        crate::core::Rect::new(0.0, 0.0, 200.0, TextField::height(&theme)),
        &theme,
    );
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    let washes = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::Rect { color, .. }
            if color[3] == theme.glass.wash_alpha)
        })
        .count();
    assert_eq!(washes, 1, "one selection wash under the text");
}

#[test]
fn text_field_click_places_the_caret_on_the_glyph() {
    use crate::core::{Rect, WidgetEvent};
    use crate::form::TextField;
    let theme = engine::theme::Theme::hoff();
    let mut field = TextField::new("").with_text("hello");
    let bounds = Rect::new(10.0, 10.0, 200.0, TextField::height(&theme));
    let style = TextField::text_style(&theme);
    let caret_3 = engine::text::TextMeasurer::cursor_x_styled("hello", &style, None, 3);
    let x = bounds.x + theme.spacing.md + caret_3;
    let r = field.handle_event(&WidgetEvent::MouseDown { x, y: 20.0 }, bounds, &theme);
    assert!(r.clicked);
    assert!(field.is_focused());
    assert_eq!(field.input.buffer.cursor(), 3);
    // A press outside blurs.
    let r = field.handle_event(
        &WidgetEvent::MouseDown { x: 400.0, y: 400.0 },
        bounds,
        &theme,
    );
    assert!(r.changed && !field.is_focused());
}
