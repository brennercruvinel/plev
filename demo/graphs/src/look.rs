//! How each page looks: the gallery page that draws it in gramorph
//! (`D3ForceDemo.tsx`, `CytoscapeDemo.tsx`), copied in theme tones.

use engine::widgets::graph::{EdgeLook, Graph, LabelAt, Look, NodeLook, Tone};

/// The d3 demo's five tiers, by `kind % 5`: accent, text secondary, text
/// muted, border active, border.
const TIERS: [Tone; 5] = [
    Tone::Accent,
    Tone::TextMid,
    Tone::TextDim,
    Tone::BorderActive,
    Tone::Divider,
];

/// The cytoscape demo's node: a 76 px circle with its label inside.
pub const CY_RADIUS: f32 = 38.0;

/// d3's: radius 10 for a hub or kind 0 and 7 otherwise, filled by tier, the
/// label 14 px above; links as wide as their weight, at 0.6.
pub fn d3(graph: &Graph) -> Look {
    let look = Look::of(
        graph,
        |n| NodeLook {
            radius: if n.id.starts_with("hub") || n.kind == 0 {
                10.0
            } else {
                7.0
            },
            fill: TIERS[n.kind % TIERS.len()],
            stroke: Tone::BorderActive,
            label_at: LabelAt::Above(14.0),
        },
        |e| EdgeLook {
            width: e.weight,
            alpha: 0.6,
            tone: Tone::Divider,
            arrow: e.directed,
        },
    );
    Look {
        node_font: 9.0,
        ..look
    }
}

/// cytoscape's: every node the same circle with its label inside, a head on
/// every directed edge.
pub fn cytoscape(graph: &Graph) -> Look {
    Look::of(
        graph,
        |_| NodeLook {
            radius: CY_RADIUS,
            fill: Tone::Fill,
            stroke: Tone::Divider,
            label_at: LabelAt::Center,
        },
        |e| EdgeLook {
            width: 1.0,
            alpha: 1.0,
            tone: Tone::TextDim,
            arrow: e.directed,
        },
    )
}

/// The rosetta's ir: a 28 px node, a cylinder (kind 2) and a record (kind 3)
/// told apart by tone until the widgets draw shapes, a dashed edge (kind 1)
/// dimmer, a head on every directed edge.
pub fn elk(graph: &Graph) -> Look {
    Look::of(
        graph,
        |n| NodeLook {
            radius: 28.0,
            fill: match n.kind {
                2 => Tone::Accent,
                3 => Tone::TextMid,
                _ => Tone::Fill,
            },
            stroke: Tone::BorderActive,
            label_at: LabelAt::Center,
        },
        |e| EdgeLook {
            width: 1.0,
            alpha: if e.kind == 1 { 0.5 } else { 1.0 },
            tone: Tone::TextDim,
            arrow: e.directed,
        },
    )
}
