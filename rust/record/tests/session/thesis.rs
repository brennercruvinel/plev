//! the thesis session (generative-runtime-thesis.md, sections 8, 9 and
//! 11): the agent observes A, B and C staged, the person unstages B, the
//! agent's commit comes back stale with removed [B], the agent observes
//! again and its commit completes with a sha. recorded through the
//! dispatcher's sink and explicit calls, then read back.

use accord::{
    Actor, Change, Dispatcher, Millis, Rejection, Request, Snapshot, Stage, StateRef, Value,
};
use record::{InputKind, Kind, Limits, Recorder, Session};

use crate::toys::{Git, agent, id, look, sha};

type Fallible = Result<(), Box<dyn std::error::Error>>;

/// the session, what the app projected at each instant it projected, and
/// the app as it ended.
type Recorded = (Session, Vec<(Millis, Snapshot)>, Git);

fn record() -> Result<Recorded, Box<dyn std::error::Error>> {
    let rec = Recorder::new(Limits::default());
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut git = Git::new(&["A", "B", "C"], "fix login");
    let mut projected = Vec::new();

    let seen_by_agent = look(&git);
    rec.snapshot(1_000, &seen_by_agent);
    projected.push((1_000, seen_by_agent.clone()));
    rec.observed(1_000, agent(), None, seen_by_agent.revision());
    let commit = Request::observe(&seen_by_agent, agent(), "commit.submit", "submit", None)?;

    rec.input(
        1_100,
        InputKind::PointerDown,
        Some([40.0, 120.0]),
        Some(id("changes")),
    );
    let seen_by_human = look(&git);
    let unstage = Request::observe(
        &seen_by_human,
        Actor::Human,
        "changes",
        "unstage",
        Some("B".into()),
    )?;
    dispatcher.dispatch(&mut git, unstage, 1_100);
    let now = look(&git);
    rec.snapshot(1_100, &now);
    projected.push((1_100, now));

    dispatcher.dispatch(&mut git, commit, 1_300);

    let seen_again = look(&git);
    rec.snapshot(1_400, &seen_again);
    rec.observed(1_400, agent(), None, seen_again.revision());
    let again = Request::observe(&seen_again, agent(), "commit.submit", "submit", None)?;
    dispatcher.dispatch(&mut git, again, 1_500);
    let now = look(&git);
    rec.snapshot(1_500, &now);
    projected.push((1_500, now));

    Ok((rec.snapshot_window(), projected, git))
}

/// one line per event: its time, its kind, and the stage name for a
/// stage.
fn lines(session: &Session) -> Vec<(Millis, String)> {
    session
        .events()
        .iter()
        .map(|e| {
            let what = match &e.kind {
                Kind::Stage { id, stage, .. } => format!("stage {id} {}", stage.name()),
                other => other.name().to_owned(),
            };
            (e.t, what)
        })
        .collect()
}

#[test]
fn every_event_reads_back_in_order() -> Fallible {
    let (session, _, _) = record()?;
    let expected: Vec<(Millis, &str)> = vec![
        (1_000, "snapshot"),
        (1_000, "observed"),
        (1_100, "input"),
        (1_100, "stage 1 requested"),
        (1_100, "stage 1 accepted"),
        (1_100, "stage 1 executing"),
        (1_100, "stage 1 completed"),
        (1_100, "changes"),
        (1_300, "stage 2 requested"),
        (1_300, "stage 2 rejected"),
        (1_400, "observed"),
        (1_500, "stage 3 requested"),
        (1_500, "stage 3 accepted"),
        (1_500, "stage 3 executing"),
        (1_500, "stage 3 completed"),
        (1_500, "changes"),
    ];
    let got = lines(&session);
    let got: Vec<(Millis, &str)> = got.iter().map(|(t, s)| (*t, s.as_str())).collect();
    assert_eq!(got, expected);
    Ok(())
}

