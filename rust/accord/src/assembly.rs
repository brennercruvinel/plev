//! projection. the app describes its current state through
//! `Semantics::project`; the builder collects the description and
//! `finish` validates it as a whole.
//!
//! the builder takes ids, keys, action names and state refs either typed or
//! as text. text is parsed on the spot and a bad one is recorded, never
//! panicked on, so a projection needs neither `?` nor `unwrap` and `finish`
//! reports every mistake at once.

use std::borrow::Cow;
use std::panic::Location;

use indexmap::IndexMap;

use crate::controls::{Node, Source};
use crate::failures::{SemError, SemErrors};
use crate::identity::SemId;
use crate::manifest::{ActionDecl, Constraints, ParamSpec, Policy};
use crate::notation::{ActionName, StateKey};
use crate::property::{StateRef, Value};
use crate::snapshot::Snapshot;
use crate::taxonomy::Role;
use crate::validate;

/// a type whose state has a meaning.
pub trait Semantics {
    /// describes the current state into `b`. called on demand (a frame, an
    /// agent observation, the check of an action), always from the live
    /// state; `sem` keeps no reference to it between calls. what is not
    /// projected does not exist for the contract, so hover, animation and
    /// position can never invalidate anything.
    fn project(&self, b: &mut SnapshotBuilder);
}

/// collects one projection. the same sequence of calls yields the same
/// snapshot on every run and every target.
#[derive(Debug, Default)]
pub struct SnapshotBuilder {
    nodes: Vec<Node>,
    focus: Option<SemId>,
    errors: Vec<SemError>,
}

pub(crate) fn resolve<T>(input: impl TryInto<T, Error: Into<SemError>>) -> Result<T, SemError> {
    input.try_into().map_err(Into::into)
}

impl SnapshotBuilder {
    pub fn new() -> Self {
        SnapshotBuilder::default()
    }

    /// declares a node. it is enabled, not inert, focusable when its role
    /// is interactive, and records the calling file, line and column as its
    /// `source`.
    #[track_caller]
    pub fn node(
        &mut self,
        id: impl TryInto<SemId, Error: Into<SemError>>,
        role: Role,
        label: impl Into<String>,
    ) -> NodeBuilder<'_> {
        let at = Location::caller();
        let slot = match resolve(id) {
            Ok(id) => {
                let slot = self.nodes.len();
                self.nodes.push(Node {
                    id,
                    role,
                    label: label.into(),
                    description: None,
                    parent: None,
                    state: IndexMap::new(),
                    actions: Vec::new(),
                    constraints: Constraints::default(),
                    enabled: true,
                    inert: false,
                    focusable: role.is_interactive(),
                    source: Some(Source {
                        file: Cow::Borrowed(at.file()),
                        line: at.line(),
                        column: at.column(),
                    }),
                });
                Some(slot)
            }
            Err(e) => {
                self.errors.push(e);
                None
            }
        };
        NodeBuilder { b: self, slot }
    }

    /// the nodes declared so far in this projection, in the order they
    /// were declared (a node whose id did not parse is not among them).
    /// what a projection adds after the app's own reads the app's
    /// declarations here (task-70: a composite reads what its steps read).
    pub fn declared(&self) -> &[Node] {
        &self.nodes
    }

    /// goes on declaring the node `id`, declared earlier in this
    /// projection (the first with that id): more state, more actions, as
    /// the chain `node` returned. `None` when no node with that id was
    /// declared.
    pub fn reopen(&mut self, id: &SemId) -> Option<NodeBuilder<'_>> {
        let slot = self.nodes.iter().position(|n| n.id == *id)?;
        Some(NodeBuilder {
            b: self,
            slot: Some(slot),
        })
    }

    /// the node that has focus. the last call wins.
    pub fn focus(&mut self, id: impl TryInto<SemId, Error: Into<SemError>>) {
        match resolve(id) {
            Ok(id) => self.focus = Some(id),
            Err(e) => self.errors.push(e),
        }
    }

    /// validates the projection and returns the snapshot, or every
    /// violation found, in projection order.
    pub fn finish(self) -> Result<Snapshot, SemErrors> {
        validate::finish(self.nodes, self.focus, self.errors)
    }
}

/// the node being declared. methods chain; `action` opens the action being
/// declared and ends the chain for the node.
///
/// a node whose id did not parse is still accepted by every method, so the
/// mistakes in its keys and refs are reported too, and it is left out of
/// the snapshot.
pub struct NodeBuilder<'a> {
    b: &'a mut SnapshotBuilder,
    slot: Option<usize>,
}

