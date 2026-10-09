//! `why`: the observation a request rested on (found, or not recorded and
//! why), the request with what it read, what moved in between and who ran
//! then (another actor, nobody from the app, or several), the stages and
//! the result.

use replay::{Basis, By, Ending, Inspector, Unrecorded, Why};
use record::InputKind;
use accord::{Actor, Change, Rejection, RequestId, Stage, Value};

use crate::apps::{Lab, agent, id, rid, sha};
use crate::thesis::record;

type Fallible = Result<(), Box<dyn std::error::Error>>;

fn set(items: &[&str]) -> Value {
    Value::Set(items.iter().map(|i| (*i).to_owned()).collect())
}

fn why(session: &record::Session, id: RequestId) -> Result<Why, Box<dyn std::error::Error>> {
    Ok(Inspector::new(session)
        .why(0, id)?
        .ok_or("no such request")?)
}

#[test]
fn the_stale_submit_rests_on_the_agents_observation() -> Fallible {
    let thesis = record();
    let why = why(&thesis.session, thesis.stale)?;
    let Basis::Observed {
        at,
        scope,
        revision,
        ..
    } = &why.basis
    else {
        panic!("expected the observation, got {:?}", why.basis);
    };
    assert_eq!((*at, scope), (0, &None));
    assert_eq!(*revision, why.request.revision);
    assert_eq!(why.request.actor, agent());
    assert_eq!(why.request.target, id("commit.submit"));
    assert_eq!(why.request.action.as_str(), "submit");
    assert_eq!(why.request.at, 300);
    let control = why
        .request
        .control
        .as_ref()
        .ok_or("the button was projected")?;
    assert_eq!(control.label, "Commit");
    let reads: Vec<(String, Option<Value>, Option<bool>)> = why
        .request
        .reads
        .iter()
        .map(|r| (r.at.to_string(), r.seen.clone(), r.in_scope))
        .collect();
    assert_eq!(
        reads,
        [
            (
                "changes:staged".into(),
                Some(set(&["A", "B", "C"])),
                Some(true)
            ),
            (
                "commit.message:text".into(),
                Some("fix login".into()),
                Some(true)
            ),
        ]
    );
    Ok(())
}

#[test]
fn a_move_between_the_observation_and_the_request_names_the_person_who_ran() -> Fallible {
    let thesis = record();
    let why = why(&thesis.session, thesis.stale)?;
    assert_eq!(why.between.from.map(|s| s.at), Some(0));
    assert_eq!(why.between.to.at, 300);
    let staged = &why.between.reads[0];
    assert_eq!(staged.at.to_string(), "changes:staged");
    assert_eq!(staged.before_request, Some(set(&["A", "C"])));
    let [moved] = staged.moves.as_slice() else {
        panic!("one move, got {:?}", staged.moves);
    };
    assert_eq!(moved.at, 100);
    assert_eq!(moved.since.map(|s| s.at), Some(0), "the state before it");
    assert_eq!(
        (moved.before.clone(), moved.after.clone()),
        (Some(set(&["A", "B", "C"])), Some(set(&["A", "C"])))
    );
    assert_eq!(
        (moved.added.as_slice(), moved.removed.as_slice()),
        (&[][..], &["B".to_owned()][..])
    );
    let By::Receipt { cause } = &moved.by else {
        panic!("one receipt, got {:?}", moved.by);
    };
    assert_eq!(cause.id, thesis.unstage);
    assert_eq!(cause.actor, Actor::Human);
    assert_eq!(cause.action.as_str(), "unstage");
    assert_eq!(cause.param, Some(Value::Text("B".into())));
    assert_eq!(
        (cause.stage, cause.at, cause.same_actor),
        ("executing", 100, false)
    );
    assert!(
        why.between.reads[1].moves.is_empty(),
        "the message did not move"
    );
    let inputs: Vec<(u64, InputKind)> =
        why.between.inputs.iter().map(|i| (i.at, i.input)).collect();
    assert_eq!(inputs, [(100, InputKind::PointerDown)]);
    Ok(())
}

