#[cfg(test)]
mod tests {
    use crate::builder::*;
    use crate::color::Color;
    use crate::compositor::SceneNode;
    use crate::view::{View, ViewContext};

    fn test_cx() -> ViewContext {
        ViewContext::new(800.0, 600.0)
    }

    #[test]
    fn div_produces_no_nodes_without_bg() {
        let el = div();
        let nodes = el.render(&mut test_cx());
        assert!(nodes.is_empty());
    }

    #[test]
    fn div_with_bg_and_size_produces_rect() {
        let el = div().bg("blue").w(100.0).h(50.0);
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        let SceneNode::Rect { w, h, color, .. } = &nodes[0] else {
            panic!("Expected Rect, got {:?}", &nodes[0]);
        };
        assert_eq!(*w, 100.0);
        assert_eq!(*h, 50.0);
        assert_eq!(*color, Color::BLUE.to_array());
    }

    #[test]
    fn text_produces_text_node() {
        let el = text("Hello");
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        let SceneNode::Text { key, color, .. } = &nodes[0] else {
            panic!("Expected Text, got {:?}", &nodes[0]);
        };
        assert_eq!(key.text, "Hello");
        assert_eq!(key.font_size_bits, 16.0_f32.to_bits());
        assert_eq!(*color, Color::WHITE.to_array());
    }

    #[test]
    fn text_font_size_changes_size_and_line_height() {
        let el = text("Hi").font_size(32.0);
        let nodes = el.render(&mut test_cx());
        let SceneNode::Text { key, .. } = &nodes[0] else {
            panic!("Expected Text, got {:?}", &nodes[0]);
        };
        assert_eq!(key.font_size_bits, 32.0_f32.to_bits());
        assert_eq!(key.line_height_bits, (32.0_f32 * 1.3).to_bits());
    }

    #[test]
    fn text_color_applied() {
        let el = text("Hi").text_color("red");
        let nodes = el.render(&mut test_cx());
        let SceneNode::Text { color, .. } = &nodes[0] else {
            panic!("Expected Text, got {:?}", &nodes[0]);
        };
        assert_eq!(*color, Color::RED.to_array());
    }

