//! Tessellation: a usvg tree into `engine` tessellated paths, in
//! document paint order (usvg keeps children back-to-front, so the
//! output vector is already bottom-first; no reversal). usvg has already
//! resolved groups, transforms, use/defs and lowered every shape (rect,
//! circle, ellipse, line, polyline, polygon) to a path, so this module
//! only walks paths, bakes each one's absolute transform into its points,
//! and tessellates fill then stroke through `PathBuilder`.
//!
//! colors follow the engine convention lot also uses: sRGB channels as
//! 0..1 floats passed straight through (the gpu linearizes on write),
//! alpha from fill/stroke opacity. gradients and patterns collapse to a
//! representative solid color; filters, clip paths, masks, images and
//! text are skipped with a one-time log, never a panic.

use std::collections::HashSet;

use engine::path::{PathBuilder, TessellatedPath};
use usvg::tiny_skia_path::{Path as SkPath, Point, Transform};
use usvg::{Group, Node, Paint};

/// One transformed sub-path command, in final canvas coordinates. Built
/// once per path and replayed into a fresh builder for fill and stroke.
enum Seg {
    Move(f32, f32),
    Line(f32, f32),
    Quad([f32; 2], [f32; 2]),
    Cubic([f32; 2], [f32; 2], [f32; 2]),
    Close,
}

/// Tessellate every visible path in the tree into drawable geometry.
pub fn tessellate(tree: &usvg::Tree) -> Vec<TessellatedPath> {
    let mut out = Vec::new();
    let mut warned = HashSet::new();
    walk(tree.root(), &mut out, &mut warned);
    out
}

fn warn_once(warned: &mut HashSet<&'static str>, msg: &'static str) {
    if warned.insert(msg) {
        log::warn!("svg: {msg}");
    }
}

fn walk(group: &Group, out: &mut Vec<TessellatedPath>, warned: &mut HashSet<&'static str>) {
    for node in group.children() {
        match node {
            Node::Group(g) => walk(g, out, warned),
            Node::Path(p) => emit_path(p, out, warned),
            Node::Image(_) => warn_once(warned, "image element skipped (raster, not vector)"),
            Node::Text(_) => warn_once(warned, "text element skipped (build without fontdb)"),
        }
    }
}

fn emit_path(
    path: &usvg::Path,
    out: &mut Vec<TessellatedPath>,
    warned: &mut HashSet<&'static str>,
) {
    if path.fill().is_none() && path.stroke().is_none() {
        return;
    }
    let abs = path.abs_transform();
    let segs = to_segs(path.data(), &abs);
    if segs.is_empty() {
        return;
    }
    // svg default paint order: fill under stroke.
    if let Some(fill) = path.fill() {
        let color = paint_color(fill.paint(), fill.opacity().get(), warned);
        push_if_visible(build(&segs).fill(color), color, out);
    }
    if let Some(stroke) = path.stroke() {
        let color = paint_color(stroke.paint(), stroke.opacity().get(), warned);
        // width is in the path's local units; the geometry is already in
        // canvas space, so scale the width by the transform's scale too.
        let w = (stroke.width().get() * scale_factor(&abs)).max(0.1);
        push_if_visible(build(&segs).stroke(color, w), color, out);
    }
}

fn push_if_visible(tess: TessellatedPath, color: [f32; 4], out: &mut Vec<TessellatedPath>) {
    if color[3] > 0.0 && !tess.vertices.is_empty() {
        out.push(tess);
    }
}

/// Bake the path's absolute transform into every point, once.
fn to_segs(data: &SkPath, abs: &Transform) -> Vec<Seg> {
    let map = |x: f32, y: f32| -> [f32; 2] {
        let mut p = Point::from_xy(x, y);
        abs.map_point(&mut p);
        [p.x, p.y]
    };
    let mut segs = Vec::new();
    let mut open = false;
    for seg in data.segments() {
        use usvg::tiny_skia_path::PathSegment as S;
        match seg {
            S::MoveTo(p) => {
                if open {
                    segs.push(Seg::Close);
                }
                let m = map(p.x, p.y);
                segs.push(Seg::Move(m[0], m[1]));
                open = true;
            }
            S::LineTo(p) => {
                let m = map(p.x, p.y);
                segs.push(Seg::Line(m[0], m[1]));
            }
            S::QuadTo(c, p) => segs.push(Seg::Quad(map(c.x, c.y), map(p.x, p.y))),
            S::CubicTo(c1, c2, p) => {
                segs.push(Seg::Cubic(map(c1.x, c1.y), map(c2.x, c2.y), map(p.x, p.y)))
            }
            S::Close => {
                segs.push(Seg::Close);
                open = false;
            }
        }
    }
    segs
}

fn build(segs: &[Seg]) -> PathBuilder {
    let mut b = PathBuilder::new();
    for s in segs {
        b = match s {
            Seg::Move(x, y) => b.move_to(*x, *y),
            Seg::Line(x, y) => b.line_to(*x, *y),
            Seg::Quad(c, p) => b.quadratic_bezier_to(*c, *p),
            Seg::Cubic(c1, c2, p) => b.cubic_bezier_to(*c1, *c2, *p),
            Seg::Close => b.close(),
        };
    }
    b
}

/// Solid rgba for a paint. Solid colors pass through; gradients average
/// their stops (rgb and opacity); patterns fall back to a neutral grey.
fn paint_color(paint: &Paint, opacity: f32, warned: &mut HashSet<&'static str>) -> [f32; 4] {
    match paint {
        Paint::Color(c) => rgba(c.red, c.green, c.blue, opacity),
        Paint::LinearGradient(g) => {
            warn_once(warned, "linear gradient approximated as solid color");
            avg_stops(g.stops(), opacity)
        }
        Paint::RadialGradient(g) => {
            warn_once(warned, "radial gradient approximated as solid color");
            avg_stops(g.stops(), opacity)
        }
        Paint::Pattern(_) => {
            warn_once(warned, "pattern fill approximated as neutral grey");
            rgba(128, 128, 128, opacity)
        }
    }
}

fn avg_stops(stops: &[usvg::Stop], opacity: f32) -> [f32; 4] {
    if stops.is_empty() {
        return rgba(128, 128, 128, opacity);
    }
    let n = stops.len() as f32;
    let (mut r, mut g, mut b, mut a) = (0.0, 0.0, 0.0, 0.0);
    for s in stops {
        let c = s.color();
        r += c.red as f32;
        g += c.green as f32;
        b += c.blue as f32;
        a += s.opacity().get();
    }
    [r / n / 255.0, g / n / 255.0, b / n / 255.0, opacity * a / n]
}

fn rgba(r: u8, g: u8, b: u8, a: f32) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a]
}

/// Uniform scale estimate (sqrt of |determinant|), for stroke widths.
fn scale_factor(t: &Transform) -> f32 {
    (t.sx * t.sy - t.kx * t.ky).abs().sqrt()
}
