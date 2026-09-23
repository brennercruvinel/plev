//! a secret never reaches the recorder. the snapshot already keeps only
//! the length of a `secret` field (docs/ADR/semantic-contract-as-pure-projection.md);
//! what travels outside the snapshot does not: a request's parameter goes
//! as given, and so can an observed value, a stale delta, an outcome or
//! an error text. everything aimed at a secret node is reduced here, with
//! the same rule the snapshot applies: `Text` becomes `Int(chars)`, `List`
//! and `Set` become `Int(items)`, `Bool`, `Int` and `Num` stay.
//!
//! which node is secret comes from the last snapshot the app gave the
//! recorder. a node that snapshot does not hold, or no snapshot at all,
//! counts as secret: the recorder fails closed.

use sem::{Change, Outcome, Rejection, Request, SemId, Snapshot, Stage, Value};

/// what replaces a free text (an error, a refusal reason) about a secret
/// target, which could quote the secret.
pub(crate) const REDACTED: &str = "redacted: the target is a secret field";

/// true when `id` is secret in `state`, or `state` does not know it.
pub(crate) fn secret(state: Option<&Snapshot>, id: &SemId) -> bool {
    state
        .and_then(|s| s.get(id))
        .is_none_or(|node| node.constraints.secret)
}

/// the value as a secret field keeps it: its length.
pub(crate) fn value(v: &Value) -> Value {
    match v {
        Value::Text(text) => Value::Int(count(text.chars().count())),
        Value::List(items) => Value::Int(count(items.len())),
        Value::Set(items) => Value::Int(count(items.len())),
        Value::Bool(_) | Value::Int(_) | Value::Num(_) => v.clone(),
    }
}

fn count(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// the request with its parameter and every observed value on a secret
/// node reduced to their length.
pub(crate) fn request(req: &Request, state: Option<&Snapshot>) -> Request {
    let mut out = req.clone();
    if secret(state, &req.target) {
        out.param = out.param.as_ref().map(value);
    }
    for (at, seen) in &mut out.observed.values {
        if secret(state, &at.node) {
            *seen = seen.as_ref().map(value);
        }
    }
    out
}

/// the stage with what it carries about secret state reduced: a stale
/// delta on a secret node keeps lengths and loses its items; the refusal
/// reason of a parameter, the outcome and the error of a secret target
/// are not kept.
pub(crate) fn stage(stage: &Stage, target: &SemId, state: Option<&Snapshot>) -> Stage {
    let hidden = secret(state, target);
    match stage {
        Stage::Rejected(Rejection::Stale { changes }) => Stage::Rejected(Rejection::Stale {
            changes: changes.iter().map(|c| change(c, state)).collect(),
        }),
        Stage::Rejected(Rejection::InvalidParam { .. }) if hidden => {
            Stage::Rejected(Rejection::InvalidParam {
                reason: REDACTED.to_owned(),
            })
        }
        Stage::Completed(outcome) if hidden => Stage::Completed(Outcome {
            result: outcome
                .result
                .iter()
                .map(|(k, v)| (k.clone(), value(v)))
                .collect(),
        }),
        Stage::Failed { .. } if hidden => Stage::Failed {
            error: REDACTED.to_owned(),
        },
        other => other.clone(),
    }
}

fn change(c: &Change, state: Option<&Snapshot>) -> Change {
    match c {
        Change::StateChanged {
            at, before, after, ..
        } if secret(state, &at.node) => Change::StateChanged {
            at: at.clone(),
            before: before.as_ref().map(value),
            after: after.as_ref().map(value),
            added: Vec::new(),
            removed: Vec::new(),
        },
        other => other.clone(),
    }
}
