//! a secret never leaves the process through the port. a snapshot already
//! holds only the length of a `secret` field; a stage message also carries
//! the request as the dispatcher took it (its parameter and observed
//! values) and whatever the app answered (a stale delta, an outcome, an
//! error, a refusal reason). everything in it aimed at a secret node goes
//! out as the snapshot's rule has it: a `Text` as `Int(chars)`, a `List`
//! or a `Set` as `Int(items)`, a `Bool`, an `Int` or a `Num` as it is; a
//! free text about a secret target (an error, a refusal reason) does not go
//! out at all.
//!
//! which nodes are secret is read from the live snapshot when the request
//! is dispatched. a node that snapshot does not hold counts as secret: the
//! port fails closed.

use sem::{Change, Outcome, Rejection, Request, SemId, Snapshot, Stage, Value};

/// what replaces a free text about a secret target, which could quote it.
pub const REDACTED: &str = "redacted: the target is a secret field";

/// the nodes of one request whose values are secret: the target and every
/// node its observed values name, as the live snapshot marks them.
#[derive(Clone, Debug, Default)]
pub(crate) struct Secrets {
    nodes: Vec<SemId>,
}

impl Secrets {
    pub(crate) fn of(req: &Request, live: Option<&Snapshot>) -> Secrets {
        let hidden = |id: &SemId| {
            live.and_then(|s| s.get(id))
                .is_none_or(|node| node.constraints.secret)
        };
        let named =
            std::iter::once(&req.target).chain(req.observed.values.iter().map(|(at, _)| &at.node));
        let mut nodes: Vec<SemId> = named.filter(|id| hidden(id)).cloned().collect();
        nodes.sort();
        nodes.dedup();
        Secrets { nodes }
    }

    fn holds(&self, id: &SemId) -> bool {
        self.nodes.binary_search(id).is_ok()
    }

    /// the request with its parameter (when the target is secret) and every
    /// observed value of a secret node as their length.
    pub(crate) fn request(&self, req: &Request) -> Request {
        let mut out = req.clone();
        if self.holds(&req.target) {
            out.param = out.param.as_ref().map(length);
        }
        for (at, seen) in &mut out.observed.values {
            if self.holds(&at.node) {
                *seen = seen.as_ref().map(length);
            }
        }
        out
    }

    /// the stage with what it says about secret state reduced.
    pub(crate) fn stage(&self, target: &SemId, stage: &Stage) -> Stage {
        let secret = self.holds(target);
        match stage {
            Stage::Rejected(Rejection::Stale { changes }) => Stage::Rejected(Rejection::Stale {
                changes: changes.iter().map(|c| self.change(c)).collect(),
            }),
            Stage::Rejected(Rejection::InvalidParam { .. }) if secret => {
                Stage::Rejected(Rejection::InvalidParam {
                    reason: REDACTED.to_owned(),
                })
            }
            Stage::Completed(outcome) if secret => Stage::Completed(Outcome {
                result: outcome
                    .result
                    .iter()
                    .map(|(name, v)| (name.clone(), length(v)))
                    .collect(),
            }),
            Stage::Failed { .. } if secret => Stage::Failed {
                error: REDACTED.to_owned(),
            },
            other => other.clone(),
        }
    }

    fn change(&self, c: &Change) -> Change {
        match c {
            Change::StateChanged {
                at, before, after, ..
            } if self.holds(&at.node) => Change::StateChanged {
                at: at.clone(),
                before: before.as_ref().map(length),
                after: after.as_ref().map(length),
                added: Vec::new(),
                removed: Vec::new(),
            },
            other => other.clone(),
        }
    }
}

/// a secret value as the snapshot keeps it: its length.
pub(crate) fn length(v: &Value) -> Value {
    let n = match v {
        Value::Text(text) => text.chars().count(),
        Value::List(items) => items.len(),
        Value::Set(items) => items.len(),
        Value::Bool(_) | Value::Int(_) | Value::Num(_) => return v.clone(),
    };
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}