#[test]
fn the_stale_refusal_carries_the_delta_and_who_acted() -> Fallible {
    let thesis = record();
    let why = why(&thesis.session, thesis.stale)?;
    let names: Vec<(u64, &str)> = why.stages.iter().map(|s| (s.at, s.stage.name())).collect();
    assert_eq!(names, [(300, "requested"), (300, "rejected")]);
    let Ending::Rejected {
        rejection: Rejection::Stale { changes, acted },
        at_truth,
        ..
    } = &why.ending
    else {
        panic!("a stale refusal, got {:?}", why.ending);
    };
    assert!(!at_truth);
    assert!(matches!(
        changes.as_slice(),
        [Change::StateChanged { removed, .. }] if removed == &["B".to_owned()]
    ));
    let acted: Vec<(RequestId, &Actor)> = acted.iter().map(|a| (a.id, &a.actor)).collect();
    assert_eq!(acted, [(thesis.unstage, &Actor::Human)]);
    Ok(())
}

#[test]
fn the_completed_submit_has_its_sha_and_files() -> Fallible {
    let thesis = record();
    let why = why(&thesis.session, thesis.done)?;
    assert!(matches!(why.basis, Basis::Observed { at: 400, .. }));
    assert!(why.between.reads.iter().all(|r| r.moves.is_empty()));
    let names: Vec<&str> = why.stages.iter().map(|s| s.stage.name()).collect();
    assert_eq!(names, ["requested", "accepted", "executing", "completed"]);
    let Ending::Completed { outcome, at, .. } = &why.ending else {
        panic!("completed, got {:?}", why.ending);
    };
    assert_eq!(*at, 500);
    let files = vec!["A".to_owned(), "C".to_owned()];
    assert_eq!(
        outcome.result.get("commit"),
        Some(&Value::Text(sha(&files, "fix login")))
    );
    assert_eq!(outcome.result.get("files"), Some(&Value::List(files)));
    Ok(())
}

/// a person has no recorded observation: the recorder keeps the ones the
/// app reports. the state still says when it was at the revision the
/// request carries, and the effect the request had in the same
/// millisecond is not a move before it.
#[test]
fn a_persons_request_has_no_recorded_observation() -> Fallible {
    let thesis = record();
    let why = why(&thesis.session, thesis.unstage)?;
    let Basis::NotRecorded { reason, state } = &why.basis else {
        panic!("not recorded, got {:?}", why.basis);
    };
    assert_eq!(*reason, Unrecorded::NoObservation);
    assert_eq!(state.map(|s| s.at), Some(0));
    assert_eq!(why.between.from.map(|s| s.at), Some(0));
    assert!(why.request.reads.iter().all(|r| r.in_scope.is_none()));
    assert!(
        why.between.reads.iter().all(|r| r.moves.is_empty()),
        "the unstage's own effect comes after it: {:?}",
        why.between.reads
    );
    assert_eq!(why.between.inputs.len(), 1, "the click before it");
    assert!(matches!(why.ending, Ending::Completed { .. }));
    Ok(())
}

/// an agent that acts on a revision it never observed through the port
/// (the last stage of an earlier act carries one): the observations it
/// has are at other revisions.
#[test]
fn an_act_on_a_revision_no_observation_served() -> Fallible {
    let mut lab = Lab::new(&["A", "B", "C"], "fix login", 0);
    let first = lab.observe(10, agent());
    lab.person_unstages(20, "B");
    let now = lab.look();
    let submit = lab.act(30, &now, agent(), "commit.submit", "submit", None);
    let why = why(&lab.session(), submit)?;
    let Basis::NotRecorded { reason, state } = &why.basis else {
        panic!("not recorded, got {:?}", why.basis);
    };
    assert_eq!(
        *reason,
        Unrecorded::OtherRevision {
            t: 10,
            at: 10,
            revision: first.revision()
        }
    );
    assert_eq!(state.map(|s| s.at), Some(20), "the state after the unstage");
    assert!(why.between.reads.iter().all(|r| r.moves.is_empty()));
    assert!(matches!(why.ending, Ending::Completed { .. }));
    Ok(())
}

/// a revision the recorded state never had: no stretch to read, only what
/// the actor saw beside what the state held right before the request.
#[test]
fn a_revision_the_recording_never_held() -> Fallible {
    let mut lab = Lab::new(&["A", "B"], "fix login", 0);
    let mut elsewhere = crate::apps::Git::new(&["A", "B", "C"], "fix login");
    elsewhere.message = "other".into();
    let seen = crate::apps::look(&elsewhere);
    let submit = lab.act(10, &seen, agent(), "commit.submit", "submit", None);
    let why = why(&lab.session(), submit)?;
    assert_eq!(
        why.basis,
        Basis::NotRecorded {
            reason: Unrecorded::NoObservation,
            state: None
        }
    );
    assert_eq!(why.between.from, None);
    let staged = &why.between.reads[0];
    assert_eq!(staged.seen, Some(set(&["A", "B", "C"])));
    assert_eq!(staged.before_request, Some(set(&["A", "B"])));
    assert!(staged.moves.is_empty());
    assert!(matches!(why.ending, Ending::Rejected { .. }));
    Ok(())
}

