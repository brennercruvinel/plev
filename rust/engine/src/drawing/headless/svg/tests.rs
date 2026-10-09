//! The SVG export against the frame path: the glyphs it writes are the
//! ones the frame shaped, the structure follows the scene, and every
//! approximation is on the report. GPU tests return early when the
//! machine has no adapter, as the rest of the headless tests do.

use super::super::{HeadlessError, Renderer, Viewport};
use super::write::{escape, hex, num, opacity};
use super::*;
use crate::drawing::compositor::{
    Compositor, GradientRectParams, RoundedRectParams, SceneNode, TextNodeKey,
};
use crate::drawing::gpu::RenderConfig;
use crate::drawing::path::PathBuilder;
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::hoff::PAGE_BG;

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

/// A scene of everything the SVG approximates: a drop and an inset
/// shadow, a gradient, a backdrop blur, a clip, an image. Each one is on
/// the report, once, and the file still parses.
fn approximate_scene(c: &mut Compositor) {
    c.begin_frame();
    c.push_clip(0.0, 0.0, 200.0, 120.0);
    c.push(SceneNode::Shadow {
        x: 20.0,
        y: 20.0,
        w: 100.0,
        h: 50.0,
        corner_radius: 8.0,
        blur_radius: 12.0,
        offset: [0.0, 4.0],
        color: [0.0, 0.0, 0.0, 0.5],
        inset: false,
    });
    c.draw_gradient_rect(GradientRectParams {
        x: 20.0,
        y: 20.0,
        w: 100.0,
        h: 50.0,
        color: [1.0, 0.0, 0.0, 1.0],
        color2: [0.0, 0.0, 1.0, 1.0],
        angle_deg: 90.0,
        corner_radius: 8.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
    c.push(SceneNode::Shadow {
        x: 20.0,
        y: 20.0,
        w: 100.0,
        h: 50.0,
        corner_radius: 8.0,
        blur_radius: 16.0,
        offset: [2.0, 4.0],
        color: [1.0, 1.0, 1.0, 0.1],
        inset: true,
    });
    c.draw_backdrop_blur(130.0, 20.0, 60.0, 60.0, 6.0, 10.0);
    c.draw_image(
        130.0,
        90.0,
        20.0,
        20.0,
        crate::drawing::gpu::image::ImageHandle {
            id: 7,
            atlas_x: 1,
            atlas_y: 1,
            width: 32,
            height: 32,
        },
        4.0,
    );
    c.pop_clip();
}

#[test]
fn every_approximation_is_on_the_report_once_and_the_file_still_parses() {
    let Some(mut r) = renderer(Viewport::new(200.0, 120.0, 1.0)) else {
        return;
    };
    let mut c = Compositor::new();
    approximate_scene(&mut c);
    let export = match r.svg(&c, Some([1.0, 1.0, 1.0, 1.0])) {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };
    let svg = &export.svg;
    let kinds: Vec<&str> = export
        .report
        .iter()
        .map(|a| {
            a.node
                .rsplit('(')
                .next()
                .unwrap_or("")
                .trim_end_matches(')')
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "clip",
            "shadow",
            "gradient rect",
            "shadow",
            "backdrop blur",
            "image"
        ],
        "{}",
        export.report_text()
    );
    assert!(export.report[1].what.starts_with("drop shadow"));
    assert!(export.report[3].what.starts_with("inset shadow"));
    assert_eq!(export.report[1].node, "layer 0, node 1 (shadow)");
    assert_eq!(count(&export.report_text(), "\n"), 6);
    assert_eq!(
        count(svg, "<clipPath "),
        2,
        "the clip and the inset's bounds"
    );
    assert_eq!(count(svg, "<filter "), 2);
    assert_eq!(count(svg, "<linearGradient "), 1);
    assert!(svg.contains("color-interpolation=\"linearRGB\""));
    // the gradient runs left to right across the box at 90 degrees
    assert!(
        svg.contains("x1=\"20\" y1=\"45\" x2=\"120\" y2=\"45\""),
        "{svg}"
    );
    assert!(svg.contains("fill=\"url(#gradient"));
    assert!(svg.contains("<!-- image 32x32 at 130 90"));
    assert!(svg.contains("<!-- backdrop blur at 130 20"));
    assert_eq!(count(svg, "<g"), count(svg, "</g>"));
    if let Err(e) = ::svg::convert(svg, "approximate") {
        panic!("the importer refused the export: {e}\n{svg}");
    }
}

/// A shadow entirely off the viewport is culled as the frame culls it;
/// a pop without a push is reported; a push without a pop still closes
/// its group.
#[test]
fn culled_and_unbalanced_nodes_follow_the_frame() {
    let Some(mut r) = renderer(Viewport::new(100.0, 100.0, 1.0)) else {
        return;
    };
    let mut c = Compositor::new();
    c.begin_frame();
    c.pop_clip();
    c.push(SceneNode::Shadow {
        x: 500.0,
        y: 500.0,
        w: 10.0,
        h: 10.0,
        corner_radius: 0.0,
        blur_radius: 4.0,
        offset: [0.0; 2],
        color: [0.0, 0.0, 0.0, 1.0],
        inset: false,
    });
    c.draw_rect(500.0, 0.0, 10.0, 10.0, [1.0, 0.0, 0.0, 1.0]);
    c.push_clip(10.0, 10.0, 50.0, 50.0);
    c.push_clip(40.0, 40.0, 50.0, 50.0);
    c.draw_rect(0.0, 0.0, 100.0, 100.0, [0.0, 1.0, 0.0, 1.0]);
    let export = match r.svg(&c, None) {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };
    let svg = &export.svg;
    assert!(
        !svg.contains("<filter"),
        "the off-screen shadow was written"
    );
    assert!(!svg.contains("#ff0000"), "the off-screen rect was written");
    assert!(svg.contains("#00ff00"));
    // the second clip is the intersection of both
    assert!(svg.contains("<rect x=\"40\" y=\"40\" width=\"20\" height=\"20\"/></clipPath>"));
    assert_eq!(count(svg, "<g"), count(svg, "</g>"));
    assert_eq!(export.report.len(), 3, "{}", export.report_text());
    assert!(export.report[0].what.contains("PopClip without"));
}

/// A tessellated shape comes back as the polygon bounding its triangles:
/// a filled rect is one four-corner loop, a frame with a hole is two
/// loops, and both draw without a report.
#[test]
fn a_tessellated_path_is_the_outline_of_its_triangles() {
    let Some(mut r) = renderer(Viewport::new(100.0, 100.0, 1.0)) else {
        return;
    };
    let mut c = Compositor::new();
    c.begin_frame();
    c.draw_path(
        PathBuilder::new()
            .move_to(10.0, 10.0)
            .line_to(50.0, 10.0)
            .line_to(50.0, 30.0)
            .line_to(10.0, 30.0)
            .close()
            .fill([0.0, 0.0, 1.0, 1.0]),
    );
    // a square frame: outer square, inner square the other way round
    c.draw_path(
        PathBuilder::new()
            .move_to(60.0, 10.0)
            .line_to(90.0, 10.0)
            .line_to(90.0, 40.0)
            .line_to(60.0, 40.0)
            .close()
            .move_to(70.0, 20.0)
            .line_to(70.0, 30.0)
            .line_to(80.0, 30.0)
            .line_to(80.0, 20.0)
            .close()
            .fill([0.0, 1.0, 0.0, 0.5]),
    );
    let export = match r.svg(&c, None) {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };
    let svg = &export.svg;
    assert!(export.report.is_empty(), "{}", export.report_text());
    let paths: Vec<&str> = svg
        .lines()
        .filter(|l| l.trim_start().starts_with("<path "))
        .collect();
    assert_eq!(paths.len(), 2, "{svg}");
    let rect = paths[0];
    assert!(rect.contains("fill=\"#0000ff\""));
    assert_eq!(count(rect, "M"), 1, "{rect}");
    assert_eq!(count(rect, "L"), 3, "{rect}");
    for corner in ["10 10", "50 10", "50 30", "10 30"] {
        assert!(rect.contains(corner), "{rect} lacks {corner}");
    }
    let frame = paths[1];
    assert!(frame.contains("fill=\"#00ff00\" fill-opacity=\"0.5\""));
    assert_eq!(count(frame, "M"), 2, "{frame}");
    assert_eq!(count(frame, "Z"), 2, "{frame}");
    if let Err(e) = ::svg::convert(svg, "paths") {
        panic!("the importer refused the export: {e}\n{svg}");
    }
}

/// A mesh that is not a clean partition (two triangles walking one edge
/// the same way) falls back to one subpath per triangle, reported.
#[test]
fn a_mesh_that_is_not_a_partition_falls_back_to_triangles() {
    let Some(mut r) = renderer(Viewport::new(100.0, 100.0, 1.0)) else {
        return;
    };
    let v = |x: f32, y: f32| crate::drawing::compositor::QuadVertex {
        position: [x, y],
        color: [1.0, 0.0, 0.0, 1.0],
    };
    let data = crate::drawing::path::TessellatedPath {
        vertices: vec![v(0.0, 0.0), v(10.0, 0.0), v(0.0, 10.0), v(10.0, 10.0)],
        indices: vec![0, 1, 2, 0, 1, 3],
        hash: 1,
    };
    let mut c = Compositor::new();
    c.begin_frame();
    c.draw_path(data);
    let export = match r.svg(&c, None) {
        Ok(e) => e,
        Err(e) => panic!("{e}"),
    };
    assert_eq!(export.report.len(), 1);
    assert!(
        export.report[0].what.contains("seams"),
        "{}",
        export.report[0]
    );
    let svg = &export.svg;
    assert_eq!(count(svg, "M"), 2, "{svg}");
    assert_eq!(count(svg, "Z"), 2, "{svg}");
}
