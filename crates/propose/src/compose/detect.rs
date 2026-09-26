//! `Detector`: what happened on an app's dispatcher, read stage by stage,
//! and the sequence repeated at the end of it.
//!
//! the input is the stream of stages of one dispatcher, in the order they
//! happened: `(id, request, stage)`. live, a `Listener` hears it as the
//! dispatcher's sink; recorded, it is the session's stage events of one
//! origin, in the order of the file (`rec::Kind::Stage`). the same calls,
//! the same answer.
//!
//! a request enters the history when it ends, in the order of its id (the
//! order the dispatcher took them): one still open holds the ones after
//! it. a request taken while a composite of the same actor runs is a step
//! of that composite, not something its actor did: it does not enter. a
//! request enters as a step only when it completed without a parameter,
//! never waited for a person and is not a composite; anything else enters
//! as a cut, which no sequence spans.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};

use sem::{ActionName, Actor, Receipt, ReceiptSink, Request, RequestId, SemId, Stage};

use super::{Found, Shape, Step, Target, item_of};

/// the longest sequence looked for, in steps.
pub const MAX_STEPS: usize = 6;

/// how many requests the history keeps: enough for the longest sequence
/// repeated many times over.
const KEPT: usize = 64;

/// one request in the history.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Entry {
    /// an action that ran to its end and a composite could repeat.
    Step {
        actor: Actor,
        target: SemId,
        action: ActionName,
    },
    /// anything else: no sequence goes across it.
    Cut,
}

/// a request taken and not in the history yet.
#[derive(Clone, Debug)]
struct Open {
    actor: Actor,
    target: SemId,
    action: ActionName,
    /// it can never be a step: a parameter, a wait for a person, a
    /// composite, or a request first heard after its start.
    cut: bool,
    /// taken while a composite of its actor ran: one of its steps.
    nested: bool,
    /// how it ended: completed or not; `None` while it runs.
    ended: Option<bool>,
}

/// finds the sequence of actions an actor repeats, from the stages of one
/// dispatcher.
#[derive(Clone, Debug)]
pub struct Detector {
    repeats: usize,
    composites: BTreeSet<ActionName>,
    open: BTreeMap<RequestId, Open>,
    /// the highest id that entered the history: a stage of one at or
    /// below it is of a request already read.
    read: Option<RequestId>,
    /// the composites running now, with who runs them.
    running: Vec<(RequestId, Actor)>,
    history: VecDeque<Entry>,
}

impl Detector {
    /// a detector that finds a sequence once it is repeated `repeats`
    /// times in a row (two at least).
    pub fn new(repeats: usize) -> Detector {
        Detector {
            repeats: repeats.max(2),
            composites: BTreeSet::new(),
            open: BTreeMap::new(),
            read: None,
            running: Vec::new(),
            history: VecDeque::new(),
        }
    }

    /// how many times in a row a sequence is repeated before it is found.
    pub fn repeats(&self) -> usize {
        self.repeats
    }

    /// `action` is a composite: a request for it runs its steps, and the
    /// requests its actor makes while it runs are those steps.
    pub fn composite(&mut self, action: ActionName) {
        self.composites.insert(action);
    }

    /// one stage of the request `id`, in the order the dispatcher went
    /// through them.
    pub fn stage(&mut self, id: RequestId, req: &Request, stage: &Stage) {
        if self.read.is_some_and(|read| id <= read) {
            return;
        }
        if !self.open.contains_key(&id) {
            let nested = self.running.iter().any(|(_, actor)| *actor == req.actor);
            self.open.insert(
                id,
                Open {
                    actor: req.actor.clone(),
                    target: req.target.clone(),
                    action: req.action.clone(),
                    cut: *stage != Stage::Requested
                        || req.param.is_some()
                        || self.composites.contains(&req.action),
                    nested,
                    ended: None,
                },
            );
        }
        match stage {
            Stage::AwaitingConfirmation | Stage::Confirmed { .. } => {
                if let Some(open) = self.open.get_mut(&id) {
                    open.cut = true;
                }
            }
            Stage::Executing if self.composites.contains(&req.action) => {
                self.running.push((id, req.actor.clone()));
            }
            _ => {}
        }
        if stage.is_terminal() {
            self.running.retain(|(running, _)| *running != id);
            if let Some(open) = self.open.get_mut(&id) {
                open.ended = Some(matches!(stage, Stage::Completed(_)));
            }
            self.settle();
        }
    }

