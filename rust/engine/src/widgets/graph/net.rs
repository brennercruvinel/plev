//! The data a [`LiveGraph`](super::live::LiveGraph) draws: nodes with a label,
//! a radius and a tone; edges with a width, an alpha and an optional arrow;
//! groups (compound nodes) that hold nodes and other groups; and the layout
//! to run. Nothing here knows a source format: an app reads d3, cytoscape or
//! any graph language into this, the way `GraphSpec` is fed for `GraphView`.

use crate::styling::theme::Theme;

/// A theme color by role, resolved at draw time so a graph follows the theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Accent,
    Text,
    TextMid,
    TextDim,
    BorderActive,
    Divider,
    /// The hovered-surface fill: a node body on the panel.
    Fill,
    Panel,
}

impl Tone {
    pub fn color(self, theme: &Theme) -> [f32; 4] {
        let c = &theme.colors;
        match self {
            Tone::Accent => c.accent.0,
            Tone::Text => c.text.0,
            Tone::TextMid => c.text_mid.0,
            Tone::TextDim => c.text_dim.0,
            Tone::BorderActive => c.border_active.0,
            Tone::Divider => c.divider.0,
            Tone::Fill => c.bg_hover.0,
            Tone::Panel => c.bg_panel.0,
        }
    }
}

/// Where a node writes its label.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LabelAt {
    /// Centered on the node (cytoscape).
    Center,
    /// Centered `gap` px above the node's center (d3's `dy`).
    Above(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetNode {
    pub label: String,
    /// World px.
    pub radius: f32,
    pub fill: Tone,
    pub stroke: Tone,
    pub label_at: LabelAt,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetEdge {
    pub from: usize,
    pub to: usize,
    /// World px (a d3 link's `value`).
    pub width: f32,
    pub alpha: f32,
    pub tone: Tone,
    /// A head at `to`, the line stopping at the node's border.
    pub arrow: bool,
}

/// A compound node: the nodes and the groups it holds directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NetGroup {
    pub label: String,
    pub nodes: Vec<usize>,
    pub groups: Vec<usize>,
}

/// The forces, as d3 names them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForceParams {
    pub link_distance: f64,
    pub charge: f64,
    /// Collision radius of a node, world px.
    pub collide: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NetLayout {
    /// A live simulation in a fixed `w × h` world (d3's viewBox), centered.
    Force {
        params: ForceParams,
        world: (f32, f32),
    },
    /// Groups solved from the inside out, once (`compound::layout`).
    Compound { params: ForceParams },
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetSpec {
    pub nodes: Vec<NetNode>,
    pub edges: Vec<NetEdge>,
    pub groups: Vec<NetGroup>,
    pub layout: NetLayout,
    /// Label sizes in world px: a node's and a group's.
    pub node_font: f32,
    pub group_font: f32,
}

impl NetSpec {
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
