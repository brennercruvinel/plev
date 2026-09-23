//! the checks `finish` runs over a whole projection, and the two
//! derivations it applies once the projection is valid: inherited `inert`
//! and the redaction of `secret` values.

use indexmap::IndexMap;

use crate::error::{SemError, SemErrors};
use crate::id::SemId;
use crate::node::Node;
use crate::snapshot::Snapshot;
use crate::value::Value;

/// turns the collected nodes into a snapshot, or reports every violation.
/// `errors` holds what the builder already rejected (ids, keys, refs that
/// did not parse); the structural checks append to it.
pub(crate) fn finish(
    nodes: Vec<Node>,
    focus: Option<SemId>,
    mut errors: Vec<SemError>,
) -> Result<Snapshot, SemErrors> {
    let mut map: IndexMap<SemId, Node> = IndexMap::with_capacity(nodes.len());
    for node in nodes {
        if map.contains_key(&node.id) {
            errors.push(SemError::DuplicateId(node.id));
        } else {
            map.insert(node.id.clone(), node);
        }
    }
    for node in map.values() {
        check_node(node, &map, &mut errors);
    }
    check_cycles(&map, &mut errors);
    if let Some(f) = &focus {
        if !map.contains_key(f) {
            errors.push(SemError::UnknownFocus(f.clone()));
        }
    }
    if !errors.is_empty() {
        return Err(SemErrors(errors));
    }
    inherit_inert(&mut map);
    redact_secrets(&mut map);
    Ok(Snapshot::from_parts(map, focus))
}

fn check_node(node: &Node, map: &IndexMap<SemId, Node>, errors: &mut Vec<SemError>) {
    if let Some(parent) = &node.parent {
        if !map.contains_key(parent) {
            errors.push(SemError::UnknownParent {
                node: node.id.clone(),
                parent: parent.clone(),
            });
        }
    }
    if node.role.is_interactive() && node.label.trim().is_empty() {
        errors.push(SemError::MissingLabel(node.id.clone()));
    }
    for (at, action) in node.actions.iter().enumerate() {
        if node.actions.iter().take(at).any(|a| a.name == action.name) {
            errors.push(SemError::DuplicateAction {
                node: node.id.clone(),
                action: action.name.clone(),
            });
        }
    }
    for reason in node.constraints.faults() {
        errors.push(SemError::InvalidConstraint {
            node: node.id.clone(),
            reason,
        });
    }
    for action in &node.actions {
        for read in &action.reads {
            let found = map
                .get(&read.node)
                .is_some_and(|target| target.state.contains_key(&read.key));
            if !found {
                errors.push(SemError::DanglingRead {
                    node: node.id.clone(),
                    action: action.name.clone(),
                    read: read.clone(),
                });
            }
        }
    }
}

fn parent_index(map: &IndexMap<SemId, Node>, at: usize) -> Option<usize> {
    let (_, node) = map.get_index(at)?;
    map.get_index_of(node.parent.as_ref()?)
}

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    New,
    Open,
    Done,
}

/// reports each parent cycle once, naming its first node in projection
/// order. a node that is its own parent is a cycle of one.
fn check_cycles(map: &IndexMap<SemId, Node>, errors: &mut Vec<SemError>) {
    let mut mark = vec![Mark::New; map.len()];
    for start in 0..map.len() {
        let mut path = Vec::new();
        let mut cur = Some(start);
        while let Some(at) = cur {
            match mark.get(at).copied() {
                Some(Mark::New) => {
                    if let Some(m) = mark.get_mut(at) {
                        *m = Mark::Open;
                    }
                    path.push(at);
                    cur = parent_index(map, at);
                }
                Some(Mark::Open) => {
                    let first = path
                        .iter()
                        .position(|&p| p == at)
                        .and_then(|pos| path.get(pos..))
                        .and_then(|cycle| cycle.iter().copied().min());
                    if let Some((id, _)) = first.and_then(|i| map.get_index(i)) {
                        errors.push(SemError::ParentCycle(id.clone()));
                    }
                    cur = None;
                }
                Some(Mark::Done) | None => cur = None,
            }
        }
        for at in path {
            if let Some(m) = mark.get_mut(at) {
                *m = Mark::Done;
            }
        }
    }
}

/// a node is inert when it or any ancestor declared itself inert. runs on
/// a validated projection (parents exist, no cycle); the walk is bounded
/// anyway.
fn inherit_inert(map: &mut IndexMap<SemId, Node>) {
    let len = map.len();
    let parents: Vec<Option<usize>> = (0..len).map(|at| parent_index(map, at)).collect();
    let own: Vec<bool> = map.values().map(|n| n.inert).collect();
    let mut effective: Vec<Option<bool>> = vec![None; len];
    for start in 0..len {
        let mut chain = Vec::new();
        let mut cur = Some(start);
        let mut inherited = false;
        while let Some(at) = cur {
            if let Some(known) = effective.get(at).copied().flatten() {
                inherited = known;
                break;
            }
            if chain.len() > len {
                break;
            }
            chain.push(at);
            cur = parents.get(at).copied().flatten();
        }
        for &at in chain.iter().rev() {
            inherited = inherited || own.get(at).copied().unwrap_or(false);
            if let Some(slot) = effective.get_mut(at) {
                *slot = Some(inherited);
            }
        }
    }
    for (node, value) in map.values_mut().zip(effective) {
        node.inert = value.unwrap_or(node.inert);
    }
}

/// a secret node keeps only the length of its text-like values: `Text`
/// becomes `Int(chars)`, `List` and `Set` become `Int(items)`. scalars
/// (`Bool`, `Int`, `Num`) describe the field, not the secret, and stay.
fn redact_secrets(map: &mut IndexMap<SemId, Node>) {
    for node in map.values_mut().filter(|n| n.constraints.secret) {
        for value in node.state.values_mut() {
            let len = match value {
                Value::Text(text) => text.chars().count(),
                Value::List(items) => items.len(),
                Value::Set(items) => items.len(),
                Value::Bool(_) | Value::Int(_) | Value::Num(_) => continue,
            };
            *value = Value::Int(i64::try_from(len).unwrap_or(i64::MAX));
        }
    }
}
