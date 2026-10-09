//! `why`: the causal chain of one request, read from the recording: the
//! observation it rested on, the request with the values it read, what
//! moved in those values between the observation and the request and which
//! receipt ran then, the stages, the verdict with its delta, the result.

use rec::{InputKind, Kind, ReadError, State};
use sem::{
    ActionName, Actor, Change, Millis, Node, Outcome, Rejection, Request, RequestId, Revision,
    SemId, Stage, StateRef, Value,
};

use crate::index::{Inspector, Stamp, Tracked};

/// the causal chain of one request.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Why {
    /// the dispatcher the request came from (one per sink the recorder
    /// gave out) and its id there.
    pub origin: u16,
    pub id: RequestId,
    /// false when the window starts after the request did: its first
    /// stages were cut. the request itself is whole, every stage carries
    /// it.
    pub whole: bool,
    pub request: Asked,
    pub basis: Basis,
    pub between: Between,
    pub stages: Vec<StageAt>,
    pub ending: Ending,
}

/// what was asked, by whom, of which control, having seen what.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Asked {
    /// the first stage the window holds (the request itself, when whole).
    pub t: Millis,
    pub at: Millis,
    pub actor: Actor,
    pub target: SemId,
    pub action: ActionName,
    pub param: Option<Value>,
    /// the question the actor asked a person before the action ran
    /// (`Request::ask`, task-92), as recorded: `REDACTED` when the target
    /// is secret. not written when it asked none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<String>,
    /// the target as the recorded state held it right before the request;
    /// `None` when it did not hold it.
    pub control: Option<Node>,
    /// the revision of the snapshot the actor saw (it traces, it never
    /// decides).
    pub revision: Revision,
    /// the precondition: the value the actor saw for each state the action
    /// reads.
    pub reads: Vec<Read>,
}

/// one state the action reads, as the actor saw it.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Read {
    pub at: StateRef,
    pub seen: Option<Value>,
    /// whether the node of this state is inside the scope of the recorded
    /// observation the request rested on. a scoped observation serves one
    /// subtree, and the agent port rebuilds the values outside it from the
    /// whole snapshot at that revision, so `false` means the actor was not
    /// served this value by that observation. `None` when no observation
    /// was recorded, or the recorded state does not place the node.
    pub in_scope: Option<bool>,
}

/// the observation the request rested on, or why the recording has none.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Basis {
    /// the last observation by the same actor, at the revision the request
    /// carries, before the request.
    Observed {
        t: Millis,
        at: Millis,
        scope: Option<SemId>,
        revision: Revision,
    },
    /// no such observation is recorded. `state` is the last instant before
    /// the request at which the recorded state had the revision the
    /// request carries, when there is one.
    NotRecorded {
        reason: Unrecorded,
        state: Option<Stamp>,
    },
}

/// why no observation backs a request.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Unrecorded {
    /// no observation by this actor before the request: a person acts on
    /// the frame on screen, and the recorder keeps only the observations
    /// the app reports (the agent port reports an agent's); or the window
    /// starts after it.
    NoObservation,
    /// the actor's observations before the request are at other
    /// revisions; this is the last one. the revision came another way (the
    /// last stage of an earlier act carries the app's revision).
    OtherRevision {
        t: Millis,
        at: Millis,
        revision: Revision,
    },
}

/// what the recording shows between the observation and the request.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Between {
    /// where the stretch starts: the observation, or else the last
    /// instant the state had the observed revision; `None` when neither is
    /// recorded, and then nothing is said about when a value moved.
    pub from: Option<Stamp>,
    /// the request.
    pub to: Stamp,
    /// every state the action reads, and how it moved in the stretch.
    pub reads: Vec<ReadMoves>,
    /// a person's input in the stretch.
    pub inputs: Vec<InputAt>,
}

/// one state the action reads, over the stretch.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ReadMoves {
    pub at: StateRef,
    /// the value the actor saw.
    pub seen: Option<Value>,
    /// the recorded value right before the request.
    pub before_request: Option<Value>,
    /// every recorded change of the value in the stretch, in order.
    pub moves: Vec<Move>,
}