    #[test]
    fn child_composition() {
        let el = div()
            .bg("dark_gray")
            .w(200.0)
            .h(100.0)
            .child(text("A"))
            .child(text("B"));
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 3);
        assert!(matches!(&nodes[0], SceneNode::Rect { .. }));
        assert!(matches!(&nodes[1], SceneNode::Text { .. }));
        assert!(matches!(&nodes[2], SceneNode::Text { .. }));
    }

    #[test]
    fn children_iter() {
        let items = vec!["A", "B", "C"];
        let el = div()
            .w(200.0)
            .h(100.0)
            .bg("gray")
            .children(items.into_iter().map(text));
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 4);
    }

    #[test]
    fn column_layout_stacks_vertically() {
        let el = div()
            .col()
            .gap(10.0)
            .child(text("First"))
            .child(text("Second"));
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 2);
        if let (SceneNode::Text { y: y1, .. }, SceneNode::Text { y: y2, .. }) =
            (&nodes[0], &nodes[1])
        {
            assert!(*y2 > *y1, "Column layout: y2={} > y1={}", y2, y1);
        }
    }

    #[test]
    fn row_layout_stacks_horizontally() {
        let el = div()
            .row()
            .gap(10.0)
            .child(text("Left"))
            .child(text("Right"));
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 2);
        if let (SceneNode::Text { x: x1, .. }, SceneNode::Text { x: x2, .. }) =
            (&nodes[0], &nodes[1])
        {
            assert!(*x2 > *x1, "Row layout: x2={} > x1={}", x2, x1);
        }
    }

    #[test]
    fn padding_offsets_children() {
        let el = div().p(20.0).child(text("Padded"));
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        if let SceneNode::Text { x, y, .. } = &nodes[0] {
            assert_eq!(*x, 20.0);
            assert_eq!(*y, 20.0);
        }
    }

    #[test]
    fn nested_divs() {
        let el = div()
            .w(400.0)
            .h(300.0)
            .bg("dark_gray")
            .child(div().w(200.0).h(100.0).bg("blue").child(text("Nested")));
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn button_sugar() {
        let el = button("Click me");
        let nodes = el.render(&mut test_cx());
        assert!(!nodes.is_empty());
        assert!(nodes.iter().any(|n| matches!(n, SceneNode::Text { .. })));
    }

    #[test]
    fn on_click_stub_compiles() {
        let _el = div().on_click(|_e| {});
    }
    #[test]
    fn on_hover_stub_compiles() {
        let _el = div().on_hover(|_e| {});
    }

    #[test]
    fn rounded_and_shadow_stored() {
        let el = div().rounded(8.0).shadow(4.0);
        assert_eq!(el.style.corner_radius, 8.0);
        assert_eq!(el.style.shadow, 4.0);
    }

    #[test]
    fn rounded_named_presets() {
        assert_eq!(div().rounded("md").style.corner_radius, 4.0);
        assert_eq!(div().rounded("xl").style.corner_radius, 12.0);
        assert_eq!(div().rounded("full").style.corner_radius, 9999.0);
    }

    #[test]
    fn complex_tree() {
        let el = div()
            .col()
            .gap(10.0)
            .p(20.0)
            .bg("dark_gray")
            .w(400.0)
            .h(300.0)
            .child(text("Hello, plev!").font_size(32.0).text_color("white"))
            .child(
                div()
                    .row()
                    .gap(8.0)
                    .child(button("OK").on_click(|_| {}))
                    .child(button("Cancel")),
            );
        let nodes = el.render(&mut test_cx());
        assert!(nodes.len() >= 3, "Expected >= 3 nodes, got {}", nodes.len());
    }

    #[test]
    fn into_f32_accepts_ints() {
        let el = div().gap(4).p(8).w(100).h(50);
        assert_eq!(el.layout.gap, 4.0);
    }

    #[test]
    fn text_child_merges_content() {
        let el = text("").child("Hello");
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        let SceneNode::Text { key, .. } = &nodes[0] else {
            panic!("Expected Text, got {:?}", &nodes[0]);
        };
        assert_eq!(key.text, "Hello");
    }

    #[test]
    fn text_child_format_string() {
        let el = text("").child(format!("Count: {}", 42));
        let nodes = el.render(&mut test_cx());
        let SceneNode::Text { key, .. } = &nodes[0] else {
            panic!("Expected Text, got {:?}", &nodes[0]);
        };
        assert_eq!(key.text, "Count: 42");
    }

    #[test]
    fn string_child_on_div_adds_text_node() {
        let el = div().child("Hello");
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], SceneNode::Text { .. }));
    }

    #[test]
    fn child_if_works() {
        let el = div()
            .w(100.0)
            .h(50.0)
            .bg("gray")
            .child_if(|| true, || text("shown"));
        assert_eq!(el.render(&mut test_cx()).len(), 2);

        let el = div()
            .w(100.0)
            .h(50.0)
            .bg("gray")
            .child_if(|| false, || text("hidden"));
        assert_eq!(el.render(&mut test_cx()).len(), 1);
    }

    #[test]
    fn children_each_works() {
        let el = div().children_each(|| vec!["a", "b", "c"], |item| text(item));
        assert_eq!(el.render(&mut test_cx()).len(), 3);
    }

    #[test]
    fn bold_italic_flags() {
        let el = text("styled").bold().italic();
        assert!(el.style.bold);
        assert!(el.style.italic);
    }

    #[test]
    fn centered_alias() {
        let el = div().centered();
        assert_eq!(el.layout.align, Align::Center);
        assert_eq!(el.layout.justify, Justify::Center);
    }

    #[test]
    fn px_py_padding() {
        let el = div().px(4).py(2);
        assert_eq!(el.layout.padding.left, 4.0);
        assert_eq!(el.layout.padding.right, 4.0);
        assert_eq!(el.layout.padding.top, 2.0);
        assert_eq!(el.layout.padding.bottom, 2.0);
    }

    #[test]
    fn path_element_produces_path_node() {
        let circle = crate::path::PathBuilder::circle(50.0, 50.0, 25.0).fill([1.0, 0.0, 0.0, 1.0]);
        let el = path(circle);
        assert_eq!(el.render(&mut test_cx()).len(), 1);
    }

    #[test]
    fn path_in_div() {
        let circle = crate::path::PathBuilder::circle(50.0, 50.0, 25.0).fill([1.0, 0.0, 0.0, 1.0]);
        let el = div().w(200.0).h(200.0).bg("dark_gray").child(path(circle));
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 2);
        assert!(matches!(&nodes[0], SceneNode::Rect { .. }));
        assert!(matches!(&nodes[1], SceneNode::Path { .. }));
    }

    #[test]
    fn border_emits_rounded_rect() {
        let el = div().w(100.0).h(50.0).border(1.0).border_color("gray");
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], SceneNode::RoundedRect { .. }));
    }

    #[test]
    fn rounded_bg_emits_rounded_rect() {
        let el = div().w(100.0).h(50.0).bg("blue").rounded(8.0);
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], SceneNode::RoundedRect { .. }));
    }

    #[test]
    fn image_src_bytes_emits_image_node_with_natural_size() {
        // Tiny in-memory PNG (the png feature also enables encoding)
        let img = ::image::RgbaImage::from_pixel(6, 4, ::image::Rgba([1, 2, 3, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        img.write_to(&mut png, ::image::ImageFormat::Png).unwrap();

        let el = image().src_bytes(png.into_inner()).rounded(3.0);
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        let SceneNode::Image {
            w,
            h,
            image: handle,
            corner_radius,
            ..
        } = &nodes[0]
        else {
            panic!("Expected Image, got {:?}", &nodes[0]);
        };
        // Natural size drives layout when no explicit w/h is set
        assert_eq!((*w, *h), (6.0, 4.0));
        assert_eq!((handle.width, handle.height), (6, 4));
        assert_eq!(*corner_radius, 3.0);
    }

    #[test]
    fn image_without_source_emits_nothing() {
        let el = image().w(50.0).h(50.0);
        let nodes = el.render(&mut test_cx());
        assert!(nodes.is_empty());
    }

    #[test]
    fn clip_children_wraps_children_in_push_pop() {
        let el = div()
            .w(100.0)
            .h(50.0)
            .bg("blue")
            .clip_children()
            .child(div().w(300.0).h(20.0).bg("red"))
            .child(div().w(300.0).h(20.0).bg("green"));
        let nodes = el.render(&mut test_cx());

        // parent rect, PushClip, 2 child rects, PopClip
        assert_eq!(nodes.len(), 5);
        assert!(matches!(&nodes[0], SceneNode::Rect { .. }));
        let SceneNode::PushClip { x, y, w, h } = &nodes[1] else {
            panic!("Expected PushClip, got {:?}", &nodes[1]);
        };
        assert_eq!((*x, *y, *w, *h), (0.0, 0.0, 100.0, 50.0));
        assert!(matches!(&nodes[2], SceneNode::Rect { .. }));
        assert!(matches!(&nodes[3], SceneNode::Rect { .. }));
        assert!(matches!(&nodes[4], SceneNode::PopClip));
    }

    #[test]
    fn nested_clip_children_balance_push_pop() {
        let el = div().w(100.0).h(100.0).clip_children().child(
            div()
                .w(80.0)
                .h(80.0)
                .clip_children()
                .child(div().w(200.0).h(10.0).bg("red")),
        );
        let nodes = el.render(&mut test_cx());

        let pushes = nodes
            .iter()
            .filter(|n| matches!(n, SceneNode::PushClip { .. }))
            .count();
        let pops = nodes
            .iter()
            .filter(|n| matches!(n, SceneNode::PopClip))
            .count();
        assert_eq!(pushes, 2);
        assert_eq!(pops, 2);
        // Last node closes the outer clip
        assert!(matches!(nodes.last(), Some(SceneNode::PopClip)));
    }

    #[test]
    fn clip_children_without_children_emits_no_clip_nodes() {
        let el = div().w(100.0).h(50.0).bg("blue").clip_children();
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], SceneNode::Rect { .. }));
    }

    #[test]
    fn shadow_drop_emits_shadow_before_rect() {
        let el = div().w(100.0).h(50.0).bg("blue").rounded(8.0).shadow_drop(
            16.0,
            4.0,
            [0.0, 0.0, 0.0, 0.5],
        );
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 2);
        let SceneNode::Shadow {
            w,
            h,
            corner_radius,
            blur_radius,
            offset,
            color,
            ..
        } = &nodes[0]
        else {
            panic!("Expected Shadow first, got {:?}", &nodes[0]);
        };
        assert_eq!((*w, *h), (100.0, 50.0));
        assert_eq!(*corner_radius, 8.0);
        assert_eq!(*blur_radius, 16.0);
        assert_eq!(*offset, [0.0, 4.0]);
        assert_eq!(*color, [0.0, 0.0, 0.0, 0.5]);
        assert!(matches!(&nodes[1], SceneNode::RoundedRect { .. }));
    }

    #[test]
    fn shadow_inset_emits_inset_shadow_after_fill() {
        let el = div().w(100.0).h(50.0).bg("blue").rounded(8.0).shadow_inset(
            16.0,
            [2.0, 4.0],
            [248.0 / 255.0, 248.0 / 255.0, 248.0 / 255.0, 0.06],
        );
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 2);
        assert!(matches!(&nodes[0], SceneNode::RoundedRect { .. }));
        let SceneNode::Shadow {
            w,
            h,
            corner_radius,
            blur_radius,
            offset,
            inset,
            ..
        } = &nodes[1]
        else {
            panic!("Expected inset Shadow after fill, got {:?}", &nodes[1]);
        };
        assert_eq!((*w, *h), (100.0, 50.0));
        assert_eq!(*corner_radius, 8.0);
        assert_eq!(*blur_radius, 16.0);
        assert_eq!(*offset, [2.0, 4.0]);
        assert!(*inset);
    }

    #[test]
    fn bg_linear_emits_gradient_rect() {
        let el = div().w(100.0).h(50.0).rounded(8.0).bg_linear(
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            45.0,
        );
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        let SceneNode::GradientRect {
            color,
            color2,
            angle_deg,
            corner_radius,
            ..
        } = &nodes[0]
        else {
            panic!("Expected GradientRect, got {:?}", &nodes[0]);
        };
        assert_eq!(*color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(*color2, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(*angle_deg, 45.0);
        assert_eq!(*corner_radius, 8.0);
    }

    #[test]
    fn bg_linear_takes_precedence_over_bg() {
        let el = div()
            .w(100.0)
            .h(50.0)
            .bg("blue")
            .bg_linear("red", "green", 0.0);
        let nodes = el.render(&mut test_cx());
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], SceneNode::GradientRect { .. }));
    }

    #[test]
    fn border_bottom_emits_thin_rect() {
        let el = div().w(200.0).h(40.0).border_bottom(1.0, "gray");
        let nodes = el.render(&mut test_cx());
        // border_bottom emits a thin Rect at the bottom
        assert_eq!(nodes.len(), 1);
        let SceneNode::Rect { y, h, .. } = &nodes[0] else {
            panic!("Expected Rect for border-bottom, got {:?}", &nodes[0]);
        };
        assert_eq!(*h, 1.0);
        assert_eq!(*y, 39.0); // 40.0 - 1.0
    }
}
