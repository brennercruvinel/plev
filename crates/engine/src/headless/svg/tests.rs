//! The SVG export against the frame path: the glyphs it writes are the
//! ones the frame shaped, the structure follows the scene, and every
//! approximation is on the report. GPU tests return early when the
//! machine has no adapter, as the rest of the headless tests do.

use super::super::{HeadlessError, Renderer, Viewport};
use super::write::{escape, hex, num, opacity};
use super::*;
use crate::compositor::{Compositor, RoundedRectParams, SceneNode, TextNodeKey};
use crate::gpu::RenderConfig;
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::hoff::PAGE_BG;

fn renderer(viewport: Viewport) -> Option<Renderer> {
    match Renderer::new(viewport, RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// A rect, a rounded rect with a border and a text run: everything the
/// SVG holds exactly.
fn exact_scene(c: &mut Compositor) {
    c.begin_frame();
    c.draw_rect(10.0, 10.0, 20.0, 20.0, [1.0, 0.0, 0.0, 1.0]);
    c.draw_rounded_rect(RoundedRectParams {
        x: 40.0,
        y: 40.0,
        w: 120.0,
        h: 60.0,
        color: [0.2, 0.4, 0.9, 1.0],
        corner_radius: 12.0,
        border_width: 1.0,
        border_color: [1.0, 1.0, 1.0, 0.3],
    });
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(
            "headless",
            &TextStyle::new(16.0).with_line_height(22.4),
            None,
        ),
        x: 50.0,
        y: 60.0,
        color: [1.0, 1.0, 1.0, 1.0],
    });
}

#[test]
fn numbers_are_short_and_never_negative_zero() {
    assert_eq!(num(12.0), "12");
    assert_eq!(num(12.5), "12.5");
    assert_eq!(num(12.345), "12.35");
    assert_eq!(num(-0.001), "0");
    assert_eq!(num(-3.25), "-3.25");
    assert_eq!(num(f32::NAN), "0");
}

#[test]
fn colors_are_srgb_hex_with_the_alpha_apart() {
    assert_eq!(hex([1.0, 0.0, 0.5, 1.0]), "#ff0080");
    assert_eq!(hex([0.1882, 0.1882, 0.1882, 1.0]), "#303030");
    assert_eq!(hex([2.0, -1.0, 0.0, 0.0]), "#ff0000");
    assert_eq!(opacity("fill-opacity", 1.0), "");
    assert_eq!(opacity("fill-opacity", 0.5), " fill-opacity=\"0.5\"");
    assert_eq!(
        fill([0.0, 0.0, 0.0, 0.25]),
        " fill=\"#000000\" fill-opacity=\"0.25\""
    );
}

#[test]
fn labels_are_escaped_for_an_attribute() {
    assert_eq!(
        escape("a < b & \"c\"\nd"),
        "a &lt; b &amp; &quot;c&quot;&#10;d"
    );
    assert_eq!(escape("tab\tbell\u{7}"), "tab&#9;bell");
}

#[test]
fn a_degenerate_viewport_is_an_error_here_too() {
    let mut c = Compositor::new();
    c.begin_frame();
    let Some(mut r) = renderer(Viewport::new(8.0, 8.0, 1.0)) else {
        return;
    };
    let bad = Viewport::new(0.0, 8.0, 1.0);
    match export(&c, &mut r.text_system, bad, None) {
        Err(HeadlessError::InvalidViewport(v)) => assert_eq!(v, bad),
        other => panic!("expected InvalidViewport, got {:?}", other.map(|_| ())),
    }
}

/// The document's frame: physical size as width and height, logical size
/// as the viewBox, one element per node, the text as a labelled group of
/// glyph paths, and nothing on the report.
#[test]
fn an_exact_scene_exports_one_element_per_node_and_reports_nothing() {
    let Some(mut r) = renderer(Viewport::new(240.0, 140.0, 2.0)) else {
        return;
    };
    let mut c = Compositor::new();
    exact_scene(&mut c);
    let export = match r.svg(&c, Some(PAGE_BG.to_array())) {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };
    let svg = &export.svg;
    assert!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg "));
    assert!(svg.contains(" width=\"480\" height=\"280\" viewBox=\"0 0 240 140\">"));
    // background, the rect, the rounded rect's fill and its border stroke
    assert_eq!(count(svg, "<rect "), 4, "{svg}");
    assert!(svg.contains("<rect width=\"240\" height=\"140\" fill=\"#303030\"/>"));
    assert!(svg.contains("<rect x=\"10\" y=\"10\" width=\"20\" height=\"20\" fill=\"#ff0000\"/>"));
    // the fill is inset by the border, the stroke sits half a border in
    assert!(svg.contains(
        "<rect x=\"41\" y=\"41\" width=\"118\" height=\"58\" rx=\"11\" fill=\"#3366e6\"/>"
    ));
    assert!(svg.contains(
        "<rect x=\"40.5\" y=\"40.5\" width=\"119\" height=\"59\" rx=\"11.5\" fill=\"none\" \
         stroke=\"#ffffff\" stroke-opacity=\"0.3\" stroke-width=\"1\"/>"
    ));
    assert_eq!(
        count(svg, "<g aria-label=\"headless\" fill=\"#ffffff\">"),
        1
    );
    // eight letters, each with a contour
    assert_eq!(count(svg, "<path d=\"M"), 8, "{svg}");
    assert_eq!(count(svg, "<g"), count(svg, "</g>"));
    assert!(svg.ends_with("</svg>\n"));
    assert!(export.report.is_empty(), "{:?}", export.report);
    assert_eq!(export.report_text(), "");
}

