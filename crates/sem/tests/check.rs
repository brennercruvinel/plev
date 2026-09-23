//! `check`: the validation order, the stale delta, semantic invalidation,
//! parameters and confirmation.

use std::collections::BTreeSet;

use proptest::prelude::*;
use sem::{
    ActionName, Actor, Actors, Change, Confirm, Constraints, ParamSpec, Policy, Rejection, Request,
    Role, SemId, Snapshot, SnapshotBuilder, StateRef, Value, ValueKind, Verdict, check,
};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("valid id")
}

fn at(text: &str) -> StateRef {
    StateRef::parse(text).expect("valid state ref")
}

fn set(items: &[&str]) -> Value {
    Value::Set(items.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>())
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn agent() -> Actor {
    Actor::Agent {
        name: "reviewer".into(),
    }
}

fn test_actor() -> Actor {
    Actor::Test {
        name: "check".into(),
    }
}

/// the git client of the thesis, with a knob for every condition `check`
/// looks at.
#[derive(Clone)]
struct Git {
    staged: Vec<&'static str>,
    message: &'static str,
    cursor: i64,
    submit_projected: bool,
    action: &'static str,
    node_enabled: bool,
    action_enabled: bool,
    inert: bool,
    policy: Policy,
    param: Option<ParamSpec>,
}

impl Git {
    fn base() -> Self {
        Git {
            staged: vec!["A", "B", "C"],
            message: "fix login",
            cursor: 0,
            submit_projected: true,
            action: "submit",
            node_enabled: true,
            action_enabled: true,
            inert: false,
            policy: Policy::default(),
            param: None,
        }
    }

    fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        b.node("changes", Role::List, "changes")
            .state("staged", set(&self.staged))
            .state("cursor", self.cursor);
        b.node("commit.message", Role::TextInput, "commit message")
            .state("text", self.message);
        if self.submit_projected {
            let mut node = b.node("commit.submit", Role::Button, "commit");
            node.enabled(self.node_enabled).inert(self.inert);
            let mut action = node.action(self.action);
            action
                .enabled(self.action_enabled)
                .reads("changes:staged")
                .reads("commit.message:text")
                .policy(self.policy);
            if let Some(spec) = self.param {
                action.param(spec);
            }
        }
        b.finish().unwrap()
    }

    fn request(&self, actor: Actor, param: Option<Value>) -> Request {
        Request::observe(&self.snapshot(), actor, "commit.submit", "submit", param).unwrap()
    }
}

fn rejection(verdict: Verdict) -> Rejection {
    match verdict {
        Verdict::Rejected(why) => why,
        other => panic!("expected a rejection, got {other:?}"),
    }
}

#[test]
fn nothing_changed_is_accepted() {
    let git = Git::base();
    let req = git.request(agent(), None);
    assert_eq!(check(&req, &git.snapshot()), Verdict::Accepted);
}

#[test]
fn a_moved_cursor_is_state_the_commit_does_not_read_and_is_accepted() {
    let seen = Git::base();
    let req = seen.request(agent(), None);
    let now = Git { cursor: 2, ..seen };
    assert_ne!(now.snapshot().revision(), req.observed.revision);
    assert_eq!(check(&req, &now.snapshot()), Verdict::Accepted);
}

#[test]
fn a_file_unstaged_after_the_observation_is_stale_with_removed_b() {
    let seen = Git::base();
    let req = seen.request(agent(), None);
    let now = Git {
        staged: vec!["A", "C"],
        cursor: 1,
        ..seen
    };
    assert_eq!(
        check(&req, &now.snapshot()),
        Verdict::Rejected(Rejection::Stale {
            changes: vec![Change::StateChanged {
                at: at("changes:staged"),
                before: Some(set(&["A", "B", "C"])),
                after: Some(set(&["A", "C"])),
                added: vec![],
                removed: strings(&["B"]),
            }],
        })
    );
}

