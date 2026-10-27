//! Compound graphs laid out level by level. A group is solved from the
//! inside: its members run a force simulation of their own, the box they
//! fill (padding, and room for the label above) becomes one round body in
//! the level that holds the group, and an edge that crosses groups pulls on
//! the two ancestors that meet at that level. Members keep their place
//! relative to their group, so no group box covers a node it does not hold:
//! the overlap a flat layout plus member bounding boxes produces (measured in
//! gramorph on mermaid subgraphs: 11 nodes covered by groups not theirs).

use super::Forces;
use super::force::{Link, Params, Sim};
use crate::widgets::graph::look::Look;
use crate::widgets::graph::model::Graph;

/// Cytoscape's compound padding, and the room a group label takes above its box.
pub const PAD: f32 = 10.0;
pub const LABEL: f32 = 22.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Member {
    Node(usize),
    Group(usize),
}

/// `x0, y0, x1, y1`.
type Bounds = (f64, f64, f64, f64);

/// A level solved: each member's center relative to the level's origin (a
/// group with its own level solved inside), and the box they fill.
struct Solved {
    members: Vec<(Member, (f64, f64), Option<Solved>)>,
    bounds: Bounds,
}

/// The center of every node, in `graph.nodes` order, around the origin; a
/// node takes the room of its radius in `look`.
pub fn layout(graph: &Graph, look: &Look, forces: Forces) -> Vec<(f32, f32)> {
    let (node_parent, group_parent) = graph.parents();
    let ctx = Ctx {
        graph,
        look,
        node_parent: &node_parent,
        group_parent: &group_parent,
        forces,
    };
    let solved = ctx.solve(None, 0);
    let mut centers = vec![(0.0, 0.0); graph.nodes.len()];
    place(&solved, (0.0, 0.0), &mut centers);
    centers
}

struct Ctx<'a> {
    graph: &'a Graph,
    look: &'a Look,
    node_parent: &'a [Option<usize>],
    group_parent: &'a [Option<usize>],
    forces: Forces,
}

impl Ctx<'_> {
    fn members(&self, level: Option<usize>) -> Vec<Member> {
        let nodes = (0..self.graph.nodes.len()).filter(|&i| self.node_parent[i] == level);
        let groups = (0..self.graph.groups.len()).filter(|&g| self.group_parent[g] == level);
        nodes
            .map(Member::Node)
            .chain(groups.map(Member::Group))
            .collect()
    }

    /// The member of `level` that holds `node`, walking up the groups.
    fn ancestor_at(&self, node: usize, level: Option<usize>) -> Option<Member> {
        let mut at = Member::Node(node);
        for _ in 0..=self.graph.groups.len() {
            let up = match at {
                Member::Node(i) => self.node_parent[i],
                Member::Group(g) => self.group_parent[g],
            };
            if up == level {
                return Some(at);
            }
            at = Member::Group(up?);
        }
        None
    }

    fn solve(&self, level: Option<usize>, depth: usize) -> Solved {
        let members = self.members(level);
        let mut inner: Vec<Option<Solved>> = members
            .iter()
            .map(|m| match m {
                Member::Group(g) if depth < self.graph.groups.len() => {
                    Some(self.solve(Some(*g), depth + 1))
                }
                _ => None,
            })
            .collect();
        let sizes: Vec<(f64, f64)> = members
            .iter()
            .zip(&inner)
            .map(|(m, solved)| match (m, solved) {
                (Member::Node(i), _) => {
                    let d = 2.0 * self.look.nodes[*i].radius as f64;
                    (d, d)
                }
                (Member::Group(_), Some(s)) => body_of(s.bounds),
                (Member::Group(_), None) => (0.0, 0.0),
            })
            .collect();
        let radii: Vec<f64> = members
            .iter()
            .zip(&sizes)
            .map(|(m, (w, h))| match m {
                Member::Node(_) => self.forces.collide,
                Member::Group(_) => 0.5 * (w * w + h * h).sqrt() + PAD as f64,
            })
            .collect();
        let mut links = Vec::new();
        for e in &self.graph.edges {
            let a = self.ancestor_at(e.from, level);
            let b = self.ancestor_at(e.to, level);
            let (Some(a), Some(b)) = (a, b) else { continue };
            let s = members.iter().position(|m| *m == a);
            let t = members.iter().position(|m| *m == b);
            if let (Some(s), Some(t)) = (s, t)
                && s != t
            {
                links.push(Link {
                    source: s,
                    target: t,
                });
            }
        }
        let params = Params {
            link_distance: self.forces.link_distance,
            charge: self.forces.charge,
            center: (0.0, 0.0),
        };
        let mut sim = Sim::new(&radii, &links, params);
        sim.settle();
        let mut bounds = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        let mut placed = Vec::with_capacity(members.len());
        for (k, ((m, (w, h)), b)) in members.iter().zip(&sizes).zip(&sim.bodies).enumerate() {
            bounds = (
                bounds.0.min(b.x - w / 2.0),
                bounds.1.min(b.y - h / 2.0),
                bounds.2.max(b.x + w / 2.0),
                bounds.3.max(b.y + h / 2.0),
            );
            placed.push((*m, (b.x, b.y), inner[k].take()));
        }
        if placed.is_empty() {
            bounds = (0.0, 0.0, 0.0, 0.0);
        }
        Solved {
            members: placed,
            bounds,
        }
    }
}

