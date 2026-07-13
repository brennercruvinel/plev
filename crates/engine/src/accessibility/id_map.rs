//! ViewId <-> NodeId mapping for AccessKit integration.

use accesskit::NodeId;

use crate::input::ViewId;

/// accesskit 0.24: NodeId wraps u64. We reserve u64::MAX for root.
pub(crate) const ROOT_NODE_ID: NodeId = NodeId(u64::MAX);

pub(crate) fn view_id_to_node_id(id: ViewId) -> NodeId {
    // ViewId(0) maps to NodeId(1), ViewId(1) -> NodeId(2), etc.
    // We add 1 so that ViewId(0) doesn't become NodeId(0).
    NodeId(id.0.wrapping_add(1))
}

#[allow(dead_code)]
pub(crate) fn node_id_to_view_id(id: NodeId) -> Option<ViewId> {
    if id == ROOT_NODE_ID {
        return None;
    }
    if id.0 == 0 {
        return None;
    }
    Some(ViewId(id.0 - 1))
}