#[test]
fn every_read_that_moved_is_one_change_in_the_order_of_the_reads() {
    let seen = Git::base();
    let req = seen.request(agent(), None);
    let now = Git {
        staged: vec!["A", "C", "D"],
        message: "fix logout",
        ..seen
    };
    let Rejection::Stale { changes } = rejection(check(&req, &now.snapshot())) else {
        panic!("expected stale");
    };
    assert_eq!(
        changes,
        [
            Change::StateChanged {
                at: at("changes:staged"),
                before: Some(set(&["A", "B", "C"])),
                after: Some(set(&["A", "C", "D"])),
                added: strings(&["D"]),
                removed: strings(&["B"]),
            },
            Change::StateChanged {
                at: at("commit.message:text"),
                before: Some(Value::Text("fix login".into())),
                after: Some(Value::Text("fix logout".into())),
                added: vec![],
                removed: vec![],
            },
        ]
    );
}

#[test]
fn a_read_the_actor_never_observed_is_stale_as_seen_without_a_value() {
    let git = Git::base();
    let mut req = git.request(agent(), None);
    req.observed
        .values
        .retain(|(r, _)| *r != at("changes:staged"));
    let Rejection::Stale { changes } = rejection(check(&req, &git.snapshot())) else {
        panic!("expected stale");
    };
    assert_eq!(
        changes,
        [Change::StateChanged {
            at: at("changes:staged"),
            before: None,
            after: Some(set(&["A", "B", "C"])),
            added: strings(&["A", "B", "C"]),
            removed: vec![],
        }]
    );
}

#[test]
fn observed_values_outside_the_reads_and_the_revision_decide_nothing() {
    let git = Git::base();
    let mut req = git.request(agent(), None);
    req.observed
        .values
        .push((at("changes:cursor"), Some(Value::Int(99))));
    req.observed.revision = Git {
        cursor: 5,
        ..Git::base()
    }
    .snapshot()
    .revision();
    assert_eq!(check(&req, &git.snapshot()), Verdict::Accepted);
}

#[test]
fn a_read_declared_twice_is_one_change() {
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Button, "a")
        .state("n", 1i64)
        .action("go")
        .reads("a:n")
        .reads("a:n");
    let seen = b.finish().unwrap();
    let req = Request::observe(&seen, Actor::Human, "a", "go", None).unwrap();
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Button, "a")
        .state("n", 2i64)
        .action("go")
        .reads("a:n")
        .reads("a:n");
    let Rejection::Stale { changes } = rejection(check(&req, &b.finish().unwrap())) else {
        panic!("expected stale");
    };
    assert_eq!(changes.len(), 1);
}

#[test]
fn a_target_that_is_gone_is_not_found() {
    let seen = Git::base();
    let req = seen.request(agent(), None);
    let now = Git {
        submit_projected: false,
        ..seen
    };
    assert_eq!(
        rejection(check(&req, &now.snapshot())),
        Rejection::NotFound {
            target: id("commit.submit")
        }
    );
}

#[test]
fn an_action_the_target_no_longer_declares_is_unknown_action() {
    let seen = Git::base();
    let req = seen.request(agent(), None);
    let now = Git {
        action: "amend",
        ..seen
    };
    assert_eq!(
        rejection(check(&req, &now.snapshot())),
        Rejection::UnknownAction {
            target: id("commit.submit"),
            action: ActionName::parse("submit").unwrap(),
        }
    );
}

#[test]
fn each_actor_is_authorized_by_its_own_column() {
    type Deny = fn(&mut Actors);
    let columns: [(Actor, Deny); 4] = [
        (Actor::Human, |a| a.human = false),
        (agent(), |a| a.agent = false),
        (Actor::Assistive, |a| a.assistive = false),
        (test_actor(), |a| a.test = false),
    ];
    for (actor, deny) in &columns {
        let mut git = Git::base();
        deny(&mut git.policy.actors);
        let req = git.request(actor.clone(), None);
        assert_eq!(
            rejection(check(&req, &git.snapshot())),
            Rejection::NotAuthorized {
                actor: actor.clone()
            }
        );
        for (other, _) in columns.iter().filter(|(o, _)| o != actor) {
            let req = git.request(other.clone(), None);
            assert_eq!(check(&req, &git.snapshot()), Verdict::Accepted, "{other}");
        }
    }
}

