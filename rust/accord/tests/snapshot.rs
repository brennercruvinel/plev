//! `Snapshot`: queries, `subtree` and the stable `Revision`.

use std::collections::BTreeSet;

use accord::{Constraints, Role, SemId, Snapshot, SnapshotBuilder, StateRef, Value};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("valid id")
}

fn names<'a>(nodes: impl Iterator<Item = &'a accord::Node>) -> Vec<&'a str> {
    nodes.map(|n| n.id.as_str()).collect()
}

/// the git form of the thesis, with every knob a revision must see.
#[derive(Clone)]
struct Git {
    staged: Vec<&'static str>,
    message: &'static str,
    submit_label: &'static str,
    submit_enabled: bool,
    max_len: Option<usize>,
    message_parent: &'static str,
    changes_first: bool,
    description: Option<&'static str>,
    focus: Option<&'static str>,
}

impl Git {
    fn base() -> Self {
        Git {
            staged: vec!["lexer.rs", "parser.rs"],
            message: "fix parser",
            submit_label: "commit",
            submit_enabled: true,
            max_len: Some(72),
            message_parent: "commit",
            changes_first: true,
            description: None,
            focus: Some("commit.message"),
        }
    }

    fn project(&self, b: &mut SnapshotBuilder) {
        let staged: BTreeSet<String> = self.staged.iter().map(|s| s.to_string()).collect();
        if self.changes_first {
            b.node("changes", Role::List, "changes")
                .state("staged", staged.clone());
        }
        b.node("commit", Role::Group, "commit");
        b.node("other", Role::Group, "other");
        let mut message = b.node("commit.message", Role::TextInput, "message");
        message
            .parent(self.message_parent)
            .state("text", self.message)
            .constraints(Constraints {
                editable: true,
                max_len: self.max_len,
                ..Constraints::default()
            });
        if let Some(d) = self.description {
            message.description(d);
        }
        b.node("commit.submit", Role::Button, self.submit_label)
            .parent("commit")
            .action("submit")
            .enabled(self.submit_enabled)
            .reads("changes:staged")
            .reads("commit.message:text");
        if !self.changes_first {
            b.node("changes", Role::List, "changes")
                .state("staged", staged);
        }
        if let Some(f) = self.focus {
            b.focus(f);
        }
    }

    fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish().expect("valid projection")
    }
}

#[test]
fn children_follow_projection_order_even_before_the_parent_is_declared() {
    let mut b = SnapshotBuilder::new();
    b.node("list.b", Role::ListItem, "b").parent("list");
    b.node("list", Role::List, "files");
    b.node("list.a", Role::ListItem, "a").parent("list");
    b.node("list.c", Role::ListItem, "c").parent("list");
    b.node("footer", Role::Status, "3 files");
    let snap = b.finish().unwrap();
    assert_eq!(
        names(snap.children(&id("list"))),
        ["list.b", "list.a", "list.c"]
    );
    assert_eq!(names(snap.children(&id("list.a"))), Vec::<&str>::new());
    assert_eq!(names(snap.children(&id("ghost"))), Vec::<&str>::new());
    assert_eq!(names(snap.roots()), ["list", "footer"]);
}

#[test]
fn value_reads_through_a_state_ref() {
    let snap = Git::base().snapshot();
    let staged = StateRef::parse("changes:staged").unwrap();
    let set: BTreeSet<String> = ["lexer.rs".to_owned(), "parser.rs".to_owned()].into();
    assert_eq!(snap.value(&staged), Some(&Value::Set(set)));
    assert_eq!(
        snap.value(&StateRef::parse("commit.message:text").unwrap()),
        Some(&Value::Text("fix parser".into()))
    );
    assert_eq!(snap.value(&StateRef::parse("changes:ghost").unwrap()), None);
    assert_eq!(snap.value(&StateRef::parse("ghost:text").unwrap()), None);
}

