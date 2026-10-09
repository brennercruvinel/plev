//! Lucide icons as tessellated vector paths.
//!
//! Geometry is embedded from Lucide (<https://lucide.dev>, ISC license) as
//! the original 24x24 SVG primitives, stroked at width 2 with round caps and
//! joins — exactly how the icon set is designed to render. Each (name, size)
//! pair is tessellated once via `src/path` (lyon) and cached; per-call work
//! for a cache hit is a vertex copy with offset + tint applied.
//!
//! ```rust
//! use engine::widgets::icons;
//! let node = icons::icon_at("folder", 16.0, [1.0, 1.0, 1.0, 1.0], 8.0, 8.0);
//! assert!(node.is_some());
//! ```

mod data;
mod svg;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

use crate::drawing::compositor::{QuadVertex, SceneNode};
use crate::drawing::path::{PathBuilder, TessellatedPath};

/// Lucide design constants: 24x24 viewBox, stroke-width 2.
const VIEWBOX: f32 = 24.0;
const STROKE_WIDTH: f32 = 2.0;

/// Cache cap — icons come in few sizes, this is just a safety valve.
const CACHE_CAPACITY: usize = 512;

use data::{ICONS, Shape};
use svg::parse_svg_path;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// All available icon names, sorted.
pub fn icon_names() -> Vec<&'static str> {
    ICONS.iter().map(|(n, _)| *n).collect()
}

/// `true` if `name` is a known icon.
pub fn has_icon(name: &str) -> bool {
    ICONS.iter().any(|(n, _)| *n == name)
}

/// Tessellate `name` at `size` (square, top-left at origin), tinted `color`.
/// Returns `None` for unknown names.
pub fn icon(name: &str, size: f32, color: [f32; 4]) -> Option<SceneNode> {
    icon_at(name, size, color, 0.0, 0.0)
}

/// Like [`icon`], but translated so the icon's top-left is at `(x, y)`.
pub fn icon_at(name: &str, size: f32, color: [f32; 4], x: f32, y: f32) -> Option<SceneNode> {
    icon_path(name, size, color, x, y).map(|data| SceneNode::Path { data })
}

/// Raw tessellated geometry for callers that compose paths themselves.
pub fn icon_path(
    name: &str,
    size: f32,
    color: [f32; 4],
    x: f32,
    y: f32,
) -> Option<TessellatedPath> {
    let (canonical, shapes) = ICONS.iter().find(|(n, _)| *n == name)?;
    ICON_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() >= CACHE_CAPACITY {
            cache.clear();
        }
        let cached = cache
            .entry((canonical, size.to_bits()))
            .or_insert_with(|| tessellate_icon(shapes, size));

        // Cache hits only pay for this copy: translate + tint.
        let vertices = cached
            .vertices
            .iter()
            .map(|v| QuadVertex {
                position: [v.position[0] + x, v.position[1] + y],
                color,
            })
            .collect();

        // The base hash only covers geometry commands; fold in everything
        // else that affects pixels so compositor diffing stays correct.
        let mut h = FxHasher::default();
        cached.base_hash.hash(&mut h);
        x.to_bits().hash(&mut h);
        y.to_bits().hash(&mut h);
        for c in color {
            c.to_bits().hash(&mut h);
        }
        Some(TessellatedPath {
            vertices,
            indices: cached.indices.clone(),
            hash: h.finish(),
        })
    })
}

// ---------------------------------------------------------------------------
// Tessellation cache
// ---------------------------------------------------------------------------

struct CachedIcon {
    /// White geometry at origin; tinted/translated per request.
    vertices: Vec<QuadVertex>,
    indices: Vec<u32>,
    base_hash: u64,
}

thread_local! {
    static ICON_CACHE: RefCell<HashMap<(&'static str, u32), CachedIcon>> =
        RefCell::new(HashMap::new());
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

fn tessellate_icon(shapes: &[Shape], size: f32) -> CachedIcon {
    let s = size / VIEWBOX;
    let width = STROKE_WIDTH * s;

    let mut vertices: Vec<QuadVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut hasher = FxHasher::default();
    size.to_bits().hash(&mut hasher);

    // Each primitive is stroked independently (caps/joins never span SVG
    // elements) and the buffers merged with re-based indices.
    for shape in shapes {
        let path = match shape {
            Shape::Path(d) => match parse_svg_path(d, s) {
                Some(pb) => pb.stroke_round(WHITE, width),
                None => continue,
            },
            Shape::Circle(cx, cy, r) => {
                PathBuilder::circle(cx * s, cy * s, r * s).stroke_round(WHITE, width)
            }
            Shape::Line(x1, y1, x2, y2) => PathBuilder::new()
                .move_to(x1 * s, y1 * s)
                .line_to(x2 * s, y2 * s)
                .end_open()
                .stroke_round(WHITE, width),
            Shape::Polyline(points) => match poly_path(points, s, false) {
                Some(pb) => pb.stroke_round(WHITE, width),
                None => continue,
            },
            Shape::Polygon(points) => match poly_path(points, s, true) {
                Some(pb) => pb.stroke_round(WHITE, width),
                None => continue,
            },
            Shape::Rect(x, y, w, h, rx) => {
                PathBuilder::rounded_rect(x * s, y * s, w * s, h * s, rx * s)
                    .stroke_round(WHITE, width)
            }
        };

        path.hash.hash(&mut hasher);
        let base = vertices.len() as u32;
        vertices.extend(path.vertices);
        indices.extend(path.indices.iter().map(|i| i + base));
    }

    CachedIcon {
        vertices,
        indices,
        base_hash: hasher.finish(),
    }
}

/// Build a polyline/polygon path from an SVG `points` attribute.
fn poly_path(points: &str, scale: f32, close: bool) -> Option<PathBuilder> {
    let mut nums = points
        .split([' ', ','])
        .filter(|t| !t.is_empty())
        .map(|t| t.parse::<f32>().ok());
    let mut pb = PathBuilder::new();
    let (x, y) = (nums.next()??, nums.next()??);
    pb = pb.move_to(x * scale, y * scale);
    while let (Some(x), Some(y)) = (nums.next(), nums.next()) {
        pb = pb.line_to(x? * scale, y? * scale);
    }
    Some(if close { pb.close() } else { pb.end_open() })
}
