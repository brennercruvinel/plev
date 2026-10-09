//! the capability registry: what an app declares it has, the formal space
//! a generated surface is made in.
//!
//! it is built from the app's own projection (`Capabilities::from_snapshot`),
//! never declared a second time by hand: every `StateRef` of the snapshot
//! with the kind of its value, and every `ActionDecl` with its target, its
//! `ParamSpec`, its `Policy` and its label. an IR can only reference what
//! is here.

use std::collections::{BTreeMap, BTreeSet};

use accord::{ActionName, ParamSpec, Policy, SemId, Snapshot, StateRef, ValueKind};
use serde::Serialize;

/// one piece of state the app shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DataCap {
    pub at: StateRef,
    pub kind: ValueKind,
    /// the label of the node that holds it.
    pub label: String,
}

/// one action the app runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ActionCap {
    pub target: SemId,
    pub action: ActionName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<ParamSpec>,
    pub policy: Policy,
    /// the action's own label, or its node's.
    pub label: String,
}

impl ActionCap {
    /// an action with no parameter, the default policy, and its name as
    /// its label.
    pub fn new(target: SemId, action: ActionName) -> ActionCap {
        let label = action.as_str().to_owned();
        ActionCap {
            target,
            action,
            param: None,
            policy: Policy::default(),
            label,
        }
    }
}

/// the registry: data and actions in projection order, each found by its
/// reference.
///
/// a secret field is in it as the snapshot keeps it, its length, so no
/// generated surface can show or bind a secret as text.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Capabilities {
    data: Vec<DataCap>,
    actions: Vec<ActionCap>,
    #[serde(skip)]
    data_at: BTreeMap<StateRef, usize>,
    #[serde(skip)]
    action_at: BTreeMap<(SemId, ActionName), usize>,
    #[serde(skip)]
    nodes: BTreeSet<SemId>,
}

impl Capabilities {
    /// every state and every action `snap` declares, whatever their
    /// `enabled` or `inert` now: those move at runtime and the dispatcher
    /// checks them when an action is asked for.
    pub fn from_snapshot(snap: &Snapshot) -> Capabilities {
        let mut caps = Capabilities::default();
        for node in snap.nodes().values() {
            for (key, value) in &node.state {
                caps = caps.with_data(DataCap {
                    at: StateRef::new(node.id.clone(), key.clone()),
                    kind: value.kind(),
                    label: node.label.clone(),
                });
            }
            for decl in &node.actions {
                caps = caps.with_action(ActionCap {
                    target: node.id.clone(),
                    action: decl.name.clone(),
                    param: decl.param,
                    policy: decl.policy,
                    label: decl.label.clone().unwrap_or_else(|| node.label.clone()),
                });
            }
        }
        caps
    }

    /// adds a piece of state by hand (a test builds a registry this way);
    /// one with the same reference is replaced.
    pub fn with_data(mut self, cap: DataCap) -> Capabilities {
        self.nodes.insert(cap.at.node.clone());
        match self.data_at.get(&cap.at) {
            Some(&at) => self.data[at] = cap,
            None => {
                self.data_at.insert(cap.at.clone(), self.data.len());
                self.data.push(cap);
            }
        }
        self
    }

    /// adds an action by hand; one with the same target and name is
    /// replaced.
    pub fn with_action(mut self, cap: ActionCap) -> Capabilities {
        self.nodes.insert(cap.target.clone());
        let key = (cap.target.clone(), cap.action.clone());
        match self.action_at.get(&key) {
            Some(&at) => self.actions[at] = cap,
            None => {
                self.action_at.insert(key, self.actions.len());
                self.actions.push(cap);
            }
        }
        self
    }

    /// every piece of state, in projection order.
    pub fn data(&self) -> &[DataCap] {
        &self.data
    }

    /// every action, in projection order.
    pub fn actions(&self) -> &[ActionCap] {
        &self.actions
    }

    pub fn datum(&self, at: &StateRef) -> Option<&DataCap> {
        self.data_at.get(at).and_then(|&i| self.data.get(i))
    }