#[test]
fn subtree_scopes_a_node_and_its_descendants() {
    let mut b = SnapshotBuilder::new();
    b.node("app", Role::Window, "plev");
    b.node("app.side", Role::Region, "side").parent("app");
    b.node("app.main", Role::Region, "main").parent("app");
    b.node("files", Role::List, "files")
        .parent("app.side")
        .state("staged", Value::Set(BTreeSet::new()));
    b.node(id("files").key("a.rs"), Role::ListItem, "a.rs")
        .parent("files");
    b.node(id("files").key("b.rs"), Role::ListItem, "b.rs")
        .parent("files");
    b.node("commit.submit", Role::Button, "commit")
        .parent("app.main")
        .action("submit")
        .reads("files:staged");
    b.focus(id("files").key("b.rs"));
    let snap = b.finish().unwrap();

    let side = snap.subtree(&id("app.side"));
    assert_eq!(
        names(side.nodes().values()),
        ["app.side", "files", "files[a.rs]", "files[b.rs]"]
    );
    assert_eq!(side.get(&id("app.side")).unwrap().parent, Some(id("app")));
    assert_eq!(names(side.roots()), ["app.side"]);
    assert_eq!(side.focused(), Some(&id("files[b.rs]")));

    let main = snap.subtree(&id("app.main"));
    assert_eq!(names(main.nodes().values()), ["app.main", "commit.submit"]);
    assert_eq!(main.focused(), None);
    let submit = main.get(&id("commit.submit")).unwrap();
    assert_eq!(
        submit.actions[0].reads,
        [StateRef::parse("files:staged").unwrap()]
    );

    assert_eq!(snap.subtree(&id("app")), snap);
    assert!(snap.subtree(&id("ghost")).is_empty());
    assert_ne!(side.revision(), snap.revision());
}

#[test]
fn equal_projections_have_equal_revisions() {
    let a = Git::base().snapshot();
    let b = Git::base().snapshot();
    assert_eq!(a, b);
    assert_eq!(a.revision(), b.revision());
}

#[test]
fn the_revision_changes_with_every_semantic_field() {
    let base = Git::base().snapshot().revision();
    let variants: [(&str, Git); 9] = [
        (
            "state",
            Git {
                staged: vec!["lexer.rs"],
                ..Git::base()
            },
        ),
        (
            "state text",
            Git {
                message: "fix lexer",
                ..Git::base()
            },
        ),
        (
            "label",
            Git {
                submit_label: "commit now",
                ..Git::base()
            },
        ),
        (
            "action enabled",
            Git {
                submit_enabled: false,
                ..Git::base()
            },
        ),
        (
            "constraints",
            Git {
                max_len: Some(50),
                ..Git::base()
            },
        ),
        (
            "parent",
            Git {
                message_parent: "other",
                ..Git::base()
            },
        ),
        (
            "order",
            Git {
                changes_first: false,
                ..Git::base()
            },
        ),
        (
            "description",
            Git {
                description: Some("first line under 72"),
                ..Git::base()
            },
        ),
        (
            "focus",
            Git {
                focus: Some("commit.submit"),
                ..Git::base()
            },
        ),
    ];
    let mut seen = BTreeSet::from([base]);
    for (what, git) in variants {
        let snap = git.snapshot();
        assert_ne!(snap.revision(), base, "{what} did not change the revision");
        assert_ne!(snap, Git::base().snapshot(), "{what}");
        assert!(seen.insert(snap.revision()), "{what} collided");
    }
}

