//! `check`, the decision on a request against the current state, and its
//! answer: `Verdict`, with a `Rejection` that says why.

use crate::decl::{ActionDecl, Actors, Confirm, Constraints};
use crate::diff::{Change, state_change};
use crate::id::SemId;
use crate::name::ActionName;
use crate::request::{Actor, Request};
use crate::snapshot::Snapshot;
use crate::value::{StateRef, Value};

/// the answer to a request.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Verdict {
    /// every precondition holds: the action may run.
    Accepted,
    /// every precondition holds, and the policy asks a human to confirm
    /// before the action runs.
    NeedsConfirmation,
    Rejected(Rejection),
}

/// why a request was refused. `check` returns the first of these that
/// holds, in the order the variants are listed here.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Rejection {
    /// the target is not in the current snapshot.
    NotFound { target: SemId },
    /// the target does not declare the action.
    UnknownAction { target: SemId, action: ActionName },
    /// the action's policy does not let this actor ask.
    NotAuthorized { actor: Actor },
    /// state the action reads is no longer what the actor saw. one
    /// `Change::StateChanged` per read that differs, in the order of the
    /// action's `reads`: `before` is the observed value, `after` the
    /// current one, and `added` and `removed` name the items that came and
    /// went, as `Snapshot::diff` computes them.
    Stale { changes: Vec<Change> },
    /// the target is inert (an open modal, a blocked region).
    Inert { target: SemId },
    /// the target or the action is disabled.
    Disabled { target: SemId, action: ActionName },
    /// the parameter does not fit the action.
    InvalidParam { reason: String },
}

/// decides `req` against `now`, the current state. pure: the same request
/// and snapshot give the same verdict.
///
/// the order is part of the contract, the first that holds wins:
///
/// 1. `NotFound`: the target is not in `now`.
/// 2. `UnknownAction`: the target does not declare the action.
/// 3. `NotAuthorized`: the policy's `Actors` leave this actor out.
/// 4. `Stale`: a value the action reads differs from what the actor
///    observed. it comes before inert and disabled because the delta
///    explains why the rest changed.
/// 5. `Inert`: the target is inert.
/// 6. `Disabled`: the target or the action is disabled.
/// 7. `InvalidParam`: the parameter does not fit (see below).
/// 8. `NeedsConfirmation`: `Confirm::Always`, or `Confirm::ForAgents` and
///    an agent asks.
/// 9. `Accepted`.
///
/// only the action's `reads`, as `now` declares them, are compared. state
/// the action does not read never makes a request stale, however much it
/// moved: that is semantic invalidation. the observed revision is never
/// compared, since it moves with every projected value. a read that the
/// observation does not hold counts as observed without a value, so an
/// action that starts to read something the actor never saw is stale.
/// a `secret` value is its length in the snapshot, so on a secret field
/// only a change of length is seen.
///
/// the parameter: an action without a `ParamSpec` takes none; an action
/// with one needs a value of that kind. when the spec is `constrained`,
/// the node's `Constraints` apply in this order: `editable` must be on,
/// `required` refuses an empty value (blank text, empty list or set),
/// `range` bounds an `Int` or a `Num` (inclusive; `NaN` is outside),
/// `max_len` bounds the characters of a text or the items of a list or
/// set, and `options` holds the accepted text or items. `step` is not
/// checked.
pub fn check(req: &Request, now: &Snapshot) -> Verdict {
    let Some(node) = now.get(&req.target) else {
        return reject(Rejection::NotFound {
            target: req.target.clone(),
        });
    };
    let Some(decl) = node.actions.iter().find(|a| a.name == req.action) else {
        return reject(Rejection::UnknownAction {
            target: req.target.clone(),
            action: req.action.clone(),
        });
    };
    if !allows(&decl.policy.actors, &req.actor) {
        return reject(Rejection::NotAuthorized {
            actor: req.actor.clone(),
        });
    }
    let changes = stale(req, decl, now);
    if !changes.is_empty() {
        return reject(Rejection::Stale { changes });
    }
    if node.inert {
        return reject(Rejection::Inert {
            target: req.target.clone(),
        });
    }
    if !node.enabled || !decl.enabled {
        return reject(Rejection::Disabled {
            target: req.target.clone(),
            action: req.action.clone(),
        });
    }
    if let Err(reason) = param(decl, &node.constraints, req.param.as_ref()) {
        return reject(Rejection::InvalidParam { reason });
    }
    if needs_confirmation(decl.policy.confirm, &req.actor) {
        return Verdict::NeedsConfirmation;
    }
    Verdict::Accepted
}