/// the index moved from a terminal: no receipt ran, so the move is from
/// outside the app.
#[test]
fn a_move_no_receipt_explains_is_from_outside_the_app() -> Fallible {
    let mut lab = Lab::new(&["A", "B", "C"], "fix login", 0);
    let seen = lab.observe(10, agent());
    lab.outside_unstages(20, "C");
    let submit = lab.act(30, &seen, agent(), "commit.submit", "submit", None);
    let why = why(&lab.session(), submit)?;
    let [moved] = why.between.reads[0].moves.as_slice() else {
        panic!("one move, got {:?}", why.between.reads[0].moves);
    };
    assert_eq!((moved.at, moved.by.clone()), (20, By::Outside));
    assert_eq!(moved.removed, ["C".to_owned()]);
    let Ending::Rejected {
        rejection: Rejection::Stale { acted, .. },
        ..
    } = &why.ending
    else {
        panic!("stale, got {:?}", why.ending);
    };
    assert!(acted.is_empty(), "the dispatcher names nobody either");
    Ok(())
}

/// two receipts ran before the one projection that shows the change: the
/// recording cannot tell which moved the value, and says so.
#[test]
fn two_receipts_in_the_window_are_ambiguous_and_both_shown() -> Fallible {
    let mut lab = Lab::new(&["A", "B", "C"], "fix login", 0);
    let seen = lab.observe(10, agent());
    // each one acts on the app as it is, and the app projects only once
    // after both: one recorded change, two receipts before it.
    let person = accord::Request::observe(
        &lab.look(),
        Actor::Human,
        "changes",
        "unstage",
        Some("B".into()),
    )?;
    let one = lab.dispatcher.dispatch(&mut lab.git, person, 20);
    let assistive = accord::Request::observe(
        &lab.look(),
        Actor::Assistive,
        "changes",
        "unstage",
        Some("C".into()),
    )?;
    let two = lab.dispatcher.dispatch(&mut lab.git, assistive, 21);
    lab.rec.snapshot(21, &lab.look());
    let submit = lab.act(30, &seen, agent(), "commit.submit", "submit", None);
    let why = why(&lab.session(), submit)?;
    let [moved] = why.between.reads[0].moves.as_slice() else {
        panic!("one move, got {:?}", why.between.reads[0].moves);
    };
    assert_eq!(moved.removed, ["B".to_owned(), "C".to_owned()]);
    let By::Ambiguous { causes } = &moved.by else {
        panic!("ambiguous, got {:?}", moved.by);
    };
    let who: Vec<(RequestId, &Actor)> = causes.iter().map(|c| (c.id, &c.actor)).collect();
    assert_eq!(who, [(one, &Actor::Human), (two, &Actor::Assistive)]);
    Ok(())
}

/// a refused request ran nothing, so it never explains a move; the
/// actor's own earlier act does, and is marked as the same actor.
#[test]
fn a_refused_request_explains_nothing_and_the_same_actor_is_marked() -> Fallible {
    let mut lab = Lab::new(&["A", "B", "C"], "fix login", 0);
    let seen = lab.observe(10, agent());
    let stale_seen = crate::apps::look(&crate::apps::Git::new(&["A"], "x"));
    let refused = lab.act(
        20,
        &stale_seen,
        Actor::Human,
        "commit.submit",
        "submit",
        None,
    );
    let own = lab.act(20, &seen, agent(), "changes", "unstage", Some("C".into()));
    let submit = lab.act(30, &seen, agent(), "commit.submit", "submit", None);
    let session = lab.session();
    assert!(matches!(
        why(&session, refused)?.ending,
        Ending::Rejected { .. }
    ));
    let why = why(&session, submit)?;
    let [moved] = why.between.reads[0].moves.as_slice() else {
        panic!("one move, got {:?}", why.between.reads[0].moves);
    };
    let By::Receipt { cause } = &moved.by else {
        panic!("one receipt, got {:?}", moved.by);
    };
    assert_eq!((cause.id, cause.same_actor), (own, true));
    Ok(())
}

