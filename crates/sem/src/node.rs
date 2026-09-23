//! `Node`, one control as the contract sees it, and `Source`, the place in
//! the code that projected it.

use std::borrow::Cow;

use indexmap::IndexMap;

use crate::decl::{ActionDecl, Constraints};
use crate::id::SemId;
use crate::name::StateKey;
use crate::role::Role;
use crate::value::Value;

/// one control: who it is, what it means, its state, what it accepts and
/// what bounds it.
///
/// `parent` is declared, never derived from the id text. `inert` is the
/// effective value: a node is inert when it or any ancestor is (an open
/// modal leaves the background inert). `state` keeps projection order.
///
/// equality ignores `source` and the order state keys were declared in:
/// two nodes that mean the same are equal wherever and however they were
/// projected.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node {
    pub id: SemId,
    pub role: Role,
    pub label: String,
    pub description: Option<String>,
    pub parent: Option<SemId>,
    pub state: IndexMap<StateKey, Value>,
    pub actions: Vec<ActionDecl>,
    pub constraints: Constraints,
    pub enabled: bool,
    pub inert: bool,
    pub focusable: bool,
    pub source: Option<Source>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.role == other.role
            && self.label == other.label
            && self.description == other.description
            && self.parent == other.parent
            && self.state == other.state
            && self.actions == other.actions
            && self.constraints == other.constraints
            && self.enabled == other.enabled
            && self.inert == other.inert
            && self.focusable == other.focusable
    }
}

impl Eq for Node {}

/// where a node was projected: file, line and column of the call to
/// `SnapshotBuilder::node`, captured by `#[track_caller]`. it points at the
/// projection site, not at the render code. it is metadata: it never takes
/// part in equality, in the revision or in a diff.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Source {
    pub file: Cow<'static, str>,
    pub line: u32,
    pub column: u32,
}
