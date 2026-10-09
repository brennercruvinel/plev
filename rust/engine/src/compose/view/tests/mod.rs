//! Tests for the view module.

use super::*;
use crate::compositor::{SceneNode, TextNodeKey};
use crate::layout::{ComputedBounds, LayoutStyle};

fn test_cx() -> ViewContext {
    ViewContext::new(800.0, 600.0)
}

fn test_cx_with_bounds(bounds: ComputedBounds) -> ViewContext {
    ViewContext::with_bounds(800.0, 600.0, bounds)
}

#[test]
fn rect_view_produces_rect_node() {
    let view = RectView {
        x: 10.0,
        y: 20.0,
        w: 100.0,
        h: 50.0,
        color: [1.0, 0.0, 0.0, 1.0],
    };
    let bounds = ComputedBounds {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
    };
    let mut cx = test_cx_with_bounds(bounds);
    let nodes = view.render(&mut cx);
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        SceneNode::Rect { x, y, w, h, color } => {
            assert_eq!(*x, 10.0);
            assert_eq!(*y, 20.0);
            assert_eq!(*w, 100.0);
            assert_eq!(*h, 50.0);
            assert_eq!(*color, [1.0, 0.0, 0.0, 1.0]);
        }
        _ => panic!("Expected SceneNode::Rect"),
    }
}

#[test]
fn text_view_produces_text_node() {
    let view = TextView {
        text: "hello".to_string(),
        font_size: 16.0,
        line_height: 20.0,
        max_width: Some(400.0),
        x: 5.0,
        y: 10.0,
        color: [1.0, 1.0, 1.0, 1.0],
    };
    let bounds = ComputedBounds {
        x: 5.0,
        y: 10.0,
        width: 400.0,
        height: 20.0,
    };
    let mut cx = test_cx_with_bounds(bounds);
    let nodes = view.render(&mut cx);
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        SceneNode::Text { key, x, y, color } => {
            assert_eq!(key.text, "hello");
            assert_eq!(key.font_size_bits, 16.0_f32.to_bits());
            assert_eq!(key.line_height_bits, 20.0_f32.to_bits());
            assert_eq!(key.max_width_bits, Some(400.0_f32.to_bits()));
            assert_eq!(*x, 5.0);
            assert_eq!(*y, 10.0);
            assert_eq!(*color, [1.0, 1.0, 1.0, 1.0]);
        }
        _ => panic!("Expected SceneNode::Text"),
    }
}

#[test]
fn custom_view_composes_multiple_nodes() {
    struct CardView;

    impl View for CardView {
        fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode> {
            vec![
                SceneNode::Rect {
                    x: cx.bounds.x,
                    y: cx.bounds.y,
                    w: 200.0,
                    h: 100.0,
                    color: [0.2, 0.2, 0.3, 1.0],
                },
                SceneNode::Text {
                    key: TextNodeKey::new("Card title", 16.0, 20.0, Some(180.0)),
                    x: cx.bounds.x + 10.0,
                    y: cx.bounds.y + 10.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ]
        }
    }

    let view = CardView;
    let bounds = ComputedBounds {
        x: 50.0,
        y: 50.0,
        width: 200.0,
        height: 100.0,
    };
    let mut cx = test_cx_with_bounds(bounds);
    let nodes = view.render(&mut cx);
    assert_eq!(nodes.len(), 2);
    assert!(matches!(&nodes[0], SceneNode::Rect { .. }));
    assert!(matches!(&nodes[1], SceneNode::Text { .. }));
}

#[test]
fn dyn_view_dispatch_works() {
    let views: Vec<Box<dyn View>> = vec![
        Box::new(RectView {
            x: 0.0,
            y: 0.0,
            w: 50.0,
            h: 50.0,
            color: [1.0, 0.0, 0.0, 1.0],
        }),
        Box::new(TextView {
            text: "test".to_string(),
            font_size: 14.0,
            line_height: 18.0,
            max_width: None,
            x: 0.0,
            y: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }),
    ];
    let mut cx = test_cx();
    let total_nodes: usize = views.iter().map(|v| v.render(&mut cx).len()).sum();
    assert_eq!(total_nodes, 2);
}

#[test]
fn container_view_with_background() {
    let container = ContainerView {
        style: LayoutStyle::default(),
        children: vec![],
        background: Some([0.1, 0.2, 0.3, 1.0]),
    };
    let bounds = ComputedBounds {
        x: 10.0,
        y: 20.0,
        width: 300.0,
        height: 200.0,
    };
    let mut cx = test_cx_with_bounds(bounds);
    let nodes = container.render(&mut cx);
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        SceneNode::Rect { x, y, w, h, .. } => {
            assert_eq!(*x, 10.0);
            assert_eq!(*y, 20.0);
            assert_eq!(*w, 300.0);
            assert_eq!(*h, 200.0);
        }
        _ => panic!("Expected Rect"),
    }
}

#[test]
fn container_view_no_background() {
    let container = ContainerView {
        style: LayoutStyle::default(),
        children: vec![],
        background: None,
    };
    let mut cx = test_cx();
    let nodes = container.render(&mut cx);
    assert!(nodes.is_empty());
}

#[test]
fn rect_view_layout_returns_fixed_size() {
    let view = RectView {
        x: 0.0,
        y: 0.0,
        w: 150.0,
        h: 75.0,
        color: [1.0; 4],
    };
    let style = view.layout();
    assert_eq!(style.width, Some(150.0));
    assert_eq!(style.height, Some(75.0));
}
