//! Text measurement integration (taffy measure function).

use super::container;
use crate::compose::layout::*;

fn text_leaf(content: &str, font_size: f32) -> LayoutItem {
    LayoutItem {
        style: LayoutStyle::default(),
        children: vec![],
        text: Some(TextMeasureSpec {
            content: content.to_string(),
            style: crate::drawing::text::TextStyle::new(font_size).with_family("Inclusive Sans"),
            max_width: None,
        }),
    }
}

#[test]
fn test_text_leaf_gets_measured_size() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Row,
                align: Align::Start,
                ..Default::default()
            },
            vec![1],
        ),
        text_leaf("Hello, World!", 16.0),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    let (w, h) = crate::drawing::text::TextMeasurer::measure_styled(
        "Hello, World!",
        &crate::drawing::text::TextStyle::new(16.0).with_family("Inclusive Sans"),
        None,
    );
    assert!(
        (bounds[1].width - w).abs() < 1.0,
        "{} vs {}",
        bounds[1].width,
        w
    );
    assert!(
        (bounds[1].height - h).abs() < 1.0,
        "{} vs {}",
        bounds[1].height,
        h
    );
}

#[test]
fn test_text_leaf_proportional_widths_differ() {
    let mut engine = LayoutEngine::new();
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                align: Align::Start,
                ..Default::default()
            },
            vec![1, 2],
        ),
        text_leaf("iiii", 16.0),
        text_leaf("WWWW", 16.0),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);
    assert!(
        bounds[1].width < bounds[2].width,
        "'iiii' ({}) must lay out narrower than 'WWWW' ({})",
        bounds[1].width,
        bounds[2].width
    );
}

#[test]
fn test_text_leaf_wraps_in_narrow_container() {
    let mut engine = LayoutEngine::new();
    let long = "the quick brown fox jumps over the lazy dog";
    let items = vec![
        container(
            LayoutStyle {
                direction: Direction::Column,
                align: Align::Stretch,
                width: Some(80.0),
                ..Default::default()
            },
            vec![1],
        ),
        text_leaf(long, 16.0),
    ];
    let bounds = engine.compute(&items, 800.0, 600.0);

    let style = crate::drawing::text::TextStyle::new(16.0).with_family("Inclusive Sans");
    let (_, single_line_h) = crate::drawing::text::TextMeasurer::measure_styled("a", &style, None);
    assert!(bounds[1].width <= 80.0 + 0.5);
    assert!(
        bounds[1].height >= single_line_h * 2.0,
        "text in an 80px container must wrap to multiple lines (h={})",
        bounds[1].height
    );
}

fn text_leaf_wrapping(content: &str, wrap: crate::drawing::text::OverflowWrap) -> LayoutItem {
    let mut leaf = text_leaf(content, 14.0);
    if let Some(spec) = leaf.text.as_mut() {
        spec.style = spec.style.clone().with_overflow_wrap(wrap);
    }
    leaf
}

/// A row narrower than a word: the text's min-content is the word
/// (`Normal`), so it keeps one line and runs past the row; only an
/// `Anywhere` text shrinks to the row, a letter or so a line.
#[test]
fn a_word_keeps_its_min_content_width_unless_it_may_break_anywhere() {
    use crate::drawing::text::{OverflowWrap, TextMeasurer};
    let row = |child| {
        container(
            LayoutStyle {
                direction: Direction::Row,
                align: Align::Start,
                width: Some(30.0),
                ..Default::default()
            },
            vec![child],
        )
    };
    let mut engine = LayoutEngine::new();
    let normal = text_leaf_wrapping("Collapsible", OverflowWrap::Normal);
    let line = normal.text.as_ref().map_or(0.0, |t| t.style.line_height);
    let (word_w, _) =
        TextMeasurer::measure_styled("Collapsible", &normal.text.as_ref().unwrap().style, None);
    let bounds = engine.compute(&[row(1), normal], 800.0, 600.0);
    assert!(
        (bounds[1].width - word_w).abs() < 1.0,
        "{} vs {word_w}",
        bounds[1].width
    );
    assert!(
        (bounds[1].height - line).abs() < 0.5,
        "one line: {}",
        bounds[1].height
    );

    let anywhere = text_leaf_wrapping("Collapsible", OverflowWrap::Anywhere);
    let bounds = engine.compute(&[row(1), anywhere], 800.0, 600.0);
    assert!(bounds[1].width <= 30.5, "{}", bounds[1].width);
    assert!(
        bounds[1].height > line * 1.5,
        "it breaks: {}",
        bounds[1].height
    );
}
