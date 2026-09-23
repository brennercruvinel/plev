//! `Change`, the delta between two snapshots: what a rejected action hands
//! back ("removed: B") and what an incremental observation sends.

use std::collections::{HashMap, HashSet};

use crate::decl::{ActionDecl, Constraints};
use crate::id::SemId;
use crate::name::StateKey;
use crate::node::Node;
use crate::role::Role;
use crate::snapshot::Snapshot;
use crate::value::{StateRef, Value};

/// one semantic difference between two snapshots.
///
/// `diff` lists nothing exactly when the two snapshots are equal, so every
/// field the revision sees has a variant: the spec's seven, plus role,
/// description, constraints, sibling order and focus.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Change {
    NodeAdded(SemId),
    NodeRemoved(SemId),
    /// a state value appeared, changed or went away. for `List` and `Set`
    /// (a missing value counts as empty) `added` and `removed` hold the
    /// items that came and went, as a multiset in value order; for scalars
    /// they are empty.
    StateChanged {
        at: StateRef,
        before: Option<Value>,
        after: Option<Value>,
        added: Vec<String>,
        removed: Vec<String>,
    },
    LabelChanged {
        node: SemId,
        before: String,
        after: String,
    },
    /// the action list differs in any way: a name, a field, the order.
    ActionsChanged {
        node: SemId,
        before: Vec<ActionDecl>,
        after: Vec<ActionDecl>,
    },
    /// each flag that changed, with its new value; `None` when unchanged.
    FlagsChanged {
        node: SemId,
        enabled: Option<bool>,
        inert: Option<bool>,
        focusable: Option<bool>,
    },
    ParentChanged {
        node: SemId,
        before: Option<SemId>,
        after: Option<SemId>,
    },
    RoleChanged {
        node: SemId,
        before: Role,
        after: Role,
    },
    DescriptionChanged {
        node: SemId,
        before: Option<String>,
        after: Option<String>,
    },
    ConstraintsChanged {
        node: SemId,
        before: Constraints,
        after: Constraints,
    },
    /// the children present in both snapshots (the roots, for `None`) are
    /// in another order.
    Reordered {
        parent: Option<SemId>,
    },
    FocusChanged {
        before: Option<SemId>,
        after: Option<SemId>,
    },
}

impl Snapshot {
    /// every semantic difference from `before` to `after`, in a fixed
    /// order: removed nodes (in `before` order), then per node of `after`
    /// either `NodeAdded` or its field changes (role, label, description,
    /// parent, state, actions, constraints, flags), then reorders, then
    /// focus. `source` is never compared.
    pub fn diff(before: &Snapshot, after: &Snapshot) -> Vec<Change> {
        let mut out = Vec::new();
        for id in before.nodes().keys() {
            if after.get(id).is_none() {
                out.push(Change::NodeRemoved(id.clone()));
            }
        }
        for (id, new) in after.nodes() {
            match before.get(id) {
                None => out.push(Change::NodeAdded(id.clone())),
                Some(old) => node_changes(old, new, &mut out),
            }
        }
        let roots = |s: &Snapshot| s.roots().map(|n| n.id.clone()).collect::<Vec<_>>();
        if reordered(&roots(before), &roots(after)) {
            out.push(Change::Reordered { parent: None });
        }
        for id in after.nodes().keys() {
            if before.get(id).is_some() {
                let kids = |s: &Snapshot| s.children(id).map(|n| n.id.clone()).collect::<Vec<_>>();
                if reordered(&kids(before), &kids(after)) {
                    out.push(Change::Reordered {
                        parent: Some(id.clone()),
                    });
                }
            }
        }
        if before.focused() != after.focused() {
            out.push(Change::FocusChanged {
                before: before.focused().cloned(),
                after: after.focused().cloned(),
            });
        }
        out
    }
}

