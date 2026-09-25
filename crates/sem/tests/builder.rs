//! `SnapshotBuilder`: every `SemError`, aggregation, defaults, inherited
//! `inert`, `secret`, `source`.

use std::collections::BTreeSet;

use sem::{
    ActionName, Actors, Confirm, Constraints, ParamSpec, Policy, Role, SemError, SemErrors, SemId,
    Semantics, Snapshot, SnapshotBuilder, StateRef, Value, ValueKind,
};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("valid id")
}

fn errors(b: SnapshotBuilder) -> Vec<SemError> {
    match b.finish() {
        Err(SemErrors(all)) => all,
        Ok(snap) => panic!("expected violations, got {snap:?}"),
    }
}

fn snapshot(b: SnapshotBuilder) -> Snapshot {
    b.finish().expect("valid projection")
}

#[test]
fn an_id_that_does_not_parse_is_invalid_id() {
    let mut b = SnapshotBuilder::new();
    b.node("Commit", Role::Group, "");
    let all = errors(b);
    assert!(matches!(&all[..], [SemError::InvalidId { text, .. }] if text == "Commit"));
}

#[test]
fn bad_state_keys_and_action_names_are_invalid_key() {
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Button, "a")
        .state("Text", "x")
        .state("ok", 1i64)
        .state("ok", 2i64)
        .action("Submit");
    let all = errors(b);
    assert_eq!(all.len(), 3, "{all:?}");
    assert!(all.iter().all(|e| matches!(e, SemError::InvalidKey { .. })));
    assert!(
        matches!(&all[1], SemError::InvalidKey { text, reason } if text == "ok" && reason.contains("twice"))
    );
}

#[test]
fn a_repeated_id_is_duplicate_id() {
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Group, "first");
    b.node("a", Role::Group, "second");
    assert_eq!(errors(b), [SemError::DuplicateId(id("a"))]);
}

#[test]
fn a_parent_that_is_not_projected_is_unknown_parent() {
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Group, "").parent("ghost");
    assert_eq!(
        errors(b),
        [SemError::UnknownParent {
            node: id("a"),
            parent: id("ghost")
        }]
    );
}

#[test]
fn a_loop_in_the_parent_chain_is_one_parent_cycle() {
    let mut b = SnapshotBuilder::new();
    b.node("root", Role::Group, "");
    b.node("b", Role::Group, "").parent("c");
    b.node("c", Role::Group, "").parent("d");
    b.node("d", Role::Group, "").parent("b");
    b.node("leaf", Role::Group, "").parent("c");
    b.node("me", Role::Group, "").parent("me");
    assert_eq!(
        errors(b),
        [
            SemError::ParentCycle(id("b")),
            SemError::ParentCycle(id("me"))
        ]
    );
}

#[test]
fn a_read_of_missing_state_is_dangling_read() {
    let mut b = SnapshotBuilder::new();
    b.node("changes", Role::List, "changes")
        .state("staged", Value::Set(BTreeSet::new()));
    b.node("commit.submit", Role::Button, "commit")
        .action("submit")
        .reads("changes:staged")
        .reads("changes:unstaged")
        .reads("ghost:text");
    let submit = ActionName::parse("submit").unwrap();
    assert_eq!(
        errors(b),
        [
            SemError::DanglingRead {
                node: id("commit.submit"),
                action: submit.clone(),
                read: StateRef::parse("changes:unstaged").unwrap(),
            },
            SemError::DanglingRead {
                node: id("commit.submit"),
                action: submit,
                read: StateRef::parse("ghost:text").unwrap(),
            },
        ]
    );
}

#[test]
fn an_interactive_role_without_label_is_missing_label() {
    let mut b = SnapshotBuilder::new();
    b.node("empty", Role::Button, "");
    b.node("blank", Role::TextInput, "  ");
    b.node("group", Role::Group, "");
    b.node("row", Role::ListItem, "");
    assert_eq!(
        errors(b),
        [
            SemError::MissingLabel(id("empty")),
            SemError::MissingLabel(id("blank"))
        ]
    );
}

#[test]
fn every_interactive_role_needs_a_label() {
    for role in Role::ALL {
        let mut b = SnapshotBuilder::new();
        b.node("x", role, "");
        assert_eq!(b.finish().is_err(), role.is_interactive(), "{role:?}");
    }
    let interactive: Vec<&str> = Role::ALL
        .into_iter()
        .filter(|r| r.is_interactive())
        .map(Role::as_str)
        .collect();
    assert_eq!(
        interactive,
        [
            "button",
            "toggle",
            "checkbox",
            "radio",
            "text_input",
            "slider",
            "select",
            "option",
            "tab",
            "tree_item",
            "menu_item",
            "link"
        ]
    );
}

#[test]
fn the_same_action_twice_is_duplicate_action() {
    let mut b = SnapshotBuilder::new();
    let mut n = b.node("a", Role::Button, "a");
    n.action("press");
    n.action("press");
    assert_eq!(
        errors(b),
        [SemError::DuplicateAction {
            node: id("a"),
            action: ActionName::parse("press").unwrap()
        }]
    );
}