#[test]
fn the_semantic_state_is_right_at_every_instant() -> Fallible {
    let (session, projected, _) = record()?;
    for t in (900..=1_700).step_by(50) {
        let expected = projected.iter().rev().find(|(at, _)| *at <= t);
        let state = session.state_at(t)?;
        match (expected, state) {
            (None, None) => {}
            (Some((_, snapshot)), Some(state)) => {
                let want: Vec<_> = snapshot.nodes().values().collect();
                let got: Vec<_> = state.nodes().iter().collect();
                assert_eq!(got, want, "nodes at t={t}");
                assert_eq!(state.focused(), snapshot.focused(), "focus at t={t}");
                assert_eq!(state.revision()?, snapshot.revision(), "revision at t={t}");
                for node in snapshot.nodes().values() {
                    let kept = state.get(&node.id).and_then(|n| n.source.clone());
                    assert_eq!(kept, node.source, "source of {} at t={t}", node.id);
                }
            }
            (want, got) => panic!("t={t}: expected {want:?}, got {got:?}"),
        }
    }
    let staged = StateRef::parse("changes:staged")?;
    let at = |t| -> Result<Option<Value>, Box<dyn std::error::Error>> {
        Ok(session.state_at(t)?.and_then(|s| s.value(&staged).cloned()))
    };
    let set = |items: &[&str]| Value::Set(items.iter().map(|i| (*i).to_owned()).collect());
    assert_eq!(at(1_050)?, Some(set(&["A", "B", "C"])));
    assert_eq!(at(1_299)?, Some(set(&["A", "C"])));
    assert_eq!(at(1_500)?, Some(set(&[])));
    Ok(())
}

#[test]
fn the_receipts_for_commit_submit_are_right_up_to_each_instant() -> Fallible {
    let (session, _, git) = record()?;
    let submit = id("commit.submit");
    assert!(session.receipts_for(&submit, 1_299).is_empty());

    let at_1300 = session.receipts_for(&submit, 1_300);
    assert_eq!(at_1300.len(), 1);
    let rejected = &at_1300[0];
    assert!(rejected.whole);
    assert_eq!(rejected.receipt.request.actor, agent());
    let names: Vec<&str> = rejected
        .receipt
        .stages
        .iter()
        .map(|(s, _)| s.name())
        .collect();
    assert_eq!(names, ["requested", "rejected"]);
    match &rejected.receipt.stages[1].0 {
        Stage::Rejected(Rejection::Stale { changes, .. }) => match changes.as_slice() {
            [Change::StateChanged { at, removed, .. }] => {
                assert_eq!(at.to_string(), "changes:staged");
                assert_eq!(removed, &["B".to_owned()]);
            }
            other => panic!("expected one staged change, got {other:?}"),
        },
        other => panic!("expected a stale rejection, got {other:?}"),
    }

    // the request carries what the agent observed, at the revision its
    // observation was recorded with.
    let observed_at_1000 = session
        .until(1_000)
        .iter()
        .find_map(|e| match &e.kind {
            Kind::Observed { revision, .. } => Some(*revision),
            _ => None,
        })
        .ok_or("no observation at 1000")?;
    assert_eq!(rejected.receipt.request.observed.revision, observed_at_1000);

    let at_1600 = session.receipts_for(&submit, 1_600);
    assert_eq!(at_1600.len(), 2);
    let done = &at_1600[1].receipt;
    let names: Vec<&str> = done.stages.iter().map(|(s, _)| s.name()).collect();
    assert_eq!(names, ["requested", "accepted", "executing", "completed"]);
    let Some((Stage::Completed(outcome), 1_500)) = done.stages.last() else {
        panic!("expected completed at 1500, got {:?}", done.stages.last());
    };
    let files = vec!["A".to_owned(), "C".to_owned()];
    assert_eq!(
        outcome.result.get("commit"),
        Some(&Value::Text(sha(&files, "fix login")))
    );
    assert_eq!(outcome.result.get("files"), Some(&Value::List(files)));
    assert_eq!(git.commits.len(), 1);

    // the unstage touched the list, not the button.
    let on_changes = session.receipts_for(&id("changes"), 1_600);
    assert_eq!(
        on_changes.len(),
        3,
        "unstage targets it, both commits read it"
    );
    Ok(())
}
