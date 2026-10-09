//! Bidi in the single-line field: the arrow keys walk the screen, the
//! caret sits where the next letter appears at a change of direction,
//! and a logical selection draws as the boxes it covers. The corpus is
//! the phase-1 audit's two mixed phrases and one right-to-left phrase;
//! Latin keeps every x it had.
//!
//! With `TEXT_FIELD_BIDI_OUT=<dir>` and a GPU, the field test also
//! writes `text-field-bidi.png` there, to look at.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::drawing::text::{ClusterBox, TextMeasurer, TextStyle};
use crate::styling::theme::Theme;

use crate::widgets::form::text_input::*;
use crate::widgets::form::{EditKey, TextField};
use crate::widgets::widget::{Rect, WidgetEvent};

const MIXED_HEBREW: &str = "abc אבג def";
const MIXED_ARABIC: &str = "hello مرحبا 123 world";
const HEBREW: &str = "שלום עולם";

fn style() -> TextStyle {
    TextStyle::new(16.0).with_family("Inclusive Sans")
}

fn input(text: &str, cursor: usize) -> TextInput {
    let mut ti = TextInput::new();
    ti.buffer.set_text(text);
    ti.focus();
    ti.buffer.set_cursor(cursor);
    ti
}

/// Every offset Right visits from `start`, pressed until it stops.
fn walk_right(text: &str, start: usize) -> Vec<usize> {
    let mut ti = input(text, start);
    let mut seen = vec![start];
    loop {
        ti.handle_right();
        let pos = ti.buffer.cursor();
        if pos == *seen.last().unwrap() {
            return seen;
        }
        seen.push(pos);
    }
}

/// Every offset Left visits from `start`, pressed until it stops.
fn walk_left(text: &str, start: usize) -> Vec<usize> {
    let mut ti = input(text, start);
    let mut seen = vec![start];
    loop {
        ti.handle_left();
        let pos = ti.buffer.cursor();
        if pos == *seen.last().unwrap() {
            return seen;
        }
        seen.push(pos);
    }
}

fn caret_x(text: &str, pos: usize) -> f32 {
    input(text, pos).caret_x(&style(), None)
}

/// The shaped cluster that starts at byte `start`.
fn cluster(text: &str, start: usize) -> ClusterBox {
    TextMeasurer::clusters_styled(text, &style(), None)
        .into_iter()
        .find(|c| c.start == start)
        .unwrap()
}

/// Right walks the screen from its left end and Left walks it back; the
/// caret moves right at every press and each offset shows up once.
fn assert_screen_walk(text: &str, leftmost: usize, expected: &[usize]) {
    let right = walk_right(text, leftmost);
    assert_eq!(right, expected, "Right through {text:?}");
    let mut left = walk_left(text, *expected.last().unwrap());
    left.reverse();
    assert_eq!(left, expected, "Left back through {text:?}");
    let xs: Vec<f32> = expected.iter().map(|&p| caret_x(text, p)).collect();
    assert!(
        xs.windows(2).all(|w| w[0] < w[1]),
        "the caret moves right at every press in {text:?}: {xs:?}"
    );
    let mut sorted = expected.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), expected.len(), "every offset once");
}

#[test]
fn right_arrow_walks_the_screen_through_hebrew_in_latin() {
    // On screen: abc ␠ ג ב א ␠ def. Inside the RTL run Right goes back in
    // the text (8, then 6); the run's edges belong to the Latin side.
    assert_screen_walk(MIXED_HEBREW, 0, &[0, 1, 2, 3, 4, 8, 6, 10, 11, 12, 13, 14]);
}

#[test]
fn right_arrow_walks_the_screen_through_arabic_and_digits_in_latin() {
    // On screen: hello␠ 123 ␠ ا ب ح ر م ␠world. The digits read left to
    // right inside the RTL run; Left inside the Arabic goes forward in
    // the text (8, 10, 12, 14, 16).
    assert_screen_walk(
        MIXED_ARABIC,
        0,
        &[
            0, 1, 2, 3, 4, 5, 6, 18, 19, 17, 16, 14, 12, 10, 8, 20, 21, 22, 23, 24, 25, 26,
        ],
    );
    assert_eq!(
        walk_left(MIXED_ARABIC, 8),
        vec![8, 10, 12, 14, 16, 17, 19, 18, 6, 5, 4, 3, 2, 1, 0]
    );
}

