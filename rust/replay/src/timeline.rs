//! the timeline: every event of the session, one entry each, with its
//! time from the start, and the tallies by kind and by actor.

use accord::{ActionName, Actor, Millis, RequestId, Revision, SemId};
use record::{InputKind, Kind};

use crate::describe;
use crate::index::{Inspector, Stamp};

/// every event of a session, summarized.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Timeline {
    pub start: Millis,
    pub end: Millis,
    /// how many events of each kind, in the order `observed`, `snapshot`,
    /// `changes`, `stage`, `input`; kinds with none are left out.
    pub kinds: Vec<Count>,
    /// what each actor did, in the order they first appear.
    pub actors: Vec<Tally>,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Count {
    pub kind: &'static str,
    pub count: usize,
}

/// what one actor did in the session. a person's input counts for
/// `human`: the recorder records input as a person's.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Tally {
    pub actor: Actor,
    pub observations: usize,
    /// receipts whose request this actor made.
    pub requests: usize,
    pub stages: usize,
    pub inputs: usize,
}

/// one event: when, its index in the session, and what it was.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Entry {
    pub t: Millis,
    pub at: Millis,
    pub index: usize,
    pub what: What,
}

/// an event, summarized. changes are one line each (see `text`).
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum What {
    /// an actor observed the app, whole (`scope` none) or a subtree.
    Observed {
        actor: Actor,
        scope: Option<SemId>,
        revision: Revision,
    },
    /// a full snapshot, the keyframe of the state: how many nodes, and the
    /// changes from the state before it (none when the recorder re-keyed
    /// an unchanged state).
    Snapshot { nodes: usize, changes: Vec<String> },
    /// changes applied to the state before.
    Changes { changes: Vec<String> },
    /// one stage of one receipt.
    Stage {
        origin: u16,
        id: RequestId,
        actor: Actor,
        target: SemId,
        action: ActionName,
        stage: &'static str,
        detail: Option<String>,
    },
    /// a person's input, and the control under it.
    Input {
        input: InputKind,
        target: Option<SemId>,
    },
}

impl What {
    /// the event kind, the name `rec` gives it.
    pub fn kind(&self) -> &'static str {
        match self {
            What::Observed { .. } => "observed",
            What::Snapshot { .. } => "snapshot",
            What::Changes { .. } => "changes",
            What::Stage { .. } => "stage",
            What::Input { .. } => "input",
        }
    }
}

const KINDS: [&str; 5] = ["observed", "snapshot", "changes", "stage", "input"];

impl Inspector<'_> {
    /// every event, summarized, with the tallies.
    pub fn timeline(&self) -> Timeline {
        let entries: Vec<Entry> = self
            .session()
            .events()
            .iter()
            .enumerate()
            .map(|(index, event)| {
                let Stamp { t, at } = self.stamp(event.t);
                Entry {
                    t,
                    at,
                    index,
                    what: what(&event.kind),
                }
            })
            .collect();
        let kinds = KINDS
            .iter()
            .map(|kind| Count {
                kind,
                count: entries.iter().filter(|e| e.what.kind() == *kind).count(),
            })
            .filter(|c| c.count > 0)
            .collect();
        Timeline {
            start: self.start(),
            end: self.end(),
            kinds,
            actors: self.tallies(),
            entries,
        }
    }

    /// what each actor did, in the order they first appear.
    pub fn tallies(&self) -> Vec<Tally> {
        let mut out: Vec<Tally> = Vec::new();
        for event in self.session().events() {
            match &event.kind {
                Kind::Observed { actor, .. } => slot(&mut out, actor).observations += 1,
                Kind::Stage { request, .. } => slot(&mut out, &request.actor).stages += 1,
                Kind::Input { .. } => slot(&mut out, &Actor::Human).inputs += 1,
                Kind::Snapshot { .. } | Kind::Changes(_) => {}
            }
        }
        for tracked in self.tracked() {
            slot(&mut out, &tracked.at.receipt.request.actor).requests += 1;
        }
        out
    }
}

/// the tally of `actor`, added at the end the first time it appears.
fn slot<'a>(out: &'a mut Vec<Tally>, actor: &Actor) -> &'a mut Tally {
    let at = match out.iter().position(|t| t.actor == *actor) {
        Some(at) => at,
        None => {
            out.push(Tally {
                actor: actor.clone(),
                observations: 0,
                requests: 0,
                stages: 0,
                inputs: 0,
            });
            out.len() - 1
        }
    };
    &mut out[at]
}

fn what(kind: &Kind) -> What {
    match kind {
        Kind::Observed {
            actor,
            scope,
            revision,
        } => What::Observed {
            actor: actor.clone(),
            scope: scope.clone(),
            revision: *revision,
        },
        Kind::Snapshot { snapshot, changes } => What::Snapshot {
            nodes: snapshot.len(),
            changes: changes.iter().map(describe::change).collect(),
        },
        Kind::Changes(changes) => What::Changes {
            changes: changes.iter().map(describe::change).collect(),
        },
        Kind::Stage {
            origin,
            id,
            request,
            stage,
            ..
        } => What::Stage {
            origin: *origin,
            id: *id,
            actor: request.actor.clone(),
            target: request.target.clone(),
            action: request.action.clone(),
            stage: stage.name(),
            detail: describe::stage_detail(stage),
        },
        Kind::Input { kind, target, .. } => What::Input {
            input: *kind,
            target: target.clone(),
        },
    }
}
