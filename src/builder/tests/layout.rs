//! Layout pipeline: positioning and text measurement parity
//! (one `TextStyle` shared by measurement and drawing).

use super::test_cx;
use crate::builder::*;
use crate::compositor::SceneNode;
use crate::view::View;

#[test]
fn column_layout_stacks_vertically() {
    let el = div()
        .col()
        .gap(10.0)
        .child(text("First"))
        .child(text("Second"));
    let nodes = el.render(&mut test_cx());
    assert_eq!(nodes.len(), 2);
    if let (SceneNode::Text { y: y1, .. }, SceneNode::Text { y: y2, .. }) = (&nodes[0], &nodes[1]) {
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
    if let (SceneNode::Text { x: x1, .. }, SceneNode::Text { x: x2, .. }) = (&nodes[0], &nodes[1]) {
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

// -- letter spacing: ONE TextStyle shared by measurement and drawing --

/// `.tracking()` must reach the measure spec, otherwise layout sizes the
/// box without the tracking the renderer draws and text overflows.
#[test]
fn tracking_reaches_text_measure_spec() {
    let el = text("Research Social").tracking(2.0);
    let mut items = Vec::new();
    let mut elements = Vec::new();
    crate::builder::layout_pipeline::collect_layout_items(&el, &mut items, &mut elements);
    let spec = items[0]
        .text
        .as_ref()
        .expect("text leaf must carry a measure spec");
    assert_eq!(spec.style.letter_spacing, 2.0);
}

/// `.tracking()` must reach the draw key, otherwise the shaper renders
/// without the tracking layout reserved space for.
#[test]
fn tracking_reaches_text_node_key() {
    let el = text("Research Social").tracking(2.0);
    let nodes = el.render(&mut test_cx());
    let SceneNode::Text { key, .. } = &nodes[0] else {
        panic!("Expected Text, got {:?}", &nodes[0]);
    };
    assert_eq!(key.letter_spacing_bits, 2.0_f32.to_bits());
}

/// Measure spec and draw key are built from the same resolved TextStyle:
/// every typographic field must agree (weight via `.bold()`, size,
/// line height, tracking).
#[test]
fn measure_spec_and_node_key_share_one_text_style() {
    let el = text("Quarterly").font_size(18.0).bold().tracking(0.35);

    let mut items = Vec::new();
    let mut elements = Vec::new();
    crate::builder::layout_pipeline::collect_layout_items(&el, &mut items, &mut elements);
    let spec = items[0].text.as_ref().expect("measure spec").clone();

    let nodes = el.render(&mut test_cx());
    let SceneNode::Text { key, .. } = &nodes[0] else {
        panic!("Expected Text, got {:?}", &nodes[0]);
    };

    assert_eq!(key.font_size_bits, spec.style.font_size.to_bits());
    assert_eq!(key.line_height_bits, spec.style.line_height.to_bits());
    assert_eq!(key.font_weight, spec.style.font_weight);
    assert_eq!(key.font_weight, 700);
    assert_eq!(key.letter_spacing_bits, spec.style.letter_spacing.to_bits());
    assert_eq!(key.font_family, spec.style.font_family);
}

/// End to end through the builder pipeline: a tracked run must measure
/// wider than the same run without tracking (layout reserves the extra
/// advance the renderer will draw).
#[test]
fn tracking_widens_builder_text_measurement() {
    let measure = |el: Element| {
        let mut items = Vec::new();
        let mut elements = Vec::new();
        crate::builder::layout_pipeline::collect_layout_items(&el, &mut items, &mut elements);
        let mut engine = crate::layout::LayoutEngine::new();
        engine.compute(&items, 800.0, 600.0)[0].width
    };
    let plain = measure(text("Research Social"));
    let tracked = measure(text("Research Social").tracking(2.0));
    assert!(
        tracked > plain,
        "tracking must widen the measured run ({plain} -> {tracked})"
    );
}
