//! How a graph looks: one [`NodeLook`] per node and one [`EdgeLook`] per
//! edge, in the [`Graph`]'s order, and the label sizes. A look is computed
//! from the data once ([`Look::of`]): a palette by a node's `kind`, a width
//! by an edge's `weight`, a head on a directed edge. Colors are theme roles
//! ([`Tone`]), resolved when the graph draws, so a graph follows the theme.

use super::model::{Edge, Graph, Node};
use crate::styling::theme::Theme;

/// A theme color by role, resolved at draw time.
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeLook {
    /// World px.
    pub radius: f32,
    pub fill: Tone,
    pub stroke: Tone,
    pub label_at: LabelAt,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeLook {
    /// World px.
    pub width: f32,
    pub alpha: f32,
    pub tone: Tone,
    /// A head at the edge's `to`, the line stopping at the node's border.
    pub arrow: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    /// One per node of the graph, in its order.
    pub nodes: Vec<NodeLook>,
    /// One per edge of the graph, in its order.
    pub edges: Vec<EdgeLook>,
    /// Label sizes in world px.
    pub node_font: f32,
    pub group_font: f32,
}

impl Look {
    /// Default label sizes in world px; `Look { node_font, ..Look::of(..) }` changes them.
    pub const NODE_FONT: f32 = 11.0;
    pub const GROUP_FONT: f32 = 13.0;

    /// The look of every element of `graph`, computed from its data.
    pub fn of(
        graph: &Graph,
        node: impl Fn(&Node) -> NodeLook,
        edge: impl Fn(&Edge) -> EdgeLook,
    ) -> Look {
        Look {
            nodes: graph.nodes.iter().map(node).collect(),
            edges: graph.edges.iter().map(edge).collect(),
            node_font: Self::NODE_FONT,
            group_font: Self::GROUP_FONT,
        }
    }

    /// True when it has one look per node and per edge of `graph`.
    pub fn fits(&self, graph: &Graph) -> bool {
        self.nodes.len() == graph.nodes.len() && self.edges.len() == graph.edges.len()
    }
}
