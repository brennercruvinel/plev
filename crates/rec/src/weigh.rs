//! the byte estimate the ring's budget counts: the inline size of each
//! value plus the text and the collections it owns, from their lengths.
//! allocator overhead and spare capacity are not counted. a shared text
//! (`SemId`, `ActionName` and `StateKey` are `Arc<str>`) is counted at
//! every holder, so the estimate bounds what the ring holds from above.

use std::mem::size_of;

use sem::{
    ActionDecl, Actor, Change, Constraints, Node, Outcome, Rejection, Request, SemErrors, SemId,
    Snapshot, Stage, StateRef, Value,
};

use crate::event::{Event, Kind};

/// one event, its payload included, except the snapshot of a
/// `Kind::Snapshot` (see `snapshot`), which the track charges.
pub(crate) fn event(e: &Event) -> usize {
    size_of::<Event>()
        + match &e.kind {
            Kind::Observed {
                actor: a, scope, ..
            } => actor(a) + scope.as_ref().map_or(0, id),
            Kind::Snapshot { changes, .. } => list(changes, change),
            Kind::Changes(changes) => list(changes, change),
            Kind::Stage {
                request: r,
                stage: s,
                ..
            } => request(r) + stage(s),
            Kind::Input { target, .. } => target.as_ref().map_or(0, id),
        }
}

/// a whole snapshot: its nodes, the index over them and the tree.
pub(crate) fn snapshot(s: &Snapshot) -> usize {
    let per_node = size_of::<SemId>() + 4 * size_of::<usize>();
    size_of::<Snapshot>()
        + s.nodes().values().map(node).sum::<usize>()
        + s.len() * per_node
        + s.focused().map_or(0, id)
}

pub(crate) fn node(n: &Node) -> usize {
    size_of::<Node>()
        + id(&n.id)
        + n.label.len()
        + n.description.as_ref().map_or(0, String::len)
        + n.parent.as_ref().map_or(0, id)
        + n.state
            .iter()
            .map(|(k, v)| size_of::<(SemId, Value)>() + k.as_str().len() + value(v))
            .sum::<usize>()
        + list(&n.actions, decl)
        + constraints(&n.constraints)
        + n.source.as_ref().map_or(0, |s| s.file.len())
}

/// the heap a value owns (its inline size is counted by its holder).
pub(crate) fn value(v: &Value) -> usize {
    match v {
        Value::Bool(_) | Value::Int(_) | Value::Num(_) => 0,
        Value::Text(text) => text.len(),
        Value::List(items) => items.iter().map(|i| size_of::<String>() + i.len()).sum(),
        Value::Set(items) => items
            .iter()
            .map(|i| size_of::<String>() + 2 * size_of::<usize>() + i.len())
            .sum(),
    }
}

/// the heap an id owns.
pub(crate) fn id(id: &SemId) -> usize {
    2 * size_of::<usize>() + id.as_str().len()
}

fn at(r: &StateRef) -> usize {
    id(&r.node) + 2 * size_of::<usize>() + r.key.as_str().len()
}

fn decl(a: &ActionDecl) -> usize {
    size_of::<ActionDecl>()
        + a.name.as_str().len()
        + a.label.as_ref().map_or(0, String::len)
        + a.reads
            .iter()
            .map(|r| size_of::<StateRef>() + at(r))
            .sum::<usize>()
}

fn constraints(c: &Constraints) -> usize {
    c.options.as_ref().map_or(0, |options| {
        options.iter().map(|o| size_of::<String>() + o.len()).sum()
    })
}

fn actor(a: &Actor) -> usize {
    match a {
        Actor::Human | Actor::Assistive => 0,
        Actor::Agent { name } | Actor::Test { name } => name.len(),
    }
}

pub(crate) fn request(r: &Request) -> usize {
    size_of::<Request>()
        + actor(&r.actor)
        + id(&r.target)
        + r.action.as_str().len()
        + r.param.as_ref().map_or(0, value)
        + r.observed
            .values
            .iter()
            .map(|(a, v)| {
                size_of::<(StateRef, Option<Value>)>() + at(a) + v.as_ref().map_or(0, value)
            })
            .sum::<usize>()
}

fn stage(s: &Stage) -> usize {
    match s {
        Stage::Requested | Stage::Accepted | Stage::AwaitingConfirmation | Stage::Executing => 0,
        Stage::Confirmed { by } => actor(by),
        Stage::Rejected(why) => rejection(why),
        Stage::Completed(outcome) => self::outcome(outcome),
        Stage::Failed { error } => error.len(),
    }
}

fn outcome(o: &Outcome) -> usize {
    o.result
        .iter()
        .map(|(k, v)| size_of::<(String, Value)>() + k.len() + value(v))
        .sum()
}

fn rejection(r: &Rejection) -> usize {
    match r {
        Rejection::NotFound { target } | Rejection::Inert { target } => id(target),
        Rejection::UnknownAction { target, action } | Rejection::Disabled { target, action } => {
            id(target) + action.as_str().len()
        }
        Rejection::NotAuthorized { actor: a } => actor(a),
        Rejection::Stale { changes } => list(changes, change),
        Rejection::InvalidParam { reason } => reason.len(),
        Rejection::InvalidProjection { errors } => errors_weight(errors),
    }
}

fn errors_weight(errors: &SemErrors) -> usize {
    errors
        .0
        .iter()
        .map(|e| size_of_val(e) + e.to_string().len())
        .sum()
}

pub(crate) fn change(c: &Change) -> usize {
    size_of::<Change>()
        + match c {
            Change::NodeAdded(n) | Change::NodeRemoved(n) => id(n),
            Change::StateChanged {
                at: a,
                before,
                after,
                added,
                removed,
            } => {
                at(a)
                    + before.as_ref().map_or(0, value)
                    + after.as_ref().map_or(0, value)
                    + added
                        .iter()
                        .chain(removed)
                        .map(|i| size_of::<String>() + i.len())
                        .sum::<usize>()
            }
            Change::LabelChanged {
                node,
                before,
                after,
                ..
            } => id(node) + before.len() + after.len(),
            Change::ActionsChanged {
                node,
                before,
                after,
                ..
            } => id(node) + list(before, decl) + list(after, decl),
            Change::FlagsChanged { node, .. } | Change::RoleChanged { node, .. } => id(node),
            Change::ParentChanged {
                node,
                before,
                after,
                ..
            } => id(node) + before.as_ref().map_or(0, id) + after.as_ref().map_or(0, id),
            Change::DescriptionChanged {
                node,
                before,
                after,
                ..
            } => {
                id(node)
                    + before.as_ref().map_or(0, String::len)
                    + after.as_ref().map_or(0, String::len)
            }
            Change::ConstraintsChanged {
                node,
                before,
                after,
                ..
            } => id(node) + constraints(before) + constraints(after),
            Change::Reordered { parent } => parent.as_ref().map_or(0, id),
            Change::FocusChanged { before, after } => {
                before.as_ref().map_or(0, id) + after.as_ref().map_or(0, id)
            }
        }
}

fn list<T>(items: &[T], each: fn(&T) -> usize) -> usize {
    items.iter().map(each).sum()
}
