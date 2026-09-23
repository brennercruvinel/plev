//! The shapes: rounded rects as the SDF shader draws them, gradients as
//! `<linearGradient>`, tessellated paths as the outline of their
//! triangles, and shadows as blurred silhouettes. What the frame computes
//! analytically or in linear space is approximated here and reported.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::compositor::{QuadVertex, gradient_direction, shadow_padding, shadow_sigma};
use crate::path::TessellatedPath;

use super::write::{Svg, fill, hex, num, opacity};

/// A rounded rect's box and corner radius.
#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
    pub(super) radius: f32,
}

impl Bounds {
    fn rect(&self, paint: &str) -> String {
        format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\"{paint}/>",
            num(self.x),
            num(self.y),
            num(self.w),
            num(self.h),
            num(self.radius.max(0.0))
        )
    }

    /// The box shrunk by `d` on every side, its radius following.
    fn inset(&self, d: f32) -> Bounds {
        Bounds {
            x: self.x + d,
            y: self.y + d,
            w: self.w - 2.0 * d,
            h: self.h - 2.0 * d,
            radius: self.radius - d,
        }
    }

    /// The box as path data (arcs at the corners), for shapes a `<rect>`
    /// cannot express, like a rounded hole.
    fn path_d(&self) -> String {
        let r = self.radius.clamp(0.0, (self.w.min(self.h) / 2.0).max(0.0));
        let (x0, y0, x1, y1) = (self.x, self.y, self.x + self.w, self.y + self.h);
        if r <= 0.0 {
            return format!(
                "M{} {}H{}V{}H{}Z",
                num(x0),
                num(y0),
                num(x1),
                num(y1),
                num(x0)
            );
        }
        let arc = |x: f32, y: f32| format!("A{} {} 0 0 1 {} {}", num(r), num(r), num(x), num(y));
        format!(
            "M{} {}H{}{}V{}{}H{}{}V{}{}Z",
            num(x0 + r),
            num(y0),
            num(x1 - r),
            arc(x1, y0 + r),
            num(y1 - r),
            arc(x1 - r, y1),
            num(x0 + r),
            arc(x0, y1 - r),
            num(y0 + r),
            arc(x0 + r, y0)
        )
    }
}

/// A rounded rect as the SDF shader draws it: with a border, the ring of
/// `border_width` inside the outer edge holds the border color and only
/// the inset box holds the fill; without one, the fill reaches the edge.
/// `fill_attrs` is the fill (a color, or a gradient reference).
pub(super) fn rounded_rect(
    svg: &mut Svg,
    b: Bounds,
    fill_attrs: &str,
    border_width: f32,
    border: [f32; 4],
) {
    if border_width <= 0.0 {
        svg.element(&b.rect(fill_attrs));
        return;
    }
    let bw = border_width;
    if b.w > 2.0 * bw && b.h > 2.0 * bw {
        svg.element(&b.inset(bw).rect(fill_attrs));
    }
    if border[3] > 0.0 && b.w > bw && b.h > bw {
        // A stroke centered half a border inside the edge covers exactly
        // the ring between the outer edge and the inset box.
        let stroke = format!(
            " fill=\"none\" stroke=\"{}\"{} stroke-width=\"{}\"",
            hex(border),
            opacity("stroke-opacity", border[3]),
            num(bw)
        );
        svg.element(&b.inset(bw / 2.0).rect(&stroke));
    }
}

