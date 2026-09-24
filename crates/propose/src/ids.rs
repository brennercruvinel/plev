//! the ids of a proposal. a node whose capability the current surface
//! already shows takes the id (and the label) it has there, so a change
//! of form reconciles by `SemId` and the widget, its focus and its caret
//! go on; a container takes the id of the current container that holds
//! most of the same capabilities; everything else gets an id of its own,
//! named after the app node it shows and never one the current surface
//! uses for something else.

use std::collections::{BTreeMap, BTreeSet};

use sem::{SemId, Snapshot, StateRef};
use uir::{CapabilityRef, Ir, IrNode};

/// what a bound node is there for: the capability it operates, or the
/// state it shows when it operates none.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Anchor {
    Action(CapabilityRef),
    State(StateRef),
}

/// the anchor of `node`: its `set_value` (a field is there to be
/// written), else its first action, else its first state; none for a
/// node that binds nothing.
pub(crate) fn anchor(node: &IrNode) -> Option<Anchor> {
    let actions = &node.bind.actions;
    if let Some(cap) = actions.get("set_value").or_else(|| actions.values().next()) {
        return Some(Anchor::Action(cap.clone()));
    }
    node.bind.state.values().next().cloned().map(Anchor::State)
}

/// the anchors of every bound node under `node`, `node` included.
pub(crate) fn anchors(node: &IrNode) -> BTreeSet<Anchor> {
    let mut out = BTreeSet::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        out.extend(anchor(n));
        stack.extend(n.children.iter());
    }
    out
}

/// a container of the current surface and what it holds.
struct Held {
    id: SemId,
    label: Option<String>,
    anchors: BTreeSet<Anchor>,
}

/// the smallest share of a new container's capabilities a current one
/// must have held for the new one to take its id.
const SAME_CONTAINER: f32 = 0.5;

pub(crate) struct Namer {
    surface: SemId,
    root: Option<(SemId, Option<String>)>,
    bound: BTreeMap<Anchor, (SemId, Option<String>)>,
    containers: Vec<Held>,
    /// every id the current surface uses, and every id given so far.
    taken: BTreeSet<SemId>,
    given: BTreeSet<SemId>,
}

impl Namer {
    /// ids for a proposal over `current`, or for a new surface with an id
    /// no node of `app` has or starts with.
    pub fn new(current: Option<&Ir>, app: &Snapshot) -> Namer {
        let mut namer = Namer {
            surface: fresh_surface(app),
            root: None,
            bound: BTreeMap::new(),
            containers: Vec::new(),
            taken: BTreeSet::new(),
            given: BTreeSet::new(),
        };
        let Some(ir) = current else {
            return namer;
        };
        namer.surface = ir.surface.id.clone();
        let root = &ir.surface.root;
        if root.component.is_none_or(|c| c.spec().container) {
            namer.root = Some((root.sem.clone(), root.label.clone()));
        }
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            namer.taken.insert(n.sem.clone());
            match anchor(n) {
                Some(a) => {
                    namer
                        .bound
                        .entry(a)
                        .or_insert_with(|| (n.sem.clone(), n.label.clone()));
                }
                None if n.sem != root.sem && !n.children.is_empty() => {
                    namer.containers.push(Held {
                        id: n.sem.clone(),
                        label: n.label.clone(),
                        anchors: anchors(n),
                    });
                }
                None => {}
            }
            // a repeat's template names its instances, not one node: its
            // ids are never kept, only kept away from
            if let Some(repeat) = &n.repeat {
                namer.taken.insert(repeat.item.sem.clone());
            }
            stack.extend(n.children.iter());
        }
        namer
    }

    pub fn surface(&self) -> &SemId {
        &self.surface
    }

    /// the root's id and label: the current surface's root when it holds
    /// children.
    pub fn root(&mut self, fallback_label: Option<String>) -> (SemId, Option<String>) {
        match self.root.take() {
            Some((id, label)) => {
                self.given.insert(id.clone());
                (id, label.or(fallback_label))
            }
            None => (self.derive("card"), fallback_label),
        }
    }

    /// the id and label of a node bound to `anchor`: the current
    /// surface's node for it, or one named after `app_node`.
    pub fn bound(&mut self, anchor: &Anchor, name: &str, label: String) -> (SemId, String) {
        if let Some((id, kept)) = self.bound.get(anchor) {
            if !self.given.contains(id) {
                let id = id.clone();
                self.given.insert(id.clone());
                return (id, kept.clone().unwrap_or(label));
            }
        }
        (self.derive(name), label)
    }

    /// the id and label of a container holding `held`: the current
    /// container that held at least half of it (the one that held the
    /// most, then the one that held least else), or one named `name`.
    /// containers are named from the inside out, so a group takes its
    /// own id before a column that held it can.
    pub fn container(
        &mut self,
        held: &BTreeSet<Anchor>,
        name: &str,
        label: Option<String>,
    ) -> (SemId, Option<String>) {
        let mut best: Option<((f32, f32), usize)> = None;
        for (at, c) in self.containers.iter().enumerate() {
            if self.given.contains(&c.id) {
                continue;
            }
            let shared = c.anchors.intersection(held).count() as f32;
            let kept = shared / held.len().max(1) as f32;
            let tight = shared / c.anchors.union(held).count().max(1) as f32;
            let score = (kept, tight);
            if kept >= SAME_CONTAINER && best.is_none_or(|(s, _)| score > s) {
                best = Some((score, at));
            }
        }
        match best.and_then(|(_, at)| self.containers.get(at)) {
            Some(c) => {
                let (id, kept) = (c.id.clone(), c.label.clone());
                self.given.insert(id.clone());
                (id, kept.or(label))
            }
            None => (self.derive(name), label),
        }
    }

    /// `<surface>.<name>`, or with a number after it when that id is
    /// taken.
    fn derive(&mut self, name: &str) -> SemId {
        let base = format!("{}.{name}", self.surface);
        let mut n = 1usize;
        loop {
            let text = match n {
                1 => base.clone(),
                n if base.ends_with(']') => format!("{base}.{n}"),
                n => format!("{base}_{n}"),
            };
            match SemId::parse(&text) {
                Ok(id) if !self.taken.contains(&id) && !self.given.contains(&id) => {
                    self.given.insert(id.clone());
                    return id;
                }
                Ok(_) => n += 1,
                Err(_) => return self.derive("node"),
            }
        }
    }
}

/// `proposed`, or `proposed_2` and on, the first that no node of `app`
/// is or starts with.
fn fresh_surface(app: &Snapshot) -> SemId {
    let mut n = 1usize;
    loop {
        let text = match n {
            1 => "proposed".to_owned(),
            n => format!("proposed_{n}"),
        };
        let clash = app.nodes().keys().any(|id| {
            let id = id.as_str();
            id == text || id.starts_with(&format!("{text}.")) || id.starts_with(&format!("{text}["))
        });
        if !clash {
            if let Ok(id) = SemId::name(&text) {
                return id;
            }
        }
        n += 1;
    }
}