#[test]
fn in_a_right_to_left_paragraph_left_goes_forward_in_the_text() {
    // The end of the text is the left end of the line.
    assert_screen_walk(HEBREW, 17, &[17, 15, 13, 11, 9, 8, 6, 4, 2, 0]);
    assert_eq!(walk_left(HEBREW, 0), vec![0, 2, 4, 6, 8, 9, 11, 13, 15, 17]);
}

#[test]
fn the_caret_sits_on_the_latin_side_of_a_change_of_direction() {
    // `abc |אבג def`: after the space, not inside the Hebrew.
    let space = cluster(MIXED_HEBREW, 3);
    assert_eq!(caret_x(MIXED_HEBREW, 4), space.x + space.w);
    // `abc אבג| def`: before the second space, the right end of the run.
    assert_eq!(caret_x(MIXED_HEBREW, 10), cluster(MIXED_HEBREW, 10).x);
    // Inside the run the caret is between the two letters it separates:
    // 6 between ב and א, the right edge of ב.
    let bet = cluster(MIXED_HEBREW, 6);
    assert_eq!(caret_x(MIXED_HEBREW, 6), bet.x + bet.w);
    // `hello مرحبا |123`: the digits' left side belongs to the Arabic
    // space before them in the text, whose left edge touches the 3.
    let three = cluster(MIXED_ARABIC, 19);
    assert_eq!(caret_x(MIXED_ARABIC, 17), cluster(MIXED_ARABIC, 16).x);
    assert!((caret_x(MIXED_ARABIC, 17) - (three.x + three.w)).abs() < 0.01);
    // `hello |مرحبا`: right after `hello `, left of the digits.
    assert_eq!(caret_x(MIXED_ARABIC, 6), cluster(MIXED_ARABIC, 17).x);
}

#[test]
fn a_right_to_left_paragraph_hangs_from_the_right_edge_of_its_box() {
    let ti = input(HEBREW, 0);
    let line_w = TextMeasurer::measure_styled(HEBREW, &style(), None).0;
    assert!((ti.caret_x(&style(), None) - line_w).abs() < 0.01);
    assert!((ti.caret_x(&style(), Some(200.0)) - 200.0).abs() < 0.01);
    let end = input(HEBREW, HEBREW.len());
    assert!(end.caret_x(&style(), None).abs() < 0.01);
    assert!((end.caret_x(&style(), Some(200.0)) - (200.0 - line_w)).abs() < 0.01);
}

#[test]
fn shift_arrows_extend_one_logical_range_drawn_as_the_boxes_it_covers() {
    let mut ti = input(MIXED_ARABIC, 3);
    for _ in 0..6 {
        ti.handle_select_right();
    }
    // 3, 4, 5, 6, 18, 19, 17: the range is `lo مرحبا ` in the text.
    assert_eq!(ti.buffer.selection(), Some((3, 17)));
    assert_eq!(ti.buffer.selected_text(), Some("lo مرحبا "));
    let boxes = ti.selection_boxes(&style(), None);
    assert_eq!(boxes.len(), 2, "{boxes:?}");
    // `lo ` on the left, then the Arabic word with its space; the digits
    // between them stay unselected.
    let (l, space, one, three) = (
        cluster(MIXED_ARABIC, 3),
        cluster(MIXED_ARABIC, 5),
        cluster(MIXED_ARABIC, 17),
        cluster(MIXED_ARABIC, 19),
    );
    assert_eq!(boxes[0].0, l.x);
    assert!((boxes[0].0 + boxes[0].1 - (space.x + space.w)).abs() < 0.01);
    assert!(boxes[0].0 + boxes[0].1 <= one.x + 0.01);
    assert!(boxes[1].0 >= three.x + three.w - 0.01);
    let mim = cluster(MIXED_ARABIC, 6);
    assert!((boxes[1].0 + boxes[1].1 - (mim.x + mim.w)).abs() < 0.01);
    // Back over the boundary the anchor holds.
    ti.handle_select_left();
    assert_eq!(ti.buffer.selection(), Some((3, 19)));
    // A plain arrow drops the selection.
    ti.handle_left();
    assert_eq!(ti.buffer.selection(), None);
}

