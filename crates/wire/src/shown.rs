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

use std::collections::{BTreeSet, VecDeque};

use sem::{Change, Revision, SemId, Snapshot, StateKey, StateRef, Value};

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

/// a revision given to an actor, with what came with it.
#[derive(Clone, Debug)]
struct Given {
    revision: Revision,
    part: Part,
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

    /// `actor` was given `revision` with `part`. a terminal stage at the
    /// revision the actor was last given adds nothing, since it shows
    /// nothing.
    pub(crate) fn record(&mut self, actor: &str, revision: Revision, part: Part) {
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
            given.push_back(Given { revision, part });
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
    pub(crate) fn values(
        &self,
        actor: &str,
        revision: Revision,
        reads: &[StateRef],
        ring: &Ring,
    ) -> Result<Vec<Option<Value>>, Vec<StateRef>> {
        let given = self
            .until(actor, revision)
            .map(|(given, until)| given.range(..=until));
        let mut values = Vec::with_capacity(reads.len());
        let mut missing: Vec<StateRef> = Vec::new();
        for read in reads {
            match given
                .clone()
                .and_then(|given| last_shown(given, read, ring))
            {
                Some(value) => values.push(value),
                None if missing.contains(read) => {}
                None => missing.push(read.clone()),
            }
        }
        if missing.is_empty() {
            Ok(values)
        } else {
            Err(missing)
        }
    }

    /// the revisions given to `actor` and where the last giving of
    /// `revision` sits among them.
    fn until(&self, actor: &str, revision: Revision) -> Option<(&VecDeque<Given>, usize)> {
        let (_, given) = self.actors.iter().find(|(name, _)| name == actor)?;
        let until = given.iter().rposition(|g| g.revision == revision)?;
        Some((given, until))
    }
}

/// the value the last of `given` that showed `read` holds for it. `None`
/// when none showed it, and when the last one that did (or may have) can
/// no longer be read: an older showing is not what the actor saw last.
fn last_shown<'a>(
    given: impl DoubleEndedIterator<Item = &'a Given>,
    read: &StateRef,
    ring: &Ring,
) -> Option<Option<Value>> {
    for g in given.rev() {
        let at = ring.get(g.revision);
        match g.part.shows(read, at) {
            Some(false) => continue,
            Some(true) => return at.map(|snapshot| snapshot.value(read).cloned()),
            None => return None,
        }
    }
    None
}
