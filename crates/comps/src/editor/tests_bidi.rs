//! Bidi in the editor: the arrow keys walk each line on screen, crossing
//! to the neighbouring line in the paragraph's direction; Shift extends
//! one logical range drawn as the boxes it covers; Home and End stay
//! logical. The lines are the phase-1 audit's two mixed phrases and one
//! right-to-left phrase.

use rope::{Document, Selection};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use engine::compositor::{Compositor, SceneNode};
use engine::layout::ComputedBounds;
use engine::text::{ClusterBox, TextMeasurer};

use super::{EditorConfig, EditorTheme, EditorView};

/// Line 0 starts at 0, line 1 at 15, line 2 at 42; the text ends at 59.
const DOC: &str = "abc אבג def\nhello مرحبا 123 world\nשלום עולם";
const L1: usize = 15;
const L2: usize = 42;

fn bounds() -> ComputedBounds {
    ComputedBounds {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 400.0,
    }
}

fn editor(text: &str) -> EditorView {
    let mut ed = EditorView::new(Document::load(text));
    ed.set_bounds(bounds());
    ed
}

fn at(ed: &mut EditorView, pos: usize) {
    ed.document.set_selections(rope::SelectionSet::caret(pos));
}

fn press(ed: &mut EditorView, key: NamedKey, mods: ModifiersState) -> usize {
    assert!(ed.handle_key(&Key::Named(key), mods));
    ed.document.selections().primary().head
}

/// The heads `key` visits from `from`, `n` presses.
fn walk(ed: &mut EditorView, from: usize, key: NamedKey, n: usize) -> Vec<usize> {
    at(ed, from);
    let mut seen = vec![from];
    for _ in 0..n {
        seen.push(press(ed, key, ModifiersState::empty()));
    }
    seen
}

/// `(x, w)` of every rect the editor draws in `color`.
fn rects(ed: &mut EditorView, color: [f32; 4]) -> Vec<(f32, f32)> {
    let mut comp = Compositor::new();
    comp.begin_frame();
    ed.render(&mut comp, bounds(), &EditorTheme::default());
    comp.layers()[0]
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Rect { x, w, color: c, .. } if *c == color => Some((*x, *w)),
            _ => None,
        })
        .collect()
}

fn clusters(text: &str) -> Vec<ClusterBox> {
    TextMeasurer::clusters_styled(text, &EditorConfig::default().text_style(), None)
}

fn cluster(text: &str, start: usize) -> ClusterBox {
    clusters(text)
        .into_iter()
        .find(|c| c.start == start)
        .unwrap()
}

#[test]
fn right_walks_each_line_on_screen_and_crosses_at_the_ends() {
    let mut ed = editor(DOC);
    let line0 = walk(&mut ed, 0, NamedKey::ArrowRight, 12);
    // Past the right end of line 0 the next Right goes to line 1's start.
    assert_eq!(line0, vec![0, 1, 2, 3, 4, 8, 6, 10, 11, 12, 13, 14, L1]);
    let rel: Vec<usize> = walk(&mut ed, L1, NamedKey::ArrowRight, 21)
        .into_iter()
        .map(|p| p - L1)
        .collect();
    assert_eq!(
        rel,
        vec![
            0, 1, 2, 3, 4, 5, 6, 18, 19, 17, 16, 14, 12, 10, 8, 20, 21, 22, 23, 24, 25, 26
        ]
    );
    // Left walks it back, and past line 1's left end goes to line 0's end.
    let back = walk(&mut ed, L1 + 26, NamedKey::ArrowLeft, 22);
    let mut expected: Vec<usize> = rel.iter().map(|p| p + L1).collect();
    expected.reverse();
    expected.push(14);
    assert_eq!(back, expected);
}