#[test]
fn an_inert_target_is_inert() {
    let git = Git {
        inert: true,
        ..Git::base()
    };
    let req = git.request(agent(), None);
    assert_eq!(
        rejection(check(&req, &git.snapshot())),
        Rejection::Inert {
            target: id("commit.submit")
        }
    );
}

#[test]
fn a_disabled_node_or_a_disabled_action_is_disabled() {
    let disabled = Rejection::Disabled {
        target: id("commit.submit"),
        action: ActionName::parse("submit").unwrap(),
    };
    for git in [
        Git {
            node_enabled: false,
            ..Git::base()
        },
        Git {
            action_enabled: false,
            ..Git::base()
        },
    ] {
        let req = git.request(agent(), None);
        assert_eq!(rejection(check(&req, &git.snapshot())), disabled);
    }
}

#[test]
fn a_parameter_for_an_action_that_takes_none_is_invalid_param() {
    let git = Git::base();
    let req = git.request(agent(), Some(Value::Int(1)));
    assert!(matches!(
        rejection(check(&req, &git.snapshot())),
        Rejection::InvalidParam { reason } if reason.contains("takes no parameter")
    ));
}

#[test]
fn the_validation_order_holds_when_every_condition_fails_at_once() {
    // everything is wrong at once: observed {A,B,C}, now {A,C}; the agent
    // is not allowed; the node is inert and disabled; the parameter is not
    // the declared kind; the policy asks agents to confirm. fixing one
    // condition at a time walks the verdict down the documented order.
    let seen = Git {
        param: Some(ParamSpec {
            kind: ValueKind::Text,
            constrained: false,
        }),
        ..Git::base()
    };
    let req = seen.request(agent(), Some(Value::Int(1)));
    let mut now = Git {
        staged: vec!["A", "C"],
        inert: true,
        node_enabled: false,
        policy: Policy {
            actors: Actors {
                agent: false,
                ..Actors::default()
            },
            confirm: Confirm::ForAgents,
            destructive: true,
        },
        ..seen.clone()
    };
    let walk = |now: &Git| check(&req, &now.snapshot());

    let gone = Git {
        submit_projected: false,
        ..now.clone()
    };
    assert!(matches!(rejection(walk(&gone)), Rejection::NotFound { .. }));
    let renamed = Git {
        action: "amend",
        ..now.clone()
    };
    assert!(matches!(
        rejection(walk(&renamed)),
        Rejection::UnknownAction { .. }
    ));
    assert!(matches!(
        rejection(walk(&now)),
        Rejection::NotAuthorized { .. }
    ));
    now.policy.actors.agent = true;
    assert!(matches!(rejection(walk(&now)), Rejection::Stale { .. }));
    now.staged = vec!["A", "B", "C"];
    assert!(matches!(rejection(walk(&now)), Rejection::Inert { .. }));
    now.inert = false;
    assert!(matches!(rejection(walk(&now)), Rejection::Disabled { .. }));
    now.node_enabled = true;
    assert!(matches!(
        rejection(walk(&now)),
        Rejection::InvalidParam { .. }
    ));
    let fixed = Request {
        param: Some(Value::Text("x".into())),
        ..req.clone()
    };
    assert_eq!(check(&fixed, &now.snapshot()), Verdict::NeedsConfirmation);
    now.policy.confirm = Confirm::Never;
    assert_eq!(check(&fixed, &now.snapshot()), Verdict::Accepted);
}

#[test]
fn stale_wins_over_disabled_when_unstaging_disables_the_commit() {
    let seen = Git::base();
    let req = seen.request(agent(), None);
    let now = Git {
        staged: vec![],
        node_enabled: false,
        ..seen
    };
    assert!(matches!(
        rejection(check(&req, &now.snapshot())),
        Rejection::Stale { .. }
    ));
}

