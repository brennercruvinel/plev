//! `Receipt`, `Stage` and the legal transitions, `RequestId`.

use accord::{
    Actor, Change, Outcome, Receipt, Rejection, Request, RequestId, Role, SemError, SemId,
    SnapshotBuilder, Stage, Value,
};
use indexmap::IndexMap;

fn one_of_each() -> Vec<Stage> {
    vec![
        Stage::Requested,
        Stage::Accepted,
        Stage::AwaitingConfirmation,
        Stage::Confirmed { by: Actor::Human },
        Stage::Rejected(Rejection::stale(vec![Change::NodeRemoved(
            SemId::parse("x").unwrap(),
        )])),
        Stage::Executing,
        Stage::Completed(Outcome::default()),
        Stage::Failed {
            error: "disk full".into(),
        },
    ]
}

#[test]
fn only_the_legal_transitions_are_allowed() {
    let legal: &[(&str, &[&str])] = &[
        (
            "requested",
            &["accepted", "rejected", "awaiting_confirmation"],
        ),
        ("accepted", &["executing"]),
        ("awaiting_confirmation", &["confirmed", "rejected"]),
        ("confirmed", &["executing"]),
        ("rejected", &[]),
        ("executing", &["completed", "failed", "rejected"]),
        ("completed", &[]),
        ("failed", &[]),
    ];
    let stages = one_of_each();
    for from in &stages {
        let (_, allowed) = legal.iter().find(|(name, _)| *name == from.name()).unwrap();
        for to in &stages {
            assert_eq!(
                from.can_advance_to(to),
                allowed.contains(&to.name()),
                "{} -> {}",
                from.name(),
                to.name()
            );
        }
    }
}

#[test]
fn terminal_stages_are_the_ones_nothing_follows() {
    let stages = one_of_each();
    for s in &stages {
        let followed = stages.iter().any(|next| s.can_advance_to(next));
        assert_eq!(s.is_terminal(), !followed, "{}", s.name());
    }
    let terminal: Vec<&str> = stages
        .iter()
        .filter(|s| s.is_terminal())
        .map(Stage::name)
        .collect();
    assert_eq!(terminal, ["rejected", "completed", "failed"]);
}

#[test]
fn a_request_id_reads_and_prints_its_decimal_text() {
    for text in ["0", "1", "42", "18446744073709551615"] {
        let id = RequestId::parse(text).unwrap();
        assert_eq!(id.to_string(), text);
        assert_eq!(text.parse::<RequestId>().unwrap(), id);
    }
    assert_eq!(RequestId::parse("42").unwrap().as_u64(), 42);
    for bad in [
        "",
        "-1",
        "+1",
        "01",
        "1.0",
        " 1",
        "x",
        "18446744073709551616",
    ] {
        assert!(
            matches!(RequestId::parse(bad), Err(SemError::InvalidId { .. })),
            "{bad:?}"
        );
    }
}

#[test]
fn a_receipt_is_at_its_last_stage_and_finished_at_a_terminal_one() {
    let mut b = SnapshotBuilder::new();
    b.node("a", Role::Button, "a").action("go");
    let seen = b.finish().unwrap();
    let mut receipt = Receipt {
        id: RequestId::parse("1").unwrap(),
        request: Request::observe(&seen, Actor::Human, "a", "go", None).unwrap(),
        validated_against: Some(seen.revision()),
        stages: vec![],
    };
    assert_eq!(receipt.stage(), None);
    assert!(!receipt.is_finished());
    receipt.stages.push((Stage::Requested, 5));
    receipt.stages.push((Stage::Accepted, 5));
    receipt.stages.push((Stage::Executing, 6));
    assert_eq!(receipt.stage(), Some(&Stage::Executing));
    assert!(!receipt.is_finished());
    let outcome = Outcome {
        result: IndexMap::from([("n".to_owned(), Value::Int(1))]),
    };
    receipt.stages.push((Stage::Completed(outcome.clone()), 9));
    assert_eq!(receipt.stage(), Some(&Stage::Completed(outcome)));
    assert!(receipt.is_finished());
}

#[cfg(feature = "serde")]
#[test]
fn stage_serde_names_match_the_stable_names() {
    for stage in one_of_each() {
        let json = serde_json::to_value(&stage).unwrap();
        let name = match &json {
            serde_json::Value::String(name) => name.clone(),
            serde_json::Value::Object(map) => map.keys().next().unwrap().clone(),
            other => panic!("unexpected form {other}"),
        };
        assert_eq!(name, stage.name());
        assert_eq!(serde_json::from_value::<Stage>(json).unwrap(), stage);
    }
}
