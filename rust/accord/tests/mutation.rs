//! `Snapshot::diff` and `Change`.

use std::collections::BTreeSet;

use proptest::prelude::*;
use accord::{Change, Constraints, Role, SemId, Snapshot, SnapshotBuilder, StateRef, Value};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("valid id")
}

fn at(text: &str) -> StateRef {
    StateRef::parse(text).expect("valid state ref")
}

fn set(items: &[&str]) -> Value {
    Value::Set(items.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>())
}

fn list(items: &[&str]) -> Value {
    Value::List(items.iter().map(|s| s.to_string()).collect())
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// one node `a` holding one state value `v`.
fn holding(value: Value) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Group, "a").state("v", value);
    b.finish().unwrap()
}

fn diff_values(before: Value, after: Value) -> Vec<Change> {
    Snapshot::diff(&holding(before), &holding(after))
}

#[test]
fn a_set_that_loses_b_reports_removed_b() {
    assert_eq!(
        diff_values(set(&["A", "B", "C"]), set(&["A", "C"])),
        [Change::StateChanged {
            at: at("a:v"),
            before: Some(set(&["A", "B", "C"])),
            after: Some(set(&["A", "C"])),
            added: vec![],
            removed: strings(&["B"]),
        }]
    );
}

#[test]
fn a_list_reports_items_as_a_multiset_in_order() {
    let grown = diff_values(list(&["a", "b"]), list(&["a", "b", "c", "a"]));
    assert!(matches!(
        &grown[..],
        [Change::StateChanged { added, removed, .. }] if *added == strings(&["c", "a"]) && removed.is_empty()
    ));
    let shrunk = diff_values(list(&["a", "b", "a"]), list(&["b"]));
    assert!(matches!(
        &shrunk[..],
        [Change::StateChanged { added, removed, .. }] if added.is_empty() && *removed == strings(&["a", "a"])
    ));
    let reordered = diff_values(list(&["a", "b"]), list(&["b", "a"]));
    assert!(matches!(
        &reordered[..],
        [Change::StateChanged { before, after, added, removed, .. }]
            if *before == Some(list(&["a", "b"])) && *after == Some(list(&["b", "a"]))
                && added.is_empty() && removed.is_empty()
    ));
}

#[test]
fn a_scalar_reports_before_and_after_only() {
    for (before, after) in [
        (Value::Text("fix".into()), Value::Text("fix parser".into())),
        (Value::Int(1), Value::Int(2)),
        (Value::Num(0.0), Value::Num(-0.0)),
        (Value::Bool(true), Value::Bool(false)),
        (Value::Text("3".into()), Value::Int(3)),
        (set(&["a"]), Value::Text("a".into())),
    ] {
        assert_eq!(
            diff_values(before.clone(), after.clone()),
            [Change::StateChanged {
                at: at("a:v"),
                before: Some(before),
                after: Some(after),
                added: vec![],
                removed: vec![],
            }]
        );
    }
    assert!(diff_values(Value::Num(f64::NAN), Value::Num(f64::NAN)).is_empty());
}

#[test]
fn a_state_key_that_appears_or_goes_counts_as_empty_on_the_other_side() {
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Group, "a")
        .state("files", set(&["x", "y"]))
        .state("n", 1i64);
    let full = b.finish().unwrap();
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Group, "a");
    let bare = b.finish().unwrap();
    assert_eq!(
        Snapshot::diff(&bare, &full),
        [
            Change::StateChanged {
                at: at("a:files"),
                before: None,
                after: Some(set(&["x", "y"])),
                added: strings(&["x", "y"]),
                removed: vec![],
            },
            Change::StateChanged {
                at: at("a:n"),
                before: None,
                after: Some(Value::Int(1)),
                added: vec![],
                removed: vec![],
            },
        ]
    );
    let back = Snapshot::diff(&full, &bare);
    assert!(matches!(
        &back[0],
        Change::StateChanged { after: None, removed, .. } if *removed == strings(&["x", "y"])
    ));
}