/// A group's body: its members' box with padding, and the label's room above, as `w, h`.
fn body_of(b: Bounds) -> (f64, f64) {
    let pad = PAD as f64;
    (b.2 - b.0 + 2.0 * pad, b.3 - b.1 + 2.0 * pad + LABEL as f64)
}

/// Absolute centers: a level's origin plus each member's place; a group's
/// content is centered in its body, below the label's room.
fn place(solved: &Solved, origin: (f64, f64), centers: &mut [(f32, f32)]) {
    for (m, (x, y), inner) in &solved.members {
        let at = (origin.0 + x, origin.1 + y);
        match (m, inner) {
            (Member::Node(i), _) => centers[*i] = (at.0 as f32, at.1 as f32),
            (Member::Group(_), Some(inner)) => {
                let cx = (inner.bounds.0 + inner.bounds.2) / 2.0;
                let cy = (inner.bounds.1 + inner.bounds.3) / 2.0;
                place(inner, (at.0 - cx, at.1 - cy + LABEL as f64 / 2.0), centers);
            }
            (Member::Group(_), None) => {}
        }
    }
}

/// Every group's box (`x, y, w, h`, world px) from where its members are now:
/// the members' box with padding, a nested group counted with its label's
/// room. The label sits above the box.
pub fn group_boxes(graph: &Graph, look: &Look, centers: &[(f32, f32)]) -> Vec<Option<[f32; 4]>> {
    let mut boxes: Vec<Option<[f32; 4]>> = vec![None; graph.groups.len()];
    for _ in 0..=graph.groups.len() {
        for (g, group) in graph.groups.iter().enumerate() {
            if boxes[g].is_some() {
                continue;
            }
            let inner: Option<Vec<[f32; 4]>> = group.groups.iter().map(|&k| boxes[k]).collect();
            let Some(inner) = inner else { continue };
            let mut b: Option<[f32; 4]> = None;
            let mut grow = |r: [f32; 4]| {
                b = Some(match b {
                    None => r,
                    Some(a) => [
                        a[0].min(r[0]),
                        a[1].min(r[1]),
                        a[2].max(r[2]),
                        a[3].max(r[3]),
                    ],
                });
            };
            for &n in &group.nodes {
                let (x, y) = centers[n];
                let r = look.nodes[n].radius;
                grow([x - r, y - r, x + r, y + r]);
            }
            for [x, y, w, h] in inner {
                grow([x, y - LABEL, x + w, y + h]);
            }
            if let Some([x0, y0, x1, y1]) = b {
                boxes[g] = Some([x0 - PAD, y0 - PAD, x1 - x0 + 2.0 * PAD, y1 - y0 + 2.0 * PAD]);
            }
        }
    }
    boxes
}