/// one change of a value the action reads.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Move {
    pub t: Millis,
    pub at: Millis,
    /// the last recorded state before this change: the window `by` looks
    /// in runs from after it to the change. `None` when the change is the
    /// first recorded state.
    pub since: Option<Stamp>,
    pub before: Option<Value>,
    pub after: Option<Value>,
    /// the items that came and went, as the recorded change lists them.
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub by: By,
}

/// who moved a value, by correlation in time: the receipts that ran
/// (reached executing, completed or failed; a refused one ran nothing)
/// between the recorded state before the change and the change. it is not
/// a declaration: the contract declares what an action reads, never what
/// it writes.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum By {
    /// no receipt ran in that window: a change from outside the app, with
    /// no request.
    Outside,
    /// one receipt ran in that window.
    Receipt { cause: Cause },
    /// two or more did; the recording cannot tell which moved the value.
    Ambiguous { causes: Vec<Cause> },
}

/// a receipt that ran in the window of a change.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Cause {
    pub origin: u16,
    pub id: RequestId,
    pub actor: Actor,
    pub target: SemId,
    pub action: ActionName,
    pub param: Option<Value>,
    /// the stage of it that fell in the window, and when.
    pub stage: &'static str,
    pub t: Millis,
    pub at: Millis,
    /// true when the actor is the one of the request explained.
    pub same_actor: bool,
}

/// a person's input: its kind and the control under it.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct InputAt {
    pub t: Millis,
    pub at: Millis,
    pub input: InputKind,
    pub target: Option<SemId>,
}

/// one stage of the request, when it happened.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct StageAt {
    pub t: Millis,
    pub at: Millis,
    pub stage: Stage,
}

/// how the request ended, as far as the window goes.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Ending {
    Completed {
        t: Millis,
        at: Millis,
        outcome: Outcome,
    },
    /// refused: nothing ran. `at_truth` when it was refused after it
    /// reached executing (the point of truth checked again).
    Rejected {
        t: Millis,
        at: Millis,
        rejection: Rejection,
        at_truth: bool,
    },
    Failed {
        t: Millis,
        at: Millis,
        error: String,
    },
    /// not finished when the window ends: its last stage.
    Open {
        t: Millis,
        at: Millis,
        stage: &'static str,
    },
}