#[test]
fn home_and_end_are_logical_and_shift_extends_to_them() {
    let mut ti = input(MIXED_ARABIC, 10);
    ti.handle_select_home();
    assert_eq!(ti.buffer.selection(), Some((10, 0)));
    ti.handle_select_end();
    assert_eq!(ti.buffer.selection(), Some((10, 26)));
    ti.handle_home();
    assert_eq!((ti.buffer.cursor(), ti.buffer.selection()), (0, None));
    ti.handle_end();
    assert_eq!(ti.buffer.cursor(), 26);
    // In the RTL paragraph too: Home is the start of the text, drawn at
    // the right end.
    let mut rtl = input(HEBREW, 9);
    rtl.handle_home();
    assert_eq!(rtl.buffer.cursor(), 0);
}

#[test]
fn a_click_lands_where_the_caret_draws() {
    for text in [MIXED_HEBREW, MIXED_ARABIC, HEBREW, "Wide and iiii"] {
        let mut ti = input(text, 0);
        let mut stops: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        stops.push(text.len());
        for pos in stops {
            let x = caret_x(text, pos);
            ti.handle_click(x, &style());
            assert_eq!(
                ti.buffer.cursor(),
                pos,
                "click at the caret of {pos} in {text:?}"
            );
        }
    }
    // The left half of ג is the edge after `abc `: 4, not 10.
    let gimel = cluster(MIXED_HEBREW, 8);
    let mut ti = input(MIXED_HEBREW, 0);
    ti.handle_click(gimel.x + gimel.w * 0.25, &style());
    assert_eq!(ti.buffer.cursor(), 4);
}

#[test]
fn latin_keeps_every_caret_and_the_one_selection_box() {
    let text = "Wide and iiii";
    let old = |pos| TextMeasurer::cursor_x_styled(text, &style(), None, pos);
    assert_eq!(walk_right(text, 0), (0..=text.len()).collect::<Vec<_>>());
    for pos in 0..=text.len() {
        assert_eq!(
            caret_x(text, pos).to_bits(),
            old(pos).to_bits(),
            "caret {pos}"
        );
    }
    let mut ti = input(text, 2);
    for _ in 0..5 {
        ti.handle_select_right();
    }
    assert_eq!(ti.buffer.selection(), Some((2, 7)));
    let boxes = ti.selection_boxes(&style(), Some(300.0));
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].0.to_bits(), old(2).to_bits());
    assert_eq!(boxes[0].1.to_bits(), (old(7) - old(2)).to_bits());
}

#[test]
fn text_buffer_select_to_keeps_the_anchor() {
    let mut buf = TextBuffer::with_text("hello");
    buf.set_cursor(1);
    buf.select_to(4);
    assert_eq!((buf.selection(), buf.cursor()), (Some((1, 4)), 4));
    buf.select_to(0);
    assert_eq!(buf.selection(), Some((1, 0)));
    buf.select_to(1);
    assert_eq!(
        buf.selection(),
        None,
        "a range closed to nothing is dropped"
    );
    buf.set_cursor(99);
    assert_eq!(buf.cursor(), 5);
}

/// The field as drawn: the washes and the caret of `field` in `bounds`.
fn drawn(field: &TextField, bounds: Rect, theme: &Theme) -> (Vec<(f32, f32)>, Vec<f32>) {
    let mut c = Compositor::new();
    c.begin_frame();
    field.render(&mut c, bounds, theme);
    let mut washes = Vec::new();
    let mut carets = Vec::new();
    for n in c.layer(LayerId::DEFAULT).unwrap().nodes() {
        if let SceneNode::Rect { x, w, color, .. } = n {
            if color[3] == theme.glass.wash_alpha {
                washes.push((*x, *w));
            } else if *color == theme.colors.accent.0 {
                carets.push(*x);
            }
        }
    }
    (washes, carets)
}

