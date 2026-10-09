//! `Shown`, what a port showed each actor: the revisions it gave it, in
//! order, each with the part of the app that came with it, so an act on
//! `from_revision` rests on what its actor was shown and on nothing else.
//!
//! an observation shows a part. a snapshot shows the nodes under its scope
//! (every node without one) with their state under its keys (every key
//! without them). a delta shows only what it carries: a state that
//! changed, the state under its keys of a node added or removed; a state
//! of its cut that did not move keeps the value the actor was shown
//! before, if it was ever shown one. a terminal stage gives a revision and
//! shows no state. the values are read from the ring, which holds the
//! whole snapshot of each revision served: a part whose revision left the
//! ring shows nothing any more.
//!
//! the actor is the name declared in `hello`, the identity of receipts and
//! of `acted`: two connections under one name are one actor, and what one
//! was shown the other was. it is bounded: the last `cap` revisions given
//! to each actor, and at most `MAX_ACTORS` actors, the one served longest
//! ago forgotten first. whatever was forgotten counts as never shown.
//!
//! each revision given carries the dispatcher's mark at that instant
//! (`sem::Mark`, followed by the port's sink in `Looks`), so an act says
//! when its actor looked at what it reads: the oldest of the observations
//! its reads come from. the dispatcher names what ran after that look in a
//! stale refusal (task-91); the mark only names.

use std::collections::{BTreeSet, VecDeque};

use accord::{
    Actor, Change, Mark, Receipt, RequestId, Revision, SemId, Snapshot, Stage, StateKey, StateRef,
    Value,
};

use crate::ring::Ring;

/// how many actors a port remembers what it showed.
pub const MAX_ACTORS: usize = 16;

/// the part of the app that came with a revision.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Part {
    /// a snapshot: the nodes under `scope`, their state under `keys`.
    Cut {
        scope: Option<SemId>,
        keys: Option<Vec<StateKey>>,
    },
    /// a delta: the states that changed, and the nodes added or removed,
    /// whose state under `keys` it shows.
    Moved {
        states: BTreeSet<StateRef>,
        nodes: BTreeSet<SemId>,
        keys: Option<Vec<StateKey>>,
    },
    /// a terminal stage: the name of a revision, no state.
    Nothing,
}

impl Part {
    /// what a delta that served `changes` showed.
    pub(crate) fn moved(changes: &[Change], keys: Option<Vec<StateKey>>) -> Part {
        let mut states = BTreeSet::new();
        let mut nodes = BTreeSet::new();
        for change in changes {
            match change {
                Change::StateChanged { at, .. } => {
                    states.insert(at.clone());
                }
                Change::NodeAdded(node) => {
                    nodes.insert(node.id.clone());
                }
                Change::NodeRemoved(id) => {
                    nodes.insert(id.clone());
                }
                _ => {}
            }
        }
        Part::Moved {
            states,
            nodes,
            keys,
        }
    }

    /// whether this part showed `read`, `at` being the snapshot of its
    /// revision if the ring still holds it; `None` when that cannot be
    /// told any more (a scoped snapshot whose revision left the ring).
    fn shows(&self, read: &StateRef, at: Option<&Snapshot>) -> Option<bool> {
        match self {
            Part::Nothing => Some(false),
            Part::Moved {
                states,
                nodes,
                keys,
            } => Some(
                states.contains(read) || (nodes.contains(&read.node) && keeps(keys, &read.key)),
            ),
            Part::Cut { keys, .. } if !keeps(keys, &read.key) => Some(false),
            Part::Cut { scope: None, .. } => Some(true),
            Part::Cut {
                scope: Some(scope), ..
            } => at.map(|snapshot| under(snapshot, &read.node, scope)),
        }
    }
}

/// true when a cut by `keys` keeps state under `key`: every key when it
/// names none.
pub(crate) fn keeps(keys: &Option<Vec<StateKey>>, key: &StateKey) -> bool {
    keys.as_ref().is_none_or(|keys| keys.contains(key))
}

