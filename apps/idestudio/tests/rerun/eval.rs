//! the ide's fixture, and the sessions of the laboratory rerun against it.
//!
//! the fixture is the laboratory's own start (`src/bin/lab/bench.rs`,
//! `Bench::open`): the repository of the experiment built in a temporary
//! directory (`ide::lab_repo::Repo::build`, the same code the laboratory
//! runs), a real git worker on it, and the `Workbench` connected and
//! loaded. the worker only ever runs `git` in that directory; nothing
//! reaches the network.
//!
//! the workbench stamps its receipts on its own monotonic clock and takes
//! none: the times the harness gives are not used. nothing the harness
//! compares reads that clock (stages by name, outcomes by key, the
//! projection by value), and the toasts' timers move only through
//! `Workbench::wake`, which the fixture never calls.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use git::{GitClient, GitEvent};
use ide::lab_repo::Repo;
use ide::workbench::Workbench;
use rec::Session;
use sem::{Actor, Millis, Receipt, Request, RequestId, Snapshot};

use crate::harness::{Fixture, rerun};
use crate::marks;
use crate::report::Report;

/// the window the laboratory draws for (`src/bin/lab/bench.rs`).
const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 800.0;

/// how long a git answer may take before the rerun fails.
const WAIT: Duration = Duration::from_secs(10);

/// the laboratory's workbench on a fresh repository of the experiment.
pub struct Lab {
    /// the temporary directory lives as long as the fixture.
    repo: Repo,
    bench: Workbench,
    events: Receiver<GitEvent>,
    /// the refreshes asked and not answered yet: the connection's, one
    /// per mutation the worker answered, one per settle.
    owed: usize,
}

impl Lab {
    /// the repository built, the worker spawned, the workbench connected
    /// and loaded (status, log and branches applied).
    pub fn start() -> Result<Lab, String> {
        let repo = Repo::build()?;
        let (tx, events) = channel();
        let client = GitClient::spawn(repo.path(), move |event| {
            let _ = tx.send(event);
        })
        .map_err(|e| format!("the git worker on the repository: {e}"))?;
        let mut bench = Workbench::new(WIDTH, HEIGHT);
        bench.connect(client);
        let mut lab = Lab {
            repo,
            bench,
            events,
            owed: 1,
        };
        lab.drain()?;
        Ok(lab)
    }

    /// applies the worker's events, in order, until every refresh owed
    /// was answered. the worker answers in the order it was asked, a
    /// refresh ends with its `Branches`, and every mutation it answers
    /// asks for one more (`GitSession::apply`).
    fn drain(&mut self) -> Result<(), String> {
        while self.owed > 0 {
            self.take()?;
        }
        Ok(())
    }

    /// applies the worker's answers, one at a time and in order, while
    /// `busy` holds for the workbench; every refresh asked on the way
    /// stays owed, for a later `drain` (task-80: the rerun with the
    /// reads in flight, `crate::inflight`).
    pub fn answer_while(&mut self, busy: impl Fn(&Workbench) -> bool) -> Result<(), String> {
        while busy(&self.bench) {
            self.take()?;
        }
        Ok(())
    }

    /// the worker's next answer applied, counting the refresh a mutation
    /// answered asks for and the one a `Branches` ends.
    fn take(&mut self) -> Result<(), String> {
        let event = self
            .events
            .recv_timeout(WAIT)
            .map_err(|e| format!("the git worker did not answer in time: {e}"))?;
        match event {
            GitEvent::Staged { .. }
            | GitEvent::Unstaged { .. }
            | GitEvent::Discarded { .. }
            | GitEvent::Ignored { .. }
            | GitEvent::Committed(_)
            | GitEvent::CommitRejected { .. } => self.owed += 1,
            GitEvent::Branches(_) => self.owed = self.owed.saturating_sub(1),
            _ => {}
        }
        self.bench.apply(event);
        Ok(())
    }

    /// `git` in the repository, for a test that looks at it after.
    pub fn git(&self, args: &[&str]) -> Result<String, String> {
        self.repo.git(args)
    }
}

impl Fixture for Lab {
    fn dispatch(&mut self, req: Request, _t: Millis) -> RequestId {
        self.bench.dispatch(req)
    }

    fn confirm(&mut self, id: RequestId, by: Actor, _t: Millis) -> Result<(), String> {
        self.bench.confirm(id, by).map_err(|e| e.to_string())
    }

    fn decline(&mut self, id: RequestId, by: Actor, _t: Millis) -> Result<(), String> {
        self.bench.decline(id, by).map_err(|e| e.to_string())
    }

    /// a refresh asked now is answered after everything sent before it,
    /// so once it is, every execution that left for the worker ended and
    /// every read landed.
    fn settle(&mut self) -> Result<(), String> {
        self.bench.refresh();
        self.owed += 1;
        self.drain()
    }

    fn project(&self) -> Result<Snapshot, String> {
        self.bench.snapshot().map_err(|e| e.to_string())
    }

    fn receipt(&self, id: RequestId) -> Option<Receipt> {
        self.bench.receipt(id).cloned()
    }
}

/// `crates/ide/tests/fixtures/sessions/<name>.monster` and its marks.
pub fn fixture(name: &str) -> (Session, Vec<marks::Mark>) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sessions");
    let excerpt = dir.join(format!("{name}.monster"));
    let session = match Session::open(&excerpt) {
        Ok(session) => session,
        Err(e) => panic!("{}: {e}", excerpt.display()),
    };
    let marks = dir.join(format!("{name}.marks"));
    let text = match std::fs::read_to_string(&marks) {
        Ok(text) => text,
        Err(e) => panic!("{}: {e}", marks.display()),
    };
    match marks::parse(&text) {
        Ok(marks) => (session, marks),
        Err(e) => panic!("{}: {e}", marks.display()),
    }
}

fn rerun_fixture(name: &str, lab: &mut Lab) -> Report {
    let (session, marks) = fixture(name);
    match rerun(&session, &marks, || Ok(lab)) {
        Ok(report) => report,
        Err(e) => panic!("{name}: the rerun could not be made: {e}"),
    }
}

/// the third round's a2ppa session (an LLM agent, a scripted person who
/// unstages a file right before the agent's submit reaches the
/// dispatcher), exported with `plev-inspect --export`: the stale refusal
/// and the recovery come out as recorded, and the commit holds the three
/// files of the task.
#[test]
fn the_a2ppa_session_reruns_as_it_was_recorded() {
    let mut lab = match Lab::start() {
        Ok(lab) => lab,
        Err(e) => panic!("the fixture: {e}"),
    };
    let report = rerun_fixture("a2ppa", &mut lab);
    assert!(!report.diverged(), "{report}");
    println!("{report}");
    assert_eq!(report.requests.len(), 9);
    assert_eq!(report.marks.len(), 6);
    // the rerun's commit is in the temporary repository, and only there.
    let files = lab.git(&["show", "--name-only", "--format=%s", "HEAD"]);
    assert_eq!(
        files
            .as_deref()
            .map(str::lines)
            .map(Iterator::collect::<Vec<_>>),
        Ok(vec![
            "fix login session timeout",
            "",
            "src/login.rs",
            "src/session.rs",
            "tests/login_test.rs"
        ])
    );
}
