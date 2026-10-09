//! the transformation proposer (task-70 phase 2): with a surface shown, an
//! intent becomes operations of `uir::transform` instead of a whole new
//! IR.
//!
//! a `Transformer` reads what a person or an agent asked for (`Adapt`:
//! the intent, the app's registry and snapshot, the surface as its
//! transformations left it, the policy it was verified under and who
//! asks) and answers operations, never trusted as they are, or `Undo`.
//! the way out is `transform_verified`, which applies every operation in
//! order over the surface, each one verified again, and returns the
//! operations only with the verified surface they make:
//!
//! ```text
//! intent -> Transformer -> [Op] -> Adapted::apply, one by one -> Adapted -> the host
//! ```
//!
//! the `Proposer` of task-69 is untouched: a transformer is its sibling,
//! and `Rules` and `Llm` are both.

use sem::{Actor, Snapshot};
use uir::{Adapted, Capabilities, Context, GenerationPolicy, Op, TransformError};

use crate::error::ProposeError;

/// what a transformer is asked: the intent, and the surface shown now
/// with everything it was made in.
#[derive(Clone, Copy, Debug)]
pub struct Adapt<'a> {
    /// what the person or the agent wants changed, in its own words.
    pub intent: &'a str,
    /// what the app declares: the only state and actions a surface binds.
    pub caps: &'a Capabilities,
    /// the app projected now: roles, parents, labels and values.
    pub app: &'a Snapshot,
    /// the surface as its transformations left it, with the nodes set
    /// aside.
    pub current: &'a Adapted,
    /// the policy the surface was verified under: every operation is
    /// verified under it again.
    pub policy: &'a GenerationPolicy,
    /// who asks: a promotion is only of an action the app lets them ask.
    pub by: &'a Actor,
}

impl<'a> Adapt<'a> {
    /// what an operation is checked against.
    pub fn context(&self) -> Context<'a> {
        Context {
            caps: self.caps,
            policy: self.policy,
        }
    }
}

/// what a transformer answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Adaptation {
    /// these operations, in this order.
    Ops(Vec<Op>),
    /// the last change asked, undone: the host knows what it was.
    Undo,
}

/// changes the surface shown for an intent. the operations are not
/// checked yet: the caller applies them (`transform_verified`).
pub trait Transformer {
    fn transform(&mut self, ask: &Adapt<'_>) -> Result<Adaptation, ProposeError>;
}

/// what came out of `transform_verified`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transformed {
    /// the operations, in order, and the verified surface they make over
    /// the one asked about (boxed: an undo holds nothing).
    Changed { ops: Vec<Op>, adapted: Box<Adapted> },
    /// the last change asked is to be undone.
    Undo,
}

/// `ops` applied in order over the surface of `ask`, each verified again:
/// the surface they make, or the first refusal and where it is.
pub(crate) fn apply_all(ops: &[Op], ask: &Adapt<'_>) -> Result<Adapted, (usize, TransformError)> {
    let mut now = ask.current.clone();
    for (at, op) in ops.iter().enumerate() {
        now = now
            .apply(op, ask.by, ask.context())
            .map(|(next, _)| next)
            .map_err(|e| (at, e))?;
    }
    Ok(now)
}

/// asks `transformer` and applies what it answers over `ask.current`,
/// under `ask.policy`, by `ask.by`: the operations with the verified
/// surface they make, `Undo`, or why there is neither (the transformer's
/// error, no operation at all, or the first operation refused, with why).
/// a surface the verifier refuses never comes out of here.
pub fn transform_verified<T: Transformer + ?Sized>(
    transformer: &mut T,
    ask: &Adapt<'_>,
) -> Result<Transformed, ProposeError> {
    let ops = match transformer.transform(ask)? {
        Adaptation::Undo => return Ok(Transformed::Undo),
        Adaptation::Ops(ops) => ops,
    };
    if ops.is_empty() {
        return Err(ProposeError::NoChange(
            "no operation was proposed: the surface stays as it is".to_owned(),
        ));
    }
    match apply_all(&ops, ask) {
        Ok(adapted) => Ok(Transformed::Changed {
            ops,
            adapted: Box::new(adapted),
        }),
        Err((at, error)) => Err(ProposeError::Unapplied {
            at: at + 1,
            op: ops.get(at).map(ToString::to_string).unwrap_or_default(),
            error,
        }),
    }
}