/// an observation of one subtree: the read outside it was not served by
/// that observation.
#[test]
fn a_scoped_observation_says_which_reads_it_served() -> Fallible {
    let mut lab = Lab::new(&["A", "B"], "fix login", 0);
    let seen = lab.observe_scope(10, agent(), "commit.message");
    let submit = lab.act(20, &seen, agent(), "commit.submit", "submit", None);
    let why = why(&lab.session(), submit)?;
    assert!(
        matches!(&why.basis, Basis::Observed { scope: Some(s), .. } if *s == id("commit.message"))
    );
    let scoped: Vec<Option<bool>> = why.request.reads.iter().map(|r| r.in_scope).collect();
    assert_eq!(scoped, [Some(false), Some(true)]);
    Ok(())
}

#[test]
fn a_request_the_window_does_not_hold_is_none() -> Fallible {
    let thesis = record();
    let inspector = Inspector::new(&thesis.session);
    assert!(inspector.why(0, rid(99))?.is_none());
    assert!(inspector.why(1, thesis.stale)?.is_none(), "another origin");
    assert_eq!(inspector.origins_of(thesis.stale), [0]);
    Ok(())
}

/// a window that starts after the request: its first stages were cut, the
/// request is still whole.
#[test]
fn a_request_cut_by_the_window_is_not_whole() -> Fallible {
    let thesis = record();
    let events: Vec<record::Event> = thesis
        .session
        .events()
        .iter()
        .filter(|e| {
            !matches!(&e.kind, record::Kind::Stage { id, stage: Stage::Requested, .. } if *id == thesis.stale)
        })
        .cloned()
        .collect();
    let cut = record::Session::from_events(events);
    let why = why(&cut, thesis.stale)?;
    assert!(!why.whole);
    assert_eq!(why.stages.len(), 1);
    assert!(matches!(why.ending, Ending::Rejected { .. }));
    assert!(matches!(why.basis, Basis::Observed { at: 0, .. }));
    Ok(())
}

/// a submit refused at the point of truth after it started: it ends
/// rejected at the truth, and since it ran nothing it never explains a
/// move.
#[test]
fn a_refusal_at_the_point_of_truth_ran_nothing() -> Fallible {
    let mut lab = Lab::new(&["A", "B", "C"], "fix login", 0);
    lab.git.slow = true;
    let seen = lab.observe(10, agent());
    let now = lab.look();
    let person = lab.act(20, &now, Actor::Human, "commit.submit", "submit", None);
    lab.dispatcher
        .reject_at_truth(person, Rejection::stale(Vec::new()), 30)?;
    lab.outside_unstages(40, "C");
    lab.git.slow = false;
    let submit = lab.act(50, &seen, agent(), "commit.submit", "submit", None);
    let session = lab.session();
    assert!(matches!(
        why(&session, person)?.ending,
        Ending::Rejected { at_truth: true, .. }
    ));
    let why = why(&session, submit)?;
    let [moved] = why.between.reads[0].moves.as_slice() else {
        panic!("one move, got {:?}", why.between.reads[0].moves);
    };
    assert_eq!(moved.by, By::Outside);
    Ok(())
}

/// the documented limit of the correlation: a receipt that started before
/// the recorded state before a change and ended after the change has no
/// stage in the change's window, so it is not named and the move reads as
/// one from outside the app. the ide does not do this (its view moves
/// right after the act starts); an app whose effect lands long after the
/// start, with other projections in between, would.
#[test]
fn a_receipt_running_across_the_window_is_not_named() -> Fallible {
    let mut lab = Lab::new(&["A", "B"], "fix login", 0);
    lab.git.slow = true;
    let seen = lab.observe(10, agent());
    let now = lab.look();
    let person = lab.act(20, &now, Actor::Human, "commit.submit", "submit", None);
    lab.rec.snapshot_full(25, &lab.look());
    lab.git.staged.clear();
    lab.rec.snapshot(40, &lab.look());
    lab.dispatcher
        .complete(person, accord::Outcome::default(), 50)?;
    lab.git.slow = false;
    let submit = lab.act(60, &seen, agent(), "commit.submit", "submit", None);
    let why = why(&lab.session(), submit)?;
    let [moved] = why.between.reads[0].moves.as_slice() else {
        panic!("one move, got {:?}", why.between.reads[0].moves);
    };
    assert_eq!((moved.at, moved.since.map(|s| s.at)), (40, Some(25)));
    assert_eq!(moved.by, By::Outside);
    Ok(())
}