#[test]
fn nodes_added_and_removed_are_named() {
    let mut b = SnapshotBuilder::new();
    b.node("keep", Role::Group, "keep");
    b.node("gone", Role::Group, "gone");
    let before = b.finish().unwrap();
    let mut b = SnapshotBuilder::new();
    b.node("keep", Role::Group, "keep");
    b.node("new", Role::Button, "new");
    let after = b.finish().unwrap();
    let new = after.get(&id("new")).unwrap().clone();
    assert_eq!(
        Snapshot::diff(&before, &after),
        [
            Change::NodeRemoved(id("gone")),
            Change::NodeAdded(Box::new(new))
        ]
    );
}

/// a node that came carries every field as `after` holds it, its source
/// included, and a secret field as its length: the delta holds what a
/// snapshot would, so nobody needs a second observation to know it.
#[test]
fn a_node_added_is_the_node_as_after_holds_it() {
    let project = |with_pin: bool| {
        let mut b = SnapshotBuilder::new();
        b.node("form", Role::Group, "login");
        if with_pin {
            b.node("form.pin", Role::TextInput, "pin")
                .parent("form")
                .description("four digits")
                .state("text", "1234")
                .state("tries", 2i64)
                .constraints(Constraints {
                    secret: true,
                    editable: true,
                    ..Constraints::default()
                })
                .focusable(true)
                .action("set")
                .reads("form.pin:tries");
        }
        b.finish().unwrap()
    };
    let (before, after) = (project(false), project(true));
    let pin = after.get(&id("form.pin")).unwrap();
    let delta = Snapshot::diff(&before, &after);
    let [Change::NodeAdded(node)] = &delta[..] else {
        panic!("one node added: {delta:?}");
    };
    assert_eq!(**node, *pin);
    assert_eq!(node.parent, Some(id("form")));
    assert_eq!(node.description.as_deref(), Some("four digits"));
    assert_eq!(node.state.get("text"), Some(&Value::Int(4)), "a length");
    assert_eq!(node.state.get("tries"), Some(&Value::Int(2)));
    assert_eq!(node.actions, pin.actions);
    assert!(node.constraints.secret && node.focusable);
    assert!(node.source.is_some() && node.source == pin.source);
    assert!(!format!("{delta:?}").contains("1234"));
    // and the other way it is only named.
    assert_eq!(
        Snapshot::diff(&after, &before),
        [Change::NodeRemoved(id("form.pin"))]
    );
}

#[test]
fn flags_report_only_what_changed() {
    let project = |enabled: bool, modal: bool| {
        let mut b = SnapshotBuilder::new();
        b.node("app", Role::Region, "app").inert(modal);
        b.node("app.save", Role::Button, "save")
            .parent("app")
            .enabled(enabled);
        b.finish().unwrap()
    };
    assert_eq!(
        Snapshot::diff(&project(true, false), &project(false, false)),
        [Change::FlagsChanged {
            node: id("app.save"),
            enabled: Some(false),
            inert: None,
            focusable: None,
        }]
    );
    assert_eq!(
        Snapshot::diff(&project(true, false), &project(true, true)),
        [
            Change::FlagsChanged {
                node: id("app"),
                enabled: None,
                inert: Some(true),
                focusable: None,
            },
            Change::FlagsChanged {
                node: id("app.save"),
                enabled: None,
                inert: Some(true),
                focusable: None,
            },
        ]
    );
}

#[test]
fn every_other_node_field_has_its_change() {
    let project = |label: &str,
                   role: Role,
                   parent: &str,
                   go_enabled: bool,
                   max: usize,
                   desc: Option<&str>| {
        let mut b = SnapshotBuilder::new();
        b.node("p", Role::Group, "p");
        b.node("q", Role::Group, "q");
        let mut n = b.node("x", role, label);
        n.parent(parent).constraints(Constraints {
            max_len: Some(max),
            ..Constraints::default()
        });
        if let Some(d) = desc {
            n.description(d);
        }
        n.action("go").enabled(go_enabled);
        b.finish().unwrap()
    };
    let base = project("old", Role::Button, "p", true, 10, None);
    let changed = project("new", Role::Link, "q", false, 20, Some("hint"));
    let delta = Snapshot::diff(&base, &changed);
    let kinds: Vec<&str> = delta
        .iter()
        .map(|c| match c {
            Change::RoleChanged { before, after, .. } => {
                assert_eq!((*before, *after), (Role::Button, Role::Link));
                "role"
            }
            Change::LabelChanged { before, after, .. } => {
                assert_eq!((before.as_str(), after.as_str()), ("old", "new"));
                "label"
            }
            Change::DescriptionChanged { after, .. } => {
                assert_eq!(after.as_deref(), Some("hint"));
                "description"
            }
            Change::ParentChanged { before, after, .. } => {
                assert_eq!(
                    (before.clone(), after.clone()),
                    (Some(id("p")), Some(id("q")))
                );
                "parent"
            }
            Change::ActionsChanged { before, after, .. } => {
                assert!(before[0].enabled && !after[0].enabled);
                "actions"
            }
            Change::ConstraintsChanged { after, .. } => {
                assert_eq!(after.max_len, Some(20));
                "constraints"
            }
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "role",
            "label",
            "description",
            "parent",
            "actions",
            "constraints"
        ]
    );
}

