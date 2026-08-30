//! Pure graph geometry for the [`GraphView`](crate::ui::widgets::GraphView)
//! widget: a CSR adjacency built from an edge list, a deterministic
//! force-directed layout, and the pan/zoom [`ViewTransform`]. No GPU, no
//! widget state — the same code is unit-testable headless and reusable by
//! apps that precompute layouts off the UI thread.
//!
//! Layout: initial positions come from a splitmix64 hash of the node id
//! (a deterministic golden-angle scatter), then fixed-dt force iterations
//! (O(n²) repulsion + edge springs + center gravity). Above
//! [`MAX_LAYOUT_NODES`] the graph is subsampled by BFS from the
//! highest-degree node — O(n²) past ~2.5k nodes is not interactive, and a
//! neighborhood around the busiest node is the interesting part to look at
//! first. The layout is deterministic: same input, same scene.

/// Above this many nodes the layout subsamples by BFS (see module docs).
pub const MAX_LAYOUT_NODES: usize = 2_500;

/// Force iterations for small graphs; large graphs cut this to keep the
/// O(n²·iters) work bounded (~1s worst case on one thread).
const ITERS_SMALL: usize = 300;
const ITERS_LARGE: usize = 80;
const LARGE_N: usize = 800;

/// One directed edge: `from → to` of a given `kind` (an app-defined u8 —
/// the widget maps kinds to theme tones via `set_edge_tone`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphEdge {
    pub from: u32,
    pub to: u32,
    pub kind: u8,
}

/// Graph input: a node count plus the edge list. Node payloads stay with
/// the app — everything here indexes nodes by ordinal.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphSpec {
    pub n_nodes: usize,
    pub edges: Vec<GraphEdge>,
}

/// CSR adjacency built from a [`GraphSpec`]: `offsets[node..=node]` bounds
/// the node's run in `neighbors`/`kinds`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphData {
    pub n_nodes: usize,
    /// Row pointers, len `n_nodes + 1`.
    pub offsets: Vec<u32>,
    pub neighbors: Vec<u32>,
    /// Edge kind per neighbor entry.
    pub kinds: Vec<u8>,
}

impl GraphData {
    /// CSR-build from an edge list (edges to out-of-range nodes are
    /// dropped; the input order is preserved within each node).
    pub fn from_spec(spec: &GraphSpec) -> Self {
        let mut data = GraphData {
            n_nodes: spec.n_nodes,
            offsets: vec![0; spec.n_nodes + 1],
            neighbors: Vec::with_capacity(spec.edges.len()),
            kinds: Vec::with_capacity(spec.edges.len()),
        };
        // Count out-degrees, prefix-sum into offsets, then fill.
        for e in &spec.edges {
            if (e.from as usize) < spec.n_nodes && (e.to as usize) < spec.n_nodes {
                data.offsets[e.from as usize + 1] += 1;
            }
        }
        for i in 0..spec.n_nodes {
            data.offsets[i + 1] += data.offsets[i];
        }
        let mut cursor = data.offsets.clone();
        let total = data.offsets[spec.n_nodes] as usize;
        data.neighbors = vec![0; total];
        data.kinds = vec![0; total];
        for e in &spec.edges {
            if (e.from as usize) < spec.n_nodes && (e.to as usize) < spec.n_nodes {
                let slot = cursor[e.from as usize] as usize;
                data.neighbors[slot] = e.to;
                data.kinds[slot] = e.kind;
                cursor[e.from as usize] += 1;
            }
        }
        data
    }

    pub fn neighbors(&self, node: usize) -> &[u32] {
        if node + 1 >= self.offsets.len() {
            return &[];
        }
        &self.neighbors[self.offsets[node] as usize..self.offsets[node + 1] as usize]
    }

    pub fn kind(&self, node: usize, i: usize) -> Option<u8> {
        if node + 1 >= self.offsets.len() {
            return None;
        }
        let idx = self.offsets[node] as usize + i;
        (idx < self.offsets[node + 1] as usize).then(|| self.kinds[idx])
    }

    pub fn degree(&self, node: usize) -> usize {
        self.neighbors(node).len()
    }
}

