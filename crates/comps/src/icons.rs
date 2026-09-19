//! Lucide icons as tessellated vector paths.
//!
//! Geometry is embedded from Lucide (<https://lucide.dev>, ISC license) as
//! the original 24x24 SVG primitives, stroked at width 2 with round caps and
//! joins — exactly how the icon set is designed to render. Each (name, size)
//! pair is tessellated once via `src/path` (lyon) and cached; per-call work
//! for a cache hit is a vertex copy with offset + tint applied.
//!
//! ```rust
//! use engine::ui::icons;
//! let node = icons::icon_at("folder", 16.0, [1.0, 1.0, 1.0, 1.0], 8.0, 8.0);
//! assert!(node.is_some());
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

use crate::compositor::{QuadVertex, SceneNode};
use crate::path::{PathBuilder, TessellatedPath};

/// Lucide design constants: 24x24 viewBox, stroke-width 2.
const VIEWBOX: f32 = 24.0;
const STROKE_WIDTH: f32 = 2.0;

/// Cache cap — icons come in few sizes, this is just a safety valve.
const CACHE_CAPACITY: usize = 512;

// ---------------------------------------------------------------------------
// Icon data — Lucide v0.469.0 primitives, verbatim
// ---------------------------------------------------------------------------

/// One SVG primitive of an icon, in the 24x24 viewBox.
enum Shape {
    /// SVG path `d` data.
    Path(&'static str),
    /// `<circle cx cy r>`.
    Circle(f32, f32, f32),
    /// `<line x1 y1 x2 y2>`.
    Line(f32, f32, f32, f32),
    /// `<polyline points>`.
    Polyline(&'static str),
    /// `<polygon points>` (closed).
    Polygon(&'static str),
    /// `<rect x y w h rx>`.
    Rect(f32, f32, f32, f32, f32),
}

use Shape::{Circle, Line, Path, Polygon, Polyline, Rect as SvgRect};

static ICONS: &[(&str, &[Shape])] = &[
    (
        "alert-triangle",
        &[
            Path("m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"),
            Path("M12 9v4"),
            Path("M12 17h.01"),
        ],
    ),
    (
        "book-open",
        &[
            Path("M12 7v14"),
            Path(
                "M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z",
            ),
        ],
    ),
    ("check", &[Path("M20 6 9 17l-5-5")]),
    ("chevron-down", &[Path("m6 9 6 6 6-6")]),
    ("chevron-left", &[Path("m15 18-6-6 6-6")]),
    ("chevron-right", &[Path("m9 18 6-6-6-6")]),
    ("chevron-up", &[Path("m18 15-6-6-6 6")]),
    ("circle", &[Circle(12.0, 12.0, 10.0)]),
    (
        "clipboard",
        &[
            SvgRect(8.0, 2.0, 8.0, 4.0, 1.0),
            Path("M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"),
        ],
    ),
    (
        "code",
        &[Polyline("16 18 22 12 16 6"), Polyline("8 6 2 12 8 18")],
    ),
    (
        "copy",
        &[
            SvgRect(8.0, 8.0, 14.0, 14.0, 2.0),
            Path("M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"),
        ],
    ),
    (
        "eye",
        &[
            Path(
                "M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0",
            ),
            Circle(12.0, 12.0, 3.0),
        ],
    ),
    (
        "file",
        &[
            Path("M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"),
            Path("M14 2v4a2 2 0 0 0 2 2h4"),
        ],
    ),
    (
        "folder",
        &[Path(
            "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z",
        )],
    ),
    (
        "folder-open",
        &[Path(
            "m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2",
        )],
    ),
    (
        "git-branch",
        &[
            Line(6.0, 3.0, 6.0, 15.0),
            Circle(18.0, 6.0, 3.0),
            Circle(6.0, 18.0, 3.0),
            Path("M18 9a9 9 0 0 1-9 9"),
        ],
    ),
    (
        "git-commit",
        &[
            Circle(12.0, 12.0, 3.0),
            Line(3.0, 12.0, 9.0, 12.0),
            Line(15.0, 12.0, 21.0, 12.0),
        ],
    ),
    (
        "heart",
        &[Path(
            "M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z",
        )],
    ),
    (
        "house",
        &[
            Path("M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"),
            Path(
                "M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
            ),
        ],
    ),
    (
        "info",
        &[
            Circle(12.0, 12.0, 10.0),
            Path("M12 16v-4"),
            Path("M12 8h.01"),
        ],
    ),
    (
        "layers",
        &[
            Path(
                "M12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83z",
            ),
            Path("M2 12a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 12"),
            Path("M2 17a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 17"),
        ],
    ),
    (
        "layout-grid",
        &[
            SvgRect(3.0, 3.0, 7.0, 7.0, 1.0),
            SvgRect(14.0, 3.0, 7.0, 7.0, 1.0),
            SvgRect(14.0, 14.0, 7.0, 7.0, 1.0),
            SvgRect(3.0, 14.0, 7.0, 7.0, 1.0),
        ],
    ),
    ("minus", &[Path("M5 12h14")]),
    ("moon", &[Path("M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z")]),
    ("play", &[Polygon("6 3 20 12 6 21 6 3")]),
    ("plus", &[Path("M5 12h14"), Path("M12 5v14")]),
    (
        "redo",
        &[
            Path("M21 7v6h-6"),
            Path("M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7"),
        ],
    ),
    (
        "save",
        &[
            Path(
                "M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z",
            ),
            Path("M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7"),
            Path("M7 3v4a1 1 0 0 0 1 1h7"),
        ],
    ),
    ("search", &[Circle(11.0, 11.0, 8.0), Path("m21 21-4.3-4.3")]),
    (
        "settings",
        &[
            Path(
                "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
            ),
            Circle(12.0, 12.0, 3.0),
        ],
    ),
    ("square", &[SvgRect(3.0, 3.0, 18.0, 18.0, 2.0)]),
    (
        "sun",
        &[
            Circle(12.0, 12.0, 4.0),
            Path("M12 2v2"),
            Path("M12 20v2"),
            Path("m4.93 4.93 1.41 1.41"),
            Path("m17.66 17.66 1.41 1.41"),
            Path("M2 12h2"),
            Path("M20 12h2"),
            Path("m6.34 17.66-1.41 1.41"),
            Path("m19.07 4.93-1.41 1.41"),
        ],
    ),
    (
        "terminal",
        &[Polyline("4 17 10 11 4 5"), Line(12.0, 19.0, 20.0, 19.0)],
    ),
    (
        "trash",
        &[
            Path("M3 6h18"),
            Path("M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"),
            Path("M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"),
        ],
    ),
    (
        "undo",
        &[
            Path("M3 7v6h6"),
            Path("M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"),
        ],
    ),
    ("x", &[Path("M18 6 6 18"), Path("m6 6 12 12")]),
];

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

// ---------------------------------------------------------------------------
// SVG path data parser
// ---------------------------------------------------------------------------

/// Parse an SVG path `d` string into a [`PathBuilder`], scaling every
/// coordinate by `scale`. Supports the full command set Lucide uses
/// (M/L/H/V/C/S/Q/T/A/Z, absolute and relative, implicit repetition).
fn parse_svg_path(d: &str, scale: f32) -> Option<PathBuilder> {
    let mut p = Tokens::new(d);
    let mut pb = PathBuilder::new();

    let mut cmd = b' ';
    // Current point, subpath start, and previous control point (for S/T
    // reflection), all in unscaled (viewBox) coordinates.
    let (mut cx, mut cy) = (0.0_f32, 0.0_f32);
    let (mut sx, mut sy) = (0.0_f32, 0.0_f32);
    let mut prev_cubic_ctrl: Option<(f32, f32)> = None;
    let mut prev_quad_ctrl: Option<(f32, f32)> = None;
    let mut open = false;

    macro_rules! ensure_open {
        () => {
            if !open {
                pb = pb.move_to(cx * scale, cy * scale);
                (sx, sy) = (cx, cy);
                open = true;
            }
        };
    }

    while let Some(next) = p.next_command_or_number() {
        match next {
            Token::Command(c) => {
                cmd = c;
                if cmd == b'Z' || cmd == b'z' {
                    if open {
                        pb = pb.close();
                        open = false;
                    }
                    (cx, cy) = (sx, sy);
                    prev_cubic_ctrl = None;
                    prev_quad_ctrl = None;
                    continue;
                }
            }
            Token::NumberStart => {
                // Implicit repetition: M becomes L (m becomes l) after the
                // first coordinate pair; everything else repeats as-is.
                cmd = match cmd {
                    b'M' => b'L',
                    b'm' => b'l',
                    c => c,
                }
            }
        }

        let rel = cmd.is_ascii_lowercase();
        match cmd.to_ascii_uppercase() {
            b'M' => {
                let (x, y) = (p.number()?, p.number()?);
                if open {
                    pb = pb.end_open();
                }
                (cx, cy) = if rel { (cx + x, cy + y) } else { (x, y) };
                pb = pb.move_to(cx * scale, cy * scale);
                (sx, sy) = (cx, cy);
                open = true;
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
            }
            b'L' => {
                let (x, y) = (p.number()?, p.number()?);
                ensure_open!();
                (cx, cy) = if rel { (cx + x, cy + y) } else { (x, y) };
                pb = pb.line_to(cx * scale, cy * scale);
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
            }
            b'H' => {
                let x = p.number()?;
                ensure_open!();
                cx = if rel { cx + x } else { x };
                pb = pb.line_to(cx * scale, cy * scale);
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
            }
            b'V' => {
                let y = p.number()?;
                ensure_open!();
                cy = if rel { cy + y } else { y };
                pb = pb.line_to(cx * scale, cy * scale);
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
            }
            b'C' | b'S' => {
                let (c1x, c1y) = if cmd.eq_ignore_ascii_case(&b'C') {
                    let (a, b) = (p.number()?, p.number()?);
                    if rel { (cx + a, cy + b) } else { (a, b) }
                } else {
                    // S: first control point reflects the previous one.
                    match prev_cubic_ctrl {
                        Some((px, py)) => (2.0 * cx - px, 2.0 * cy - py),
                        None => (cx, cy),
                    }
                };
                let (a, b, ex, ey) = (p.number()?, p.number()?, p.number()?, p.number()?);
                let (c2x, c2y) = if rel { (cx + a, cy + b) } else { (a, b) };
                let (nx, ny) = if rel { (cx + ex, cy + ey) } else { (ex, ey) };
                ensure_open!();
                pb = pb.cubic_bezier_to(
                    [c1x * scale, c1y * scale],
                    [c2x * scale, c2y * scale],
                    [nx * scale, ny * scale],
                );
                prev_cubic_ctrl = Some((c2x, c2y));
                prev_quad_ctrl = None;
                (cx, cy) = (nx, ny);
            }
            b'Q' | b'T' => {
                let (qx, qy) = if cmd.eq_ignore_ascii_case(&b'Q') {
                    let (a, b) = (p.number()?, p.number()?);
                    if rel { (cx + a, cy + b) } else { (a, b) }
                } else {
                    match prev_quad_ctrl {
                        Some((px, py)) => (2.0 * cx - px, 2.0 * cy - py),
                        None => (cx, cy),
                    }
                };
                let (ex, ey) = (p.number()?, p.number()?);
                let (nx, ny) = if rel { (cx + ex, cy + ey) } else { (ex, ey) };
                ensure_open!();
                pb = pb.quadratic_bezier_to([qx * scale, qy * scale], [nx * scale, ny * scale]);
                prev_quad_ctrl = Some((qx, qy));
                prev_cubic_ctrl = None;
                (cx, cy) = (nx, ny);
            }
            b'A' => {
                let (rx, ry) = (p.number()?, p.number()?);
                let rot = p.number()?;
                let large_arc = p.flag()?;
                let sweep = p.flag()?;
                let (ex, ey) = (p.number()?, p.number()?);
                let (nx, ny) = if rel { (cx + ex, cy + ey) } else { (ex, ey) };
                ensure_open!();
                pb = arc_to_cubics(pb, cx, cy, rx, ry, rot, large_arc, sweep, nx, ny, scale);
                prev_cubic_ctrl = None;
                prev_quad_ctrl = None;
                (cx, cy) = (nx, ny);
            }
            _ => return None, // unknown command — refuse the whole path
        }
    }

    if open {
        pb = pb.end_open();
    }
    Some(pb)
}

enum Token {
    Command(u8),
    /// Next token is a number — caller re-reads it via `number()`.
    NumberStart,
}

struct Tokens<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Tokens<'a> {
    fn new(d: &'a str) -> Self {
        Self {
            bytes: d.as_bytes(),
            pos: 0,
        }
    }

    fn skip_separators(&mut self) {
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_whitespace() || self.bytes[self.pos] == b',')
        {
            self.pos += 1;
        }
    }

    fn next_command_or_number(&mut self) -> Option<Token> {
        self.skip_separators();
        let b = *self.bytes.get(self.pos)?;
        if b.is_ascii_alphabetic() && b != b'e' && b != b'E' {
            self.pos += 1;
            Some(Token::Command(b))
        } else {
            Some(Token::NumberStart)
        }
    }

    /// Parse `[+-]?digits[.digits][(e|E)[+-]digits]`.
    fn number(&mut self) -> Option<f32> {
        self.skip_separators();
        let start = self.pos;
        if matches!(self.bytes.get(self.pos), Some(b'+') | Some(b'-')) {
            self.pos += 1;
        }
        let mut seen_digit = false;
        while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
            self.pos += 1;
            seen_digit = true;
        }
        if self.bytes.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                self.pos += 1;
                seen_digit = true;
            }
        }
        if !seen_digit {
            self.pos = start;
            return None;
        }
        if matches!(self.bytes.get(self.pos), Some(b'e') | Some(b'E')) {
            let exp_start = self.pos;
            self.pos += 1;
            if matches!(self.bytes.get(self.pos), Some(b'+') | Some(b'-')) {
                self.pos += 1;
            }
            if self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                    self.pos += 1;
                }
            } else {
                self.pos = exp_start;
            }
        }
        std::str::from_utf8(&self.bytes[start..self.pos])
            .ok()?
            .parse()
            .ok()
    }

    /// Arc flags are single `0`/`1` chars that may not be separated from the
    /// following number (`"a1 1 0 01-.7"`), so they cannot use `number()`.
    fn flag(&mut self) -> Option<bool> {
        self.skip_separators();
        match self.bytes.get(self.pos)? {
            b'0' => {
                self.pos += 1;
                Some(false)
            }
            b'1' => {
                self.pos += 1;
                Some(true)
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Elliptical arc -> cubic beziers (SVG 1.1 spec, appendix B.2.4)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn arc_to_cubics(
    mut pb: PathBuilder,
    x1: f32,
    y1: f32,
    rx: f32,
    ry: f32,
    rotation_deg: f32,
    large_arc: bool,
    sweep: bool,
    x2: f32,
    y2: f32,
    scale: f32,
) -> PathBuilder {
    if rx == 0.0 || ry == 0.0 || (x1 == x2 && y1 == y2) {
        return pb.line_to(x2 * scale, y2 * scale);
    }
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    let plev = rotation_deg.to_radians();
    let (sin_plev, cos_plev) = plev.sin_cos();

    // Step 1: midpoint coordinates.
    let dx2 = (x1 - x2) / 2.0;
    let dy2 = (y1 - y2) / 2.0;
    let x1p = cos_plev * dx2 + sin_plev * dy2;
    let y1p = -sin_plev * dx2 + cos_plev * dy2;

    // Correct out-of-range radii.
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda > 1.0 {
        let l = lambda.sqrt();
        rx *= l;
        ry *= l;
    }

    // Step 2: center in primed coordinates.
    let rx_sq = rx * rx;
    let ry_sq = ry * ry;
    let num = rx_sq * ry_sq - rx_sq * y1p * y1p - ry_sq * x1p * x1p;
    let den = rx_sq * y1p * y1p + ry_sq * x1p * x1p;
    let mut coef = if den == 0.0 {
        0.0
    } else {
        (num / den).max(0.0).sqrt()
    };
    if large_arc == sweep {
        coef = -coef;
    }
    let cxp = coef * rx * y1p / ry;
    let cyp = -coef * ry * x1p / rx;

    // Step 3: center in original coordinates.
    let cx = cos_plev * cxp - sin_plev * cyp + (x1 + x2) / 2.0;
    let cy = sin_plev * cxp + cos_plev * cyp + (y1 + y2) / 2.0;

    // Step 4: start angle and sweep extent.
    let angle = |ux: f32, uy: f32, vx: f32, vy: f32| -> f32 {
        let dot = ux * vx + uy * vy;
        let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
        let mut a = (dot / len).clamp(-1.0, 1.0).acos();
        if ux * vy - uy * vx < 0.0 {
            a = -a;
        }
        a
    };
    let theta1 = angle(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut dtheta = angle(
        (x1p - cxp) / rx,
        (y1p - cyp) / ry,
        (-x1p - cxp) / rx,
        (-y1p - cyp) / ry,
    );
    if !sweep && dtheta > 0.0 {
        dtheta -= 2.0 * std::f32::consts::PI;
    } else if sweep && dtheta < 0.0 {
        dtheta += 2.0 * std::f32::consts::PI;
    }

    // Split into <= 90 degree segments, each approximated by one cubic.
    let segments = (dtheta.abs() / std::f32::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let delta = dtheta / segments as f32;
    let alpha = 4.0 / 3.0 * (delta / 4.0).tan();

    let point = |t: f32| -> (f32, f32) {
        let (sin_t, cos_t) = t.sin_cos();
        (
            cx + rx * cos_t * cos_plev - ry * sin_t * sin_plev,
            cy + rx * cos_t * sin_plev + ry * sin_t * cos_plev,
        )
    };
    let derivative = |t: f32| -> (f32, f32) {
        let (sin_t, cos_t) = t.sin_cos();
        (
            -rx * sin_t * cos_plev - ry * cos_t * sin_plev,
            -rx * sin_t * sin_plev + ry * cos_t * cos_plev,
        )
    };

    let mut t = theta1;
    for _ in 0..segments {
        let t_next = t + delta;
        let (px, py) = point(t);
        let (nx, ny) = point(t_next);
        let (dx1, dy1) = derivative(t);
        let (dx2_, dy2_) = derivative(t_next);
        pb = pb.cubic_bezier_to(
            [(px + alpha * dx1) * scale, (py + alpha * dy1) * scale],
            [(nx - alpha * dx2_) * scale, (ny - alpha * dy2_) * scale],
            [nx * scale, ny * scale],
        );
        t = t_next;
    }
    pb
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_tessellates_with_vertices() {
        for name in icon_names() {
            let path = icon_path(name, 16.0, WHITE, 0.0, 0.0).unwrap();
            assert!(
                !path.vertices.is_empty(),
                "icon '{name}' produced no vertices"
            );
            assert!(
                !path.indices.is_empty(),
                "icon '{name}' produced no indices"
            );
            assert_eq!(path.indices.len() % 3, 0, "icon '{name}' not triangles");
        }
    }

    /// `icon_names` documents its result as sorted, and it returns the table
    /// verbatim - so the table is what has to stay sorted. Nothing enforced
    /// that before, and an icon appended at the end would have made the doc
    /// quietly false for every caller that renders the set as a list.
    #[test]
    fn the_icon_table_is_sorted_by_name() {
        let names = icon_names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted, "ICONS must stay in alphabetical order");
    }

    #[test]
    fn unknown_icon_returns_none() {
        assert!(icon("definitely-not-an-icon", 16.0, WHITE).is_none());
        assert!(!has_icon("definitely-not-an-icon"));
        assert!(has_icon("folder"));
    }

    #[test]
    fn icon_geometry_stays_within_viewbox_bounds() {
        // Stroke extends half a width past the 24x24 box; allow that margin.
        let size = 24.0;
        let margin = STROKE_WIDTH;
        for name in icon_names() {
            let path = icon_path(name, size, WHITE, 0.0, 0.0).unwrap();
            for v in &path.vertices {
                assert!(
                    v.position[0] >= -margin && v.position[0] <= size + margin,
                    "icon '{name}' x={} outside bounds",
                    v.position[0]
                );
                assert!(
                    v.position[1] >= -margin && v.position[1] <= size + margin,
                    "icon '{name}' y={} outside bounds",
                    v.position[1]
                );
            }
        }
    }

    #[test]
    fn icon_at_translates_geometry() {
        let at_origin = icon_path("x", 16.0, WHITE, 0.0, 0.0).unwrap();
        let moved = icon_path("x", 16.0, WHITE, 100.0, 50.0).unwrap();
        assert_eq!(at_origin.vertices.len(), moved.vertices.len());
        for (a, b) in at_origin.vertices.iter().zip(moved.vertices.iter()) {
            assert!((b.position[0] - a.position[0] - 100.0).abs() < 1e-4);
            assert!((b.position[1] - a.position[1] - 50.0).abs() < 1e-4);
        }
    }

    #[test]
    fn icon_color_is_applied() {
        let tint = [0.2, 0.4, 0.6, 0.8];
        let path = icon_path("check", 16.0, tint, 0.0, 0.0).unwrap();
        assert!(path.vertices.iter().all(|v| v.color == tint));
    }

    #[test]
    fn hash_distinguishes_position_and_color() {
        let a = icon_path("folder", 16.0, WHITE, 0.0, 0.0).unwrap();
        let b = icon_path("folder", 16.0, WHITE, 1.0, 0.0).unwrap();
        let c = icon_path("folder", 16.0, [1.0, 0.0, 0.0, 1.0], 0.0, 0.0).unwrap();
        let d = icon_path("folder", 16.0, WHITE, 0.0, 0.0).unwrap();
        assert_ne!(a.hash, b.hash);
        assert_ne!(a.hash, c.hash);
        assert_eq!(a.hash, d.hash, "same request must be hash-stable");
    }

    #[test]
    fn cache_returns_identical_geometry() {
        let first = icon_path("settings", 20.0, WHITE, 0.0, 0.0).unwrap();
        let second = icon_path("settings", 20.0, WHITE, 0.0, 0.0).unwrap();
        assert_eq!(first.vertices.len(), second.vertices.len());
        assert_eq!(first.indices, second.indices);
    }

    #[test]
    fn sizes_scale_geometry() {
        let small = icon_path("circle", 12.0, WHITE, 0.0, 0.0).unwrap();
        let max_small = small
            .vertices
            .iter()
            .map(|v| v.position[0].max(v.position[1]))
            .fold(f32::MIN, f32::max);
        let large = icon_path("circle", 48.0, WHITE, 0.0, 0.0).unwrap();
        let max_large = large
            .vertices
            .iter()
            .map(|v| v.position[0].max(v.position[1]))
            .fold(f32::MIN, f32::max);
        assert!(max_large > max_small * 3.0);
    }

    #[test]
    fn arc_parser_handles_compact_flags() {
        // Flags glued to the next number, as some minified SVGs emit.
        let pb = parse_svg_path("M0 0a1 1 0 01-.7.7", 1.0).unwrap();
        let path = pb.stroke_round(WHITE, 2.0);
        assert!(!path.vertices.is_empty());
    }

    #[test]
    fn parser_rejects_garbage() {
        assert!(parse_svg_path("M0 0 ? 3 4", 1.0).is_none());
    }
}