#[test]
fn sibling_order_is_a_reorder_and_an_insertion_is_not() {
    let project = |items: &[&str], roots_flipped: bool| {
        let mut b = SnapshotBuilder::new();
        if roots_flipped {
            b.node("bar", Role::Toolbar, "tools");
        }
        b.node("list", Role::List, "files");
        for item in items {
            b.node(id("list").key(*item), Role::ListItem, *item)
                .parent("list");
        }
        if !roots_flipped {
            b.node("bar", Role::Toolbar, "tools");
        }
        b.finish().unwrap()
    };
    let base = project(&["a", "b"], false);
    assert_eq!(
        Snapshot::diff(&base, &project(&["b", "a"], false)),
        [Change::Reordered {
            parent: Some(id("list"))
        }]
    );
    let inserted = project(&["a", "c", "b"], false);
    let c = inserted.get(&id("list[c]")).unwrap().clone();
    assert_eq!(
        Snapshot::diff(&base, &inserted),
        [Change::NodeAdded(Box::new(c))]
    );
    assert_eq!(
        Snapshot::diff(&base, &project(&["a", "b"], true)),
        [Change::Reordered { parent: None }]
    );
}

#[test]
fn focus_moves_are_reported() {
    let project = |focus: &str| {
        let mut b = SnapshotBuilder::new();
        b.node("a", Role::Button, "a");
        b.node("b", Role::Button, "b");
        b.focus(focus);
        b.finish().unwrap()
    };
    assert_eq!(
        Snapshot::diff(&project("a"), &project("b")),
        [Change::FocusChanged {
            before: Some(id("a")),
            after: Some(id("b"))
        }]
    );
}

#[test]
fn where_a_node_was_projected_is_never_a_change() {
    let mut here = SnapshotBuilder::new();
    here.node("a", Role::Button, "a");
    let here = here.finish().unwrap();
    let there = {
        let mut b = SnapshotBuilder::new();
        b.node("a", Role::Button, "a");
        b.finish().unwrap()
    };
    assert_ne!(
        here.get(&id("a")).unwrap().source,
        there.get(&id("a")).unwrap().source
    );
    assert!(Snapshot::diff(&here, &there).is_empty());
}

#[test]
fn a_secret_diff_carries_lengths_only() {
    let project = |password: &str| {
        let mut b = SnapshotBuilder::new();
        b.node("password", Role::TextInput, "password")
            .state("text", password)
            .constraints(Constraints {
                secret: true,
                ..Constraints::default()
            });
        b.finish().unwrap()
    };
    let delta = Snapshot::diff(&project("hunter2"), &project("hunter22"));
    assert_eq!(
        delta,
        [Change::StateChanged {
            at: at("password:text"),
            before: Some(Value::Int(7)),
            after: Some(Value::Int(8)),
            added: vec![],
            removed: vec![],
        }]
    );
    assert!(!format!("{delta:?}").contains("hunter"));
}

/// a small random tree: node `i` may only hang under a node `< i`, so no
/// generated tree loops.
#[derive(Clone, Debug)]
struct Tree {
    nodes: Vec<Leaf>,
    order: Vec<usize>,
    focus: Option<usize>,
}

#[derive(Clone, Debug)]
struct Leaf {
    parent: Option<usize>,
    label: u8,
    value: i64,
    enabled: bool,
    inert: bool,
    alive: bool,
}