/// true when `snapshot` holds `node` and it is `scope` or under it.
fn under(snapshot: &Snapshot, node: &SemId, scope: &SemId) -> bool {
    let mut at = snapshot.get(node);
    for _ in 0..=snapshot.len() {
        let Some(n) = at else {
            return false;
        };
        if n.id == *scope {
            return true;
        }
        at = n.parent.as_ref().and_then(|p| snapshot.get(p));
    }
    false
}

/// the values an act on a revision rests on, one per read, and the look
/// they come from.
pub(crate) struct Rested {
    pub(crate) values: Vec<Option<Value>>,
    /// the oldest mark of the parts the values came from; `None` with no
    /// read, or when one of those parts has no mark.
    pub(crate) look: Option<Mark>,
}

/// a revision given to an actor, with what came with it and the
/// dispatcher's mark when it was given (`None` before the port's sink was
/// on the dispatcher).
#[derive(Clone, Debug)]
struct Given {
    revision: Revision,
    part: Part,
    mark: Option<Mark>,
}

/// what a port showed each actor, bounded (see the module).
#[derive(Clone, Debug)]
pub(crate) struct Shown {
    cap: usize,
    /// the actor served longest ago first.
    actors: VecDeque<(String, VecDeque<Given>)>,
}

impl Shown {
    /// remembers the last `cap` revisions given to each actor (one at
    /// least).
    pub(crate) fn new(cap: usize) -> Self {
        Shown {
            cap: cap.max(1),
            actors: VecDeque::new(),
        }
    }

    /// `actor` was given `revision` with `part`, at `mark`. a terminal
    /// stage at the revision the actor was last given adds nothing, since
    /// it shows nothing.
    pub(crate) fn record(
        &mut self,
        actor: &str,
        revision: Revision,
        part: Part,
        mark: Option<Mark>,
    ) {
        let mut given = match self.actors.iter().position(|(name, _)| name == actor) {
            Some(at) => self
                .actors
                .remove(at)
                .map(|(_, given)| given)
                .unwrap_or_default(),
            None => VecDeque::new(),
        };
        let again = part == Part::Nothing && given.back().is_some_and(|g| g.revision == revision);
        if !again {
            given.push_back(Given {
                revision,
                part,
                mark,
            });
            while given.len() > self.cap {
                given.pop_front();
            }
        }
        self.actors.push_back((actor.to_owned(), given));
        while self.actors.len() > MAX_ACTORS {
            self.actors.pop_front();
        }
    }

    /// true when `revision` was given to `actor` and is still remembered.
    pub(crate) fn given(&self, actor: &str, revision: Revision) -> bool {
        self.until(actor, revision).is_some()
    }

    /// the value `actor` was shown for each of `reads`, as it stood the
    /// last time it was given `revision`: the value the last part up to
    /// there that showed the read holds for it. the reads no part showed,
    /// or whose last showing can no longer be read (its revision left the
    /// ring), are the error, each once, in the order of `reads`; every
    /// read is when `revision` was not given.
    ///
    /// with the values, the look they rest on (`Rested`).
    pub(crate) fn values(
        &self,
        actor: &str,
        revision: Revision,
        reads: &[StateRef],
        ring: &Ring,
    ) -> Result<Rested, Vec<StateRef>> {
        let given = self
            .until(actor, revision)
            .map(|(given, until)| given.range(..=until));
        let mut values = Vec::with_capacity(reads.len());
        let mut marks = Vec::with_capacity(reads.len());
        let mut missing: Vec<StateRef> = Vec::new();
        for read in reads {
            match given
                .clone()
                .and_then(|given| last_shown(given, read, ring))
            {
                Some((value, mark)) => {
                    values.push(value);
                    marks.push(mark);
                }
                None if missing.contains(read) => {}
                None => missing.push(read.clone()),
            }
        }
        if !missing.is_empty() {
            return Err(missing);
        }
        let marks: Option<Vec<Mark>> = marks.into_iter().collect();
        Ok(Rested {
            values,
            look: marks.and_then(|m| m.into_iter().min()),
        })
    }

