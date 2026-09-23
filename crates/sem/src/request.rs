//! a request to act on a capability: who asks (`Actor`), what they ask
//! for, and the state they saw when they decided (`Observation`).

use std::fmt;

use crate::builder::resolve;
use crate::error::SemError;
use crate::hash::Revision;
use crate::id::SemId;
use crate::name::ActionName;
use crate::snapshot::Snapshot;
use crate::value::{StateRef, Value};

/// who asks for an action. one kind per column of `Actors`, which a
/// policy maps directly: `Human` to `human`, `Agent` to `agent`,
/// `Assistive` to `assistive`, `Test` to `test`.
///
/// no actor stands for a replay. a recording plays receipts back; it never
/// asks for anything, so a replay can never create a request.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Actor {
    Human,
    Agent { name: String },
    Assistive,
    Test { name: String },
}

/// `human`, `agent:<name>`, `assistive`, `test:<name>`.
impl fmt::Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Actor::Human => f.write_str("human"),
            Actor::Agent { name } => write!(f, "agent:{name}"),
            Actor::Assistive => f.write_str("assistive"),
            Actor::Test { name } => write!(f, "test:{name}"),
        }
    }
}

/// the state an actor saw when it decided to act.
///
/// `values` holds, for each state the action reads, the value the actor
/// saw (`None` when it saw none). these are the preconditions: the action
/// runs only if every one of them still holds. `revision` is the revision
/// of the whole snapshot the actor saw; it is kept to trace the request
/// and never decides anything, since it moves with state the action does
/// not read.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Observation {
    pub revision: Revision,
    pub values: Vec<(StateRef, Option<Value>)>,
}

/// an actor asks `target` to run `action`, with an optional parameter,
/// having seen `observed`. the serde form carries `param` only when there
/// is one.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Request {
    pub actor: Actor,
    pub target: SemId,
    pub action: ActionName,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub param: Option<Value>,
    pub observed: Observation,
}

impl Request {
    /// builds the request an actor makes after seeing `seen`: it reads the
    /// action's `reads` in `seen` and keeps their values as the
    /// preconditions. a click is the same thing: `seen` is the snapshot of
    /// the frame the person looked at and the actor is `Actor::Human`.
    ///
    /// target and action are taken typed or as text, like the builder.
    /// fails when a text does not parse, with `SemError::UnknownAction`
    /// when `seen` has no such node or the node does not declare the
    /// action, and with `SemError::DanglingRead` when a read has no value
    /// in `seen`.
    ///
    /// a node marked `secret` keeps only the length of its values in the
    /// snapshot, so a precondition on a secret field sees a change of
    /// length and nothing else. that is the price of never letting the
    /// secret, or a hash of it, leave the process.
    pub fn observe(
        seen: &Snapshot,
        actor: Actor,
        target: impl TryInto<SemId, Error: Into<SemError>>,
        action: impl TryInto<ActionName, Error: Into<SemError>>,
        param: Option<Value>,
    ) -> Result<Request, SemError> {
        let target = resolve(target)?;
        let action = resolve(action)?;
        let decl = seen
            .get(&target)
            .and_then(|node| node.actions.iter().find(|a| a.name == action));
        let Some(decl) = decl else {
            return Err(SemError::UnknownAction {
                node: target,
                action,
            });
        };
        let mut values = Vec::with_capacity(decl.reads.len());
        for read in &decl.reads {
            match seen.value(read) {
                Some(value) => values.push((read.clone(), Some(value.clone()))),
                None => {
                    return Err(SemError::DanglingRead {
                        node: target,
                        action,
                        read: read.clone(),
                    });
                }
            }
        }
        Ok(Request {
            actor,
            target,
            action,
            param,
            observed: Observation {
                revision: seen.revision(),
                values,
            },
        })
    }
}