impl Tree {
    fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        for &i in &self.order {
            let leaf = &self.nodes[i];
            if !leaf.alive {
                continue;
            }
            let mut n = b.node(format!("n{i}"), Role::Group, format!("l{}", leaf.label));
            n.state("v", leaf.value)
                .enabled(leaf.enabled)
                .inert(leaf.inert);
            if let Some(p) = leaf.parent {
                n.parent(format!("n{p}"));
            }
        }
        if let Some(f) = self.focus.filter(|&f| self.nodes[f].alive) {
            b.focus(format!("n{f}"));
        }
        b.finish().expect("generated trees are valid")
    }

    fn apply(&mut self, edit: &Edit) {
        let n = self.nodes.len();
        match *edit {
            Edit::Enabled(i) => self.nodes[i % n].enabled ^= true,
            Edit::Inert(i) => self.nodes[i % n].inert ^= true,
            Edit::Label(i, l) => self.nodes[i % n].label = l,
            Edit::Value(i, v) => self.nodes[i % n].value = v,
            Edit::Parent(i, p) => {
                let i = i % n;
                self.nodes[i].parent = p.filter(|_| i > 0).map(|p| p % i.max(1));
                if self.nodes[i].parent.is_some_and(|p| !self.nodes[p].alive) {
                    self.nodes[i].parent = None;
                }
            }
            Edit::Swap(i, j) => self.order.swap(i % n, j % n),
            Edit::Remove(i) => {
                let i = i % n;
                let has_child = self.nodes.iter().any(|l| l.alive && l.parent == Some(i));
                if !has_child {
                    self.nodes[i].alive = false;
                }
            }
            Edit::Focus(f) => self.focus = f.map(|f| f % n),
        }
    }
}

#[derive(Clone, Debug)]
enum Edit {
    Enabled(usize),
    Inert(usize),
    Label(usize, u8),
    Value(usize, i64),
    Parent(usize, Option<usize>),
    Swap(usize, usize),
    Remove(usize),
    Focus(Option<usize>),
}

fn tree() -> impl Strategy<Value = Tree> {
    (1usize..7).prop_flat_map(|n| {
        let leaf = (
            proptest::option::of(0usize..8),
            0u8..3,
            0i64..3,
            any::<bool>(),
            prop::bool::weighted(0.2),
        );
        (
            prop::collection::vec(leaf, n),
            Just((0..n).collect::<Vec<_>>()).prop_shuffle(),
            proptest::option::of(0..n),
        )
            .prop_map(|(raw, order, focus)| Tree {
                nodes: raw
                    .into_iter()
                    .enumerate()
                    .map(|(i, (p, label, value, enabled, inert))| Leaf {
                        parent: p.filter(|_| i > 0).map(|p| p % i.max(1)),
                        label,
                        value,
                        enabled,
                        inert,
                        alive: true,
                    })
                    .collect(),
                order,
                focus,
            })
    })
}

fn edit() -> impl Strategy<Value = Edit> {
    prop_oneof![
        (0usize..8).prop_map(Edit::Enabled),
        (0usize..8).prop_map(Edit::Inert),
        (0usize..8, 0u8..3).prop_map(|(i, l)| Edit::Label(i, l)),
        (0usize..8, 0i64..3).prop_map(|(i, v)| Edit::Value(i, v)),
        (0usize..8, proptest::option::of(0usize..8)).prop_map(|(i, p)| Edit::Parent(i, p)),
        (0usize..8, 0usize..8).prop_map(|(i, j)| Edit::Swap(i, j)),
        (0usize..8).prop_map(Edit::Remove),
        proptest::option::of(0usize..8).prop_map(Edit::Focus),
    ]
}

proptest! {
    #[test]
    fn diff_is_empty_exactly_when_snapshots_and_revisions_are_equal(
        base in tree(),
        edits in prop::collection::vec(edit(), 0..4),
    ) {
        let mut other = base.clone();
        for e in &edits {
            other.apply(e);
        }
        let a = base.snapshot();
        let b = other.snapshot();
        let delta = Snapshot::diff(&a, &b);
        prop_assert_eq!(delta.is_empty(), a == b, "{:?}", delta);
        prop_assert_eq!(a == b, a.revision() == b.revision());
        prop_assert!(Snapshot::diff(&a, &a).is_empty());
        prop_assert_eq!(Snapshot::diff(&b, &a).is_empty(), delta.is_empty());
    }
}
