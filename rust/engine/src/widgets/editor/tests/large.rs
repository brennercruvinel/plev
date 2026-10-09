//! A document of a million lines: the editor opens it, scrolls to the end
//! and edits the middle, and what it draws is the document (task-74,
//! phase 7). Headless, real shaping.

use rope::{Document, Selection, SelectionSet};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use engine::compositor::{Compositor, SceneNode};
use engine::layout::ComputedBounds;

use super::{EditorTheme, EditorView};
use engine::clipboard::LocalClipboard;

const LINES: usize = 1_000_000;

fn bounds() -> ComputedBounds {
    ComputedBounds {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    }
}

fn million() -> EditorView {
    let text: String = (0..LINES).map(|i| format!("line {i}\n")).collect();
    let mut ed =
        EditorView::new(Document::load(&text)).with_clipboard(Box::new(LocalClipboard::new()));
    ed.set_bounds(bounds());
    ed
}

/// The text nodes of one frame, `(text, x, y)` in emission order.
fn frame(ed: &mut EditorView) -> Vec<(String, f32, f32)> {
    let mut comp = Compositor::new();
    comp.begin_frame();
    ed.render(&mut comp, bounds(), &EditorTheme::default());
    comp.layers()[0]
        .nodes()
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, x, y, .. } => Some((key.text.clone(), *x, *y)),
            _ => None,
        })
        .collect()
}

/// The line texts of a frame (the gutter numbers left out), with their y.
fn drawn_lines(ed: &mut EditorView) -> Vec<(String, f32)> {
    let origin = ed.text_origin_x();
    frame(ed)
        .into_iter()
        .filter(|(_, x, _)| (*x - origin).abs() < 0.01)
        .map(|(t, _, y)| (t, y))
        .collect()
}

/// Every drawn line is the document's line at that row, the rows are one
/// line height apart, and the gutter numbers them. An empty line draws no
/// text node, so the rows are matched through the non-empty lines.
fn assert_draws_the_document(ed: &mut EditorView) {
    let lh = ed.config.line_height;
    let range = ed.visible_lines();
    let want: Vec<(usize, String)> = range
        .clone()
        .map(|l| (l, ed.line_text(l)))
        .filter(|(_, t)| !t.is_empty())
        .collect();
    let lines = drawn_lines(ed);
    assert_eq!(lines.len(), want.len(), "one text node per non-empty line");
    for (k, ((text, y), (line, expected))) in lines.iter().zip(&want).enumerate() {
        assert_eq!(text, expected, "line {line}");
        if k > 0 {
            let step = (y - lines[k - 1].1) / (line - want[k - 1].0) as f32;
            assert!(
                (step - lh).abs() < 0.01,
                "row {line} is {step} below the last, not {lh}"
            );
        }
    }
    let numbers: Vec<String> = frame(ed)
        .into_iter()
        .filter(|(t, x, _)| *x < ed.text_origin_x() - 0.01 && t.parse::<usize>().is_ok())
        .map(|(t, _, _)| t)
        .collect();
    let want: Vec<String> = range.map(|l| (l + 1).to_string()).collect();
    assert_eq!(numbers, want, "the gutter numbers the drawn lines");
}

#[test]
fn a_million_lines_open_and_scroll_to_the_end() {
    let mut ed = million();
    assert_eq!(ed.document.len_lines(), LINES + 1);
    assert_draws_the_document(&mut ed);

    let max = ed.scroll.max_offset();
    ed.scroll.scroll_to(max);
    assert_draws_the_document(&mut ed);
    let lines = drawn_lines(&mut ed);
    let last = lines.iter().rev().find(|(t, _)| !t.is_empty());
    let (text, y) = last.expect("the end is on screen");
    assert_eq!(text, &format!("line {}", LINES - 1));
    // The scroll offset is an f32 (`ScrollState`): at 19.6 million px it
    // moves in steps of 2, so the end sits within a pixel of the edge.
    let bottom = y + 2.0 * ed.config.line_height;
    assert!(
        (bottom - bounds().height).abs() <= 1.0,
        "the empty last line ends at the bottom edge, {bottom}"
    );
}

#[test]
fn a_million_lines_scroll_through_every_region() {
    let mut ed = million();
    let max = ed.scroll.max_offset();
    for k in 0..=10 {
        ed.scroll.scroll_to(max * k as f32 / 10.0);
        assert_draws_the_document(&mut ed);
    }
}

#[test]
fn a_click_near_the_end_lands_on_the_line_under_the_pointer() {
    let mut ed = million();
    let max = ed.scroll.max_offset();
    ed.scroll.scroll_to(max);
    let lines = drawn_lines(&mut ed);
    let range = ed.visible_lines();
    let x = ed.text_origin_x() + 1.0;
    for (k, (_, y)) in lines.iter().enumerate() {
        if *y < 0.0 || *y >= bounds().height {
            continue;
        }
        let line = range.start + k;
        for dy in [0.5, ed.config.line_height - 0.5] {
            let at = ed.hit_test_point(x, y + dy);
            assert_eq!(ed.document.rope().byte_to_line(at), line, "y {}", y + dy);
        }
    }
}

#[test]
fn a_million_lines_edit_in_the_middle() {
    let mut ed = million();
    let middle = LINES / 2;
    let start = ed.document.rope().line_to_byte(middle);
    ed.document
        .set_selections(SelectionSet::single(Selection::caret(start + 1)));
    ed.scroll_to_cursor();
    assert!(ed.visible_lines().contains(&middle));

    ed.handle_key(&Key::Character("x".into()), ModifiersState::empty());
    let range = ed.visible_lines();
    let lines = drawn_lines(&mut ed);
    assert_eq!(lines[middle - range.start].0, format!("lxine {middle}"));
    assert_eq!(
        lines[middle - range.start - 1].0,
        format!("line {}", middle - 1)
    );
    assert_eq!(
        lines[middle - range.start + 1].0,
        format!("line {}", middle + 1)
    );
    assert_draws_the_document(&mut ed);

    ed.handle_key(&Key::Named(NamedKey::Backspace), ModifiersState::empty());
    assert_eq!(
        drawn_lines(&mut ed)[middle - range.start].0,
        format!("line {middle}")
    );
    assert_eq!(ed.document.len_lines(), LINES + 1);
    assert_draws_the_document(&mut ed);

    // A newline in the middle shifts every line below by one; the rows
    // still read the document.
    ed.handle_key(&Key::Named(NamedKey::Enter), ModifiersState::empty());
    assert_eq!(ed.document.len_lines(), LINES + 2);
    assert_draws_the_document(&mut ed);
}