/// splitmix64: deterministic per-index hashing for the initial scatter.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// A laid-out graph: node positions in a `w × h` world space (the widget
/// fits it with a [`ViewTransform`]), plus the app-node mapping when the
/// graph was subsampled.
#[derive(Clone, Debug)]
pub struct GraphScene {
    pub graph: GraphData,
    /// positions[i] is node i of `graph` (already reindexed when
    /// subsampled).
    pub positions: Vec<(f32, f32)>,
    /// Scene node → app node ordinal. Identity for full layouts.
    pub node_to: Vec<u32>,
    /// Whether the graph was BFS-subsampled (the widget discloses it).
    pub subsampled: bool,
}

/// Deterministic force-directed layout in a `w × h` world box.
pub fn compute_layout(data: &GraphData, w: f32, h: f32) -> GraphScene {
    let (graph, node_to, subsampled) = if data.n_nodes > MAX_LAYOUT_NODES {
        let (g, map) = subsample_bfs(data, MAX_LAYOUT_NODES);
        (g, map, true)
    } else {
        (
            data.clone(),
            (0..data.n_nodes as u32).collect::<Vec<_>>(),
            false,
        )
    };

    let n = graph.n_nodes;
    if n == 0 {
        return GraphScene {
            graph,
            positions: Vec::new(),
            node_to,
            subsampled,
        };
    }

    // Deterministic scatter: golden-angle spiral with a hashed radius
    // jitter, centered in the world box.
    let (cx, cy) = (w / 2.0, h / 2.0);
    let radius = (w.min(h) / 2.0) * 0.85;
    let mut pos: Vec<(f32, f32)> = (0..n)
        .map(|i| {
            let angle = i as f32 * 2.399_963; // golden angle
            let t = (i as f32 + 0.5) / n as f32;
            let jitter = (splitmix64(i as u64) % 1000) as f32 / 1000.0;
            let r = radius * t.sqrt() * (0.7 + 0.3 * jitter);
            (cx + r * angle.cos(), cy + r * angle.sin())
        })
        .collect();

    // Ideal spring length scales with the per-node area share.
    let spring = ((w * h) / n as f32).sqrt() * 0.5;
    let iters = if n > LARGE_N {
        ITERS_LARGE
    } else {
        ITERS_SMALL
    };
    let dt = 0.05;
    let mut vel = vec![(0.0f32, 0.0f32); n];

    for _ in 0..iters {
        let mut force = vec![(0.0f32, 0.0f32); n];
        // Repulsion (all pairs).
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = pos[i].0 - pos[j].0;
                let dy = pos[i].1 - pos[j].1;
                let d2 = (dx * dx + dy * dy).max(1.0);
                let f = (spring * spring) / d2;
                let d = d2.sqrt();
                let (fx, fy) = (f * dx / d, f * dy / d);
                force[i].0 += fx;
                force[i].1 += fy;
                force[j].0 -= fx;
                force[j].1 -= fy;
            }
        }
        // Springs along edges (each directed edge once).
        for i in 0..n {
            for &j in graph.neighbors(i) {
                let j = j as usize;
                if j >= n {
                    continue;
                }
                let dx = pos[j].0 - pos[i].0;
                let dy = pos[j].1 - pos[i].1;
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                let f = 0.05 * (d - spring);
                force[i].0 += f * dx / d;
                force[i].1 += f * dy / d;
            }
        }
        // Weak gravity keeps the cluster centered.
        for i in 0..n {
            force[i].0 += (cx - pos[i].0) * 0.01;
            force[i].1 += (cy - pos[i].1) * 0.01;
        }
        // Fixed-dt integration with damping.
        for i in 0..n {
            vel[i].0 = (vel[i].0 + force[i].0 * dt) * 0.85;
            vel[i].1 = (vel[i].1 + force[i].1 * dt) * 0.85;
            pos[i].0 += vel[i].0 * dt * 10.0;
            pos[i].1 += vel[i].1 * dt * 10.0;
        }
    }

    GraphScene {
        graph,
        positions: pos,
        node_to,
        subsampled,
    }
}