    pub fn action(&self, target: &SemId, action: &ActionName) -> Option<&ActionCap> {
        let key = (target.clone(), action.clone());
        self.action_at.get(&key).and_then(|&i| self.actions.get(i))
    }

    /// every node that holds some state or some action, in id order.
    pub fn nodes(&self) -> impl Iterator<Item = &SemId> {
        self.nodes.iter()
    }

    /// true when `id` holds some state or some action here: an app node.
    pub fn has_node(&self, id: &SemId) -> bool {
        self.nodes.contains(id)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.actions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use accord::{Confirm, Constraints, Role, SnapshotBuilder};

    use super::*;

    fn git() -> Snapshot {
        let mut b = SnapshotBuilder::new();
        b.node("changes", Role::List, "changes")
            .state("staged", BTreeSet::from(["a.rs".to_owned()]));
        let mut file = b.node("changes.file[a.rs]", Role::ListItem, "a.rs");
        file.parent("changes").state("staged", true);
        file.action("discard")
            .reads("changes.file[a.rs]:staged")
            .policy(Policy {
                confirm: Confirm::ForAgents,
                destructive: true,
                ..Policy::default()
            });
        b.node("commit.message", Role::TextInput, "commit message")
            .state("text", "fix")
            .constraints(Constraints {
                editable: true,
                ..Constraints::default()
            })
            .action("set_value")
            .label("write the message")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: true,
            });
        b.finish().unwrap()
    }

    fn at(text: &str) -> StateRef {
        text.parse().unwrap()
    }

    #[test]
    fn the_registry_is_the_projection() {
        let caps = Capabilities::from_snapshot(&git());
        let data: Vec<(String, ValueKind)> = caps
            .data()
            .iter()
            .map(|d| (d.at.to_string(), d.kind))
            .collect();
        assert_eq!(
            data,
            [
                ("changes:staged".to_owned(), ValueKind::Set),
                ("changes.file[a.rs]:staged".to_owned(), ValueKind::Bool),
                ("commit.message:text".to_owned(), ValueKind::Text),
            ]
        );
        let discard = caps
            .action(
                &"changes.file[a.rs]".parse().unwrap(),
                &"discard".parse().unwrap(),
            )
            .unwrap();
        assert!(discard.policy.destructive);
        assert_eq!(discard.policy.confirm, Confirm::ForAgents);
        assert_eq!(discard.param, None);
        assert_eq!(discard.label, "a.rs");
        let set = caps
            .action(
                &"commit.message".parse().unwrap(),
                &"set_value".parse().unwrap(),
            )
            .unwrap();
        assert_eq!(set.param.map(|p| p.kind), Some(ValueKind::Text));
        assert_eq!(set.label, "write the message");
        assert_eq!(caps.datum(&at("changes:staged")).unwrap().label, "changes");
        assert!(caps.datum(&at("changes:files")).is_none());
        assert!(caps.has_node(&"changes.file[a.rs]".parse().unwrap()));
        assert!(!caps.has_node(&"commit.submit".parse().unwrap()));
    }

    #[test]
    fn a_registry_by_hand_replaces_what_it_repeats() {
        let caps = Capabilities::default()
            .with_data(DataCap {
                at: at("todo.input:text"),
                kind: ValueKind::Text,
                label: "new todo".to_owned(),
            })
            .with_data(DataCap {
                at: at("todo.input:text"),
                kind: ValueKind::Int,
                label: "new todo".to_owned(),
            })
            .with_action(ActionCap::new(
                "todo.input".parse().unwrap(),
                "submit".parse().unwrap(),
            ));
        assert_eq!(caps.data().len(), 1);
        assert_eq!(
            caps.datum(&at("todo.input:text")).unwrap().kind,
            ValueKind::Int
        );
        assert_eq!(caps.actions()[0].label, "submit");
        assert_eq!(caps.nodes().count(), 1);
    }

    #[test]
    fn an_empty_projection_declares_nothing() {
        let caps = Capabilities::from_snapshot(&Snapshot::default());
        assert!(caps.is_empty());
        let json = serde_json::to_value(&caps).unwrap();
        assert_eq!(json, serde_json::json!({ "data": [], "actions": [] }));
    }
}