#[test]
fn impossible_constraints_are_invalid_constraint() {
    let cases = [
        Constraints {
            range: Some((2.0, 1.0)),
            ..Constraints::default()
        },
        Constraints {
            range: Some((0.0, f64::NAN)),
            ..Constraints::default()
        },
        Constraints {
            range: Some((f64::NEG_INFINITY, 1.0)),
            ..Constraints::default()
        },
        Constraints {
            step: Some(0.0),
            ..Constraints::default()
        },
        Constraints {
            step: Some(-1.0),
            ..Constraints::default()
        },
        Constraints {
            options: Some(vec!["a".into(), "b".into(), "a".into()]),
            ..Constraints::default()
        },
    ];
    for c in cases {
        let mut b = SnapshotBuilder::new();
        b.node("s", Role::Slider, "s").constraints(c.clone());
        let all = errors(b);
        assert!(
            matches!(&all[..], [SemError::InvalidConstraint { node, .. }] if *node == id("s")),
            "{c:?} gave {all:?}"
        );
    }
    let mut b = SnapshotBuilder::new();
    b.node("s", Role::Slider, "s").constraints(Constraints {
        range: Some((1.0, 1.0)),
        step: Some(0.5),
        options: Some(vec![]),
        max_len: Some(0),
        ..Constraints::default()
    });
    assert!(b.finish().is_ok());
}

#[test]
fn focus_on_a_node_that_is_not_projected_is_unknown_focus() {
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Button, "a");
    b.focus("ghost");
    assert_eq!(errors(b), [SemError::UnknownFocus(id("ghost"))]);
}

#[test]
fn finish_reports_every_violation_together_in_order() {
    let mut b = SnapshotBuilder::new();
    b.node("ok", Role::Group, "");
    b.node("Bad", Role::Group, "").state("Key", 1i64);
    b.node("ok", Role::Group, "again");
    b.node("button", Role::Button, "")
        .parent("ghost")
        .action("go")
        .reads("ok:missing");
    b.focus("nowhere");
    let all = errors(b);
    assert_eq!(all.len(), 7, "{all:?}");
    assert!(matches!(&all[0], SemError::InvalidId { text, .. } if text == "Bad"));
    assert!(matches!(&all[1], SemError::InvalidKey { text, .. } if text == "Key"));
    assert_eq!(all[2], SemError::DuplicateId(id("ok")));
    assert!(matches!(&all[3], SemError::UnknownParent { .. }));
    assert_eq!(all[4], SemError::MissingLabel(id("button")));
    assert!(matches!(&all[5], SemError::DanglingRead { .. }));
    assert_eq!(all[6], SemError::UnknownFocus(id("nowhere")));
    let text = SemErrors(all).to_string();
    assert!(text.starts_with("7 semantic violation(s); invalid id"));
}

#[test]
fn a_node_takes_defaults_from_its_role() {
    let mut b = SnapshotBuilder::new();
    b.node("button", Role::Button, "go");
    b.node("group", Role::Group, "");
    let snap = snapshot(b);
    let button = snap.get(&id("button")).unwrap();
    assert!(button.enabled && !button.inert && button.focusable);
    assert!(button.parent.is_none() && button.description.is_none());
    assert_eq!(button.constraints, Constraints::default());
    assert!(!snap.get(&id("group")).unwrap().focusable);
}

#[test]
fn the_builder_keeps_every_declared_field() {
    let mut b = SnapshotBuilder::new();
    let form = id("form");
    b.node("form", Role::Group, "form");
    let policy = Policy {
        actors: Actors {
            agent: false,
            ..Actors::default()
        },
        confirm: Confirm::Always,
        destructive: true,
    };
    b.node(id("form.volume"), Role::Slider, "volume")
        .parent(&form)
        .description("output level")
        .state("value", 0.5)
        .state("muted", false)
        .constraints(Constraints {
            editable: true,
            range: Some((0.0, 1.0)),
            ..Constraints::default()
        })
        .enabled(false)
        .focusable(false)
        .action(ActionName::parse("set").unwrap())
        .label("set volume")
        .enabled(false)
        .reads(StateRef::parse("form.volume:value").unwrap())
        .param(ParamSpec {
            kind: ValueKind::Num,
            constrained: true,
        })
        .policy(policy);
    b.focus(id("form.volume"));
    let snap = snapshot(b);
    let n = snap.get(&id("form.volume")).unwrap();
    assert_eq!(n.parent, Some(form));
    assert_eq!(n.description.as_deref(), Some("output level"));
    let keys: Vec<&str> = n.state.keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, ["value", "muted"]);
    assert!(!n.enabled && !n.focusable);
    assert_eq!(n.constraints.range, Some((0.0, 1.0)));
    let a = &n.actions[0];
    assert_eq!(a.label.as_deref(), Some("set volume"));
    assert!(!a.enabled);
    assert_eq!(a.reads, [StateRef::parse("form.volume:value").unwrap()]);
    assert_eq!(a.param.map(|p| p.kind), Some(ValueKind::Num));
    assert_eq!(a.policy, policy);
    assert_eq!(snap.focused(), Some(&id("form.volume")));
}

