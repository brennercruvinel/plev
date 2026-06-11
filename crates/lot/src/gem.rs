//! Geometry: 2x3 affine matrices, transform composition, and conversion
//! of lottie shape primitives into transformed cubic subpaths ready for
//! tessellation through `plev::path::PathBuilder`.

use crate::kfr::PathData;
use plev::path::{PathBuilder, TessellatedPath};

/// Row-major 2x3 affine matrix: x' = a*x + c*y + e; y' = b*x + d*y + f.
#[derive(Clone, Copy)]
pub struct Mat {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Mat {
    pub const IDENTITY: Mat = Mat {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    /// self * rhs (rhs applied first).
    pub fn mul(&self, r: &Mat) -> Mat {
        Mat {
            a: self.a * r.a + self.c * r.b,
            b: self.b * r.a + self.d * r.b,
            c: self.a * r.c + self.c * r.d,
            d: self.b * r.c + self.d * r.d,
            e: self.a * r.e + self.c * r.f + self.e,
            f: self.b * r.e + self.d * r.f + self.f,
        }
    }

    pub fn apply(&self, p: [f64; 2]) -> [f64; 2] {
        [
            self.a * p[0] + self.c * p[1] + self.e,
            self.b * p[0] + self.d * p[1] + self.f,
        ]
    }

    /// Uniform scale estimate (sqrt of |determinant|); used to scale
    /// stroke widths since vertices are pre-transformed on the CPU.
    pub fn scale_factor(&self) -> f64 {
        (self.a * self.d - self.b * self.c).abs().sqrt()
    }
}

/// Lottie transform order: translate(p) * rotate(r) * scale(s) * translate(-a).
pub fn trs(anchor: [f64; 2], pos: [f64; 2], scale_pct: [f64; 2], rot_deg: f64) -> Mat {
    let (sx, sy) = (scale_pct[0] / 100.0, scale_pct[1] / 100.0);
    let rad = rot_deg.to_radians();
    let (sin, cos) = rad.sin_cos();
    // rotate * scale
    let (a, b, c, d) = (cos * sx, sin * sx, -sin * sy, cos * sy);
    Mat {
        a,
        b,
        c,
        d,
        e: pos[0] - a * anchor[0] - c * anchor[1],
        f: pos[1] - b * anchor[0] - d * anchor[1],
    }
}

/// One subpath whose points are already in final (window) coordinates.
pub struct Sub {
    pub start: [f64; 2],
    /// (ctrl1, ctrl2, to) cubic segments.
    pub curves: Vec<([f64; 2], [f64; 2], [f64; 2])>,
    pub closed: bool,
}

/// Bezier path data (vertices + relative tangents) to a transformed subpath.
pub fn path_to_sub(p: &PathData, m: &Mat) -> Option<Sub> {
    let n = p.v.len();
    if n < 2 {
        return None;
    }
    let tan = |t: &Vec<[f64; 2]>, j: usize| t.get(j).copied().unwrap_or([0.0, 0.0]);
    let mut curves = Vec::with_capacity(n);
    let seg_count = if p.c { n } else { n - 1 };
    for j in 0..seg_count {
        let k = (j + 1) % n;
        let o = tan(&p.o, j);
        let i = tan(&p.i, k);
        let c1 = m.apply([p.v[j][0] + o[0], p.v[j][1] + o[1]]);
        let c2 = m.apply([p.v[k][0] + i[0], p.v[k][1] + i[1]]);
        curves.push((c1, c2, m.apply(p.v[k])));
    }
    Some(Sub {
        start: m.apply(p.v[0]),
        curves,
        closed: p.c,
    })
}

const KAPPA: f64 = 0.552_284_749_830_793_4;

/// Ellipse centered at `p` with size `s` (width, height), as 4 cubics.
pub fn ellipse_sub(p: [f64; 2], s: [f64; 2], m: &Mat) -> Sub {
    let (cx, cy, rx, ry) = (p[0], p[1], s[0] / 2.0, s[1] / 2.0);
    let (kx, ky) = (rx * KAPPA, ry * KAPPA);
    let pts = [
        // start at top, clockwise
        ([cx + kx, cy - ry], [cx + rx, cy - ky], [cx + rx, cy]),
        ([cx + rx, cy + ky], [cx + kx, cy + ry], [cx, cy + ry]),
        ([cx - kx, cy + ry], [cx - rx, cy + ky], [cx - rx, cy]),
        ([cx - rx, cy - ky], [cx - kx, cy - ry], [cx, cy - ry]),
    ];
    Sub {
        start: m.apply([cx, cy - ry]),
        curves: pts
            .iter()
            .map(|(c1, c2, to)| (m.apply(*c1), m.apply(*c2), m.apply(*to)))
            .collect(),
        closed: true,
    }
}

/// Rectangle centered at `p` with size `s`; corner roundness is ignored
/// (none of the targeted files use it), edges become degenerate cubics.
pub fn rect_sub(p: [f64; 2], s: [f64; 2], m: &Mat) -> Sub {
    let (hw, hh) = (s[0] / 2.0, s[1] / 2.0);
    let corners = [
        [p[0] + hw, p[1] - hh],
        [p[0] + hw, p[1] + hh],
        [p[0] - hw, p[1] + hh],
        [p[0] - hw, p[1] - hh],
    ];
    let start = m.apply(corners[3]);
    let mut prev = start;
    let mut curves = Vec::with_capacity(4);
    for c in corners {
        let to = m.apply(c);
        let c1 = [
            prev[0] + (to[0] - prev[0]) / 3.0,
            prev[1] + (to[1] - prev[1]) / 3.0,
        ];
        let c2 = [
            prev[0] + 2.0 * (to[0] - prev[0]) / 3.0,
            prev[1] + 2.0 * (to[1] - prev[1]) / 3.0,
        ];
        curves.push((c1, c2, to));
        prev = to;
    }
    Sub {
        start,
        curves,
        closed: true,
    }
}

fn build(subs: &[Sub]) -> PathBuilder {
    let mut b = PathBuilder::new();
    for s in subs {
        b = b.move_to(s.start[0] as f32, s.start[1] as f32);
        for (c1, c2, to) in &s.curves {
            b = b.cubic_bezier_to(
                [c1[0] as f32, c1[1] as f32],
                [c2[0] as f32, c2[1] as f32],
                [to[0] as f32, to[1] as f32],
            );
        }
        b = if s.closed { b.close() } else { b.end_open() };
    }
    b
}

pub fn tess_fill(subs: &[Sub], color: [f32; 4]) -> TessellatedPath {
    build(subs).fill(color)
}

pub fn tess_stroke(subs: &[Sub], color: [f32; 4], width: f32) -> TessellatedPath {
    build(subs).stroke_round(color, width)
}
