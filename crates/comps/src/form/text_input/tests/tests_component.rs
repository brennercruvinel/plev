use crate::form::text_input::*;

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
