//! `State`, the semantic state at an instant as the recording holds it:
//! the nodes of the governing full snapshot, in projection order, with the
//! changes recorded after it applied in place, and the focus.
//!
//! it is not a `sem::Snapshot` on purpose. a snapshot can only be built
//! inside `sem` or through its validating builder, and the builder stamps
//! every node with the builder's call site as its `source`; the recorder
//! must not rewrite what it recorded, so the nodes here keep the source
//! the app projected them with. `revision` builds that throwaway snapshot
//! only to hash it (the revision ignores `source`).

use accord::{
    Change, Node, Revision, SemErrors, SemId, Snapshot, SnapshotBuilder, StateRef, Value,
};

use crate::error::ReadError;

/// the semantic state at an instant, as recorded.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    nodes: Vec<Node>,
    focused: Option<SemId>,
}

impl State {
    /// the state a full snapshot records.
    pub(crate) fn of(snapshot: &Snapshot) -> State {
        State {
            nodes: snapshot.nodes().values().cloned().collect(),
            focused: snapshot.focused().cloned(),
        }
    }

    /// applies changes recorded after this state, in order. a change that
    /// adds a node, moves one or reorders is never recorded as a change
    /// (the snapshot is stored whole), so it is an error here, as is a
    /// change on a node the state does not hold.
    pub(crate) fn apply(&mut self, changes: &[Change]) -> Result<(), ReadError> {
        for c in changes {
            self.apply_one(c)?;
        }
        Ok(())
    }

    fn apply_one(&mut self, c: &Change) -> Result<(), ReadError> {
        match c {
            Change::NodeRemoved(id) => {
                let at = self.at(id)?;
                self.nodes.remove(at);
            }
            Change::StateChanged { at, after, .. } => {
                let node = self.node_mut(&at.node)?;
                match after {
                    Some(v) => {
                        node.state.insert(at.key.clone(), v.clone());
                    }
                    None => {
                        node.state.shift_remove(&at.key);
                    }
                }
            }
            Change::LabelChanged { node, after, .. } => self.node_mut(node)?.label = after.clone(),
            Change::DescriptionChanged { node, after, .. } => {
                self.node_mut(node)?.description = after.clone();
            }
            Change::RoleChanged { node, after, .. } => self.node_mut(node)?.role = *after,
            Change::ActionsChanged { node, after, .. } => {
                self.node_mut(node)?.actions = after.clone();
            }
            Change::ConstraintsChanged { node, after, .. } => {
                self.node_mut(node)?.constraints = after.clone();
            }
            Change::FlagsChanged {
                node,
                enabled,
                inert,
                focusable,
            } => {
                let n = self.node_mut(node)?;
                n.enabled = enabled.unwrap_or(n.enabled);
                n.inert = inert.unwrap_or(n.inert);
                n.focusable = focusable.unwrap_or(n.focusable);
            }
            Change::FocusChanged { after, .. } => self.focused = after.clone(),
            Change::NodeAdded(_) => return Err(ReadError::Structural("node_added")),
            Change::ParentChanged { .. } => return Err(ReadError::Structural("parent_changed")),
            Change::Reordered { .. } => return Err(ReadError::Structural("reordered")),
        }
        Ok(())
    }

    fn at(&self, id: &SemId) -> Result<usize, ReadError> {
        self.nodes
            .iter()
            .position(|n| &n.id == id)
            .ok_or_else(|| ReadError::UnknownNode(id.clone()))
    }

    fn node_mut(&mut self, id: &SemId) -> Result<&mut Node, ReadError> {
        let at = self.at(id)?;
        self.nodes
            .get_mut(at)
            .ok_or_else(|| ReadError::UnknownNode(id.clone()))
    }

    /// every node, in projection order.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn get(&self, id: &SemId) -> Option<&Node> {
        self.nodes.iter().find(|n| &n.id == id)
    }

    /// the value a state ref points at.
    pub fn value(&self, at: &StateRef) -> Option<&Value> {
        self.get(&at.node)?.state.get(&at.key)
    }

    /// the node that had focus, if any.
    pub fn focused(&self) -> Option<&SemId> {
        self.focused.as_ref()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// the revision of this state, the same `sem` gives the snapshot it
    /// was recorded from. it projects the nodes again through the
    /// validating builder and hashes the result; an error means the
    /// recording does not hold a valid projection.
    pub fn revision(&self) -> Result<Revision, SemErrors> {
        let mut b = SnapshotBuilder::new();
        for n in &self.nodes {
            let mut nb = b.node(n.id.clone(), n.role, n.label.clone());
            if let Some(parent) = &n.parent {
                nb.parent(parent.clone());
            }
            if let Some(text) = &n.description {
                nb.description(text.clone());
            }
            for (key, value) in &n.state {
                nb.state(key.clone(), value.clone());
            }
            nb.constraints(n.constraints.clone())
                .enabled(n.enabled)
                .inert(n.inert)
                .focusable(n.focusable);
            for decl in &n.actions {
                let mut ab = nb.action(decl.name.clone());
                if let Some(label) = &decl.label {
                    ab.label(label.clone());
                }
                ab.enabled(decl.enabled).policy(decl.policy);
                for read in &decl.reads {
                    ab.reads(read.clone());
                }
                if let Some(spec) = decl.param {
                    ab.param(spec);
                }
            }
        }
        if let Some(id) = &self.focused {
            b.focus(id.clone());
        }
        Ok(b.finish()?.revision())
    }
}
