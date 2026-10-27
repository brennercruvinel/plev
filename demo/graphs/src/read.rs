//! The two inputs of the gramorph corpus this demo draws, read into the
//! engine's `NetSpec`: a d3-force preset (`nodes` with a `group`, `links`
//! with a `value`, `chargeStrength`, `linkDistance`) and a cytoscape element
//! list (nodes with an optional `parent`, edges). The look copies the
//! gallery page that draws each (gramorph's `D3ForceDemo.tsx` and
//! `CytoscapeDemo.tsx`), in theme tones.

use serde::Deserialize;

use engine::widgets::graph::live_graph::net::{
    ForceParams, LabelAt, NetEdge, NetGroup, NetLayout, NetNode, NetSpec, Tone,
};

/// d3 demo's viewBox: its `forceCenter` pulls to the middle of it.
const D3_WORLD: (f32, f32) = (700.0, 500.0);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct D3 {
    nodes: Vec<D3Node>,
    #[serde(default)]
    links: Vec<D3Link>,
    #[serde(default = "d3_charge")]
    charge_strength: f64,
    #[serde(default = "d3_distance")]
    link_distance: f64,
}

fn d3_charge() -> f64 {
    -200.0
}

fn d3_distance() -> f64 {
    60.0
}

#[derive(Deserialize)]
struct D3Node {
    id: String,
    #[serde(default)]
    group: usize,
}

#[derive(Deserialize)]
struct D3Link {
    source: String,
    target: String,
    value: Option<f32>,
}

/// The d3 demo's five tiers, by `group % 5`: accent, text secondary, text
/// muted, border active, border.
const TIERS: [Tone; 5] = [
    Tone::Accent,
    Tone::TextMid,
    Tone::TextDim,
    Tone::BorderActive,
    Tone::Divider,
];

pub fn d3(text: &str) -> Result<NetSpec, String> {
    let d: D3 = serde_json::from_str(text).map_err(|e| format!("d3 json: {e}"))?;
    let index = |id: &str| d.nodes.iter().position(|n| n.id == id);
    let nodes = d
        .nodes
        .iter()
        .map(|n| NetNode {
            label: n.id.clone(),
            // the demo: radius 10 for a hub or group 0, 7 otherwise; the label 14 px above
            radius: if n.id.starts_with("hub") || n.group == 0 {
                10.0
            } else {
                7.0
            },
            fill: TIERS[n.group % TIERS.len()],
            stroke: Tone::BorderActive,
            label_at: LabelAt::Above(14.0),
        })
        .collect();
    let mut edges = Vec::with_capacity(d.links.len());
    for l in &d.links {
        let (Some(from), Some(to)) = (index(&l.source), index(&l.target)) else {
            return Err(format!(
                "d3 json: a link names a missing node: {} -> {}",
                l.source, l.target
            ));
        };
        let width = l.value.unwrap_or(1.0);
        edges.push(NetEdge {
            from,
            to,
            width,
            alpha: 0.6,
            tone: Tone::Divider,
            arrow: false,
        });
    }
    let params = ForceParams {
        link_distance: d.link_distance,
        charge: d.charge_strength,
        collide: 15.0,
    };
    Ok(NetSpec {
        nodes,
        edges,
        groups: Vec::new(),
        layout: NetLayout::Force {
            params,
            world: D3_WORLD,
        },
        node_font: 9.0,
        group_font: 13.0,
    })
}

#[derive(Deserialize)]
struct Cy {
    elements: Vec<CyElement>,
}

#[derive(Deserialize)]
struct CyElement {
    data: CyData,
}

#[derive(Deserialize)]
struct CyData {
    id: Option<String>,
    label: Option<String>,
    parent: Option<String>,
    source: Option<String>,
    target: Option<String>,
}

/// The cytoscape demo's node: a 76 px circle with its label inside.
const CY_RADIUS: f32 = 38.0;

pub fn cytoscape(text: &str) -> Result<NetSpec, String> {
    let cy: Cy = serde_json::from_str(text).map_err(|e| format!("cytoscape json: {e}"))?;
    let all: Vec<&CyData> = cy
        .elements
        .iter()
        .map(|e| &e.data)
        .filter(|d| d.source.is_none())
        .collect();
    let is_parent = |id: &str| all.iter().any(|d| d.parent.as_deref() == Some(id));
    let label = |d: &CyData| d.label.clone().or_else(|| d.id.clone()).unwrap_or_default();
    let (mut node_ids, mut group_ids) = (Vec::new(), Vec::new());
    let (mut nodes, mut groups) = (Vec::new(), Vec::new());
    for d in &all {
        let id =
            d.id.clone()
                .ok_or("cytoscape json: a node without `data.id`")?;
        if is_parent(&id) {
            groups.push(NetGroup {
                label: label(d),
                ..NetGroup::default()
            });
            group_ids.push(id);
        } else {
            nodes.push(NetNode {
                label: label(d),
                radius: CY_RADIUS,
                fill: Tone::Fill,
                stroke: Tone::Divider,
                label_at: LabelAt::Center,
            });
            node_ids.push(id);
        }
    }
    for d in &all {
        let (Some(id), Some(parent)) = (&d.id, &d.parent) else {
            continue;
        };
        let Some(g) = group_ids.iter().position(|p| p == parent) else {
            continue;
        };
        match (
            node_ids.iter().position(|n| n == id),
            group_ids.iter().position(|k| k == id),
        ) {
            (Some(n), _) => groups[g].nodes.push(n),
            (None, Some(k)) => groups[g].groups.push(k),
            _ => {}
        }
    }
    let mut edges = Vec::new();
    for d in cy
        .elements
        .iter()
        .map(|e| &e.data)
        .filter(|d| d.source.is_some())
    {
        let from = d
            .source
            .as_ref()
            .and_then(|s| node_ids.iter().position(|n| n == s));
        let to = d
            .target
            .as_ref()
            .and_then(|t| node_ids.iter().position(|n| n == t));
        let (Some(from), Some(to)) = (from, to) else {
            return Err(format!(
                "cytoscape json: an edge between missing nodes: {:?} -> {:?}",
                d.source, d.target
            ));
        };
        edges.push(NetEdge {
            from,
            to,
            width: 1.0,
            alpha: 1.0,
            tone: Tone::TextDim,
            arrow: true,
        });
    }
    let params = ForceParams {
        link_distance: 120.0,
        charge: -400.0,
        collide: CY_RADIUS as f64 + 10.0,
    };
    Ok(NetSpec {
        nodes,
        edges,
        groups,
        layout: NetLayout::Compound { params },
        node_font: 11.0,
        group_font: 13.0,
    })
}
