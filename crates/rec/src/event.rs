//! what the recorder keeps: `Event`s in time order, each one `Kind` of
//! fact. the semantic track is made of these; the visual track keeps
//! sampled frames beside it (see `visual`).

use std::sync::Arc;

use sem::{Actor, Change, Millis, Request, RequestId, Revision, SemId, Snapshot, Stage};

/// one recorded fact, at the time the caller gave for it.
///
/// times never go back: a time earlier than the last recorded one is
/// recorded as the last one, so the stream stays ordered (see
/// `Stats::clamped`).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub t: Millis,
    pub kind: Kind,
}

/// what happened at `Event::t`.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// an actor observed the app: the whole snapshot (`scope` none) or
    /// the subtree under `scope`, at `revision`.
    Observed {
        actor: Actor,
        scope: Option<SemId>,
        revision: Revision,
    },
    /// a full snapshot, the keyframe of the semantic track: the state at
    /// this instant is exactly `snapshot`. `changes` is the diff from the
    /// state recorded before it, kept to explain the step; it is empty for
    /// the first snapshot and when the recorder re-keys an unchanged state.
    Snapshot {
        snapshot: Arc<Snapshot>,
        changes: Vec<Change>,
    },
    /// the diff from the state recorded before, applied in place. it
    /// never holds a node added, a parent changed or a reorder: a
    /// projection with any of those is stored whole, because a `Change`
    /// does not carry the new node or the new order.
    Changes(Vec<Change>),
    /// one stage of a receipt. `origin` names the dispatcher the stage
    /// came from (one per `Recorder::sink`), `id` the request in it. every
    /// stage of one receipt shares its request, so the first stage holds
    /// it whole. a parameter or a value aimed at a secret field is
    /// redacted to its length.
    Stage {
        origin: u16,
        id: RequestId,
        request: Arc<Request>,
        stage: Stage,
        validated_against: Option<Revision>,
    },
    /// a person's input: its kind, where it landed in logical window
    /// pixels, and the control bound under it in the frame on screen.
    Input {
        kind: InputKind,
        at: Option<[f32; 2]>,
        target: Option<SemId>,
    },
}

impl Kind {
    /// the stable name, the same one serde uses.
    pub fn name(&self) -> &'static str {
        match self {
            Kind::Observed { .. } => "observed",
            Kind::Snapshot { .. } => "snapshot",
            Kind::Changes(_) => "changes",
            Kind::Stage { .. } => "stage",
            Kind::Input { .. } => "input",
        }
    }
}

/// the kind of a person's input, and nothing of its content: a key's
/// character or a typed text is never recorded, since it could be a
/// password.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    PointerDown,
    PointerUp,
    PointerMove,
    Wheel,
    Key,
    Text,
}