#[test]
fn sibling_order_is_meaning_and_interleaving_is_not() {
    let project = |order: &[&str]| {
        let mut b = SnapshotBuilder::new();
        for &name in order {
            match name {
                "list" => {
                    b.node("list", Role::List, "files");
                }
                "bar" => {
                    b.node("bar", Role::Toolbar, "tools");
                }
                item => {
                    b.node(item, Role::ListItem, item).parent("list");
                }
            }
        }
        b.finish().unwrap()
    };
    let base = project(&["list", "list.a", "list.b", "bar"]);
    let interleaved = project(&["list", "bar", "list.a", "list.b"]);
    let children_first = project(&["list.a", "list.b", "list", "bar"]);
    let swapped = project(&["list", "list.b", "list.a", "bar"]);
    let roots_swapped = project(&["bar", "list", "list.a", "list.b"]);
    for same in [&interleaved, &children_first] {
        assert_eq!(same, &base);
        assert_eq!(same.revision(), base.revision());
    }
    for other in [&swapped, &roots_swapped] {
        assert_ne!(other, &base);
        assert_ne!(other.revision(), base.revision());
    }
}

#[test]
fn state_declaration_order_is_not_meaning() {
    let project = |first: &str, second: &str| {
        let mut b = SnapshotBuilder::new();
        let mut n = b.node("a", Role::Group, "a");
        for key in [first, second] {
            n.state(key, key == "x");
        }
        b.finish().unwrap()
    };
    let xy = project("x", "y");
    let yx = project("y", "x");
    let keys: Vec<&str> = yx
        .get(&id("a"))
        .unwrap()
        .state
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, ["y", "x"]);
    assert_eq!(xy, yx);
    assert_eq!(xy.revision(), yx.revision());
}

#[test]
fn the_revision_changes_with_flags() {
    let project = |inert: bool, focusable: bool, enabled: bool| {
        let mut b = SnapshotBuilder::new();
        b.node("a", Role::Button, "a")
            .inert(inert)
            .focusable(focusable)
            .enabled(enabled);
        b.finish().unwrap().revision()
    };
    let base = project(false, true, true);
    assert_ne!(project(true, true, true), base);
    assert_ne!(project(false, false, true), base);
    assert_ne!(project(false, true, false), base);
}

fn project_here(b: &mut SnapshotBuilder) {
    b.node("a", Role::Button, "a").state("n", 1i64);
    b.node("b", Role::Group, "b").parent("a");
}

#[test]
fn the_revision_ignores_where_the_nodes_were_projected() {
    let mut here = SnapshotBuilder::new();
    project_here(&mut here);
    let here = here.finish().unwrap();

    let mut there = SnapshotBuilder::new();
    there.node("a", Role::Button, "a").state("n", 1i64);
    there.node("b", Role::Group, "b").parent("a");
    let there = there.finish().unwrap();

    let source = |s: &Snapshot| s.get(&id("a")).unwrap().source.clone();
    assert_ne!(source(&here), source(&there));
    assert_eq!(here, there);
    assert_eq!(here.revision(), there.revision());
}

/// the two values were also computed by an independent script from the
/// encoding documented in the hash module. a change here means every
/// stored revision changes: new version tag and an ADR.
#[test]
fn a_known_snapshot_has_a_pinned_revision() {
    let snap = Git::base().snapshot();
    assert_eq!(snap.revision().as_u64(), 0xa9f3_c8c5_7b29_7e42);
    assert_eq!(snap.revision().to_string(), "a9f3c8c57b297e42");
    assert_eq!(
        SnapshotBuilder::new().finish().unwrap().revision().as_u64(),
        0xe9d5_b61f_09ad_8a21
    );
}

#[cfg(feature = "serde")]
#[test]
fn a_revision_travels_as_sixteen_hex_digits() {
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error, StrDeserializer};

    let rev = Git::base().snapshot().revision();
    let text = rev.to_string();
    let d: StrDeserializer<'_, Error> = text.as_str().into_deserializer();
    assert_eq!(accord::Revision::deserialize(d).unwrap(), rev);
    for bad in ["", "12", "XYZ", "ABCDEF0123456789", "+bcdef0123456789"] {
        let d: StrDeserializer<'_, Error> = bad.into_deserializer();
        assert!(accord::Revision::deserialize(d).is_err(), "{bad:?}");
    }
}