/// A rounded rect with a two-stop linear gradient. The shader mixes the
/// stops in premultiplied linear space along the projection of the point
/// onto the direction, from one extent of the box to the other; the
/// gradient here spans the same two points and asks for linearRGB
/// interpolation, which a viewer may not honor. Reported.
#[allow(clippy::too_many_arguments)] // the node's fields
pub(super) fn gradient_rect(
    svg: &mut Svg,
    b: Bounds,
    color: [f32; 4],
    color2: [f32; 4],
    angle_deg: f32,
    border_width: f32,
    border: [f32; 4],
    at: &str,
) {
    let [dx, dy] = gradient_direction(angle_deg);
    let (hw, hh) = (b.w / 2.0, b.h / 2.0);
    let extent = (dx.abs() * hw + dy.abs() * hh).max(1e-4);
    let (cx, cy) = (b.x + hw, b.y + hh);
    let id = svg.linear_gradient(
        [cx - dx * extent, cy - dy * extent],
        [cx + dx * extent, cy + dy * extent],
        color,
        color2,
    );
    rounded_rect(
        svg,
        b,
        &format!(" fill=\"url(#{id})\""),
        border_width,
        border,
    );
    svg.note(
        at,
        "gradient stops mixed in linearRGB by the frame; the svg asks for \
         color-interpolation=\"linearRGB\", which not every viewer honors",
    );
}

/// The analytic shadow of a rounded rect, as a Gaussian blur
/// (`feGaussianBlur`, the same sigma) of the silhouette it casts: for a
/// drop shadow the offset box, for an inset shadow the region outside the
/// offset box, clipped to the casting box. The shader integrates the
/// Gaussian analytically; a viewer's blur is a close reading of the same
/// function. Reported. `None` when the frame culls it (off the viewport).
#[allow(clippy::too_many_arguments)] // the node's fields
pub(super) fn shadow(
    svg: &mut Svg,
    b: Bounds,
    blur_radius: f32,
    offset: [f32; 2],
    color: [f32; 4],
    inset: bool,
    viewport: super::Viewport,
    at: &str,
) {
    let sigma = shadow_sigma(blur_radius);
    let pad = shadow_padding(blur_radius);
    let quad = if inset {
        [b.x, b.y, b.w, b.h]
    } else {
        [
            b.x - pad + offset[0],
            b.y - pad + offset[1],
            b.w + 2.0 * pad,
            b.h + 2.0 * pad,
        ]
    };
    if super::outside(viewport, quad[0], quad[1], quad[2], quad[3]) {
        return;
    }
    let region = [b.x - pad, b.y - pad, b.w + 2.0 * pad, b.h + 2.0 * pad];
    let cast = Bounds {
        x: b.x + offset[0],
        y: b.y + offset[1],
        ..b
    };
    if inset {
        let clip = svg.clip_path_d(&b.path_d());
        let filter = svg.blur_filter(region, sigma);
        svg.open(&format!("<g clip-path=\"url(#{clip})\">"));
        // Everything outside the offset box, blurred inward and kept
        // inside the casting box: the inverted coverage the shader uses.
        let outer = Bounds {
            x: region[0],
            y: region[1],
            w: region[2],
            h: region[3],
            radius: 0.0,
        };
        svg.element(&format!(
            "<path fill-rule=\"evenodd\"{} filter=\"url(#{filter})\" d=\"{}{}\"/>",
            fill(color),
            outer.path_d(),
            cast.path_d()
        ));
        svg.close("</g>");
    } else {
        let region = [
            region[0] + offset[0],
            region[1] + offset[1],
            region[2],
            region[3],
        ];
        let filter = svg.blur_filter(region, sigma);
        svg.element(&cast.rect(&format!("{} filter=\"url(#{filter})\"", fill(color))));
    }
    svg.note(
        at,
        &format!(
            "{} shadow as a feGaussianBlur of its silhouette (sigma {}); the frame \
             integrates the Gaussian analytically",
            if inset { "inset" } else { "drop" },
            num(sigma)
        ),
    );
}

