//! a uniform spatial hash over the field, so the neighbor scan is local
//! instead of O(n^2). particles are binned by cell; `for_each_neighbor` walks
//! the 3x3 cells around a point. bins keep insertion order, so the traversal
//! is deterministic (a prerequisite for reproducible runs).

use crate::sim::particle::Particle;

/// a grid of cells, each holding the indices of the particles inside it.
pub struct Grid {
    cols: usize,
    rows: usize,
    cell: f32,
    bins: Vec<Vec<u32>>,
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cols: 1,
            rows: 1,
            cell: 1.0,
            bins: vec![Vec::new()],
        }
    }

    /// re-bin every particle for a field of `w` x `h` world pixels with the
    /// given `cell` size. reuses the bin allocations across frames.
    pub fn rebuild(&mut self, particles: &[Particle], w: f32, h: f32, cell: f32) {
        self.cell = cell.max(1.0);
        self.cols = (w / self.cell).ceil().max(1.0) as usize + 1;
        self.rows = (h / self.cell).ceil().max(1.0) as usize + 1;
        let needed = self.cols * self.rows;
        if self.bins.len() < needed {
            self.bins.resize_with(needed, Vec::new);
        }
        for bin in self.bins.iter_mut() {
            bin.clear();
        }
        for (i, p) in particles.iter().enumerate() {
            let (c, r) = self.cell_of(p.x, p.y);
            self.bins[r * self.cols + c].push(i as u32);
        }
    }

    fn cell_of(&self, x: f32, y: f32) -> (usize, usize) {
        let c = (x / self.cell) as isize;
        let r = (y / self.cell) as isize;
        (
            c.clamp(0, self.cols as isize - 1) as usize,
            r.clamp(0, self.rows as isize - 1) as usize,
        )
    }

    /// call `f` with the index of every particle in the 3x3 cell block around
    /// `(x, y)`, in deterministic order.
    pub fn for_each_neighbor(&self, x: f32, y: f32, mut f: impl FnMut(u32)) {
        let (c, r) = self.cell_of(x, y);
        let (c, r) = (c as isize, r as isize);
        for dr in -1..=1 {
            for dc in -1..=1 {
                let (nc, nr) = (c + dc, r + dr);
                if nc < 0 || nr < 0 || nc >= self.cols as isize || nr >= self.rows as isize {
                    continue;
                }
                for &idx in &self.bins[nr as usize * self.cols + nc as usize] {
                    f(idx);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, y: f32) -> Particle {
        Particle::new(x, y, 0, 2, 1.5, 0.0, 0.0, [1.0; 4])
    }

    #[test]
    fn neighbors_find_close_and_skip_far() {
        let ps = vec![at(50.0, 50.0), at(60.0, 55.0), at(500.0, 500.0)];
        let mut g = Grid::new();
        g.rebuild(&ps, 800.0, 800.0, 72.0);
        let mut seen = Vec::new();
        g.for_each_neighbor(50.0, 50.0, |i| seen.push(i));
        assert!(seen.contains(&0) && seen.contains(&1), "close pair missing");
        assert!(!seen.contains(&2), "far particle should not be a neighbor");
    }

    #[test]
    fn traversal_is_deterministic() {
        let ps = vec![at(10.0, 10.0), at(20.0, 20.0), at(30.0, 15.0)];
        let mut g = Grid::new();
        g.rebuild(&ps, 200.0, 200.0, 64.0);
        let collect = |g: &Grid| {
            let mut v = Vec::new();
            g.for_each_neighbor(20.0, 20.0, |i| v.push(i));
            v
        };
        assert_eq!(collect(&g), collect(&g));
    }
}
