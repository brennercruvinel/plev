//! the messages of protocol v1: what a client asks (`Ask`), what the app
//! answers (`Reply`), and the typed protocol errors (`ProtoError`).
//!
//! every message is one JSON object on one line with `"v": 1`, an `"id"`
//! and a `"type"`. the client picks the id (any string); every reply to a
//! message carries the id of that message, so a client matches replies by
//! id. an error about a line whose id could not be read has a null id.
//!
//! snapshots, changes, requests, observations and stages travel in the
//! serde form `sem` pins (crates/sem/tests/wire.rs): ids and refs as their
//! canonical text, a `Revision` as 16 hex digits, a `RequestId` as decimal
//! text, a `Value` as `{"kind": .., "value": ..}`.

use sem::{
    ActionName, Actor, Change, Millis, Node, Observation, Request, RequestId, Revision, SemId,
    Stage, StateKey, Value,
};
use serde::{Deserialize, Serialize};

/// the protocol version every message carries as `"v"`.
pub const VERSION: u64 = 1;

/// one message from a client: its id and what it asks.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientMsg {
    pub id: String,
    pub ask: Ask,
}

/// what a client asks.
///
/// `hello` names the actor of the connection (`Actor::Agent { name }`);
/// it comes first, and an `act` before it is refused. `confirm` and
/// `decline` exist so the protocol is complete, and are always refused on
/// a socket: only a person answers a confirmation, at the screen or
/// through assistive technology, and an agent never confirms its own
/// action.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Ask {
    Hello { name: String },
    Observe(Observe),
    Act(Act),
    Confirm { request: RequestId },
    Decline { request: RequestId },
}

/// an observation. with no field it is the whole snapshot of what the app
/// projects now.
///
/// `scope` keeps one node and its descendants. `keys` keeps only those
/// state keys (a node keeps its other fields; a change of another key is
/// left out of a delta). `since` asks for the changes from a revision the
/// app served before: the reply is a `delta` when the app still holds that
/// revision, a whole `snapshot` otherwise. `bounds` adds where each bound
/// node sits in the frame on screen; it is off by default, since an agent
/// acts by id and never by position. `source` keeps each node's `source`
/// (the file, line and column that projected it) in a `snapshot` reply;
/// it is off by default too: an agent acts by id and the code that
/// projected a control is not part of its meaning. it is for whoever goes
/// from a replay or an observation back to the code (task-65), which is
/// why the recorder always keeps it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Observe {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<SemId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<StateKey>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<Revision>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub bounds: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub source: bool,
}

/// a request to run `action` on `target`, resting on what the agent saw:
/// exactly one of `observed` (the values of the action's reads, given
/// explicitly) or `from_revision` (a revision the app served, from which
/// the app reads those values itself).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Act {
    pub target: SemId,
    pub action: ActionName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<Observation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_revision: Option<Revision>,
}

impl Act {
    /// an act resting on a revision the app served.
    pub fn from_revision(target: SemId, action: ActionName, revision: Revision) -> Self {
        Act {
            target,
            action,
            param: None,
            observed: None,
            from_revision: Some(revision),
        }
    }

    /// an act resting on values the agent gives.
    pub fn observed(target: SemId, action: ActionName, observed: Observation) -> Self {
        Act {
            target,
            action,
            param: None,
            observed: Some(observed),
            from_revision: None,
        }
    }

    /// the same act with a parameter.
    pub fn with_param(mut self, param: Value) -> Self {
        self.param = Some(param);
        self
    }
}

/// one message from the app: the id of the message it answers (`None`
/// when that line had no readable id) and the reply.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerMsg {
    pub id: Option<String>,
    pub reply: Reply,
}

/// what the app answers.
///
/// `hello` gets `welcome`; `observe` gets one `snapshot` or one `delta`;
/// `act` gets one `stage` per stage of its receipt, in order, until a
/// terminal one (rejected, completed, failed), some of them later than
/// others when the execution ends after the dispatch; anything refused
/// gets one `error`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Reply {
    Welcome { app: String, actor: Actor },
    Snapshot(SnapshotReply),
    Delta(DeltaReply),
    Stage(StageReply),
    Error(ProtoError),
}

/// the nodes an observation sees, in projection order, with the focus
/// when it is among them.
///
/// `revision` is always the revision of the app's whole snapshot at that
/// instant, also for a scoped observation: it is the value `since` and
/// `from_revision` take, and the app keeps the whole snapshot under it, so
/// an action whose reads lie outside the scope still rests on what was
/// served.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnapshotReply {
    pub revision: Revision,
    pub nodes: Vec<Node>,
    pub focused: Option<SemId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Vec<Bound>>,
}

/// what changed in the observed part from `base` to `revision`, as
/// `Snapshot::diff` lists it. empty when nothing the observation sees
/// moved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeltaReply {
    pub base: Revision,
    pub revision: Revision,
    pub changes: Vec<Change>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Vec<Bound>>,
}

/// one stage of the receipt of `request`, at the time the app's
/// dispatcher gave it. the first one, `requested`, also carries the
/// request as the dispatcher took it (`asked`), with the observed values
/// the app rebuilt from `from_revision`. a value aimed at a secret node
/// travels as its length.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StageReply {
    pub request: RequestId,
    pub stage: Stage,
    pub at: Millis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked: Option<Request>,
}

impl StageReply {
    /// true for a stage nothing follows.
    pub fn is_terminal(&self) -> bool {
        self.stage.is_terminal()
    }
}

/// where a bound node sits in the frame on screen: `(x, y, w, h)` in
/// logical window pixels, the part inside its clip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bound {
    pub id: SemId,
    pub rect: [f32; 4],
}

/// a refusal of the protocol, with the reason. a refused action is not
/// one of these: it is a `stage` with `rejected`, from the dispatcher.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ProtoError {
    /// `"v"` is missing or is not 1. `got` is its JSON text, or `missing`.
    #[error("protocol version {got} is not spoken here; this app speaks v1")]
    BadVersion { got: String },
    /// the line is not JSON, not an object, has no string id, names an
    /// unknown type, or a field does not parse (an id, a key, a value); or
    /// the message makes no sense (an act with both or neither of
    /// `observed` and `from_revision`, an action the served revision does
    /// not offer).
    #[error("bad message: {reason}")]
    BadMessage { reason: String },
    /// `from_revision` names a revision the app no longer holds (or never
    /// served): observe again.
    #[error("revision {revision} is not held; observe again")]
    UnknownRevision { revision: Revision },
    /// the app does not take this here and now: a confirmation or a
    /// decline by an agent, an act before `hello`, nothing projected or no
    /// dispatcher right now.
    #[error("not enabled: {reason}")]
    NotEnabled { reason: String },
}

fn is_false(b: &bool) -> bool {
    !*b
}
