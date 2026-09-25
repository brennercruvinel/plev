//! composite capabilities (task-70 phase 3): the sequence of actions a
//! person repeats becomes one action of the app, once the person accepts
//! it.
//!
//! - `Detector` reads the stages of an app's dispatcher, in order, the
//!   same way live (`Listener`, a `sem::ReceiptSink`) and from a recorded
//!   session (the `rec` stage events of one origin), and finds the same
//!   sequence of actions, by the same actor, in the same order, repeated
//!   at the end of what happened (`Found`, with its `Shape`).
//!
//! two steps are the same when they ask the same action of the same
//! node, or of items of the same collection (`silences.cut[3]` and
//! `silences.cut[4]`, one item for every step of one run). a step that
//! carried a parameter, was refused, failed or waited for a person breaks
//! the sequence: the detector keeps no value of any parameter, only who
//! asked which action of which node.
//!
//! pure, like the rest of the crate: no clock, no file, nothing leaves
//! the process.

mod detect;

pub use detect::{Detector, Listener, MAX_STEPS};

use sem::{ActionName, Actor, SemId};

/// where a step acts.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    /// this node, in every run.
    Node(SemId),
    /// the item of the collection `base` (`base[<key>]`): a run acts on
    /// one item, the same for every step of it that is `Item`.
    Item(SemId),
}

impl Target {
    /// the node this target is for `item` (the key segment, brackets
    /// included, as `item_of` gives it): the node itself, or the item of
    /// the collection. `None` when an item target has no item.
    pub fn node(&self, item: Option<&str>) -> Option<SemId> {
        match self {
            Target::Node(id) => Some(id.clone()),
            Target::Item(base) => SemId::parse(&format!("{base}{}", item?)).ok(),
        }
    }

    /// the template this target stands for in a generated surface: the
    /// node itself, or `base[{item}]`.
    pub fn template(&self) -> SemId {
        match self {
            Target::Node(id) => id.clone(),
            Target::Item(base) => base.key(uir::ITEM),
        }
    }
}

/// one step of a sequence: an action on a target.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Step {
    pub action: ActionName,
    pub target: Target,
}

/// a sequence of steps, by shape: what a composite runs, in order.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Shape {
    pub steps: Vec<Step>,
}

impl Shape {
    /// the collection its item steps act on, when it has one.
    pub fn collection(&self) -> Option<&SemId> {
        self.steps.iter().find_map(|s| match &s.target {
            Target::Item(base) => Some(base),
            Target::Node(_) => None,
        })
    }

    /// every node the steps name: the exact ones, and the collection of
    /// the item ones.
    pub fn nodes(&self) -> impl Iterator<Item = &SemId> {
        self.steps.iter().map(|s| match &s.target {
            Target::Node(id) | Target::Item(id) => id,
        })
    }
}

/// a sequence found at the end of what happened: who repeated it, its
/// shape, and how many times in a row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub actor: Actor,
    pub shape: Shape,
    pub repeats: usize,
}

/// the collection and the item of an id that ends with a key segment:
/// `silences.cut[3]` is `silences.cut` and `[3]`. `None` for any other id.
pub fn item_of(id: &SemId) -> Option<(SemId, &str)> {
    let base = id.parent_path()?;
    let rest = id.as_str().get(base.as_str().len()..)?;
    rest.starts_with('[').then_some((base, rest))
}
