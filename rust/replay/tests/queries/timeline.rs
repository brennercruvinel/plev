//! the timeline: one entry per event, with its time from the start, and
//! the tallies by kind and by actor.

use replay::{Inspector, What};
use accord::Actor;

use crate::toys::agent;
use crate::thesis::record;

#[test]
fn every_event_is_one_entry_timed_from_the_start() {
    let thesis = record();
    let inspector = Inspector::new(&thesis.session);
    let timeline = inspector.timeline();
    assert_eq!(timeline.start, 1_000);
    assert_eq!(timeline.end, 1_500);
    assert_eq!(timeline.entries.len(), thesis.session.events().len());
    for (entry, event) in timeline.entries.iter().zip(thesis.session.events()) {
        assert_eq!(entry.t, event.t);
        assert_eq!(entry.at, event.t - 1_000);
        assert_eq!(entry.what.kind(), event.kind.name());
    }
    let counted: usize = timeline.kinds.iter().map(|c| c.count).sum();
    assert_eq!(counted, timeline.entries.len());
    let kinds: Vec<&str> = timeline.kinds.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, ["observed", "snapshot", "changes", "stage", "input"]);
}

#[test]
fn the_entries_say_who_did_what() {
    let thesis = record();
    let timeline = Inspector::new(&thesis.session).timeline();
    let observed: Vec<(u64, &Actor)> = timeline
        .entries
        .iter()
        .filter_map(|e| match &e.what {
            What::Observed { actor, scope, .. } if scope.is_none() => Some((e.at, actor)),
            _ => None,
        })
        .collect();
    assert_eq!(observed, [(0, &agent()), (400, &agent())]);
    let stages: Vec<(u64, String, &str)> = timeline
        .entries
        .iter()
        .filter_map(|e| match &e.what {
            What::Stage { id, stage, .. } => Some((e.at, id.to_string(), *stage)),
            _ => None,
        })
        .collect();
    let first_of = |id: &str| stages.iter().find(|s| s.1 == id).map(|s| (s.0, s.2));
    let (unstage, stale, done) = (
        thesis.unstage.to_string(),
        thesis.stale.to_string(),
        thesis.done.to_string(),
    );
    assert_eq!(first_of(&unstage), Some((100, "requested")));
    assert_eq!(first_of(&stale), Some((300, "requested")));
    assert!(stages.contains(&(300, stale, "rejected")));
    assert!(stages.contains(&(500, done, "completed")));
    let rejected = timeline.entries.iter().find_map(|e| match &e.what {
        What::Stage {
            stage: "rejected",
            detail,
            ..
        } => detail.clone(),
        _ => None,
    });
    assert_eq!(rejected.as_deref(), Some("stale"));
    let moved = timeline.entries.iter().find_map(|e| match &e.what {
        What::Changes { changes } if e.at == 100 => Some(changes.clone()),
        _ => None,
    });
    assert_eq!(moved, Some(vec![r#"changes:staged: removed B"#.to_owned()]));
}

#[test]
fn the_tallies_count_each_actor() {
    let thesis = record();
    let tallies = Inspector::new(&thesis.session).tallies();
    let of = |actor: &Actor| tallies.iter().find(|t| t.actor == *actor).cloned();
    let reviewer = of(&agent()).expect("the agent is tallied");
    assert_eq!(
        (reviewer.observations, reviewer.requests, reviewer.inputs),
        (2, 2, 0)
    );
    assert_eq!(
        reviewer.stages,
        2 + 4,
        "requested and rejected; four to completed"
    );
    let person = of(&Actor::Human).expect("the person is tallied");
    assert_eq!(
        (person.observations, person.requests, person.inputs),
        (0, 1, 1)
    );
    assert_eq!(tallies[0].actor, agent(), "in the order they first appear");
}

#[test]
fn an_empty_session_has_an_empty_timeline() {
    let session = record::Session::default();
    let timeline = Inspector::new(&session).timeline();
    assert!(timeline.entries.is_empty() && timeline.kinds.is_empty() && timeline.actors.is_empty());
    assert_eq!((timeline.start, timeline.end), (0, 0));
}
