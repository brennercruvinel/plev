//! a secret never lands in the recorder: not in the state (the snapshot
//! keeps its length), not in a parameter, an observed value, a stale
//! delta, an outcome or an error aimed at a secret field.

use record::{Event, Kind, Limits, Recorder};
use accord::{Actor, Dispatcher, Observation, Rejection, Request, Stage, StateRef, Value};

use crate::toys::{Git, Login, agent, id, look};

type Fallible = Result<(), Box<dyn std::error::Error>>;

const SECRET: &str = "hunter2";

/// every rendering of the recording a secret could hide in.
fn dumps(rec: &Recorder) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let session = rec.snapshot_window();
    let file = session.to_bytes()?.0;
    Ok(vec![
        format!("{rec:?}"),
        format!("{session:?}"),
        serde_json::to_string(session.events())?,
        String::from_utf8_lossy(&file).into_owned(),
    ])
}

fn stages(events: &[Event]) -> Vec<(&Request, &Stage)> {
    events
        .iter()
        .filter_map(|e| match &e.kind {
            Kind::Stage { request, stage, .. } => Some((request.as_ref(), stage)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_secret_is_never_recorded_in_state_or_param() -> Fallible {
    let rec = Recorder::new(Limits::default());
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut login = Login {
        user: "ana".to_owned(),
        password: String::new(),
    };
    rec.snapshot(0, &look(&login));

    // the person types the password: the param goes raw to the app.
    let seen = look(&login);
    let set = Request::observe(
        &seen,
        Actor::Human,
        "login.password",
        "set",
        Some(SECRET.into()),
    )?;
    dispatcher.dispatch(&mut login, set, 10);
    assert_eq!(login.password, SECRET, "the app got the secret");
    rec.snapshot(10, &look(&login));

    // a wrong kind of param: refused, and the reason is not kept.
    let wrong = Request::observe(
        &seen,
        Actor::Human,
        "login.password",
        "set",
        Some(Value::List(vec![SECRET.to_owned()])),
    )?;
    dispatcher.dispatch(&mut login, wrong, 20);

    // an agent that saw the raw secret (a hand-built observation) asks to
    // sign in: stale, and its delta names the secret it saw.
    let seen = look(&login);
    let mut submit = Request::observe(&seen, agent(), "login.submit", "submit", None)?;
    let password = StateRef::parse("login.password:text")?;
    submit.observed = Observation {
        revision: seen.revision(),
        values: vec![
            (StateRef::parse("login.user:text")?, Some("ana".into())),
            (password.clone(), Some(SECRET.into())),
        ],
    };
    dispatcher.dispatch(&mut login, submit, 30);

    for dump in dumps(&rec)? {
        assert!(!dump.contains(SECRET), "the secret leaked into {dump}");
    }

    let session = rec.snapshot_window();
    let state = session.state_at(30)?.ok_or("no state")?;
    assert_eq!(state.value(&password), Some(&Value::Int(7)), "length only");

    let events = session.events();
    let recorded = stages(events);
    let (set, _) = recorded.first().ok_or("no stage")?;
    assert_eq!(set.param, Some(Value::Int(7)), "the param keeps its length");
    let echo = recorded.iter().find_map(|(_, s)| match s {
        Stage::Completed(o) => o.result.get("echo").cloned(),
        _ => None,
    });
    assert_eq!(echo, Some(Value::Int(7)), "the outcome keeps a length");
    let refused = recorded.iter().find_map(|(r, s)| match s {
        Stage::Rejected(Rejection::InvalidParam { reason }) => Some((r.param.clone(), reason)),
        _ => None,
    });
    let (param, reason) = refused.ok_or("no refusal")?;
    assert_eq!(param, Some(Value::Int(1)), "a list keeps its item count");
    assert!(reason.starts_with("redacted"), "reason {reason}");
    let (stale_req, stale) = recorded
        .iter()
        .find(|(_, s)| matches!(s, Stage::Rejected(Rejection::Stale { .. })))
        .ok_or("no stale rejection")?;
    let observed = stale_req
        .observed
        .values
        .iter()
        .find(|(at, _)| *at == password);
    assert_eq!(observed, Some(&(password.clone(), Some(Value::Int(7)))));
    let Stage::Rejected(Rejection::Stale { changes, .. }) = stale else {
        return Err("not stale".into());
    };
    assert!(!changes.is_empty());
    Ok(())
}

#[test]
fn a_param_is_redacted_when_the_recorder_does_not_know_its_target() -> Fallible {
    // no snapshot given: the recorder cannot know what is secret, so it
    // fails closed.
    let rec = Recorder::new(Limits::default());
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut git = Git::new(&["A", "B"], "wip");
    let seen = look(&git);
    let unstage = Request::observe(&seen, Actor::Human, "changes", "unstage", Some("B".into()))?;
    dispatcher.dispatch(&mut git, unstage, 5);

    let session = rec.snapshot_window();
    let recorded = stages(session.events());
    let (request, _) = recorded.first().ok_or("no stage")?;
    assert_eq!(request.param, Some(Value::Int(1)));
    assert!(
        request
            .observed
            .values
            .iter()
            .all(|(_, v)| matches!(v, Some(Value::Int(_)))),
        "{:?}",
        request.observed
    );

    // once the recorder knows the target is not secret, the param stays.
    rec.snapshot(6, &look(&git));
    let unstage = Request::observe(
        &look(&git),
        Actor::Human,
        "changes",
        "unstage",
        Some("A".into()),
    )?;
    dispatcher.dispatch(&mut git, unstage, 7);
    let session = rec.snapshot_window();
    let kept = stages(session.events())
        .last()
        .map(|(r, _)| r.param.clone());
    assert_eq!(kept, Some(Some(Value::Text("A".to_owned()))));
    assert_eq!(git.staged.len(), 0);
    assert!(session.receipts_for(&id("changes"), 7).len() == 2);
    Ok(())
}
