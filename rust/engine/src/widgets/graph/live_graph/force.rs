//! A live force simulation, ported from d3-force so a graph moves the way it
//! moves in a d3 page: link springs (strength 1/min degree, bias by degree),
//! many-body charge (exact, n² over the small graphs it serves), a centering
//! shift and circle collision, integrated with velocity decay while alpha
//! cools from 1 toward 0. A pinned body (`fx`, `fy`) is where a drag holds
//! it. Unlike [`compute_layout`](crate::widgets::graph::graph_view::geometry::compute_layout), which runs
//! once, this one ticks frame by frame (`LiveGraph::tick`). f64 inside, as
//! d3: the integration is long and the drawing takes f32 at the end.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub link_distance: f64,
    /// negative repels, as d3's `forceManyBody().strength`
    pub charge: f64,
    pub center: (f64, f64),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Body {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub fx: Option<f64>,
    pub fy: Option<f64>,
    /// collision radius
    pub r: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Link {
    pub source: usize,
    pub target: usize,
}

pub struct Sim {
    pub bodies: Vec<Body>,
    links: Vec<(Link, f64, f64)>,
    pub params: Params,
    pub alpha: f64,
    pub alpha_target: f64,
    alpha_min: f64,
    alpha_decay: f64,
    velocity_keep: f64,
    seed: u32,
}

impl Sim {
    /// Bodies start on d3's phyllotaxis spiral around the center, so a run is the same every time.
    pub fn new(radii: &[f64], links: &[Link], params: Params) -> Sim {
        let bodies = radii
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let radius = 10.0 * (0.5 + i as f64).sqrt();
                let angle = i as f64 * std::f64::consts::PI * (3.0 - 5f64.sqrt());
                Body {
                    x: params.center.0 + radius * angle.cos(),
                    y: params.center.1 + radius * angle.sin(),
                    r: *r,
                    ..Body::default()
                }
            })
            .collect::<Vec<_>>();
        let mut degree = vec![0usize; bodies.len()];
        for l in links {
            degree[l.source] += 1;
            degree[l.target] += 1;
        }
        let links = links
            .iter()
            .filter(|l| l.source != l.target)
            .map(|l| {
                let (ds, dt) = (degree[l.source] as f64, degree[l.target] as f64);
                (*l, 1.0 / ds.min(dt), ds / (ds + dt))
            })
            .collect();
        let alpha_min = 0.001;
        Sim {
            bodies,
            links,
            params,
            alpha: 1.0,
            alpha_target: 0.0,
            alpha_min,
            alpha_decay: 1.0 - alpha_min.powf(1.0 / 300.0),
            velocity_keep: 0.6,
            seed: 1,
        }
    }

    /// Still moving: cooling, or held warm by a drag.
    pub fn hot(&self) -> bool {
        self.alpha >= self.alpha_min || self.alpha_target > 0.0
    }

    /// Restarts the cooling from `alpha`, as d3's `simulation.alpha(a).restart()`.
    pub fn reheat(&mut self, alpha: f64) {
        self.alpha = self.alpha.max(alpha);
    }

    /// Ticks until the simulation cools, the way d3 runs a layout to rest (300 ticks).
    pub fn settle(&mut self) {
        while self.alpha >= self.alpha_min {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        self.alpha += (self.alpha_target - self.alpha) * self.alpha_decay;
        let alpha = self.alpha;
        self.link_force(alpha);
        self.charge_force(alpha);
        self.collide_force();
        // a force in d3: it shifts positions before they integrate, so a pinned body stays put
        self.center_force();
        for b in &mut self.bodies {
            match b.fx {
                Some(fx) => {
                    b.x = fx;
                    b.vx = 0.0;
                }
                None => {
                    b.vx *= self.velocity_keep;
                    b.x += b.vx;
                }
            }
            match b.fy {
                Some(fy) => {
                    b.y = fy;
                    b.vy = 0.0;
                }
                None => {
                    b.vy *= self.velocity_keep;
                    b.y += b.vy;
                }
            }
        }
    }

    /// d3's tiny random nudge for two bodies on the same spot, deterministic here.
    fn jiggle(&mut self) -> f64 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        ((self.seed >> 8) as f64 / (1u32 << 24) as f64 - 0.5) * 1e-6
    }

    fn link_force(&mut self, alpha: f64) {
        for k in 0..self.links.len() {
            let (l, strength, bias) = self.links[k];
            let (s, t) = (self.bodies[l.source], self.bodies[l.target]);
            let mut x = t.x + t.vx - s.x - s.vx;
            let mut y = t.y + t.vy - s.y - s.vy;
            if x == 0.0 {
                x = self.jiggle();
            }
            if y == 0.0 {
                y = self.jiggle();
            }
            let len = (x * x + y * y).sqrt();
            let f = (len - self.params.link_distance) / len * alpha * strength;
            let (x, y) = (x * f, y * f);
            self.bodies[l.target].vx -= x * bias;
            self.bodies[l.target].vy -= y * bias;
            self.bodies[l.source].vx += x * (1.0 - bias);
            self.bodies[l.source].vy += y * (1.0 - bias);
        }
    }

    fn charge_force(&mut self, alpha: f64) {
        let n = self.bodies.len();
        let strength = self.params.charge;
        for i in 0..n {
            let (mut ax, mut ay) = (0.0, 0.0);
            let bi = self.bodies[i];
            for (j, bj) in self.bodies.iter().enumerate() {
                if i == j {
                    continue;
                }
                let (x, y) = (bj.x - bi.x, bj.y - bi.y);
                let mut l = x * x + y * y;
                // d3's distanceMin² of 1: no blow-up when two bodies meet
                if l < 1.0 {
                    l = l.max(1e-12).sqrt();
                }
                ax += x * strength * alpha / l;
                ay += y * strength * alpha / l;
            }
            self.bodies[i].vx += ax;
            self.bodies[i].vy += ay;
        }
    }

    fn collide_force(&mut self) {
        let n = self.bodies.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (a, b) = (self.bodies[i], self.bodies[j]);
                let r = a.r + b.r;
                let mut x = a.x + a.vx - b.x - b.vx;
                let mut y = a.y + a.vy - b.y - b.vy;
                let mut l = x * x + y * y;
                if l >= r * r {
                    continue;
                }
                if x == 0.0 {
                    x = self.jiggle();
                    l += x * x;
                }
                if y == 0.0 {
                    y = self.jiggle();
                    l += y * y;
                }
                let len = l.sqrt();
                let f = (r - len) / len;
                let share = (b.r * b.r) / (a.r * a.r + b.r * b.r).max(1e-12);
                self.bodies[i].vx += x * f * share;
                self.bodies[i].vy += y * f * share;
                self.bodies[j].vx -= x * f * (1.0 - share);
                self.bodies[j].vy -= y * f * (1.0 - share);
            }
        }
    }

    /// d3's `forceCenter`: shifts every body so their mean sits on the center.
    fn center_force(&mut self) {
        let n = self.bodies.len() as f64;
        if n == 0.0 {
            return;
        }
        let (sx, sy) = self
            .bodies
            .iter()
            .fold((0.0, 0.0), |(x, y), b| (x + b.x, y + b.y));
        let (dx, dy) = (sx / n - self.params.center.0, sy / n - self.params.center.1);
        for b in &mut self.bodies {
            b.x -= dx;
            b.y -= dy;
        }
    }
}