impl Inspector<'_> {
    /// the causal chain of the request `id` of `origin`; `None` when the
    /// window holds no such request.
    pub fn why(&self, origin: u16, id: RequestId) -> Result<Option<Why>, ReadError> {
        self.why_upto(origin, id, self.session().events().len())
    }

    /// the causal chain as the first `limit` events tell it: the stages
    /// after them are left out, and a request asked after them is `None`.
    pub(crate) fn why_upto(
        &self,
        origin: u16,
        id: RequestId,
        limit: usize,
    ) -> Result<Option<Why>, ReadError> {
        let Some(tracked) = self.find(origin, id) else {
            return Ok(None);
        };
        let Some(first) = tracked.first().filter(|&f| f < limit) else {
            return Ok(None);
        };
        let request = &tracked.at.receipt.request;
        let before = self.state_upto(first)?;
        let (basis, from) = self.basis(request, first)?;
        let scope = match (&basis, from) {
            (Basis::Observed { scope, .. }, Some(i)) => {
                Some((scope.clone(), self.state_upto(i + 1)?))
            }
            _ => None,
        };
        let reads = request
            .observed
            .values
            .iter()
            .map(|(at, seen)| Read {
                at: at.clone(),
                seen: seen.clone(),
                in_scope: scope
                    .as_ref()
                    .and_then(|(scope, state)| within(state.as_ref(), &at.node, scope.as_ref())),
            })
            .collect();
        let asked = self.stamp(self.time_of(first));
        let between = self.between(tracked, from, first, before.as_ref())?;
        let stages: Vec<StageAt> = tracked
            .at
            .receipt
            .stages
            .iter()
            .zip(&tracked.index)
            .filter(|(_, i)| **i < limit)
            .map(|((stage, t), _)| {
                let Stamp { t, at } = self.stamp(*t);
                StageAt {
                    t,
                    at,
                    stage: stage.clone(),
                }
            })
            .collect();
        let ending = ending(&stages);
        Ok(Some(Why {
            origin,
            id,
            whole: tracked.at.whole,
            request: Asked {
                t: asked.t,
                at: asked.at,
                actor: request.actor.clone(),
                target: request.target.clone(),
                action: request.action.clone(),
                param: request.param.clone(),
                ask: request.ask.clone(),
                control: before
                    .as_ref()
                    .and_then(|s| s.get(&request.target))
                    .cloned(),
                revision: request.observed.revision,
                reads,
            },
            basis,
            between,
            stages,
            ending,
        }))
    }

    fn time_of(&self, i: usize) -> Millis {
        self.session().events().get(i).map_or(self.start(), |e| e.t)
    }

    /// the observation `request` rested on: the last one by its actor, at
    /// its revision, before the event `first`; with its index. without
    /// one, why not, and the last event before `first` that left the state
    /// at that revision, with its index.
    fn basis(&self, request: &Request, first: usize) -> Result<(Basis, Option<usize>), ReadError> {
        let events = self.session().events();
        let mut other: Option<(Millis, Revision)> = None;
        for (i, event) in events.iter().enumerate().take(first).rev() {
            let Kind::Observed {
                actor,
                scope,
                revision,
            } = &event.kind
            else {
                continue;
            };
            if *actor != request.actor {
                continue;
            }
            if *revision == request.observed.revision {
                let Stamp { t, at } = self.stamp(event.t);
                let basis = Basis::Observed {
                    t,
                    at,
                    scope: scope.clone(),
                    revision: *revision,
                };
                return Ok((basis, Some(i)));
            }
            other.get_or_insert((event.t, *revision));
        }
        let reason = match other {
            Some((t, revision)) => {
                let Stamp { t, at } = self.stamp(t);
                Unrecorded::OtherRevision { t, at, revision }
            }
            None => Unrecorded::NoObservation,
        };
        let held = self.last_state_at(request.observed.revision, first)?;
        let state = held.map(|i| self.stamp(self.time_of(i)));
        Ok((Basis::NotRecorded { reason, state }, held))
    }

    /// the last event before `before` that left the recorded state at
    /// `revision`.
    fn last_state_at(&self, revision: Revision, before: usize) -> Result<Option<usize>, ReadError> {
        for j in (0..before).rev() {
            if !self.sets_state(j) {
                continue;
            }
            let Some(state) = self.state_upto(j + 1)? else {
                return Ok(None);
            };
            if state.revision().ok() == Some(revision) {
                return Ok(Some(j));
            }
        }
        Ok(None)
    }

    /// every read of the request over the stretch from the event `from` to
    /// the event `first` (both left out), the moves and who ran then.
    fn between(
        &self,
        tracked: &Tracked,
        from: Option<usize>,
        first: usize,
        before: Option<&State>,
    ) -> Result<Between, ReadError> {
        let request = &tracked.at.receipt.request;
        let to = self.stamp(self.time_of(first));
        let value = |state: Option<&State>, at: &StateRef| state.and_then(|s| s.value(at)).cloned();
        let mut reads: Vec<ReadMoves> = request
            .observed
            .values
            .iter()
            .map(|(at, seen)| ReadMoves {
                at: at.clone(),
                seen: seen.clone(),
                before_request: value(before, at),
                moves: Vec::new(),
            })
            .collect();
        let Some(from) = from else {
            return Ok(Between {
                from: None,
                to,
                reads,
                inputs: Vec::new(),
            });
        };
        let start = self.state_upto(from + 1)?;
        let mut last: Vec<Option<Value>> =
            reads.iter().map(|r| value(start.as_ref(), &r.at)).collect();
        let mut inputs = Vec::new();
        let events = self.session().events();
        for (j, event) in events.iter().enumerate().take(first).skip(from + 1) {
            if let Kind::Input { kind, target, .. } = &event.kind {
                let Stamp { t, at } = self.stamp(event.t);
                inputs.push(InputAt {
                    t,
                    at,
                    input: *kind,
                    target: target.clone(),
                });
                continue;
            }
            if !self.sets_state(j) {
                continue;
            }
            let state = self.state_upto(j + 1)?;
            let mut window: Option<(By, Option<Stamp>)> = None;
            for (read, prev) in reads.iter_mut().zip(last.iter_mut()) {
                let now = value(state.as_ref(), &read.at);
                if now == *prev {
                    continue;
                }
                let (by, since) = window
                    .get_or_insert_with(|| self.correlate(tracked, j))
                    .clone();
                let (added, removed) = recorded_items(&event.kind, &read.at);
                let Stamp { t, at } = self.stamp(event.t);
                read.moves.push(Move {
                    t,
                    at,
                    since,
                    before: prev.clone(),
                    after: now.clone(),
                    added,
                    removed,
                    by,
                });
                *prev = now;
            }
        }
        Ok(Between {
            from: Some(self.stamp(self.time_of(from))),
            to,
            reads,
            inputs,
        })
    }

    /// the receipts, other than `explained`, that ran between the last
    /// event before `change` that set the state and `change` itself, and
    /// that instant.
    fn correlate(&self, explained: &Tracked, change: usize) -> (By, Option<Stamp>) {
        let prev = self.state_before(change);
        let window = prev.map_or(0, |p| p + 1)..=change;
        let actor = &explained.at.receipt.request.actor;
        let mut causes: Vec<Cause> = self
            .tracked()
            .iter()
            .filter(|other| other.key() != explained.key() && !other.refused())
            .filter_map(|other| {
                let (_, stage, t) = other.ran().find(|(i, _, _)| window.contains(i))?;
                let req = &other.at.receipt.request;
                let Stamp { t, at } = self.stamp(t);
                Some(Cause {
                    origin: other.at.origin,
                    id: other.at.receipt.id,
                    actor: req.actor.clone(),
                    target: req.target.clone(),
                    action: req.action.clone(),
                    param: req.param.clone(),
                    stage: stage.name(),
                    t,
                    at,
                    same_actor: req.actor == *actor,
                })
            })
            .collect();
        let by = match causes.len() {
            0 => By::Outside,
            1 => By::Receipt {
                cause: causes.remove(0),
            },
            _ => By::Ambiguous { causes },
        };
        (by, prev.map(|p| self.stamp(self.time_of(p))))
    }
}