#[test]
fn confirmation_follows_the_policy_and_the_actor() {
    let everyone = [Actor::Human, agent(), Actor::Assistive, test_actor()];
    for (confirm, destructive, asks) in [
        (Confirm::Never, true, [false, false, false, false]),
        (Confirm::ForAgents, false, [false, true, false, false]),
        (Confirm::ForAgents, true, [false, true, false, false]),
        (Confirm::Always, false, [true, true, true, true]),
    ] {
        let git = Git {
            policy: Policy {
                confirm,
                destructive,
                ..Policy::default()
            },
            ..Git::base()
        };
        for (actor, asks) in everyone.iter().zip(asks) {
            let verdict = check(&git.request(actor.clone(), None), &git.snapshot());
            let expected = if asks {
                Verdict::NeedsConfirmation
            } else {
                Verdict::Accepted
            };
            assert_eq!(verdict, expected, "{confirm:?} {actor}");
        }
    }
}

/// one text field with an action `set` whose parameter is `kind`,
/// constrained by `c`.
fn field(c: Constraints, kind: ValueKind) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("field", Role::TextInput, "field")
        .state("value", "x")
        .constraints(c)
        .action("set")
        .param(ParamSpec {
            kind,
            constrained: true,
        });
    b.finish().unwrap()
}

fn set_to(snap: &Snapshot, value: Option<Value>) -> Verdict {
    let req = Request::observe(snap, Actor::Human, "field", "set", value).unwrap();
    check(&req, snap)
}

fn invalid(verdict: Verdict, needle: &str) {
    match verdict {
        Verdict::Rejected(Rejection::InvalidParam { reason }) => {
            assert!(reason.contains(needle), "{reason:?} lacks {needle:?}");
        }
        other => panic!("expected invalid param with {needle:?}, got {other:?}"),
    }
}

fn editable() -> Constraints {
    Constraints {
        editable: true,
        ..Constraints::default()
    }
}

#[test]
fn a_parameter_needs_the_declared_kind_and_must_be_given() {
    let snap = field(editable(), ValueKind::Text);
    assert_eq!(set_to(&snap, Some("ok".into())), Verdict::Accepted);
    invalid(set_to(&snap, None), "none was given");
    invalid(set_to(&snap, Some(Value::Int(1))), "got int");
}

#[test]
fn a_constrained_parameter_needs_an_editable_target() {
    let snap = field(Constraints::default(), ValueKind::Text);
    invalid(set_to(&snap, Some("ok".into())), "not editable");
}

#[test]
fn an_unconstrained_parameter_ignores_the_constraints() {
    let mut b = SnapshotBuilder::new();
    b.node("field", Role::TextInput, "field")
        .state("value", "x")
        .constraints(Constraints {
            max_len: Some(1),
            ..Constraints::default()
        })
        .action("set")
        .param(ParamSpec {
            kind: ValueKind::Text,
            constrained: false,
        });
    let snap = b.finish().unwrap();
    assert_eq!(set_to(&snap, Some("long".into())), Verdict::Accepted);
}

#[test]
fn required_refuses_blank_text_and_empty_collections() {
    let required = Constraints {
        required: true,
        ..editable()
    };
    let text = field(required.clone(), ValueKind::Text);
    invalid(set_to(&text, Some("  ".into())), "required");
    assert_eq!(set_to(&text, Some("a".into())), Verdict::Accepted);
    let list = field(required.clone(), ValueKind::List);
    invalid(set_to(&list, Some(Value::List(vec![]))), "required");
    let items = field(required, ValueKind::Set);
    invalid(set_to(&items, Some(set(&[]))), "required");
    assert_eq!(set_to(&items, Some(set(&["a"]))), Verdict::Accepted);
}

#[test]
fn range_bounds_numbers_inclusively_and_refuses_nan() {
    let ranged = Constraints {
        range: Some((0.0, 1.0)),
        ..editable()
    };
    let num = field(ranged.clone(), ValueKind::Num);
    for ok in [0.0, 0.5, 1.0] {
        assert_eq!(set_to(&num, Some(Value::Num(ok))), Verdict::Accepted);
    }
    for bad in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
        invalid(set_to(&num, Some(Value::Num(bad))), "outside the range");
    }
    let int = field(ranged, ValueKind::Int);
    assert_eq!(set_to(&int, Some(Value::Int(1))), Verdict::Accepted);
    invalid(set_to(&int, Some(Value::Int(2))), "outside the range");
}