#[test]
fn the_field_draws_the_boundary_caret_and_a_selection_across_directions() {
    let theme = Theme::hoff();
    let style = TextField::text_style(&theme);
    let pad = theme.spacing.md;
    let h = TextField::height(&theme);
    let bounds = |row: f32| Rect::new(16.0, 16.0 + row * (h + 16.0), 320.0, h);
    let text_w = 320.0 - pad * 2.0;
    let line = |text| TextMeasurer::clusters_styled(text, &style, None);

    // Row 0: the caret on the boundary after `hello `, reached with Left.
    let mut caret = TextField::new("").with_text(MIXED_ARABIC);
    caret.focus();
    caret.input.buffer.set_cursor(18);
    assert!(caret.edit(EditKey::Left));
    assert_eq!(caret.input.buffer.cursor(), 6);
    let (washes, carets) = drawn(&caret, bounds(0.0), &theme);
    assert!(washes.is_empty());
    let one = line(MIXED_ARABIC)
        .into_iter()
        .find(|c| c.start == 17)
        .unwrap();
    assert_eq!(carets, vec![bounds(0.0).x + pad + one.x]);

    // Row 1: Shift+Right from `hel|lo` to the edge after the Arabic space.
    let mut sel = TextField::new("").with_text(MIXED_ARABIC);
    sel.focus();
    sel.input.buffer.set_cursor(3);
    for _ in 0..6 {
        assert!(sel.edit(EditKey::SelectRight));
    }
    let (washes, carets) = drawn(&sel, bounds(1.0), &theme);
    assert_eq!(washes.len(), 2, "two washes, the digits between them");
    let expected = sel.input.selection_boxes(&style, Some(text_w));
    let origin = bounds(1.0).x + pad;
    for ((x, w), (ex, ew)) in washes.iter().zip(&expected) {
        assert_eq!((*x, *w), (origin + ex, *ew));
    }
    assert_eq!(
        carets,
        vec![origin + sel.input.caret_x(&style, Some(text_w))]
    );

    // Row 2: an RTL paragraph, the caret at its start, the box's right
    // edge; a click there keeps it.
    let mut rtl = TextField::new("").with_text(HEBREW);
    rtl.focus();
    assert!(rtl.edit(EditKey::Home));
    let (_, carets) = drawn(&rtl, bounds(2.0), &theme);
    let right = bounds(2.0).x + pad + text_w;
    assert!((carets[0] - right).abs() < 0.01, "{carets:?} vs {right}");
    let r = rtl.handle_event(
        &WidgetEvent::MouseDown {
            x: right - 1.0,
            y: bounds(2.0).y + h / 2.0,
        },
        bounds(2.0),
        &theme,
    );
    assert!(r.clicked);
    assert_eq!(rtl.input.buffer.cursor(), 0);

    if let Some(dir) = std::env::var_os("TEXT_FIELD_BIDI_OUT") {
        write_png(&[&caret, &sel, &rtl], bounds, &theme, dir.as_ref());
    }
}

/// The three fields through the window's frame path, when a GPU answers.
fn write_png(
    fields: &[&TextField],
    bounds: impl Fn(f32) -> Rect,
    theme: &Theme,
    dir: &std::path::Path,
) {
    use crate::drawing::gpu::RenderConfig;
    use crate::drawing::headless::{HeadlessError, Renderer, Viewport};
    let last = bounds(fields.len() as f32 - 1.0);
    let viewport = Viewport::new(last.x * 2.0 + last.w, last.y + last.h + 16.0, 2.0);
    let mut renderer = match Renderer::new(viewport, RenderConfig::default()) {
        Ok(r) => r,
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter: no png");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    let mut c = Compositor::new();
    c.begin_frame();
    for (row, field) in fields.iter().enumerate() {
        field.render(&mut c, bounds(row as f32), theme);
    }
    let frame = renderer
        .render(&mut c, theme.colors.bg.to_linear_array())
        .expect("the frame draws");
    std::fs::create_dir_all(dir).expect("the out dir");
    frame
        .write_png(dir.join("text-field-bidi.png"))
        .expect("the png writes");
}
