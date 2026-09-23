//! `Snapshot`, one validated projection: an immutable value the consumer
//! keeps as long as it wants. nothing in it is invalidated by a frame.

use indexmap::IndexMap;

use crate::id::SemId;
use crate::node::Node;

/// a validated projection. nodes keep projection order, which is
/// deterministic: the same builder calls give the same snapshot.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    nodes: IndexMap<SemId, Node>,
    focused: Option<SemId>,
}

impl Snapshot {
    pub(crate) fn from_parts(nodes: IndexMap<SemId, Node>, focused: Option<SemId>) -> Self {
        Snapshot { nodes, focused }
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
}