/// BFS from the highest-degree node, keeping at most `max_nodes` nodes;
/// returns the reindexed subgraph and the scene→app mapping. Edges to
/// dropped nodes are dropped.
fn subsample_bfs(data: &GraphData, max_nodes: usize) -> (GraphData, Vec<u32>) {
    // Highest degree wins; ties keep the LOWEST id (strict `>`, so the
    // seed is deterministic and stable across runs).
    let mut seed = 0;
    for i in 1..data.n_nodes {
        if data.degree(i) > data.degree(seed) {
            seed = i;
        }
    }
    let mut keep = Vec::with_capacity(max_nodes);
    let mut kept = vec![false; data.n_nodes];
    let mut queue = std::collections::VecDeque::from([seed as u32]);
    while let Some(node) = queue.pop_front() {
        if keep.len() >= max_nodes {
            break;
        }
        if kept[node as usize] {
            continue;
        }
        kept[node as usize] = true;
        keep.push(node);
        for &next in data.neighbors(node as usize) {
            if !kept[next as usize] {
                queue.push_back(next);
            }
        }
    }
    // Any unreached nodes (disconnected components) fill the budget.
    for (i, slot) in kept.iter_mut().enumerate() {
        if keep.len() >= max_nodes {
            break;
        }
        if !*slot {
            *slot = true;
            keep.push(i as u32);
        }
    }

    // Reindex: old id → new id.
    let mut remap = vec![u32::MAX; data.n_nodes];
    for (new, &old) in keep.iter().enumerate() {
        remap[old as usize] = new as u32;
    }
    let mut graph = GraphData {
        n_nodes: keep.len(),
        offsets: Vec::with_capacity(keep.len() + 1),
        neighbors: Vec::new(),
        kinds: Vec::new(),
    };
    graph.offsets.push(0);
    for &old in &keep {
        let start = data.offsets[old as usize] as usize;
        let end = data.offsets[old as usize + 1] as usize;
        for idx in start..end {
            let dst = data.neighbors[idx];
            if kept[dst as usize] {
                graph.neighbors.push(remap[dst as usize]);
                graph.kinds.push(data.kinds[idx]);
            }
        }
        graph.offsets.push(graph.neighbors.len() as u32);
    }
    (graph, keep)
}

/// Pan/zoom mapping between world and screen space. Scale is clamped to
/// [`MIN_SCALE`..`MAX_SCALE`] so a lost scroll storm can never lose the
/// graph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub offset: (f32, f32),
    pub scale: f32,
}

pub const MIN_SCALE: f32 = 0.05;
pub const MAX_SCALE: f32 = 16.0;

impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            offset: (0.0, 0.0),
            scale: 1.0,
        }
    }
}

impl ViewTransform {
    /// Fit a `world_w × world_h` layout into a `view_w × view_h` viewport,
    /// centered, with `margin` px of breathing room.
    pub fn fit(world_w: f32, world_h: f32, view_w: f32, view_h: f32, margin: f32) -> Self {
        let scale = ((view_w - margin * 2.0) / world_w)
            .min((view_h - margin * 2.0) / world_h)
            .clamp(MIN_SCALE, MAX_SCALE);
        Self {
            offset: (
                (view_w - world_w * scale) / 2.0,
                (view_h - world_h * scale) / 2.0,
            ),
            scale,
        }
    }