/// A tessellated path as one `<path>` per fill color: the polygon that
/// bounds its triangles (the edges no two triangles share, chained into
/// loops), so a viewer draws the shape without seams between triangles.
/// A mesh that is not a clean partition (an edge walked twice the same
/// way, a chain that does not close) falls back to one subpath per
/// triangle, reported.
pub(super) fn path(svg: &mut Svg, data: &TessellatedPath, at: &str) {
    let mut groups: Vec<([f32; 4], Vec<[u32; 3]>)> = Vec::new();
    for tri in data.indices.chunks_exact(3) {
        let Some(first) = data.vertices.get(tri[0] as usize) else {
            continue;
        };
        let color = first.color;
        let same = |c: &[f32; 4]| {
            c.iter()
                .zip(&color)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        };
        let tri = [tri[0], tri[1], tri[2]];
        match groups.iter_mut().find(|(c, _)| same(c)) {
            Some((_, tris)) => tris.push(tri),
            None => groups.push((color, vec![tri])),
        }
    }
    for (color, tris) in groups {
        let d = match boundary_loops(&data.vertices, &tris) {
            Some(loops) => loops
                .iter()
                .map(|loop_| polygon_d(loop_))
                .collect::<String>(),
            None => {
                svg.note(
                    at,
                    "path as its tessellated triangles, one subpath each: the mesh is not \
                     a clean partition, so a viewer may show seams between them",
                );
                tris.iter()
                    .map(|tri| {
                        let points: Vec<[f32; 2]> = tri
                            .iter()
                            .filter_map(|&i| data.vertices.get(i as usize))
                            .map(|v| v.position)
                            .collect();
                        polygon_d(&points)
                    })
                    .collect::<String>()
            }
        };
        if d.is_empty() {
            continue;
        }
        svg.element(&format!(
            "<path fill-rule=\"nonzero\"{} d=\"{d}\"/>",
            fill(color)
        ));
    }
}

fn polygon_d(points: &[[f32; 2]]) -> String {
    let mut d = String::new();
    for (i, [x, y]) in points.iter().enumerate() {
        let _ = write!(
            d,
            "{}{} {}",
            if i == 0 { 'M' } else { 'L' },
            num(*x),
            num(*y)
        );
    }
    if !points.is_empty() {
        d.push('Z');
    }
    d
}

/// Positions quantized so vertices that coincide merge into one corner.
type Corner = (i32, i32);

const QUANTUM: f32 = 256.0;

fn corner(p: [f32; 2]) -> Corner {
    (
        (p[0] * QUANTUM).round() as i32,
        (p[1] * QUANTUM).round() as i32,
    )
}

fn point(c: Corner) -> [f32; 2] {
    [c.0 as f32 / QUANTUM, c.1 as f32 / QUANTUM]
}

/// The loops bounding a consistently wound triangle mesh: every directed
/// edge walked once whose reverse no triangle walks, chained corner to
/// corner. `None` when an edge is walked twice the same way (overlap or
/// mixed winding) or a chain does not close. Deterministic: the corners
/// are visited in coordinate order.
fn boundary_loops(vertices: &[QuadVertex], tris: &[[u32; 3]]) -> Option<Vec<Vec<[f32; 2]>>> {
    let mut walked: BTreeMap<(Corner, Corner), u32> = BTreeMap::new();
    for tri in tris {
        let c: Vec<Corner> = tri
            .iter()
            .map(|&i| vertices.get(i as usize).map(|v| corner(v.position)))
            .collect::<Option<_>>()?;
        for k in 0..3 {
            let (a, b) = (c[k], c[(k + 1) % 3]);
            if a != b {
                *walked.entry((a, b)).or_insert(0) += 1;
            }
        }
    }
    let mut out: BTreeMap<Corner, Vec<Corner>> = BTreeMap::new();
    let mut edges = 0usize;
    for (&(a, b), &n) in &walked {
        if n > 1 {
            return None;
        }
        if walked.contains_key(&(b, a)) {
            continue;
        }
        out.entry(a).or_default().push(b);
        edges += 1;
    }
    let mut loops = Vec::new();
    while let Some((&start, _)) = out.iter().find(|(_, ends)| !ends.is_empty()) {
        let mut loop_ = vec![point(start)];
        let mut at = start;
        loop {
            let next = out.get_mut(&at)?.pop()?;
            if next == start {
                break;
            }
            loop_.push(point(next));
            at = next;
            if loop_.len() > edges {
                return None;
            }
        }
        loops.push(loop_);
    }
    Some(loops)
}
