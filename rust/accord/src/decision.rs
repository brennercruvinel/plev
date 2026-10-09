//! `check`, the decision on a request against the current state, and its
//! answer: `Verdict`, with a `Rejection` that says why.

use crate::decl::{ActionDecl, Actors, Confirm, Constraints};
use crate::diff::{Change, state_change};
use crate::error::SemErrors;
use crate::id::SemId;
use crate::name::ActionName;
use crate::receipt::RequestId;
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
    /// every precondition holds, and the policy asks a person to confirm
    /// before the action runs, or an agent asks one (`Request::ask`): at
    /// the screen or through assistive technology (see
    /// `Dispatcher::confirm`), or to decline it (`Dispatcher::decline`).
    NeedsConfirmation,
    Rejected(Rejection),
}

/// why a request was refused. `check` returns the first of these that
/// holds, in the order the variants are listed here (see `check`); the
/// last two come from the dispatcher.
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
    ///
    /// `acted` names who, among the app's actors, ran something since the
    /// actor looked (see `Acted`). `check` and a point of truth
    /// leave it empty (`Rejection::stale`); the dispatcher fills it for
    /// every stale refusal it records. it is not written when empty.
    Stale {
        changes: Vec<Change>,
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Vec::is_empty")
        )]
        acted: Vec<Acted>,
    },
    /// the target is inert (an open modal, a blocked region).
    Inert { target: SemId },
    /// the target or the action is disabled.
    Disabled { target: SemId, action: ActionName },
    /// the parameter does not fit the action.
    InvalidParam { reason: String },
    /// the app's projection did not validate, so there was no current
    /// state to check against. only the dispatcher answers this, before
    /// the order above; `check` itself always has a snapshot.
    InvalidProjection { errors: SemErrors },
    /// a person refused the confirmation the request waited for, at the
    /// screen (`Actor::Human`) or through assistive technology
    /// (`Actor::Assistive`), with `Dispatcher::decline`; nothing ran.
    /// only the dispatcher answers this, never `check`.
    Declined { by: Actor },
}

impl Rejection {
    /// a stale refusal with its delta and nobody named in `acted`: what
    /// `check` builds, and what a point of truth outside the dispatcher
    /// builds. the dispatcher names who acted when it records the refusal.
    pub fn stale(changes: Vec<Change>) -> Rejection {
        Rejection::Stale {
            changes,
            acted: Vec::new(),
        }
    }
}

/// an action that ran since the author of a refused action looked: its
/// request, who asked, the target and the action. no parameter and no
/// value, so nothing of the state travels here.
///
/// exactly, among the receipts the dispatcher still holds (open, or among
/// the finished ones it retains), where a receipt ran when it reached
/// execution (executing now, completed or failed; a refused one, a
/// declined one and one awaiting confirmation never ran), in id order:
///
/// - with the look (task-91): when a sink knows when the author looked at
///   what the refused request rests on (`ReceiptSink::looked`, the
///   dispatcher's `Mark` at that instant), named are the receipts that
///   ran and were taken after it, whoever asked, the author included.
///   the agent port knows it for the acts it dispatches: the oldest of
///   the observations that showed the author what the act reads. a change
///   no receipt made (a selection by click, a `git` in a terminal) moves
///   nothing here, so the receipts after it are named all the same.
/// - without a look (task-90), the rule inside the process: the start is
///   the lowest id of a receipt, by any actor, the author included, that
///   ran and whose `validated_against` is the revision the refused
///   request observed (`Observation::revision`). named are the receipts
///   that ran, with an id from the start on, whose actor is not the
///   refused request's. ids grow in dispatch order, and the start ran on
///   the state the author saw, so it and every later one ran after the
///   author looked (unless the state came back to exactly that revision
///   later). the author's own receipts are never named and do not cut the
///   chain. with no start, nobody is named: when a change no receipt made
///   (outside the app, or state an app moves outside the dispatcher: a
///   selection, typing, focus, a toast leaving on its timer, a worker's
///   answer landing) comes first after the look, the next receipt is
///   checked on another revision and the chain has nowhere to start.
///
/// either way it says who ran something since the author looked, not that
/// what they ran moved what the refused action reads; the delta in
/// `changes` says what moved. a receipt past the retention is not named
/// (without a look, a start past it takes the chain with it), and a
/// request taken before the look whose effect lands after it is not
/// named either.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Acted {
    pub id: RequestId,
    pub actor: Actor,
    pub target: SemId,
    pub action: ActionName,
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
///    an agent asks for the action, or an agent's request carries a
///    question for a person (`Request::ask`, task-92). a question never
///    passes an earlier step: a stale, inert or disabled request is
///    refused as it would be without it.
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
        return reject(Rejection::stale(changes));
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
    if needs_confirmation(decl.policy.confirm, &req.actor) || req.asks_a_person() {
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

/// whose requests wait for a confirmation: nobody's, an agent's only, or
/// every actor's, a person's included. who may confirm is the
/// dispatcher's rule: a person, never an agent or a test.
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
