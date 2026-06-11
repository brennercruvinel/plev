//! Builder API surface: children composition and style sugar.

use super::test_cx;
use crate::builder::*;
use crate::compositor::SceneNode;
use crate::view::View;

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