    /// moves every request that ended, from the lowest id up to the first
    /// still open, into the history.
    fn settle(&mut self) {
        while let Some(entry) = self.open.first_entry() {
            let Some(completed) = entry.get().ended else {
                break;
            };
            let (id, open) = entry.remove_entry();
            self.read = Some(id);
            if open.nested {
                continue;
            }
            self.history.push_back(if completed && !open.cut {
                Entry::Step {
                    actor: open.actor,
                    target: open.target,
                    action: open.action,
                }
            } else {
                Entry::Cut
            });
            while self.history.len() > KEPT {
                self.history.pop_front();
            }
        }
    }

    /// the sequence repeated at the end of the history: the shortest one,
    /// of two steps or more and not itself a repetition of a shorter one,
    /// found `repeats` times in a row by the same actor; with how many
    /// times it is repeated there.
    pub fn found(&self) -> Option<Found> {
        let h: Vec<&Entry> = self.history.iter().collect();
        for len in 2..=MAX_STEPS {
            let Some((actor, shape)) = runs(&h, len, self.repeats) else {
                continue;
            };
            let mut repeats = self.repeats;
            while runs(&h, len, repeats + 1).is_some_and(|(_, s)| s == shape) {
                repeats += 1;
            }
            return Some(Found {
                actor,
                shape,
                repeats,
            });
        }
        None
    }
}

/// the last `n` runs of `len` steps of `h`, when they are one sequence:
/// its actor and shape.
fn runs(h: &[&Entry], len: usize, n: usize) -> Option<(Actor, Shape)> {
    let tail = h.get(h.len().checked_sub(len * n)?..)?;
    let mut steps: Vec<(&Actor, &SemId, &ActionName)> = Vec::with_capacity(tail.len());
    for entry in tail {
        match entry {
            Entry::Step {
                actor,
                target,
                action,
            } => steps.push((actor, target, action)),
            Entry::Cut => return None,
        }
    }
    let actor = steps.first()?.0;
    if steps.iter().any(|(a, _, _)| *a != actor) {
        return None;
    }
    let runs: Vec<&[(&Actor, &SemId, &ActionName)]> = steps.chunks(len).collect();
    let first = runs.first()?;
    let mut shape = Vec::with_capacity(len);
    for (at, (_, target, action)) in first.iter().enumerate() {
        let column: Vec<&SemId> = runs.iter().map(|run| run[at].1).collect();
        if runs.iter().any(|run| run[at].2 != *action) {
            return None;
        }
        let target = if column.iter().all(|t| t == target) {
            Target::Node((*target).clone())
        } else {
            let (base, _) = item_of(target)?;
            if column
                .iter()
                .any(|t| item_of(t).is_none_or(|(b, _)| b != base))
            {
                return None;
            }
            Target::Item(base)
        };
        shape.push(Step {
            action: (*action).clone(),
            target,
        });
    }
    // one item per run, the same collection for every item step
    let bases: BTreeSet<&SemId> = shape
        .iter()
        .filter_map(|s| match &s.target {
            Target::Item(base) => Some(base),
            Target::Node(_) => None,
        })
        .collect();
    if bases.len() > 1 {
        return None;
    }
    for run in &runs {
        let keys: BTreeSet<&str> = shape
            .iter()
            .zip(run.iter())
            .filter(|(s, _)| matches!(s.target, Target::Item(_)))
            .filter_map(|(_, (_, target, _))| item_of(target).map(|(_, key)| key))
            .collect();
        if keys.len() > 1 {
            return None;
        }
    }
    let primitive = (1..len)
        .filter(|d| len.is_multiple_of(*d))
        .all(|d| shape.chunks(d).any(|c| c != &shape[..d]));
    primitive.then(|| (actor.clone(), Shape { steps: shape }))
}

/// the detector heard live: a sink on the app's dispatcher that tells a
/// shared detector every stage, as it happens.
#[derive(Clone, Debug)]
pub struct Listener {
    detector: Arc<Mutex<Detector>>,
}

impl Listener {
    pub fn new(detector: Arc<Mutex<Detector>>) -> Listener {
        Listener { detector }
    }
}

impl ReceiptSink for Listener {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        self.detector
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .stage(receipt.id, &receipt.request, stage);
    }
}