fn reject(why: Rejection) -> Verdict {
    Verdict::Rejected(why)
}

/// `Human` reads `human`, `Agent` reads `agent`, `Assistive` reads
/// `assistive`, `Test` reads `test`.
fn allows(actors: &Actors, actor: &Actor) -> bool {
    match actor {
        Actor::Human => actors.human,
        Actor::Agent { .. } => actors.agent,
        Actor::Assistive => actors.assistive,
        Actor::Test { .. } => actors.test,
    }
}

fn needs_confirmation(confirm: Confirm, actor: &Actor) -> bool {
    match confirm {
        Confirm::Never => false,
        Confirm::ForAgents => matches!(actor, Actor::Agent { .. }),
        Confirm::Always => true,
    }
}

/// one `StateChanged` per read (counted once) whose observed value differs
/// from the current one.
fn stale(req: &Request, decl: &ActionDecl, now: &Snapshot) -> Vec<Change> {
    let mut out = Vec::new();
    for (at, read) in decl.reads.iter().enumerate() {
        if decl.reads.iter().take(at).any(|r| r == read) {
            continue;
        }
        let seen = observed(req, read);
        let current = now.value(read);
        if seen != current {
            out.push(state_change(&read.node, &read.key, seen, current));
        }
    }
    out
}

/// the value the actor saw at `at`; the first entry wins, and a ref the
/// observation does not hold was seen without a value.
fn observed<'a>(req: &'a Request, at: &StateRef) -> Option<&'a Value> {
    req.observed
        .values
        .iter()
        .find(|(r, _)| r == at)
        .and_then(|(_, v)| v.as_ref())
}

fn param(decl: &ActionDecl, bounds: &Constraints, given: Option<&Value>) -> Result<(), String> {
    let name = &decl.name;
    let (spec, value) = match (decl.param, given) {
        (None, None) => return Ok(()),
        (None, Some(_)) => return Err(format!("action {name} takes no parameter")),
        (Some(spec), None) => {
            return Err(format!(
                "action {name} takes a {} parameter and none was given",
                spec.kind.as_str()
            ));
        }
        (Some(spec), Some(value)) => (spec, value),
    };
    if value.kind() != spec.kind {
        return Err(format!(
            "action {name} takes a {} parameter, got {}",
            spec.kind.as_str(),
            value.kind().as_str()
        ));
    }
    if spec.constrained {
        bounded(bounds, value)
    } else {
        Ok(())
    }
}

fn bounded(c: &Constraints, value: &Value) -> Result<(), String> {
    if !c.editable {
        return Err("the target is not editable".to_owned());
    }
    let size = match value {
        Value::Text(text) => Some(text.chars().count()),
        Value::List(items) => Some(items.len()),
        Value::Set(items) => Some(items.len()),
        Value::Bool(_) | Value::Int(_) | Value::Num(_) => None,
    };
    let blank = match value {
        Value::Text(text) => text.trim().is_empty(),
        _ => size == Some(0),
    };
    if c.required && blank {
        return Err("a value is required and the parameter is empty".to_owned());
    }
    if let Some((low, high)) = c.range {
        let number = match value {
            // the bounds are f64; an i64 beyond 2^53 rounds to the nearest.
            Value::Int(v) => Some(*v as f64),
            Value::Num(v) => Some(*v),
            _ => None,
        };
        if let Some(v) = number.filter(|v| !(low..=high).contains(v)) {
            return Err(format!("{v} is outside the range [{low}, {high}]"));
        }
    }
    if let (Some(max), Some(len)) = (c.max_len, size) {
        if len > max {
            return Err(format!("length {len} is above the maximum {max}"));
        }
    }
    if let Some(options) = &c.options {
        let choices: Vec<&String> = match value {
            Value::Text(text) => vec![text],
            Value::List(items) => items.iter().collect(),
            Value::Set(items) => items.iter().collect(),
            Value::Bool(_) | Value::Int(_) | Value::Num(_) => Vec::new(),
        };
        if let Some(bad) = choices.into_iter().find(|v| !options.contains(v)) {
            return Err(format!("{bad:?} is not one of the options"));
        }
    }
    Ok(())
}