/// whether `node` is `scope` or under it in `state`; `Some(true)` for the
/// whole app, `None` when the state does not place the node.
fn within(state: Option<&State>, node: &SemId, scope: Option<&SemId>) -> Option<bool> {
    let Some(scope) = scope else {
        return Some(true);
    };
    let state = state?;
    let mut at = state.get(node)?;
    for _ in 0..=state.len() {
        if at.id == *scope {
            return Some(true);
        }
        match &at.parent {
            Some(parent) => at = state.get(parent)?,
            None => return Some(false),
        }
    }
    None
}

/// the items that came and went in `at`, as the change the event recorded
/// lists them (a full snapshot keeps the diff of its step).
fn recorded_items(kind: &Kind, at: &StateRef) -> (Vec<String>, Vec<String>) {
    let changes: &[Change] = match kind {
        Kind::Changes(changes) | Kind::Snapshot { changes, .. } => changes,
        _ => &[],
    };
    changes
        .iter()
        .find_map(|c| match c {
            Change::StateChanged {
                at: moved,
                added,
                removed,
                ..
            } if moved == at => Some((added.clone(), removed.clone())),
            _ => None,
        })
        .unwrap_or_default()
}

fn ending(stages: &[StageAt]) -> Ending {
    let ran = stages.iter().any(|s| s.stage == Stage::Executing);
    let Some(last) = stages.last() else {
        return Ending::Open {
            t: 0,
            at: 0,
            stage: "none",
        };
    };
    let (t, at) = (last.t, last.at);
    match &last.stage {
        Stage::Completed(outcome) => Ending::Completed {
            t,
            at,
            outcome: outcome.clone(),
        },
        Stage::Rejected(rejection) => Ending::Rejected {
            t,
            at,
            rejection: rejection.clone(),
            at_truth: ran,
        },
        Stage::Failed { error } => Ending::Failed {
            t,
            at,
            error: error.clone(),
        },
        other => Ending::Open {
            t,
            at,
            stage: other.name(),
        },
    }
}
