//! `Composite`: a sequence the person accepted, as one action of the
//! app: its name, its label, the steps it runs, and its declaration on
//! the app's own nodes.
//!
//! a composite is declared after the app's projection, from what the app
//! declared in it (`SnapshotBuilder::declared`, then `reopen`): over
//! items, on every item of the collection whose steps the app declares;
//! exact, on the node of its first step. it reads what its steps read,
//! it is on when its first step is (a later step may only be on after the
//! one before it ran: publishing needs the approval first; each is checked
//! when it comes), and its policy is the narrowest of theirs, so an agent
//! never runs a composite one of whose steps is kept from it.

use sem::{
    ActionDecl, ActionName, Actors, Confirm, Node, Policy, SemId, Snapshot, SnapshotBuilder,
    StateRef,
};

use super::{Shape, Target, item_of};

/// a sequence of steps the app runs as one action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composite {
    /// the action's name: the steps' actions joined by `_then_`
    /// (`preview_segment_then_apply_cut`).
    pub name: ActionName,
    /// what a person reads: the steps' labels joined by ", then "
    /// ("preview segment, then apply cut").
    pub label: String,
    pub shape: Shape,
}

impl Composite {
    /// the composite of `shape`, named and labelled from what `app`
    /// declares for each step; `taken` are the names already in use, and
    /// a name taken gets a number (`_2`, `_3`).
    pub fn new(shape: Shape, app: &Snapshot, taken: &[ActionName]) -> Result<Composite, String> {
        let joined: Vec<&str> = shape.steps.iter().map(|s| s.action.as_str()).collect();
        let base = joined.join("_then_");
        let mut name = ActionName::parse(&base).map_err(|e| e.to_string())?;
        let mut n = 2;
        while taken.contains(&name) {
            name = ActionName::parse(&format!("{base}_{n}")).map_err(|e| e.to_string())?;
            n += 1;
        }
        let labels: Vec<String> = shape.steps.iter().map(|s| label(s, app)).collect();
        Ok(Composite {
            name,
            label: labels.join(", then "),
            shape,
        })
    }

    /// declares this composite on every node of the projection so far
    /// that it fits (see the module); a node that declares an action of
    /// that name already keeps its own.
    pub fn declare(&self, b: &mut SnapshotBuilder) {
        let homes: Vec<(SemId, Option<String>)> = match self.shape.collection() {
            Some(base) => b
                .declared()
                .iter()
                .filter_map(|n| {
                    let (of, key) = item_of(&n.id)?;
                    (of == *base).then(|| (n.id.clone(), Some(key.to_owned())))
                })
                .collect(),
            None => self
                .shape
                .steps
                .first()
                .and_then(|s| s.target.node(None))
                .map(|home| vec![(home, None)])
                .unwrap_or_default(),
        };
        for (home, item) in homes {
            let nodes = b.declared();
            let taken = nodes
                .iter()
                .find(|n| n.id == home)
                .is_some_and(|n| n.actions.iter().any(|a| a.name == self.name));
            let Some(decl) = self.decl(nodes, item.as_deref()) else {
                continue;
            };
            if taken {
                continue;
            }
            let Some(mut node) = b.reopen(&home) else {
                continue;
            };
            let mut action = node.action(self.name.clone());
            action
                .label(self.label.as_str())
                .enabled(decl.enabled)
                .policy(decl.policy);
            for at in decl.reads {
                action.reads(at);
            }
        }
    }

    /// what this composite is on the node of `item`, from what the steps'
    /// nodes declare: every step's action has to be there.
    fn decl(&self, nodes: &[Node], item: Option<&str>) -> Option<ActionDecl> {
        let mut decl = ActionDecl::new(self.name.clone());
        decl.label = Some(self.label.clone());
        let mut actors = Actors::default();
        let mut confirm = Confirm::Never;
        let mut destructive = false;
        let mut reads: Vec<StateRef> = Vec::new();
        for (at, step) in self.shape.steps.iter().enumerate() {
            let id = step.target.node(item)?;
            let node = nodes.iter().find(|n| n.id == id)?;
            let declared = node.actions.iter().find(|a| a.name == step.action)?;
            if at == 0 {
                decl.enabled = declared.enabled && node.enabled;
            }
            let p = declared.policy;
            actors = Actors {
                human: actors.human && p.actors.human,
                agent: actors.agent && p.actors.agent,
                assistive: actors.assistive && p.actors.assistive,
                test: actors.test && p.actors.test,
            };
            confirm = stronger(confirm, p.confirm);
            destructive |= p.destructive;
            for at in &declared.reads {
                if !reads.contains(at) {
                    reads.push(at.clone());
                }
            }
        }
        decl.reads = reads;
        decl.policy = Policy {
            actors,
            confirm,
            destructive,
        };
        Some(decl)
    }

    /// the steps this composite runs when it is asked of `home`, the node
    /// it is declared on: each step's node and action, in order. `None`
    /// when `home` is not where it is declared: an item of its collection,
    /// or the node of its first step.
    pub fn calls(&self, home: &SemId) -> Option<Vec<(SemId, ActionName)>> {
        let item = match self.shape.collection() {
            Some(base) => match item_of(home) {
                Some((of, key)) if of == *base => Some(key),
                _ => return None,
            },
            None => {
                let first = self.shape.steps.first()?.target.node(None)?;
                if first != *home {
                    return None;
                }
                None
            }
        };
        self.shape
            .steps
            .iter()
            .map(|s| Some((s.target.node(item)?, s.action.clone())))
            .collect()
    }

    /// the node a surface binds this composite on: the template of the
    /// items of its collection (`silences.cut[{item}]`), or the node of
    /// its first step.
    pub fn template(&self) -> Option<SemId> {
        match self.shape.collection() {
            Some(base) => Some(Target::Item(base.clone()).template()),
            None => self.shape.steps.first()?.target.node(None),
        }
    }

    /// true when `action` on `home` is this composite.
    pub fn is(&self, home: &SemId, action: &ActionName) -> bool {
        *action == self.name && self.calls(home).is_some()
    }
}

/// the stronger of two confirmations: never, for agents, always.
fn stronger(a: Confirm, b: Confirm) -> Confirm {
    let rank = |c: Confirm| match c {
        Confirm::Never => 0,
        Confirm::ForAgents => 1,
        Confirm::Always => 2,
    };
    if rank(b) > rank(a) { b } else { a }
}

/// a step's label: the one the app gives its action, on the node or on
/// an item of the collection, or the action's name in words.
fn label(step: &super::Step, app: &Snapshot) -> String {
    let declared = app
        .nodes()
        .values()
        .filter(|n| match &step.target {
            Target::Node(id) => n.id == *id,
            Target::Item(base) => item_of(&n.id).is_some_and(|(b, _)| b == *base),
        })
        .flat_map(|n| n.actions.iter())
        .find(|a| a.name == step.action)
        .and_then(|a| a.label.clone());
    declared.unwrap_or_else(|| step.action.as_str().replace('_', " "))
}
