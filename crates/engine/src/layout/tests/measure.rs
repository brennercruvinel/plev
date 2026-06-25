//! Text measurement integration (taffy measure function).

use super::container;
use crate::layout::*;

fn text_leaf(content: &str, font_size: f32) -> LayoutItem {
    LayoutItem {
        style: LayoutStyle::default(),
        children: vec![],
        text: Some(TextMeasureSpec {
            content: content.to_string(),
            style: crate::text::TextStyle::new(font_size).with_family("Inter"),
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

    let (w, h) = crate::text::TextMeasurer::measure_styled(
        "Hello, World!",
        &crate::text::TextStyle::new(16.0).with_family("Inter"),
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

    let style = crate::text::TextStyle::new(16.0).with_family("Inter");
    let (_, single_line_h) = crate::text::TextMeasurer::measure_styled("a", &style, None);
    assert!(bounds[1].width <= 80.0 + 0.5);
    assert!(
        bounds[1].height >= single_line_h * 2.0,
        "text in an 80px container must wrap to multiple lines (h={})",
        bounds[1].height
    );
}