#[test]
fn on_a_right_to_left_line_left_goes_forward_in_the_text() {
    let mut ed = editor(DOC);
    let rel: Vec<usize> = walk(&mut ed, L2, NamedKey::ArrowLeft, 10)
        .into_iter()
        .map(|p| p - L2)
        .collect();
    // The last press is at the end of the document: nowhere to go.
    assert_eq!(rel, vec![0, 2, 4, 6, 8, 9, 11, 13, 15, 17, 17]);
    // Right at the line's right end (its start) goes back to line 1.
    at(&mut ed, L2);
    assert_eq!(
        press(&mut ed, NamedKey::ArrowRight, ModifiersState::empty()),
        L2 - 1
    );
    // Home and End stay logical: End is the end of the text, drawn left.
    at(&mut ed, L2 + 8);
    assert_eq!(
        press(&mut ed, NamedKey::End, ModifiersState::empty()),
        L2 + 17
    );
    assert_eq!(press(&mut ed, NamedKey::Home, ModifiersState::empty()), L2);
}

#[test]
fn shift_right_across_directions_draws_two_boxes_with_the_digits_between() {
    let mut ed = editor(DOC);
    at(&mut ed, L1 + 3);
    for _ in 0..6 {
        press(&mut ed, NamedKey::ArrowRight, ModifiersState::SHIFT);
    }
    let sel = ed.document.selections().primary();
    assert_eq!(sel, Selection::new(L1 + 3, L1 + 17));
    let washes = rects(&mut ed, EditorTheme::default().selection);
    assert_eq!(washes.len(), 2, "{washes:?}");
    let line = "hello مرحبا 123 world";
    let origin = ed.text_origin_x();
    let (l, one, three, mim) = (
        cluster(line, 3),
        cluster(line, 17),
        cluster(line, 19),
        cluster(line, 6),
    );
    assert_eq!(washes[0].0, origin + l.x);
    assert!(washes[0].0 + washes[0].1 <= origin + one.x + 0.01);
    assert!(washes[1].0 >= origin + three.x + three.w - 0.01);
    assert!((washes[1].0 + washes[1].1 - (origin + mim.x + mim.w)).abs() < 0.01);
    // A plain Left collapses to the end drawn further left: `hel|lo`.
    assert_eq!(
        press(&mut ed, NamedKey::ArrowLeft, ModifiersState::empty()),
        L1 + 3
    );
}

#[test]
fn the_caret_draws_on_the_latin_side_and_a_click_there_keeps_it() {
    let mut ed = editor(DOC);
    at(&mut ed, L1 + 6);
    let carets: Vec<f32> = rects(&mut ed, EditorTheme::default().cursor)
        .into_iter()
        .map(|(x, _)| x)
        .collect();
    let line = "hello مرحبا 123 world";
    // After `hello `, left of the digits, not inside the Arabic word.
    assert_eq!(carets, vec![ed.text_origin_x() + cluster(line, 17).x]);
    // A click on every caret stop of every line lands on it.
    let lh = ed.config.line_height;
    for (row, (start, text)) in [(0, "abc אבג def"), (L1, line), (L2, "שלום עולם")]
        .into_iter()
        .enumerate()
    {
        let mut stops: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
        stops.push(text.len());
        for rel in stops {
            let x = ed.text_origin_x() + ed.caret_x(text, rel);
            let y = row as f32 * lh + lh * 0.5;
            assert_eq!(ed.hit_test_point(x, y), start + rel, "{text:?} at {rel}");
        }
    }
}

#[test]
fn a_selected_newline_shows_on_the_side_its_paragraph_ends() {
    let mut ed = editor("שלום\nabc\n");
    ed.document
        .set_selections(rope::SelectionSet::single(Selection::new(0, 13)));
    let washes = rects(&mut ed, EditorTheme::default().selection);
    assert_eq!(washes.len(), 2, "{washes:?}");
    let origin = ed.text_origin_x();
    let ext = ed.config.font_size * 0.5;
    let hebrew_w = TextMeasurer::measure_styled("שלום", &ed.config.text_style(), None).0;
    // The RTL line ends on the left: the newline's sliver hangs there.
    assert!((washes[0].0 - (origin - ext)).abs() < 0.01, "{washes:?}");
    assert!((washes[0].0 + washes[0].1 - (origin + hebrew_w)).abs() < 0.01);
    // The Latin line ends on the right, as before.
    let abc_w = TextMeasurer::measure_styled("abc", &ed.config.text_style(), None).0;
    assert!((washes[1].0 - origin).abs() < 0.01);
    assert!((washes[1].1 - (abc_w + ext)).abs() < 0.01);
}