#[test]
fn max_len_counts_characters_and_items() {
    let short = Constraints {
        max_len: Some(3),
        ..editable()
    };
    let text = field(short.clone(), ValueKind::Text);
    assert_eq!(set_to(&text, Some("çãé".into())), Verdict::Accepted);
    invalid(set_to(&text, Some("abcd".into())), "above the maximum");
    let list = field(short, ValueKind::List);
    invalid(
        set_to(&list, Some(Value::List(strings(&["a", "b", "c", "d"])))),
        "above the maximum",
    );
}

#[test]
fn options_hold_the_accepted_text_and_items() {
    let choices = Constraints {
        options: Some(strings(&["main", "dev"])),
        ..editable()
    };
    let text = field(choices.clone(), ValueKind::Text);
    assert_eq!(set_to(&text, Some("dev".into())), Verdict::Accepted);
    invalid(set_to(&text, Some("prod".into())), "not one of the options");
    let items = field(choices, ValueKind::Set);
    assert_eq!(
        set_to(&items, Some(set(&["main", "dev"]))),
        Verdict::Accepted
    );
    invalid(
        set_to(&items, Some(set(&["main", "prod"]))),
        "not one of the options",
    );
}

/// state refs `n:k0..k4` and `m:j0..j2`, and an action `go` on a button
/// that reads the refs whose mask bit is on.
const REFS: [&str; 8] = [
    "n:k0", "n:k1", "n:k2", "n:k3", "n:k4", "m:j0", "m:j1", "m:j2",
];

fn world(values: &[i64], reads: &[bool]) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    for (node, keys) in [("n", 0..5), ("m", 5..8)] {
        let mut n = b.node(node, Role::Group, node);
        for i in keys {
            let key = REFS[i].rsplit_once(':').unwrap().1;
            n.state(key, values[i]);
        }
    }
    let mut a = b.node("act", Role::Button, "act");
    let mut go = a.action("go");
    for (r, on) in REFS.iter().zip(reads) {
        if *on {
            go.reads(*r);
        }
    }
    b.finish().unwrap()
}

proptest! {
    #[test]
    fn a_change_outside_the_reads_never_makes_a_request_stale(
        values in prop::collection::vec(any::<i64>(), 8),
        reads in prop::collection::vec(any::<bool>(), 8),
        edits in prop::collection::vec(any::<i64>(), 8),
    ) {
        let seen = world(&values, &reads);
        let req = Request::observe(&seen, test_actor(), "act", "go", None).unwrap();
        let moved: Vec<i64> = values
            .iter()
            .zip(&edits)
            .zip(&reads)
            .map(|((&v, &e), &read)| if read { v } else { e })
            .collect();
        let now = world(&moved, &reads);
        prop_assert_eq!(check(&req, &now), Verdict::Accepted);
    }

    #[test]
    fn a_stale_delta_holds_exactly_the_reads_that_moved(
        values in prop::collection::vec(any::<i64>(), 8),
        reads in prop::collection::vec(any::<bool>(), 8),
        edits in prop::collection::vec(prop::option::of(any::<i64>()), 8),
    ) {
        let seen = world(&values, &reads);
        let req = Request::observe(&seen, test_actor(), "act", "go", None).unwrap();
        let moved: Vec<i64> = values
            .iter()
            .zip(&edits)
            .map(|(&v, e)| e.unwrap_or(v))
            .collect();
        let now = world(&moved, &reads);
        let expected: Vec<Change> = (0..8)
            .filter(|&i| reads[i] && moved[i] != values[i])
            .map(|i| Change::StateChanged {
                at: at(REFS[i]),
                before: Some(Value::Int(values[i])),
                after: Some(Value::Int(moved[i])),
                added: vec![],
                removed: vec![],
            })
            .collect();
        let verdict = check(&req, &now);
        prop_assert_eq!(&verdict, &check(&req, &now));
        if expected.is_empty() {
            prop_assert_eq!(verdict, Verdict::Accepted);
        } else {
            prop_assert_eq!(verdict, Verdict::Rejected(Rejection::Stale { changes: expected }));
        }
    }
}
