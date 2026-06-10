#[cfg(test)]
mod tests_intent {
    use crate::builder::*;
    use crate::color::Color;
    use crate::compositor::SceneNode;
    use crate::view::{View, ViewContext};

    fn test_cx() -> ViewContext {
        ViewContext::new(800.0, 600.0)
    }

    #[test]
    fn uppercase_transforms_text() {
        let el = text("hello").uppercase();
        let nodes = el.render(&mut test_cx());
        let SceneNode::Text { key, .. } = &nodes[0] else {
            panic!("Expected Text, got {:?}", &nodes[0]);
        };
        assert_eq!(key.text, "HELLO");
    }

    #[test]
    fn tracking_stored() {
        let el = text("hi").tracking(0.2);
        assert_eq!(el.style.letter_spacing, 0.2);
    }

    #[test]
    fn border_color_stored() {
        let el = div().border(1.0).border_color("red");
        assert_eq!(el.style.border_color, Color::RED);
    }

    #[test]
    fn intent_stored() {
        use crate::theme::Intent;
        let el = div().intent(Intent::Destructive);
        assert_eq!(el.intent, Some(Intent::Destructive));
    }

    #[test]
    fn intent_default_none() {
        let el = div();
        assert_eq!(el.intent, None);
    }

    #[test]
    fn intent_on_text() {
        use crate::theme::Intent;
        let el = text("delete").intent(Intent::Destructive);
        assert_eq!(el.intent, Some(Intent::Destructive));
    }

    #[test]
    fn intent_on_button() {
        use crate::theme::Intent;
        let el = button("Cancel").intent(Intent::Destructive);
        assert_eq!(el.intent, Some(Intent::Destructive));
    }

    #[test]
    fn intent_resolves_bg_color_with_theme() {
        use crate::theme::{Intent, Theme};
        let theme = Theme::dark();
        let el = div().w(100.0).h(50.0).intent(Intent::Destructive);
        let mut cx = ViewContext::new(800.0, 600.0).with_theme(theme.clone());
        let nodes = el.render(&mut cx);
        assert_eq!(nodes.len(), 1);
        let SceneNode::Rect { color, .. } = &nodes[0] else {
            panic!("Expected Rect from intent-colored div, got {:?}", &nodes[0]);
        };
        let danger = theme.intent_color(Intent::Destructive).to_array();
        assert_eq!(*color, danger);
    }

    #[test]
    fn intent_resolves_text_color_with_theme() {
        use crate::theme::{Intent, Theme};
        let theme = Theme::dark();
        let el = text("Error").intent(Intent::Destructive);
        let mut cx = ViewContext::new(800.0, 600.0).with_theme(theme.clone());
        let nodes = el.render(&mut cx);
        assert_eq!(nodes.len(), 1);
        let SceneNode::Text { color, .. } = &nodes[0] else {
            panic!("Expected Text, got {:?}", &nodes[0]);
        };
        let danger = theme.intent_color(Intent::Destructive).to_array();
        assert_eq!(*color, danger);
    }

    #[test]
    fn no_intent_no_theme_unchanged() {
        let el = div().bg("blue").w(100.0).h(50.0);
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        let SceneNode::Rect { color, .. } = &nodes[0] else {
            panic!("Expected Rect, got {:?}", &nodes[0]);
        };
        assert_eq!(*color, Color::BLUE.to_array());
    }

    // -- Alignment helpers --

    #[test]
    fn flush_left_all_children_same_x() {
        let el = div()
            .flush_left()
            .pl(20.0)
            .w(400.0)
            .h(300.0)
            .child(div().bg("red").w(100.0).h(30.0))
            .child(div().bg("green").w(200.0).h(30.0))
            .child(div().bg("blue").w(50.0).h(30.0));
        let nodes = el.render(&mut test_cx());
        // 3 rects, all should have x=20 (parent padding_left)
        assert_eq!(nodes.len(), 3);
        let xs: Vec<f32> = nodes
            .iter()
            .filter_map(|n| match n {
                SceneNode::Rect { x, .. } => Some(*x),
                _ => None,
            })
            .collect();
        assert_eq!(xs.len(), 3);
        assert_eq!(xs[0], xs[1], "child 0 x={} != child 1 x={}", xs[0], xs[1]);
        assert_eq!(xs[1], xs[2], "child 1 x={} != child 2 x={}", xs[1], xs[2]);
        assert_eq!(xs[0], 20.0, "left edge should be at padding_left=20");
    }

    #[test]
    fn flush_left_children_keep_intrinsic_width() {
        let el = div()
            .flush_left()
            .w(400.0)
            .h(200.0)
            .child(div().bg("red").w(100.0).h(30.0))
            .child(div().bg("blue").w(50.0).h(30.0));
        let nodes = el.render(&mut test_cx());
        let widths: Vec<f32> = nodes
            .iter()
            .filter_map(|n| match n {
                SceneNode::Rect { w, .. } => Some(*w),
                _ => None,
            })
            .collect();
        assert_eq!(widths[0], 100.0, "first child should keep w=100");
        assert_eq!(widths[1], 50.0, "second child should keep w=50");
    }

