//! The two inputs of the gramorph corpus this demo draws, read into the
//! engine's [`Graph`]: a d3-force preset (`nodes` with a `group`, `links`
//! with a `value`, plus the preset's `chargeStrength` and `linkDistance`)
//! and a cytoscape element list (nodes with an optional `parent`, edges).
//! Only the data: how each looks is `look.rs`, where its nodes go `lib.rs`.

use serde::Deserialize;

use engine::widgets::graph::{Edge, Forces, Graph, Group, Node};

/// d3's collision radius in the demo, world px.
const D3_COLLIDE: f64 = 15.0;

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

/// A d3 preset: its graph, and the forces the preset carries.
pub fn d3(text: &str) -> Result<(Graph, Forces), String> {
    let d: D3 = serde_json::from_str(text).map_err(|e| format!("d3 json: {e}"))?;
    let index = |id: &str| d.nodes.iter().position(|n| n.id == id);
    let nodes = d
        .nodes
        .iter()
        .map(|n| Node {
            id: n.id.clone(),
            label: n.id.clone(),
            kind: n.group,
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
        edges.push(Edge {
            weight: l.value.unwrap_or(1.0),
            ..Edge::new(from, to)
        });
    }
    let forces = Forces {
        link_distance: d.link_distance,
        charge: d.charge_strength,
        collide: D3_COLLIDE,
    };
    let graph = Graph {
        nodes,
        edges,
        groups: Vec::new(),
    };
    Ok((graph, forces))
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

/// A cytoscape element list: a node that is some node's `parent` is a group.
pub fn cytoscape(text: &str) -> Result<Graph, String> {
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
            groups.push(Group {
                id: id.clone(),
                label: label(d),
                ..Group::default()
            });
            group_ids.push(id);
        } else {
            nodes.push(Node {
                id: id.clone(),
                label: label(d),
                kind: 0,
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
        edges.push(Edge {
            directed: true,
            ..Edge::new(from, to)
        });
    }
    Ok(Graph {
        nodes,
        edges,
        groups,
    })
}
