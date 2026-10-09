//! `Value`, `StateKey`, `ActionName`, `StateRef`, `Constraints`, `Policy`
//! and `ActionDecl`.

use std::collections::BTreeSet;

use accord::{
    ActionDecl, ActionName, Actors, Confirm, Constraints, Policy, SemError, SemId, StateKey,
    StateRef, Value, ValueKind,
};
use proptest::prelude::*;

fn state_ref(text: &str) -> StateRef {
    StateRef::parse(text).expect("valid state ref")
}

#[test]
fn num_equality_is_by_bits() {
    assert_eq!(Value::Num(f64::NAN), Value::Num(f64::NAN));
    assert_ne!(Value::Num(0.0), Value::Num(-0.0));
    assert_eq!(Value::Num(0.1 + 0.2), Value::Num(0.1 + 0.2));
    assert_ne!(Value::Num(0.1 + 0.2), Value::Num(0.3));
}

#[test]
fn values_of_different_kinds_are_never_equal() {
    assert_ne!(Value::Int(1), Value::Num(1.0));
    assert_ne!(Value::Text("a".into()), Value::List(vec!["a".into()]));
    let set: BTreeSet<String> = ["a".to_owned()].into();
    assert_ne!(Value::List(vec!["a".into()]), Value::Set(set));
}

#[test]
fn a_list_keeps_order_and_a_set_does_not() {
    let ab = Value::List(vec!["a".into(), "b".into()]);
    let ba = Value::List(vec!["b".into(), "a".into()]);
    assert_ne!(ab, ba);
    let s1: BTreeSet<String> = ["a".to_owned(), "b".to_owned()].into();
    let s2: BTreeSet<String> = ["b".to_owned(), "a".to_owned()].into();
    assert_eq!(Value::Set(s1), Value::Set(s2));
}

#[test]
fn values_convert_from_plain_types_and_report_their_kind() {
    let cases: [(Value, ValueKind, &str); 7] = [
        (true.into(), ValueKind::Bool, "bool"),
        (3i64.into(), ValueKind::Int, "int"),
        (0.5f64.into(), ValueKind::Num, "num"),
        ("x".into(), ValueKind::Text, "text"),
        (String::from("x").into(), ValueKind::Text, "text"),
        (vec![String::from("x")].into(), ValueKind::List, "list"),
        (
            BTreeSet::from([String::from("x")]).into(),
            ValueKind::Set,
            "set",
        ),
    ];
    for (value, kind, name) in cases {
        assert_eq!(value.kind(), kind);
        assert_eq!(kind.as_str(), name);
    }
}

#[test]
fn state_keys_and_action_names_follow_the_name_grammar() {
    assert_eq!(StateKey::parse("staged").unwrap().as_str(), "staged");
    assert_eq!(
        ActionName::parse("submit_2").unwrap().to_string(),
        "submit_2"
    );
    for bad in ["", "Staged", "a.b", "a:b", "a b", "ç"] {
        match StateKey::parse(bad) {
            Err(SemError::InvalidKey { text, reason }) => {
                assert_eq!(text, bad);
                assert!(reason.starts_with("state key"), "{reason}");
            }
            other => panic!("{bad:?} should be rejected, got {other:?}"),
        }
        match ActionName::parse(bad) {
            Err(SemError::InvalidKey { reason, .. }) => {
                assert!(reason.starts_with("action name"), "{reason}");
            }
            other => panic!("{bad:?} should be rejected, got {other:?}"),
        }
    }
}

#[test]
fn state_refs_of_the_spec_parse_and_print_back() {
    for text in [
        "changes:staged",
        "commit.message:text",
        "changes.file[c:/x]:staged",
        "a[:]:b",
    ] {
        assert_eq!(state_ref(text).to_string(), text);
    }
    let r = state_ref("changes.file[c:/x]:staged");
    assert_eq!(r.node, SemId::parse("changes.file[c:/x]").unwrap());
    assert_eq!(r.key, StateKey::parse("staged").unwrap());
    assert_eq!(StateRef::new(r.node.clone(), r.key.clone()), r);
}

