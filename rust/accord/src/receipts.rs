//! `Receipt`, the record of one request from the moment it is asked for to
//! its result, and the `Stage`s it walks through.

use std::fmt;
use std::str::FromStr;

use indexmap::IndexMap;

use crate::decision::Rejection;
use crate::encoding::text_serde;
use crate::failures::SemError;
use crate::property::Value;
use crate::requests::{Actor, Request};
use crate::revision::Revision;

/// a point in time in milliseconds, on a monotonic clock whose origin the
/// caller picks. `sem` never reads a clock: whoever dispatches passes the
/// time, so a test or a replay is deterministic. it travels as a JSON
/// number, exact up to 2^53 milliseconds (about 285 thousand years).
pub type Millis = u64;

/// the identity of one request in the dispatcher that took it. the
/// dispatcher numbers requests from 1, in the order they arrive.
///
/// text form: the number in decimal, without sign or leading zeros. it
/// travels in serde as that text, like `Revision`, because a JSON number
/// cannot hold every u64.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId(u64);

impl RequestId {
    pub(crate) fn new(n: u64) -> Self {
        RequestId(n)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// parses the text form.
    pub fn parse(text: &str) -> Result<RequestId, SemError> {
        match text.parse::<u64>() {
            Ok(n) if n.to_string() == text => Ok(RequestId(n)),
            _ => Err(SemError::InvalidId {
                text: text.to_owned(),
                reason: "a request id is a decimal number without sign or leading zeros".to_owned(),
            }),
        }
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RequestId {
    type Err = SemError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        RequestId::parse(text)
    }
}

text_serde!(RequestId);

/// a point in the order of the requests a dispatcher takes: how many it
/// had taken at that instant, the id of the last one (0 before the
/// first). ids grow in the order requests arrive, so a request was taken
/// after a mark exactly when its id is greater (`precedes`).
///
/// it marks when an actor looked: an agent port marks each observation it
/// serves with the dispatcher's mark at that instant (`Dispatcher::mark`),
/// and a stale refusal names what ran after the look its request rests on
/// (`Acted`). it only names, never decides, and never travels: it is not
/// in a request, a receipt or a message.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mark(u64);

impl Mark {
    pub(crate) fn new(taken: u64) -> Self {
        Mark(taken)
    }

    /// the mark right after `id` was taken.
    pub fn after(id: RequestId) -> Mark {
        Mark(id.0)
    }

    /// true when `id` was taken after this mark.
    pub fn precedes(self, id: RequestId) -> bool {
        id.0 > self.0
    }
}

/// what an action produced, by name: `commit`, `files`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Outcome {
    pub result: IndexMap<String, Value>,
}

/// one step in the life of a request.
///
/// ```text
/// Requested -> Accepted | Rejected | AwaitingConfirmation
/// AwaitingConfirmation -> Confirmed | Rejected   (declined, or refused by confirm's re-check)
/// Accepted | Confirmed -> Executing
/// Executing -> Completed | Failed | Rejected   (rejected at the point of truth)
/// terminal: Rejected, Completed, Failed
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Stage {
    Requested,
    Accepted,
    AwaitingConfirmation,
    Confirmed { by: Actor },
    Rejected(Rejection),
    Executing,
    Completed(Outcome),
    Failed { error: String },
}

impl Stage {
    /// the stable name, the same one serde uses.
    pub fn name(&self) -> &'static str {
        match self {
            Stage::Requested => "requested",
            Stage::Accepted => "accepted",
            Stage::AwaitingConfirmation => "awaiting_confirmation",
            Stage::Confirmed { .. } => "confirmed",
            Stage::Rejected(_) => "rejected",
            Stage::Executing => "executing",
            Stage::Completed(_) => "completed",
            Stage::Failed { .. } => "failed",
        }
    }

    /// true for a stage nothing follows: rejected, completed, failed.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Stage::Rejected(_) | Stage::Completed(_) | Stage::Failed { .. }
        )
    }

    /// true when `next` may follow this stage (the table above).
    pub fn can_advance_to(&self, next: &Stage) -> bool {
        use Stage::{
            Accepted, AwaitingConfirmation, Completed, Confirmed, Executing, Failed, Rejected,
            Requested,
        };
        matches!(
            (self, next),
            (Requested, Accepted | Rejected(_) | AwaitingConfirmation)
                | (AwaitingConfirmation, Confirmed { .. } | Rejected(_))
                | (Accepted | Confirmed { .. }, Executing)
                | (Executing, Completed(_) | Failed { .. } | Rejected(_))
        )
    }
}

/// the record of one request: what was asked, against which revision it
/// was validated, and every stage it went through with the time the
/// caller gave for it.
///
/// `validated_against` is the revision of the snapshot of the last
/// validation (at dispatch, and again at confirmation; a decline does not
/// validate); `None` until a validation ran, and when the app's projection
/// did not validate.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Receipt {
    pub id: RequestId,
    pub request: Request,
    pub validated_against: Option<Revision>,
    pub stages: Vec<(Stage, Millis)>,
}

impl Receipt {
    /// a receipt with no stage yet, not validated.
    pub(crate) fn new(id: RequestId, request: Request) -> Self {
        Receipt {
            id,
            request,
            validated_against: None,
            stages: Vec::new(),
        }
    }

    /// appends a stage the dispatcher already knows is legal: the first is
    /// `Requested`, each next one follows `can_advance_to`.
    pub(crate) fn push(&mut self, stage: Stage, at: Millis) {
        debug_assert!(
            match self.stage() {
                None => stage == Stage::Requested,
                Some(now) => now.can_advance_to(&stage),
            },
            "illegal transition to {}",
            stage.name()
        );
        self.stages.push((stage, at));
    }

    /// the current stage: the last one.
    pub fn stage(&self) -> Option<&Stage> {
        self.stages.last().map(|(stage, _)| stage)
    }

    /// true once the receipt reached a terminal stage.
    pub fn is_finished(&self) -> bool {
        self.stage().is_some_and(Stage::is_terminal)
    }
}
