//! reading a session never creates a request and never runs anything. the
//! crate holds nothing that could: no app, nothing that dispatches, no
//! recorder. an honest check on both sides: the sources name none of it,
//! and inspecting everything a recorded session holds leaves the app, its
//! dispatcher and the recording exactly as they were.

use std::path::Path;

use replay::{Inspector, prose};
use accord::{RequestId, SemId};

use crate::toys::{Lab, agent};

type Fallible = Result<(), Box<dyn std::error::Error>>;

/// every answer the inspector gives, at every instant the session holds,
/// rendered: the number of lines it printed.
fn inspect_everything(session: &record::Session) -> Result<usize, Box<dyn std::error::Error>> {
    let inspector = Inspector::new(session);
    let mut lines = prose::summary(&inspector.summary()?).lines().count();
    lines += prose::timeline(&inspector.timeline()).lines().count();
    for (origin, id) in inspector.requests() {
        let why = inspector.why(origin, id)?.ok_or("a listed request")?;
        lines += prose::why(&why).lines().count();
        lines += serde_json::to_string(&why)?.len().min(1);
    }
    let ids: Vec<SemId> = ["changes", "commit.message", "commit.submit", "nowhere"]
        .iter()
        .map(|t| SemId::parse(t))
        .collect::<Result<_, _>>()?;
    let (start, end) = (inspector.start(), inspector.end());
    for t in start.saturating_sub(5)..=end + 5 {
        for id in &ids {
            lines += prose::control(&inspector.control(id, t)?).lines().count();
        }
    }
    Ok(lines)
}

#[test]
fn inspecting_a_session_runs_nothing_and_asks_for_nothing() -> Fallible {
    let mut lab = Lab::new(&["A", "B", "C"], "fix login", 0);
    let seen = lab.observe(10, agent());
    lab.person_unstages(20, "B");
    lab.act(30, &seen, agent(), "commit.submit", "submit", None);
    let again = lab.observe(40, agent());
    lab.act(50, &again, agent(), "commit.submit", "submit", None);

    let executed = lab.git.executed;
    let commits = lab.git.commits.clone();
    let recorded = lab.rec.stats().events;
    let next = RequestId::parse("4")?;
    assert!(lab.dispatcher.receipt(next).is_none());
    let session = lab.session();

    let lines = inspect_everything(&session)?;
    assert!(lines > 0);
    // a session rebuilt from its events alone, with nothing else near it,
    // answers the same.
    let cold = record::Session::from_events(session.events().to_vec());
    assert_eq!(inspect_everything(&cold)?, lines);

    assert_eq!(lab.git.executed, executed, "nothing was executed");
    assert_eq!(lab.git.commits, commits, "no second commit");
    assert!(
        lab.dispatcher.receipt(next).is_none(),
        "no request was created"
    );
    assert_eq!(lab.rec.stats().events, recorded, "the sink saw nothing new");
    assert_eq!(lab.session(), session, "the recording is as it was");
    Ok(())
}

/// what the crate's own sources must never name: the types that run an
/// action or record one, and the calls that do.
const RUNS: [&str; 9] = [
    "Dispatcher",
    "Operable",
    "ReceiptSink",
    "Execution",
    "Recorder",
    ".dispatch(",
    ".execute(",
    ".confirm(",
    ".decline(",
];

fn sources(dir: &Path, out: &mut Vec<(String, String)>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            sources(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push((path.display().to_string(), std::fs::read_to_string(&path)?));
        }
    }
    Ok(())
}

#[test]
fn the_crate_holds_nothing_that_runs_an_action() -> Fallible {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files)?;
    assert!(files.len() >= 8, "the sources were read: {}", files.len());
    for (path, body) in &files {
        for word in RUNS {
            assert!(!body.contains(word), "{path} names {word}");
        }
    }
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|rest| rest.split("\n[").next())
        .ok_or("a [dependencies] table")?;
    let names: Vec<&str> = deps
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split(['.', ' ', '=']).next())
        .collect();
    assert_eq!(names, ["rec", "sem", "serde", "serde_json"]);
    Ok(())
}
