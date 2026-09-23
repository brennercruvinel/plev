//! `Snapshot`, one validated projection: an immutable value the consumer
//! keeps as long as it wants. nothing in it is invalidated by a frame.

use std::fmt;

use indexmap::IndexMap;

use crate::hash::{self, Revision};
use crate::id::SemId;
use crate::node::Node;
use crate::value::{StateRef, Value};

/// a validated projection.
///
/// nodes keep projection order, which is deterministic: the same builder
/// calls give the same snapshot and the same `revision` on every run and
/// target. equality follows the revision: same nodes in the same order
/// (ignoring `source`) and the same focus.
///
/// a snapshot from `finish` holds every invariant of the contract. a
/// `subtree` is a scope of one: its root keeps its parent and its actions
/// keep their reads, which may name nodes outside the scope. a deserialized
/// snapshot is taken as sent; every query stays total on it.
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "SnapshotWire", into = "SnapshotWire"))]
pub struct Snapshot {
    nodes: IndexMap<SemId, Node>,
    focused: Option<SemId>,
    /// children of each node by index, in projection order.
    kids: Vec<Vec<usize>>,
    /// nodes whose parent is none or not in this snapshot.
    roots: Vec<usize>,
}

impl Snapshot {
    pub(crate) fn from_parts(nodes: IndexMap<SemId, Node>, focused: Option<SemId>) -> Self {
        let mut kids = vec![Vec::new(); nodes.len()];
        let mut roots = Vec::new();
        for (at, node) in nodes.values().enumerate() {
            let parent = node.parent.as_ref().and_then(|p| nodes.get_index_of(p));
            match parent.and_then(|p| kids.get_mut(p)) {
                Some(list) => list.push(at),
                None => roots.push(at),
            }
        }
        Snapshot {
            nodes,
            focused,
            kids,
            roots,
        }
    }

    /// every node, in projection order.
    pub fn nodes(&self) -> &IndexMap<SemId, Node> {
        &self.nodes
    }

    pub fn get(&self, id: &SemId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// the node that has focus, if any.
    pub fn focused(&self) -> Option<&SemId> {
        self.focused.as_ref()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// the children of `id`, in projection order. empty for a leaf or an
    /// unknown id.
    pub fn children<'a>(&'a self, id: &SemId) -> impl Iterator<Item = &'a Node> + use<'a> {
        let list = self.nodes.get_index_of(id).and_then(|at| self.kids.get(at));
        list.into_iter()
            .flatten()
            .filter_map(|&at| self.node_at(at))
    }

    /// the nodes without a parent in this snapshot, in projection order.
    pub fn roots(&self) -> impl Iterator<Item = &Node> + '_ {
        self.roots.iter().filter_map(|&at| self.node_at(at))
    }

    /// the value a state ref points at.
    pub fn value(&self, at: &StateRef) -> Option<&Value> {
        self.nodes.get(&at.node)?.state.get(&at.key)
    }

    /// the stable hash of the semantic content (see the `hash` module for
    /// the encoding). ignores `source`.
    pub fn revision(&self) -> Revision {
        hash::revision(&self.nodes, self.focused.as_ref())
    }

    /// `id` and its descendants, in projection order, for an observation
    /// by scope. the focus stays when it is inside. empty for an unknown
    /// id.
    pub fn subtree(&self, id: &SemId) -> Snapshot {
        let Some(root) = self.nodes.get_index_of(id) else {
            return Snapshot::default();
        };
        let mut keep = vec![false; self.nodes.len()];
        let mut stack = vec![root];
        while let Some(at) = stack.pop() {
            match keep.get_mut(at) {
                Some(seen) if !*seen => *seen = true,
                _ => continue,
            }
            if let Some(list) = self.kids.get(at) {
                stack.extend(list.iter().copied());
            }
        }
        let nodes: IndexMap<SemId, Node> = self
            .nodes
            .iter()
            .zip(&keep)
            .filter(|&(_, &kept)| kept)
            .map(|((id, node), _)| (id.clone(), node.clone()))
            .collect();
        let focused = self.focused.clone().filter(|f| nodes.contains_key(f));
        Snapshot::from_parts(nodes, focused)
    }

    fn node_at(&self, at: usize) -> Option<&Node> {
        self.nodes.get_index(at).map(|(_, node)| node)
    }
}

impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        self.focused == other.focused
            && self.nodes.len() == other.nodes.len()
            && self.nodes.values().eq(other.nodes.values())
    }
}

impl Eq for Snapshot {}

impl fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snapshot")
            .field("nodes", &self.nodes)
            .field("focused", &self.focused)
            .finish()
    }
}

/// the serde form: nodes as a list in projection order, and the focus.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct SnapshotWire {
    nodes: Vec<Node>,
    focused: Option<SemId>,
}

#[cfg(feature = "serde")]
impl From<SnapshotWire> for Snapshot {
    fn from(wire: SnapshotWire) -> Self {
        let mut nodes = IndexMap::with_capacity(wire.nodes.len());
        for node in wire.nodes {
            nodes.entry(node.id.clone()).or_insert(node);
        }
        Snapshot::from_parts(nodes, wire.focused)
    }
}

#[cfg(feature = "serde")]
impl From<Snapshot> for SnapshotWire {
    fn from(snap: Snapshot) -> Self {
        SnapshotWire {
            nodes: snap.nodes.into_values().collect(),
            focused: snap.focused,
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::{Snapshot, SnapshotWire};
    use crate::{Role, SnapshotBuilder};

    #[test]
    fn the_wire_form_rebuilds_the_same_snapshot() {
        let mut b = SnapshotBuilder::new();
        b.node("list", Role::List, "files");
        b.node("list.a", Role::ListItem, "a").parent("list");
        b.node("list.b", Role::ListItem, "b").parent("list");
        b.focus("list.b");
        let snap = b.finish().unwrap();
        let back = Snapshot::from(SnapshotWire::from(snap.clone()));
        assert_eq!(back, snap);
        assert_eq!(back.revision(), snap.revision());
        let kids: Vec<&str> = back
            .children(&"list".parse().unwrap())
            .map(|n| n.id.as_str())
            .collect();
        assert_eq!(kids, ["list.a", "list.b"]);
    }

    #[test]
    fn a_wire_form_with_a_loop_stays_total() {
        let mut b = SnapshotBuilder::new();
        b.node("a", Role::Group, "a");
        b.node("b", Role::Group, "b").parent("a");
        let mut wire = SnapshotWire::from(b.finish().unwrap());
        wire.nodes[0].parent = Some("b".parse().unwrap());
        let dup = wire.nodes[1].clone();
        wire.nodes.push(dup);
        let snap = Snapshot::from(wire);
        assert_eq!(snap.len(), 2);
        assert_eq!(snap.roots().count(), 0);
        assert_eq!(snap.subtree(&"a".parse().unwrap()).len(), 2);
    }
}
