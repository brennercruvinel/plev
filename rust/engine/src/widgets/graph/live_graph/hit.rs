//! What lies under the pointer: every drawn primitive of a live graph (group
//! boxes, edge segments, node circles) in draw order, bucketed by a uniform
//! grid over the world, so a point asks only its cell and the topmost
//! primitive answers. A node beats the edge under it, a child beats its
//! group's box, an inner group beats the outer one.

/// A hit owner: what an item belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Node(usize),
    Edge(usize),
    Group(usize),
}

#[derive(Clone, Copy, Debug)]
enum Prim {
    Circle {
        c: (f32, f32),
        r: f32,
    },
    Segment {
        a: (f32, f32),
        b: (f32, f32),
        half: f32,
    },
    Box {
        r: [f32; 4],
    },
}

impl Prim {
    fn bounds(&self) -> [f32; 4] {
        match *self {
            Prim::Circle { c, r } => [c.0 - r, c.1 - r, c.0 + r, c.1 + r],
            Prim::Segment { a, b, half } => [
                a.0.min(b.0) - half,
                a.1.min(b.1) - half,
                a.0.max(b.0) + half,
                a.1.max(b.1) + half,
            ],
            Prim::Box { r } => [r[0], r[1], r[0] + r[2], r[1] + r[3]],
        }
    }

    fn contains(&self, p: (f32, f32), tol: f32) -> bool {
        match *self {
            Prim::Circle { c, r } => (p.0 - c.0).hypot(p.1 - c.1) <= r + tol,
            Prim::Segment { a, b, half } => seg_dist(p, a, b) <= half + tol,
            Prim::Box { r } => {
                p.0 >= r[0] - tol
                    && p.0 <= r[0] + r[2] + tol
                    && p.1 >= r[1] - tol
                    && p.1 <= r[1] + r[3] + tol
            }
        }
    }
}

fn seg_dist(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    };
    (a.0 + t * dx - p.0).hypot(a.1 + t * dy - p.1)
}

/// The primitives of one frame, in draw order, and the grid over them.
#[derive(Default)]
pub struct HitGrid {
    prims: Vec<(Prim, Hit)>,
    origin: (f32, f32),
    cell: f32,
    cols: usize,
    rows: usize,
    cells: Vec<Vec<u32>>,
}

impl HitGrid {
    /// Starts a frame's primitives; push them in draw order, then `build`.
    pub fn clear(&mut self) {
        self.prims.clear();
    }

    pub fn push_box(&mut self, r: [f32; 4], hit: Hit) {
        self.prims.push((Prim::Box { r }, hit));
    }

    pub fn push_segment(&mut self, a: (f32, f32), b: (f32, f32), width: f32, hit: Hit) {
        self.prims.push((
            Prim::Segment {
                a,
                b,
                half: width / 2.0,
            },
            hit,
        ));
    }

    pub fn push_circle(&mut self, c: (f32, f32), r: f32, hit: Hit) {
        self.prims.push((Prim::Circle { c, r }, hit));
    }

    /// Buckets the primitives: about one per cell, never finer than 16 world px.
    pub fn build(&mut self) {
        let mut b = [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        for (p, _) in &self.prims {
            let r = p.bounds();
            b = [
                b[0].min(r[0]),
                b[1].min(r[1]),
                b[2].max(r[2]),
                b[3].max(r[3]),
            ];
        }
        self.cells.clear();
        if !b[0].is_finite() {
            self.cols = 0;
            self.rows = 0;
            return;
        }
        let area = (b[2] - b[0]).max(1.0) * (b[3] - b[1]).max(1.0);
        self.cell = (area / self.prims.len().max(1) as f32).sqrt().max(16.0);
        self.origin = (b[0], b[1]);
        self.cols = ((b[2] - b[0]) / self.cell).ceil().max(1.0) as usize;
        self.rows = ((b[3] - b[1]) / self.cell).ceil().max(1.0) as usize;
        self.cells = vec![Vec::new(); self.cols * self.rows];
        for (i, (p, _)) in self.prims.iter().enumerate() {
            let r = p.bounds();
            let (c0, r0) = self.cell_of((r[0], r[1]));
            let (c1, r1) = self.cell_of((r[2], r[3]));
            for row in r0..=r1 {
                for col in c0..=c1 {
                    self.cells[row * self.cols + col].push(i as u32);
                }
            }
        }
    }

    fn cell_of(&self, p: (f32, f32)) -> (usize, usize) {
        let c = ((p.0 - self.origin.0) / self.cell).floor();
        let r = ((p.1 - self.origin.1) / self.cell).floor();
        (
            c.clamp(0.0, (self.cols - 1) as f32) as usize,
            r.clamp(0.0, (self.rows - 1) as f32) as usize,
        )
    }

    /// The topmost owner under `p` (world px); `tol` (world px) widens thin strokes.
    pub fn hit(&self, p: (f32, f32), tol: f32) -> Option<Hit> {
        if self.cells.is_empty() {
            return None;
        }
        let (c, r) = self.cell_of(p);
        self.cells[r * self.cols + c]
            .iter()
            .rev()
            .map(|&i| self.prims[i as usize])
            .find(|(prim, _)| prim.contains(p, tol))
            .map(|(_, hit)| hit)
    }
}
