//! the error model: `SemError` for one violation, returned by the parse
//! functions and for a wrong use of the request api (an unknown request,
//! an illegal transition); `SemErrors` for every violation of a projection
//! at once. a refused action is not an error: it is a `Verdict`.

use std::convert::Infallible;

use crate::id::SemId;
use crate::name::ActionName;
use crate::receipt::RequestId;
use crate::request::Actor;
use crate::value::StateRef;

/// one violation of the contract.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SemError {
    /// a text that is not the canonical form of a `SemId`, or a state ref
    /// without its `:`.
    #[error("invalid id {text:?}: {reason}")]
    InvalidId { text: String, reason: String },
    /// a text that is not a name `[a-z0-9_]+` where a state key or an
    /// action name is expected, or a state key declared twice on a node.
    #[error("invalid key {text:?}: {reason}")]
    InvalidKey { text: String, reason: String },
    /// two nodes with the same id in one projection.
    #[error("id {0} is projected more than once")]
    DuplicateId(SemId),
    /// a parent that is not in the projection.
    #[error("{node} names parent {parent}, which is not projected")]
    UnknownParent { node: SemId, parent: SemId },
    /// a parent chain that loops, named by its first node in projection
    /// order.
    #[error("the parent chain of {0} loops back on itself")]
    ParentCycle(SemId),
    /// an action that reads a node or a key the projection does not have.
    #[error("action {action} of {node} reads {read}, which is not projected")]
    DanglingRead {
        node: SemId,
        action: ActionName,
        read: StateRef,
    },
    /// an interactive role with an empty or blank label.
    #[error("{0} is interactive and has no label")]
    MissingLabel(SemId),
    /// the same action name twice on one node.
    #[error("{node} declares action {action} more than once")]
    DuplicateAction { node: SemId, action: ActionName },
    /// constraints that cannot hold (range, step, options).
    #[error("{node} has an invalid constraint: {reason}")]
    InvalidConstraint { node: SemId, reason: String },
    /// focus on a node that is not in the projection.
    #[error("focus names {0}, which is not projected")]
    UnknownFocus(SemId),
    /// a request for an action the observed snapshot does not offer: the
    /// node is not there, or it does not declare the action.
    #[error("{node} does not offer action {action} in the observed snapshot")]
    UnknownAction { node: SemId, action: ActionName },
    /// a request id the dispatcher does not hold: never issued, or finished
    /// and dropped by retention.
    #[error("request {0} is not held by the dispatcher")]
    UnknownRequest(RequestId),
    /// a stage that cannot follow the receipt's current one, named by
    /// their stable names (`Stage::name`).
    #[error("request {id} cannot go from {from} to {to}")]
    IllegalTransition {
        id: RequestId,
        from: String,
        to: String,
    },
    /// a confirmation or a decline by a machine, an agent or a test: only
    /// a person answers a request that waits for one, at the screen or
    /// through assistive technology.
    #[error("request {id} waits for a person; {by} cannot confirm or decline it")]
    ConfirmByMachine { id: RequestId, by: Actor },
}

/// the builder accepts typed values too, whose conversion cannot fail.
impl From<Infallible> for SemError {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

/// every violation of one projection, in the order they were found:
/// parse errors in call order, then the checks per node in projection
/// order, then cycles, then focus.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[error("{}", list(.0))]
pub struct SemErrors(pub Vec<SemError>);

fn list(errors: &[SemError]) -> String {
    let mut out = format!("{} semantic violation(s)", errors.len());
    for e in errors {
        out.push_str("; ");
        out.push_str(&e.to_string());
    }
    out
}
