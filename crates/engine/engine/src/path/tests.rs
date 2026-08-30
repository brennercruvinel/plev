//! Tests for the path module.

use super::*;

#[test]
fn circle_produces_vertices() {
    let path = PathBuilder::circle(100.0, 100.0, 50.0).fill([1.0, 0.0, 0.0, 1.0]);
    assert!(!path.vertices.is_empty(), "circle should produce vertices");
    assert!(!path.indices.is_empty(), "circle should produce indices");
    // Indices must be valid
    let max_idx = path.vertices.len() as u32;
    for &i in &path.indices {
        assert!(i < max_idx, "index {i} out of range (max {max_idx})");
    }
}

#[test]
fn rounded_rect_produces_vertices() {
    let path = PathBuilder::rounded_rect(10.0, 10.0, 200.0, 100.0, 12.0).fill([0.0, 1.0, 0.0, 1.0]);
    assert!(!path.vertices.is_empty());
    assert!(!path.indices.is_empty());
}

#[test]
fn ellipse_produces_vertices() {
    let path = PathBuilder::ellipse(50.0, 50.0, 40.0, 25.0).fill([0.0, 0.0, 1.0, 1.0]);
    assert!(!path.vertices.is_empty());
    assert!(!path.indices.is_empty());
}

#[test]
fn hash_is_stable() {
    let h1 = PathBuilder::circle(100.0, 100.0, 50.0)
        .fill([1.0, 0.0, 0.0, 1.0])
        .hash;
    let h2 = PathBuilder::circle(100.0, 100.0, 50.0)
        .fill([1.0, 0.0, 0.0, 1.0])
        .hash;
    assert_eq!(h1, h2, "same commands should produce same hash");
}

#[test]
fn different_paths_different_hash() {
    let h1 = PathBuilder::circle(100.0, 100.0, 50.0)
        .fill([1.0, 0.0, 0.0, 1.0])
        .hash;
    let h2 = PathBuilder::circle(200.0, 200.0, 30.0)
        .fill([1.0, 0.0, 0.0, 1.0])
        .hash;
    assert_ne!(h1, h2, "different commands should produce different hash");
}

#[test]
fn stroke_produces_vertices() {
    let path = PathBuilder::circle(100.0, 100.0, 50.0).stroke([1.0, 1.0, 1.0, 1.0], 2.0);
    assert!(!path.vertices.is_empty());
    assert!(!path.indices.is_empty());
}

#[test]
fn line_path() {
    let path = PathBuilder::new()
        .move_to(0.0, 0.0)
        .line_to(100.0, 0.0)
        .line_to(100.0, 100.0)
        .line_to(0.0, 100.0)
        .close()
        .fill([1.0, 1.0, 1.0, 1.0]);
    assert!(!path.vertices.is_empty());
    // A square should produce at least 4 vertices
    assert!(path.vertices.len() >= 4);
}

#[test]
fn zero_radius_rounded_rect_is_rect() {
    let path = PathBuilder::rounded_rect(0.0, 0.0, 100.0, 50.0, 0.0).fill([1.0, 1.0, 1.0, 1.0]);
    assert!(!path.vertices.is_empty());
}

#[test]
fn vertex_colors_match() {
    let color = [0.5, 0.3, 0.8, 1.0];
    let path = PathBuilder::circle(50.0, 50.0, 25.0).fill(color);
    for v in &path.vertices {
        assert_eq!(v.color, color);
    }
}

#[test]
fn quadratic_bezier_works() {
    let path = PathBuilder::new()
        .move_to(0.0, 0.0)
        .quadratic_bezier_to([50.0, 100.0], [100.0, 0.0])
        .close()
        .fill([1.0, 0.0, 0.0, 1.0]);
    assert!(!path.vertices.is_empty());
}

#[test]
fn open_path_stroke_does_not_panic() {
    // Regression: stroking an open sub-path (move_to/line_to, no close or
    // end_open) previously hit Lyon's "build() called before end()" abort —
    // a non-unwinding panic inside the macOS draw callback. The line-chart
    // case from the charts demo.
    let path = PathBuilder::new()
        .move_to(0.0, 0.0)
        .line_to(50.0, 20.0)
        .line_to(100.0, 0.0)
        .stroke([1.0, 1.0, 1.0, 1.0], 2.0);
    assert!(!path.vertices.is_empty());
    assert!(!path.indices.is_empty());
}

#[test]
fn open_path_fill_does_not_panic() {
    // Same guarantee for fills: an unclosed polyline must tessellate, not abort.
    let path = PathBuilder::new()
        .move_to(0.0, 0.0)
        .line_to(100.0, 0.0)
        .line_to(100.0, 100.0)
        .fill([1.0, 1.0, 1.0, 1.0]);
    assert!(!path.vertices.is_empty());
}