#[test]
fn inert_is_inherited_from_any_ancestor() {
    let mut b = SnapshotBuilder::new();
    b.node("app.list.item", Role::ListItem, "a.rs")
        .parent("app.list");
    b.node("app", Role::Region, "app").inert(true);
    b.node("app.list", Role::List, "files").parent("app");
    b.node("dialog", Role::Dialog, "confirm");
    b.node("dialog.ok", Role::Button, "ok").parent("dialog");
    let snap = snapshot(b);
    for inert in ["app", "app.list", "app.list.item"] {
        assert!(snap.get(&id(inert)).unwrap().inert, "{inert}");
    }
    for live in ["dialog", "dialog.ok"] {
        assert!(!snap.get(&id(live)).unwrap().inert, "{live}");
    }
}

#[test]
fn a_secret_value_never_reaches_the_snapshot() {
    let mut b = SnapshotBuilder::new();
    b.node("login.password", Role::TextInput, "password")
        .state("text", "hunter2")
        .state("codes", vec!["111".to_owned(), "222".to_owned()])
        .state("revealed", false)
        .constraints(Constraints {
            editable: true,
            secret: true,
            ..Constraints::default()
        });
    let snap = snapshot(b);
    let n = snap.get(&id("login.password")).unwrap();
    assert_eq!(n.state.get("text"), Some(&Value::Int(7)));
    assert_eq!(n.state.get("codes"), Some(&Value::Int(2)));
    assert_eq!(n.state.get("revealed"), Some(&Value::Bool(false)));
    let dump = format!("{snap:?}");
    assert!(!dump.contains("hunter2") && !dump.contains("111"));
}

#[test]
fn source_points_at_the_projection_site() {
    let mut b = SnapshotBuilder::new();
    let line = line!();
    b.node("a", Role::Group, "");
    let snap = snapshot(b);
    let source = snap.get(&id("a")).unwrap().source.clone().unwrap();
    assert_eq!(source.file, file!());
    assert_eq!(source.line, line + 1);
    assert!(source.column >= 5, "column {}", source.column);
}

#[test]
fn a_semantics_impl_projects_through_the_builder() {
    struct Toggle {
        on: bool,
    }
    impl Semantics for Toggle {
        fn project(&self, b: &mut SnapshotBuilder) {
            b.node("dark_mode", Role::Toggle, "dark mode")
                .state("on", self.on)
                .action("toggle")
                .reads("dark_mode:on");
        }
    }
    let mut b = SnapshotBuilder::new();
    Toggle { on: true }.project(&mut b);
    let snap = snapshot(b);
    assert_eq!(snap.len(), 1);
    assert_eq!(
        snap.get(&id("dark_mode")).unwrap().state.get("on"),
        Some(&Value::Bool(true))
    );
}

#[test]
fn snapshots_and_errors_are_shareable_values() {
    fn shared<T: Send + Sync + Clone>() {}
    shared::<Snapshot>();
    shared::<SemErrors>();
    assert!(SnapshotBuilder::new().finish().unwrap().is_empty());
}

#[cfg(feature = "serde")]
#[test]
fn role_serde_names_match_the_stable_names() {
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error, StrDeserializer};

    let mut names = BTreeSet::new();
    for role in Role::ALL {
        let d: StrDeserializer<'_, Error> = role.as_str().into_deserializer();
        assert_eq!(Role::deserialize(d).unwrap(), role);
        assert!(names.insert(role.as_str()));
    }
    assert_eq!(names.len(), 33);
}

/// task-70: what a projection adds after the app's own reads the app's
/// declarations, and goes on declaring one of its nodes.
#[test]
fn a_node_declared_earlier_is_read_and_reopened() {
    let mut b = SnapshotBuilder::new();
    b.node("docs", Role::List, "docs");
    b.node("docs.item[1]", Role::ListItem, "one")
        .parent("docs")
        .state("status", "draft")
        .action("approve")
        .reads("docs.item[1]:status");
    let declared: Vec<&str> = b.declared().iter().map(|n| n.id.as_str()).collect();
    assert_eq!(declared, ["docs", "docs.item[1]"]);
    let reads = b.declared()[1].actions[0].reads.clone();
    assert!(b.reopen(&id("docs.item[2]")).is_none(), "never declared");
    let mut item = b.reopen(&id("docs.item[1]")).expect("declared");
    let mut both = item.action("approve_then_publish");
    for at in &reads {
        both.reads(at.clone());
    }
    let snap = snapshot(b);
    let node = snap.get(&id("docs.item[1]")).unwrap();
    let names: Vec<&str> = node.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["approve", "approve_then_publish"]);
    assert_eq!(node.actions[1].reads, reads);
    assert_eq!(node.label, "one", "the node is the same one");
}