    /// the mark of the last time `actor` was given `revision`: the look an
    /// act with explicit `observed` values names.
    pub(crate) fn looked(&self, actor: &str, revision: Revision) -> Option<Mark> {
        let (given, until) = self.until(actor, revision)?;
        given.get(until)?.mark
    }

    /// the revisions given to `actor` and where the last giving of
    /// `revision` sits among them.
    fn until(&self, actor: &str, revision: Revision) -> Option<(&VecDeque<Given>, usize)> {
        let (_, given) = self.actors.iter().find(|(name, _)| name == actor)?;
        let until = given.iter().rposition(|g| g.revision == revision)?;
        Some((given, until))
    }
}

/// the value the last of `given` that showed `read` holds for it, with
/// that part's mark. `None` when none showed it, and when the last one
/// that did (or may have) can no longer be read: an older showing is not
/// what the actor saw last.
fn last_shown<'a>(
    given: impl DoubleEndedIterator<Item = &'a Given>,
    read: &StateRef,
    ring: &Ring,
) -> Option<(Option<Value>, Option<Mark>)> {
    for g in given.rev() {
        let at = ring.get(g.revision);
        match g.part.shows(read, at) {
            Some(false) => continue,
            Some(true) => return at.map(|snapshot| (snapshot.value(read).cloned(), g.mark)),
            None => return None,
        }
    }
    None
}

/// how many open requests of a port keep the look they rest on.
const MAX_LOOKS: usize = 256;

/// the dispatcher's order as the port's sink follows it (`sem::Mark`).
/// the sink learns the dispatcher's mark when it is added and follows it
/// as requests are taken, so an observation served now is marked now
/// (`now`). an act says which look it rests on right before its dispatch
/// (`expect`); the sink files that look under the request's id when the
/// dispatcher takes it, and gives it back while the request is open
/// (`of`), which is when a stale refusal can come. bounded: the looks of
/// the last `MAX_LOOKS` open requests; one past it is named by the rule
/// without a look.
#[derive(Debug, Default)]
pub(crate) struct Looks {
    now: Option<Mark>,
    /// the actor of the act about to be dispatched, and its look.
    pending: Option<(Actor, Mark)>,
    open: VecDeque<(RequestId, Mark)>,
}

impl Looks {
    /// the dispatcher took the sink at `mark`.
    pub(crate) fn attached(&mut self, mark: Mark) {
        self.now = Some(mark);
    }

    /// the dispatcher's mark now; `None` while the sink is not on it.
    pub(crate) fn now(&self) -> Option<Mark> {
        self.now
    }

    /// the next request of this actor the dispatcher takes rests on this
    /// look; `None` expects nothing.
    pub(crate) fn expect(&mut self, next: Option<(Actor, Mark)>) {
        self.pending = next;
    }

    /// a stage the sink heard: a request taken moves the mark on and
    /// takes the look expected for its actor; a terminal stage ends the
    /// request's look.
    pub(crate) fn heard(&mut self, receipt: &Receipt, stage: &Stage) {
        if matches!(stage, Stage::Requested) {
            self.now = self.now.map(|now| now.max(Mark::after(receipt.id)));
            let ours = |(actor, _): &mut (Actor, Mark)| *actor == receipt.request.actor;
            if let Some((_, look)) = self.pending.take_if(ours) {
                self.open.push_back((receipt.id, look));
                while self.open.len() > MAX_LOOKS {
                    self.open.pop_front();
                }
            }
        } else if stage.is_terminal() {
            self.open.retain(|(id, _)| *id != receipt.id);
        }
    }

    /// the look the open request `id` rests on.
    pub(crate) fn of(&self, id: RequestId) -> Option<Mark> {
        self.open
            .iter()
            .find(|(at, _)| *at == id)
            .map(|(_, look)| *look)
    }
}