    pub fn world_to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x * self.scale + self.offset.0,
            y * self.scale + self.offset.1,
        )
    }

    pub fn screen_to_world(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x - self.offset.0) / self.scale,
            (y - self.offset.1) / self.scale,
        )
    }

    /// Pan in screen pixels.
    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        self.offset.0 += dx;
        self.offset.1 += dy;
    }

    /// Zoom by `factor` keeping the world point under `screen` fixed.
    pub fn zoom_at(&mut self, screen: (f32, f32), factor: f32) {
        let world = self.screen_to_world(screen.0, screen.1);
        self.scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        self.offset = (
            screen.0 - world.0 * self.scale,
            screen.1 - world.1 * self.scale,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 4 nodes: 0→1 (kind 0), 1→2 (kind 0), 0→2 (kind 1), 3 isolated.
    fn demo_graph() -> GraphData {
        GraphData::from_spec(&GraphSpec {
            n_nodes: 4,
            edges: vec![
                GraphEdge {
                    from: 0,
                    to: 1,
                    kind: 0,
                },
                GraphEdge {
                    from: 1,
                    to: 2,
                    kind: 0,
                },
                GraphEdge {
                    from: 0,
                    to: 2,
                    kind: 1,
                },
            ],
        })
    }

    #[test]
    fn csr_accessors_match_the_edge_list() {
        let g = demo_graph();
        assert_eq!(g.neighbors(0), &[1, 2]);
        assert_eq!(g.neighbors(1), &[2]);
        assert_eq!(g.neighbors(2), &[] as &[u32]);
        assert_eq!(g.kind(0, 0), Some(0));
        assert_eq!(g.kind(0, 1), Some(1));
        assert_eq!(g.kind(0, 2), None);
        assert_eq!(g.degree(0), 2);
        // Out-of-range probes are empty, never panics.
        assert_eq!(g.neighbors(99), &[] as &[u32]);
        assert_eq!(g.kind(99, 0), None);
    }

    #[test]
    fn spec_builder_drops_out_of_range_edges() {
        let g = GraphData::from_spec(&GraphSpec {
            n_nodes: 2,
            edges: vec![
                GraphEdge {
                    from: 0,
                    to: 1,
                    kind: 0,
                },
                GraphEdge {
                    from: 0,
                    to: 9,
                    kind: 0,
                },
                GraphEdge {
                    from: 9,
                    to: 0,
                    kind: 0,
                },
            ],
        });
        assert_eq!(g.neighbors(0), &[1]);
        assert_eq!(g.neighbors(1), &[] as &[u32]);
    }

    #[test]
    fn layout_is_deterministic() {
        let g = demo_graph();
        let a = compute_layout(&g, 1000.0, 1000.0);
        let b = compute_layout(&g, 1000.0, 1000.0);
        assert_eq!(a.positions, b.positions);
        assert!(!a.subsampled);
        assert_eq!(a.node_to, vec![0, 1, 2, 3]);
    }

    #[test]
    fn layout_stays_inside_the_world_box() {
        let g = demo_graph();
        let scene = compute_layout(&g, 1000.0, 800.0);
        for &(x, y) in &scene.positions {
            assert!((-100.0..=1100.0).contains(&x), "x={x} escaped the box");
            assert!((-100.0..=900.0).contains(&y), "y={y} escaped the box");
        }
        // Springs settle linked nodes near the ideal length (spring =
        // sqrt(w*h/n) * 0.5 ≈ 224 for this box): a loose band, not an
        // exact value.
        let d01 = dist(scene.positions[0], scene.positions[1]);
        assert!(
            (80.0..=500.0).contains(&d01),
            "linked nodes settle near the spring length (d01={d01})"
        );
    }

    #[test]
    fn subsample_keeps_the_budget_and_reindexes() {
        // Chain of 3000 nodes: subsample to the cap.
        let n = 3000usize;
        let spec = GraphSpec {
            n_nodes: n,
            edges: (0..(n - 1) as u32)
                .map(|i| GraphEdge {
                    from: i,
                    to: i + 1,
                    kind: 0,
                })
                .collect(),
        };
        let scene = compute_layout(&GraphData::from_spec(&spec), 1000.0, 1000.0);
        assert!(scene.subsampled);
        assert_eq!(scene.graph.n_nodes, MAX_LAYOUT_NODES);
        assert_eq!(scene.positions.len(), MAX_LAYOUT_NODES);
        // The chain is connected: BFS from node 0 walks it in order, so
        // the mapping is the identity prefix and every edge survives.
        assert_eq!(scene.node_to[0], 0);
        assert_eq!(scene.graph.neighbors(0), &[1]);
        assert_eq!(
            scene.graph.neighbors(MAX_LAYOUT_NODES - 1),
            &[] as &[u32],
            "the last kept node's out-edge was dropped"
        );
    }

    #[test]
    fn transform_round_trips_and_zoom_anchors_the_cursor() {
        let mut t = ViewTransform::fit(1000.0, 1000.0, 800.0, 600.0, 40.0);
        let (sx, sy) = t.world_to_screen(500.0, 500.0);
        let (wx, wy) = t.screen_to_world(sx, sy);
        assert!((wx - 500.0).abs() < 0.01 && (wy - 500.0).abs() < 0.01);

        // Zooming at a screen point keeps that world point under it.
        let anchor_screen = (200.0, 150.0);
        let anchor_world = t.screen_to_world(anchor_screen.0, anchor_screen.1);
        t.zoom_at(anchor_screen, 1.5);
        let after = t.world_to_screen(anchor_world.0, anchor_world.1);
        assert!((after.0 - anchor_screen.0).abs() < 0.01);
        assert!((after.1 - anchor_screen.1).abs() < 0.01);

        // Scale clamps.
        t.zoom_at(anchor_screen, 1e9);
        assert_eq!(t.scale, MAX_SCALE);
        t.zoom_at(anchor_screen, 1e-9);
        assert_eq!(t.scale, MIN_SCALE);
    }

    #[test]
    fn empty_graph_lays_out_empty() {
        let scene = compute_layout(&GraphData::default(), 1000.0, 1000.0);
        assert!(scene.positions.is_empty());
    }

    fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
        ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
    }
}