impl NodeBuilder<'_> {
    fn node(&mut self) -> Option<&mut Node> {
        let slot = self.slot?;
        self.b.nodes.get_mut(slot)
    }

    /// the parent in the tree. declared, never taken from the id text.
    pub fn parent(&mut self, id: impl TryInto<SemId, Error: Into<SemError>>) -> &mut Self {
        match resolve(id) {
            Ok(id) => {
                if let Some(n) = self.node() {
                    n.parent = Some(id);
                }
            }
            Err(e) => self.b.errors.push(e),
        }
        self
    }

    pub fn description(&mut self, text: impl Into<String>) -> &mut Self {
        if let Some(n) = self.node() {
            n.description = Some(text.into());
        }
        self
    }

    /// one piece of state. a key declared twice on the same node is an
    /// error, not an overwrite.
    pub fn state(
        &mut self,
        key: impl TryInto<StateKey, Error: Into<SemError>>,
        value: impl Into<Value>,
    ) -> &mut Self {
        let slot = self.slot;
        let SnapshotBuilder { nodes, errors, .. } = &mut *self.b;
        match resolve(key) {
            Ok(key) => {
                if let Some(n) = slot.and_then(|i| nodes.get_mut(i)) {
                    if n.state.contains_key(&key) {
                        errors.push(SemError::InvalidKey {
                            text: key.to_string(),
                            reason: format!("state key declared twice on {}", n.id),
                        });
                    } else {
                        n.state.insert(key, value.into());
                    }
                }
            }
            Err(e) => errors.push(e),
        }
        self
    }

    /// declares an action, enabled, with the default policy, and returns
    /// it for its own chain (`label`, `enabled`, `reads`, `param`,
    /// `policy`).
    pub fn action(
        &mut self,
        name: impl TryInto<ActionName, Error: Into<SemError>>,
    ) -> ActionBuilder<'_> {
        let slot = self.slot;
        let SnapshotBuilder { nodes, errors, .. } = &mut *self.b;
        let decl = match resolve(name) {
            Ok(name) => slot.and_then(|i| nodes.get_mut(i)).and_then(|n| {
                n.actions.push(ActionDecl::new(name));
                n.actions.last_mut()
            }),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        ActionBuilder { errors, decl }
    }

    pub fn constraints(&mut self, constraints: Constraints) -> &mut Self {
        if let Some(n) = self.node() {
            n.constraints = constraints;
        }
        self
    }

    pub fn enabled(&mut self, on: bool) -> &mut Self {
        if let Some(n) = self.node() {
            n.enabled = on;
        }
        self
    }

    /// makes this node and, in the snapshot, every descendant inert.
    pub fn inert(&mut self, on: bool) -> &mut Self {
        if let Some(n) = self.node() {
            n.inert = on;
        }
        self
    }

    pub fn focusable(&mut self, on: bool) -> &mut Self {
        if let Some(n) = self.node() {
            n.focusable = on;
        }
        self
    }
}

/// the action being declared. methods chain.
pub struct ActionBuilder<'a> {
    errors: &'a mut Vec<SemError>,
    decl: Option<&'a mut ActionDecl>,
}

impl ActionBuilder<'_> {
    pub fn label(&mut self, text: impl Into<String>) -> &mut Self {
        if let Some(d) = self.decl.as_deref_mut() {
            d.label = Some(text.into());
        }
        self
    }

    pub fn enabled(&mut self, on: bool) -> &mut Self {
        if let Some(d) = self.decl.as_deref_mut() {
            d.enabled = on;
        }
        self
    }

    /// a piece of state the action depends on, typed or as
    /// `"<semid>:<key>"`. it must exist in the same snapshot.
    pub fn reads(&mut self, at: impl TryInto<StateRef, Error: Into<SemError>>) -> &mut Self {
        match resolve(at) {
            Ok(at) => {
                if let Some(d) = self.decl.as_deref_mut() {
                    d.reads.push(at);
                }
            }
            Err(e) => self.errors.push(e),
        }
        self
    }

    pub fn param(&mut self, spec: ParamSpec) -> &mut Self {
        if let Some(d) = self.decl.as_deref_mut() {
            d.param = Some(spec);
        }
        self
    }

    pub fn policy(&mut self, policy: Policy) -> &mut Self {
        if let Some(d) = self.decl.as_deref_mut() {
            d.policy = policy;
        }
        self
    }
}
