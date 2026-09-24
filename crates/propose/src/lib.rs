//! proposers of plev IR: an intent becomes a surface only through the
//! verifier.
//!
//! a `Proposer` reads what a person or an agent asked for (`Ask`: the
//! intent in text, the app's capability registry, its snapshot, the
//! surface it shows now and the policy the surface will be verified
//! under) and answers an `Ir`, never trusted as it is. the way out is
//! `propose_verified`, which verifies the proposal against the same
//! registry and policy and returns a `uir::VerifiedIr` or the reasons, so
//! nothing unverified leaves this crate:
//!
//! ```text
//! intent -> Proposer -> Ir -> uir::verify -> VerifiedIr -> the runtime
//! ```
//!
//! two proposers:
//!
//! - `Rules`, deterministic, with no model: a small grammar of intents
//!   (add, filter, counts, items, compact, two columns, everything, in
//!   english and portuguese) turned into bindings by rules over the
//!   registry's structure (roles, value kinds, parameters, parents, a
//!   list whose values key its node's children), never over an app's
//!   names; the other words of an intent choose among the app's own
//!   words. it keeps the id of every node whose capability stays on the
//!   surface, a repeated item's template included, so a change of form
//!   reconciles by `SemId`.
//! - `Llm`, a language model behind the `Model` trait: the prompt carries
//!   the IR's schema, the components, the verifier's rules, the registry
//!   and the current surface with the ids to keep; the reply is decoded
//!   and verified, and the errors go back to the model, `Llm::ATTEMPTS`
//!   times at most. no provider lives here (phase 3b).
//!
//! pure, like `uir`: no engine, no widgets, no network.

mod error;
mod ids;
mod list;
mod llm;
mod parts;
mod prompt;
mod rules;
mod words;

pub use error::ProposeError;
pub use llm::{Llm, Model, ModelError};
pub use prompt::prompt;
pub use rules::Rules;
pub use words::{GRAMMAR, Part};

use sem::Snapshot;
use uir::{Capabilities, GenerationPolicy, Ir, VerifiedIr, verify};

/// what a proposer is asked: the intent and everything it may build from.
#[derive(Clone, Copy, Debug)]
pub struct Ask<'a> {
    /// what the person or the agent wants, in its own words.
    pub intent: &'a str,
    /// what the app declares: the only state and actions a surface binds.
    pub caps: &'a Capabilities,
    /// the app projected now: roles, parents, labels and values, which
    /// the registry does not carry.
    pub app: &'a Snapshot,
    /// the surface the app shows now, when it shows one: the proposal
    /// changes it in place when it keeps its id, and keeps the ids of the
    /// nodes whose capabilities stay.
    pub current: Option<&'a VerifiedIr>,
    /// the policy the proposal is verified under (its author, its
    /// limits).
    pub policy: &'a GenerationPolicy,
}

/// proposes a surface for an intent. the answer is an `Ir` nobody has
/// checked yet: the caller verifies it (`propose_verified`).
pub trait Proposer {
    fn propose(&mut self, ask: &Ask<'_>) -> Result<Ir, ProposeError>;
}

/// asks `proposer` and verifies what it proposes against `ask.caps` under
/// `ask.policy`: a `VerifiedIr`, or why there is none (the proposer's
/// error, or every error of the verifier). an IR the verifier refuses
/// never comes out of here.
pub fn propose_verified<P: Proposer + ?Sized>(
    proposer: &mut P,
    ask: &Ask<'_>,
) -> Result<VerifiedIr, ProposeError> {
    let ir = proposer.propose(ask)?;
    verify(&ir, ask.caps, ask.policy).map_err(ProposeError::Refused)
}