fn node_changes(old: &Node, new: &Node, out: &mut Vec<Change>) {
    let node = &new.id;
    if old.role != new.role {
        out.push(Change::RoleChanged {
            node: node.clone(),
            before: old.role,
            after: new.role,
        });
    }
    if old.label != new.label {
        out.push(Change::LabelChanged {
            node: node.clone(),
            before: old.label.clone(),
            after: new.label.clone(),
        });
    }
    if old.description != new.description {
        out.push(Change::DescriptionChanged {
            node: node.clone(),
            before: old.description.clone(),
            after: new.description.clone(),
        });
    }
    if old.parent != new.parent {
        out.push(Change::ParentChanged {
            node: node.clone(),
            before: old.parent.clone(),
            after: new.parent.clone(),
        });
    }
    for (key, value) in &new.state {
        let prev = old.state.get(key);
        if prev != Some(value) {
            out.push(state_change(node, key, prev, Some(value)));
        }
    }
    for (key, value) in &old.state {
        if !new.state.contains_key(key) {
            out.push(state_change(node, key, Some(value), None));
        }
    }
    if old.actions != new.actions {
        out.push(Change::ActionsChanged {
            node: node.clone(),
            before: old.actions.clone(),
            after: new.actions.clone(),
        });
    }
    if old.constraints != new.constraints {
        out.push(Change::ConstraintsChanged {
            node: node.clone(),
            before: old.constraints.clone(),
            after: new.constraints.clone(),
        });
    }
    let flag = |a: bool, b: bool| (a != b).then_some(b);
    let enabled = flag(old.enabled, new.enabled);
    let inert = flag(old.inert, new.inert);
    let focusable = flag(old.focusable, new.focusable);
    if enabled.is_some() || inert.is_some() || focusable.is_some() {
        out.push(Change::FlagsChanged {
            node: node.clone(),
            enabled,
            inert,
            focusable,
        });
    }
}

/// the `StateChanged` for one piece of state, with `added` and `removed`
/// filled for collections. shared with the stale delta of `check`.
pub(crate) fn state_change(
    node: &SemId,
    key: &StateKey,
    before: Option<&Value>,
    after: Option<&Value>,
) -> Change {
    let (added, removed) = match (items(before), items(after)) {
        (Some(old), Some(new)) => (minus(&new, &old), minus(&old, &new)),
        _ => (Vec::new(), Vec::new()),
    };
    Change::StateChanged {
        at: StateRef::new(node.clone(), key.clone()),
        before: before.cloned(),
        after: after.cloned(),
        added,
        removed,
    }
}

/// the items of a collection value; a missing value is an empty
/// collection, a scalar has none.
fn items(value: Option<&Value>) -> Option<Vec<&String>> {
    match value {
        None => Some(Vec::new()),
        Some(Value::List(items)) => Some(items.iter().collect()),
        Some(Value::Set(items)) => Some(items.iter().collect()),
        Some(Value::Bool(_) | Value::Int(_) | Value::Num(_) | Value::Text(_)) => None,
    }
}

/// the items of `a` not matched in `b`, counting repeats, in `a` order.
fn minus(a: &[&String], b: &[&String]) -> Vec<String> {
    let mut pool: HashMap<&String, usize> = HashMap::with_capacity(b.len());
    for &item in b {
        *pool.entry(item).or_insert(0) += 1;
    }
    let mut out = Vec::new();
    for &item in a {
        match pool.get_mut(item) {
            Some(left) if *left > 0 => *left -= 1,
            _ => out.push(item.clone()),
        }
    }
    out
}

/// true when the ids present in both lists come in a different order.
fn reordered(before: &[SemId], after: &[SemId]) -> bool {
    let in_before: HashSet<&SemId> = before.iter().collect();
    let in_after: HashSet<&SemId> = after.iter().collect();
    let common_before = before.iter().filter(|id| in_after.contains(id));
    let common_after = after.iter().filter(|id| in_before.contains(id));
    !common_before.eq(common_after)
}
