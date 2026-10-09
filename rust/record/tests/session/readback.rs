//! replay never creates a request and never executes: reading a recorded
//! session back, all of it, leaves the app, its dispatcher and the live
//! recording exactly as they were. the reader holds no dispatcher and no
//! app; a session rebuilt from its events alone reads the same.

use record::{Limits, Recorder, Session};
use accord::{Actor, Dispatcher, Request, RequestId, SemId};

use crate::toys::{Git, agent, look};

type Fallible = Result<(), Box<dyn std::error::Error>>;

/// every read the session offers, at every instant it holds.
fn read_everything(session: &Session) -> Result<usize, Box<dyn std::error::Error>> {
    let (start, end) = session.span().ok_or("empty session")?;
    let mut reads = 0;
    let ids: Vec<SemId> = ["changes", "commit.message", "commit.submit"]
        .iter()
        .map(|t| SemId::parse(t))
        .collect::<Result<_, _>>()?;
    for t in start.saturating_sub(10)..=end + 10 {
        reads += session.until(t).len();
        if let Some(state) = session.state_at(t)? {
            state.revision()?;
            reads += state.len();
        }
        reads += session.keyframe_at(t).map_or(0, |_| 1);
        reads += session.receipts(t).len();
        for id in &ids {
            reads += session.receipts_for(id, t).len();
        }
    }
    Ok(reads)
}

#[test]
fn reading_a_session_never_dispatches_nor_executes() -> Fallible {
    let rec = Recorder::new(Limits::default());
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut git = Git::new(&["A", "B", "C"], "fix login");

    let seen = look(&git);
    rec.snapshot(0, &seen);
    let commit = Request::observe(&seen, agent(), "commit.submit", "submit", None)?;
    let unstage = Request::observe(&seen, Actor::Human, "changes", "unstage", Some("B".into()))?;
    dispatcher.dispatch(&mut git, unstage, 10);
    rec.snapshot(10, &look(&git));
    dispatcher.dispatch(&mut git, commit, 20);
    let again = Request::observe(&look(&git), agent(), "commit.submit", "submit", None)?;
    dispatcher.dispatch(&mut git, again, 30);
    rec.snapshot(30, &look(&git));

    let executed = git.executed;
    let commits = git.commits.clone();
    let recorded = rec.stats().events;
    let next = RequestId::parse("4")?;
    assert!(dispatcher.receipt(next).is_none());

    let session = rec.snapshot_window();
    let reads = read_everything(&session)?;
    assert!(reads > 0);

    // a session rebuilt from its events alone, no recorder, no
    // dispatcher, no app anywhere near it, reads the same.
    let cold = Session::from_events(session.events().to_vec());
    assert_eq!(read_everything(&cold)?, reads);
    assert_eq!(cold, session);

    assert_eq!(git.executed, executed, "nothing was executed again");
    assert_eq!(git.commits, commits, "no second commit");
    assert!(dispatcher.receipt(next).is_none(), "no request was created");
    assert_eq!(rec.stats().events, recorded, "the sink saw no new stage");
    Ok(())
}