/// The importer reads the export back: the file is well-formed SVG with
/// drawable content, by the same parser the project imports svg with.
#[test]
fn the_svg_importer_reads_the_export_back() {
    let Some(mut r) = renderer(Viewport::new(240.0, 140.0, 1.0)) else {
        return;
    };
    let mut c = Compositor::new();
    exact_scene(&mut c);
    let export = match r.svg(&c, Some(PAGE_BG.to_array())) {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };
    match ::svg::convert(&export.svg, "export") {
        Ok((_, stats)) => {
            assert_eq!((stats.width, stats.height), (240.0, 140.0));
            // the rects and every glyph
            assert!(stats.paths >= 4 + 8, "{stats:?}");
        }
        Err(e) => panic!("the importer refused the export: {e}\n{}", export.svg),
    }
}

/// The acceptance criterion for the text: the glyphs the SVG places span
/// exactly the width layout measured for the run, for three runs of
/// different face, weight and tracking.
#[test]
fn the_glyph_advances_of_a_run_sum_to_its_measured_width() {
    let Some(mut r) = renderer(Viewport::new(400.0, 100.0, 1.0)) else {
        return;
    };
    let runs = [
        ("hello world", TextStyle::new(16.0)),
        (
            "Inter SemiBold 600",
            TextStyle::new(14.0).with_family("Inter").with_weight(600),
        ),
        (
            "wide tracking",
            TextStyle::new(20.0)
                .with_weight(700)
                .with_letter_spacing(1.0),
        ),
    ];
    for (text, style) in runs {
        let key = TextNodeKey::from_style(text, &style, None);
        let glyphs = r.text_system.shaped_glyphs(&key);
        assert!(!glyphs.is_empty(), "{text}: no glyphs");
        assert!(glyphs.iter().all(|g| g.line == 0), "{text}: wrapped");
        let advances: f32 = glyphs.iter().map(|g| g.advance).sum();
        let (measured, _) = TextMeasurer::measure_styled(text, &style, None);
        assert!(
            (advances - measured).abs() < 0.01,
            "{text}: advances sum to {advances}, layout measured {measured}"
        );
        // the last glyph's origin plus its advance is the pen's end
        let last = glyphs.last().copied().unwrap_or(glyphs[0]);
        assert!(
            (last.x + last.advance - measured).abs() < 0.01,
            "{text}: the pen ends at {}, layout measured {measured}",
            last.x + last.advance
        );
    }
}

/// The frame and the export shape once: the glyphs asked for before a
/// draw are the glyphs the draw placed.
#[test]
fn the_glyphs_before_and_after_a_draw_are_the_same() {
    let Some(mut r) = renderer(Viewport::new(240.0, 140.0, 2.0)) else {
        return;
    };
    let key = TextNodeKey::from_style(
        "headless",
        &TextStyle::new(16.0).with_line_height(22.4),
        None,
    );
    let before = r.text_system.shaped_glyphs(&key);
    let mut c = Compositor::new();
    exact_scene(&mut c);
    if let Err(e) = r.render(&mut c, PAGE_BG.to_linear_array()) {
        panic!("{e}");
    }
    let after = r.text_system.shaped_glyphs(&key);
    assert_eq!(before, after);
    assert_eq!(before.len(), 8);
}

#[test]
fn two_exports_of_one_scene_are_identical() {
    let Some(mut a) = renderer(Viewport::new(240.0, 140.0, 2.0)) else {
        return;
    };
    let Some(mut b) = renderer(Viewport::new(240.0, 140.0, 2.0)) else {
        return;
    };
    let mut c = Compositor::new();
    exact_scene(&mut c);
    let first = a.svg(&c, None).map_err(|e| e.to_string());
    // a renderer that drew the frame first exports the same text
    if let Err(e) = b.render(&mut c, PAGE_BG.to_linear_array()) {
        panic!("{e}");
    }
    let second = b.svg(&c, None).map_err(|e| e.to_string());
    assert_eq!(first, second);
    let svg = first.map(|e| e.svg).unwrap_or_default();
    assert!(
        !svg.contains("<rect width=\"240\""),
        "no background asked for"
    );
}

/// Layers composite in z-order whatever their creation order, and a
/// hidden layer draws nothing.
#[test]
fn layers_are_written_in_composite_order_and_hidden_ones_are_not() {
    let Some(mut r) = renderer(Viewport::new(100.0, 100.0, 1.0)) else {
        return;
    };
    let mut c = Compositor::new();
    let top = c.create_layer(20);
    let hidden = c.create_layer(30);
    let mid = c.create_layer(10);
    c.set_layer_visible(hidden, false);
    c.set_layer_opacity(mid, 0.5);
    c.begin_frame();
    c.push_to_layer(
        top,
        SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: [0.0, 1.0, 0.0, 1.0],
        },
    );
    c.push_to_layer(
        hidden,
        SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: [1.0, 0.0, 1.0, 1.0],
        },
    );
    c.push_to_layer(
        mid,
        SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            color: [0.0, 0.0, 1.0, 1.0],
        },
    );
    c.draw_rect(0.0, 0.0, 10.0, 10.0, [1.0, 0.0, 0.0, 1.0]);
    let svg = match r.svg(&c, None) {
        Ok(e) => e.svg,
        Err(e) => panic!("{e}"),
    };
    let at = |needle: &str| {
        svg.find(needle)
            .unwrap_or_else(|| panic!("{needle} missing"))
    };
    assert!(at("#ff0000") < at("#0000ff"), "default layer before z 10");
    assert!(at("#0000ff") < at("#00ff00"), "z 10 before z 20");
    assert!(!svg.contains("#ff00ff"), "the hidden layer drew");
    assert!(svg.contains("<g id=\"layer-3\" opacity=\"0.5\">"));
}