#[test]
fn malformed_state_refs_are_rejected() {
    let invalid_id = |text: &str| {
        assert!(
            matches!(StateRef::parse(text), Err(SemError::InvalidId { .. })),
            "{text:?}"
        );
    };
    let invalid_key = |text: &str| {
        assert!(
            matches!(StateRef::parse(text), Err(SemError::InvalidKey { .. })),
            "{text:?}"
        );
    };
    invalid_id("changes");
    invalid_id("");
    invalid_id(":staged");
    invalid_id("Changes:staged");
    invalid_id("a..b:staged");
    invalid_id("a[x:]");
    invalid_key("changes:");
    invalid_key("changes:Staged");
    invalid_key("changes:staged.x");
}

#[test]
fn constraints_default_to_unbounded_and_compare_floats_by_bits() {
    let c = Constraints::default();
    assert!(!c.editable && !c.required && !c.secret);
    assert!(c.range.is_none() && c.step.is_none() && c.max_len.is_none());
    assert!(c.options.is_none());
    let a = Constraints {
        range: Some((0.0, 1.0)),
        ..Constraints::default()
    };
    let b = Constraints {
        range: Some((-0.0, 1.0)),
        ..Constraints::default()
    };
    assert_ne!(a, b);
    assert_eq!(a, a.clone());
}

#[test]
fn the_default_policy_lets_everyone_request_without_confirmation() {
    let p = Policy::default();
    assert_eq!(
        p.actors,
        Actors {
            human: true,
            agent: true,
            assistive: true,
            test: true
        }
    );
    assert_eq!(p.confirm, Confirm::Never);
    assert!(!p.destructive);
    assert_eq!(Confirm::ForAgents.as_str(), "for_agents");
}

#[test]
fn a_new_action_is_enabled_and_reads_nothing() {
    let a = ActionDecl::new(ActionName::parse("submit").unwrap());
    assert!(a.enabled);
    assert!(a.reads.is_empty() && a.label.is_none() && a.param.is_none());
    assert_eq!(a.policy, Policy::default());
}

#[cfg(feature = "serde")]
#[test]
fn serde_names_match_the_stable_names() {
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::{Error, StrDeserializer};

    fn from<T: for<'de> Deserialize<'de>>(text: &str) -> Result<T, Error> {
        let d: StrDeserializer<'_, Error> = text.into_deserializer();
        T::deserialize(d)
    }
    for kind in [
        ValueKind::Bool,
        ValueKind::Int,
        ValueKind::Num,
        ValueKind::Text,
        ValueKind::List,
        ValueKind::Set,
    ] {
        assert_eq!(from::<ValueKind>(kind.as_str()).unwrap(), kind);
    }
    for confirm in [Confirm::Never, Confirm::ForAgents, Confirm::Always] {
        assert_eq!(from::<Confirm>(confirm.as_str()).unwrap(), confirm);
    }
    assert_eq!(
        from::<StateRef>("commit.message:text").unwrap(),
        state_ref("commit.message:text")
    );
    assert!(from::<StateRef>("commit.message").is_err());
    assert_eq!(from::<StateKey>("text").unwrap().as_str(), "text");
    assert!(from::<ActionName>("Submit").is_err());
}

fn sem_id() -> impl Strategy<Value = SemId> {
    (
        "[a-z0-9_]{1,6}(\\.[a-z0-9_]{1,6}){0,3}",
        proptest::option::of("[a-z:\\]\\\\\\[./]{0,6}"),
    )
        .prop_map(|(path, key)| {
            let id = SemId::parse(&path).unwrap();
            match key {
                Some(k) => id.key(k),
                None => id,
            }
        })
}

proptest! {
    #[test]
    fn every_state_ref_roundtrips_through_its_text(
        node in sem_id(),
        key in "[a-z0-9_]{1,8}",
    ) {
        let r = StateRef::new(node, StateKey::parse(&key).unwrap());
        let text = r.to_string();
        prop_assert_eq!(StateRef::parse(&text), Ok(r));
    }

    #[test]
    fn state_ref_parse_never_panics(text in any::<String>()) {
        let _ = StateRef::parse(&text);
    }
}