    #[test]
    fn flush_right_children_share_right_edge() {
        let el = div()
            .flush_right()
            .w(400.0)
            .h(200.0)
            .child(div().bg("red").w(100.0).h(30.0))
            .child(div().bg("blue").w(50.0).h(30.0));
        let nodes = el.render(&mut test_cx());
        let right_edges: Vec<f32> = nodes
            .iter()
            .filter_map(|n| match n {
                SceneNode::Rect { x, w, .. } => Some(*x + *w),
                _ => None,
            })
            .collect();
        assert_eq!(right_edges.len(), 2);
        assert!(
            (right_edges[0] - right_edges[1]).abs() < 0.5,
            "right edges should align: {} vs {}",
            right_edges[0],
            right_edges[1],
        );
    }

    #[test]
    fn align_top_children_same_y() {
        let el = div()
            .align_top()
            .w(400.0)
            .h(200.0)
            .child(div().bg("red").w(80.0).h(100.0))
            .child(div().bg("green").w(80.0).h(50.0))
            .child(div().bg("blue").w(80.0).h(150.0));
        let nodes = el.render(&mut test_cx());
        let ys: Vec<f32> = nodes
            .iter()
            .filter_map(|n| match n {
                SceneNode::Rect { y, .. } => Some(*y),
                _ => None,
            })
            .collect();
        assert_eq!(ys.len(), 3);
        assert_eq!(ys[0], ys[1], "top edges: {} vs {}", ys[0], ys[1]);
        assert_eq!(ys[1], ys[2], "top edges: {} vs {}", ys[1], ys[2]);
    }

    #[test]
    fn align_bottom_children_same_bottom_edge() {
        let el = div()
            .align_bottom()
            .w(400.0)
            .h(200.0)
            .child(div().bg("red").w(80.0).h(100.0))
            .child(div().bg("green").w(80.0).h(50.0));
        let nodes = el.render(&mut test_cx());
        let bottom_edges: Vec<f32> = nodes
            .iter()
            .filter_map(|n| match n {
                SceneNode::Rect { y, h, .. } => Some(*y + *h),
                _ => None,
            })
            .collect();
        assert_eq!(bottom_edges.len(), 2);
        assert!(
            (bottom_edges[0] - bottom_edges[1]).abs() < 0.5,
            "bottom edges should align: {} vs {}",
            bottom_edges[0],
            bottom_edges[1],
        );
    }

    #[test]
    fn flush_left_sets_correct_style() {
        let el = div().flush_left();
        assert_eq!(el.layout.direction, Direction::Column);
        assert_eq!(el.layout.align, Align::Start);
    }

    #[test]
    fn align_top_sets_correct_style() {
        let el = div().align_top();
        assert_eq!(el.layout.direction, Direction::Row);
        assert_eq!(el.layout.align, Align::Start);
    }

    // -- render_interactive hit regions --

    #[test]
    fn render_interactive_no_handlers_no_hit_regions() {
        let el = div().w(100.0).h(50.0).bg("blue");
        let result = el.render_interactive(&mut test_cx());
        assert_eq!(result.nodes.len(), 1);
        assert!(result.hit_regions.is_empty());
    }

    #[test]
    fn render_interactive_on_click_creates_hit_region() {
        let el = div().w(100.0).h(50.0).bg("blue").on_click(|_| {});
        let result = el.render_interactive(&mut test_cx());
        assert_eq!(result.hit_regions.len(), 1);
        let hr = &result.hit_regions[0];
        assert_eq!(hr.bounds.width, 100.0);
        assert_eq!(hr.bounds.height, 50.0);
        assert!(hr.focusable);
    }

    #[test]
    fn render_interactive_on_hover_creates_hit_region() {
        let el = div().w(80.0).h(40.0).bg("gray").on_hover(|_| {});
        let result = el.render_interactive(&mut test_cx());
        assert_eq!(result.hit_regions.len(), 1);
    }

    #[test]
    fn render_interactive_nested_handlers_get_unique_ids() {
        let el = div()
            .w(400.0)
            .h(200.0)
            .child(div().w(100.0).h(50.0).bg("red").on_click(|_| {}))
            .child(div().w(100.0).h(50.0).bg("blue").on_click(|_| {}));
        let result = el.render_interactive(&mut test_cx());
        assert_eq!(result.hit_regions.len(), 2);
        assert_ne!(result.hit_regions[0].view_id, result.hit_regions[1].view_id);
    }

    #[test]
    fn render_interactive_nodes_still_correct() {
        let el = div()
            .w(100.0)
            .h(50.0)
            .bg("blue")
            .on_click(|_| {})
            .child(text("Click"));
        let result = el.render_interactive(&mut test_cx());
        assert_eq!(result.nodes.len(), 2); // Rect + Text
        assert_eq!(result.hit_regions.len(), 1);
    }
}
