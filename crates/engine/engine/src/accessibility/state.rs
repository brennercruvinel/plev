//! AccessibilityState -- per-frame accumulator for the accessibility tree.

use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
use rustc_hash::FxHashMap;

use crate::input::{HitRegion, ViewId};

use super::focus::FocusGraph;
use super::id_map::{ROOT_NODE_ID, view_id_to_node_id};

/// Parameters for pushing an accessibility node with intent-derived role.
pub struct IntentNodeParams<'a> {
    pub intent: Option<crate::theme::Intent>,
    pub theme: &'a crate::theme::Theme,
    pub label: Option<&'a str>,
    pub bounds: [f32; 4],
    pub focusable: bool,
    pub parent: Option<ViewId>,
}

pub struct AccessibilityState {
    active: bool,
    pub(crate) nodes: FxHashMap<ViewId, (Role, Option<String>, [f32; 4], bool)>,
    children_map: FxHashMap<ViewId, Vec<ViewId>>,
    pub(crate) root_children: Vec<ViewId>,
    focus_graph: Option<FocusGraph>,
}

impl Default for AccessibilityState {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityState {
    pub fn new() -> Self {
        Self {
            active: false,
            nodes: FxHashMap::default(),
            children_map: FxHashMap::default(),
            root_children: Vec::new(),
            focus_graph: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn activate(&mut self) {
        self.active = true;
        log::info!("Accessibility activated");
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        log::info!("Accessibility deactivated");
    }

    pub fn begin_frame(&mut self) {
        self.nodes.clear();
        self.children_map.clear();
        self.root_children.clear();
        self.focus_graph = None;
    }

    /// Called during build_scene for each interactive/labeled element.
    pub fn push_node(
        &mut self,
        view_id: ViewId,
        role: Role,
        label: Option<&str>,
        bounds: [f32; 4], // [x, y, w, h]
        focusable: bool,
        parent: Option<ViewId>,
    ) {
        self.nodes.insert(
            view_id,
            (role, label.map(|s| s.to_string()), bounds, focusable),
        );

        if let Some(parent_id) = parent {
            self.children_map
                .entry(parent_id)
                .or_default()
                .push(view_id);
        } else {
            self.root_children.push(view_id);
        }
    }

    /// Push a node with intent-derived role resolution via Theme.
    pub fn push_node_with_intent(&mut self, view_id: ViewId, params: IntentNodeParams<'_>) {
        let role = match params.intent {
            Some(i) => params.theme.intent_role(i),
            None if params.focusable => Role::Button,
            None => Role::GenericContainer,
        };
        self.push_node(
            view_id,
            role,
            params.label,
            params.bounds,
            params.focusable,
            params.parent,
        );
    }

    /// Build the tree update from accumulated nodes.
    pub fn build_tree_update(&self, focus: Option<ViewId>) -> TreeUpdate {
        let focus_id = focus.map(view_id_to_node_id).unwrap_or(ROOT_NODE_ID);

        let mut nodes: Vec<(NodeId, Node)> = Vec::with_capacity(self.nodes.len() + 1);

        // Root node
        let mut root_node = Node::new(Role::Window);
        root_node.set_children(
            self.root_children
                .iter()
                .map(|id| view_id_to_node_id(*id))
                .collect::<Vec<_>>(),
        );
        nodes.push((ROOT_NODE_ID, root_node));

        // Child nodes
        for (view_id, (role, label, bounds, focusable)) in &self.nodes {
            let mut node = Node::new(*role);
            if let Some(label) = label {
                node.set_label(label.clone());
            }

            let rect = accesskit::Rect {
                x0: f64::from(bounds[0]),
                y0: f64::from(bounds[1]),
                x1: f64::from(bounds[0] + bounds[2]),
                y1: f64::from(bounds[1] + bounds[3]),
            };
            node.set_bounds(rect);

            // Focusable nodes are indicated by their role (Button, TextInput)
            let _ = focusable;

            // Children
            if let Some(children) = self.children_map.get(view_id) {
                node.set_children(
                    children
                        .iter()
                        .map(|id| view_id_to_node_id(*id))
                        .collect::<Vec<_>>(),
                );
            }

            nodes.push((view_id_to_node_id(*view_id), node));
        }

        TreeUpdate {
            nodes,
            tree: Some(Tree::new(ROOT_NODE_ID)),
            tree_id: accesskit::TreeId::ROOT,
            focus: focus_id,
        }
    }

    /// Build focus graph from hit regions.
    pub fn update_focus_graph(&mut self, regions: &[HitRegion]) {
        self.focus_graph = Some(FocusGraph::from_hit_regions(regions));
    }

    pub fn focus_graph(&self) -> Option<&FocusGraph> {
        self.focus_graph.as_ref()
    }
}
