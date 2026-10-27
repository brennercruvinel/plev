//! What a graph is, apart from how it looks and where its nodes go: nodes,
//! edges between them, and groups (compound nodes) that hold nodes and other
//! groups. Every graph component of this category reads it; a source format
//! (d3, cytoscape, a graph language) is read into it outside the engine, and
//! [`Look`](super::look::Look) and [`Layout`](super::layout::Layout) are
//! decided apart from it.
//!
//! Elements refer to each other by position: an edge's `from` and `to` and a
//! group's members are indices into `nodes` and `groups`.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub groups: Vec<Group>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// The identity the source gave it (`data.id`, d3's `id`).
    pub id: String,
    /// What is written on it; the id when the source names none.
    pub label: String,
    /// A category from the source (d3's `group`, a class), 0 when it has
    /// none. A look may color or size by it; it is not a [`Group`].
    pub kind: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    /// A category of the relation (an app's edge type, a class), 0 when it
    /// has none. A look may color by it.
    pub kind: usize,
    /// The strength the source gives the link (d3's `value`), 1 when none.
    pub weight: f32,
    /// From `from` to `to` only; an undirected edge links both ways.
    pub directed: bool,
}

/// A compound node: the nodes and the groups it holds directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    pub id: String,
    pub label: String,
    pub nodes: Vec<usize>,
    pub groups: Vec<usize>,
}

impl Node {
    /// A node named by its id, with no kind.
    pub fn new(id: impl Into<String>) -> Node {
        let id = id.into();
        Node {
            label: id.clone(),
            id,
            kind: 0,
        }
    }
}

impl Edge {
    /// An undirected edge of weight 1 and no kind; `Edge { directed: true,
    /// ..Edge::new(a, b) }` changes what differs.
    pub fn new(from: usize, to: usize) -> Edge {
        Edge {
            from,
            to,
            kind: 0,
            weight: 1.0,
            directed: false,
        }
    }
}

impl Graph {
    /// Every index points inside the graph: each edge's ends are nodes, each
    /// group's members are nodes and groups it has. A component that draws
    /// every element (`LiveGraph`) needs it; `GraphData::from_graph` drops
    /// the edges that miss instead.
    pub fn holds(&self) -> bool {
        let (n, g) = (self.nodes.len(), self.groups.len());
        self.edges.iter().all(|e| e.from < n && e.to < n)
            && self.groups.iter().all(|group| {
                group.nodes.iter().all(|&i| i < n) && group.groups.iter().all(|&k| k < g)
            })
    }

    /// The group holding each node and each group, if any: `(node_parent, group_parent)`.
    pub fn parents(&self) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
        let mut nodes = vec![None; self.nodes.len()];
        let mut groups = vec![None; self.groups.len()];
        for (g, group) in self.groups.iter().enumerate() {
            for &n in &group.nodes {
                if let Some(slot) = nodes.get_mut(n) {
                    *slot = Some(g);
                }
            }
            for &k in &group.groups {
                if let Some(slot) = groups.get_mut(k) {
                    *slot = Some(g);
                }
            }
        }
        (nodes, groups)
    }

    /// Every node a group holds, through nested groups.
    pub fn descendants(&self, g: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack = vec![g];
        let mut seen = vec![false; self.groups.len()];
        while let Some(g) = stack.pop() {
            if g >= seen.len() || std::mem::replace(&mut seen[g], true) {
                continue;
            }
            out.extend(self.groups[g].nodes.iter().copied());
            stack.extend(self.groups[g].groups.iter().copied());
        }
        out
    }
}
