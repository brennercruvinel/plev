//! a race the rerun at rest cannot see, caught by a recorded session
//! (task-80 demo 3: the task-84 race, `crates/ide/src/session.rs`).
//!
//! the ide's fixture settles by draining every refresh owed before each
//! act (`Lab::settle`), so when the next act leaves, no read is in
//! flight, and a status read before a mutation can never land after it.
//! a live window has no such pause: the answer to a stage asks for a
//! refresh, and the agent's next act, chained on that answer, leaves
//! while the refresh is still on the worker. `InFlight` settles as that
//! window does: the worker's answers are applied one at a time, in the
//! order it gives them, while an act dispatched in the rerun is still
//! executing, and every read asked on the way stays in flight. the
//! recording allows it (each act came after the one before ended), and
//! the worker answers in the order it was asked, so the rerun is the
//! same on every run.
//!
//! this is a narrower settle than the harness asks of a fixture (every
//! read it asked for lands), on purpose and only here: the harness, the
//! `Fixture` trait and the marks are as they were.

use idestudio::capability::Workbench;
use accord::{Actor, Millis, Receipt, Request, RequestId, Snapshot, Stage};

use crate::harness::{Fixture, rerun};
use crate::lab::{Lab, fixture};

/// the laboratory's workbench, settled only as far as its executions.
pub struct InFlight<'a> {
    lab: &'a mut Lab,
    /// every request the rerun dispatched, in order.
    dispatched: Vec<RequestId>,
}

impl<'a> InFlight<'a> {
    pub fn new(lab: &'a mut Lab) -> InFlight<'a> {
        InFlight {
            lab,
            dispatched: Vec::new(),
        }
    }
}

/// true while a request `ids` names is still executing in `bench`.
fn executing(bench: &Workbench, ids: &[RequestId]) -> bool {
    ids.iter().any(|id| {
        matches!(
            bench.receipt(*id).and_then(Receipt::stage),
            Some(Stage::Executing)
        )
    })
}

impl Fixture for InFlight<'_> {
    fn dispatch(&mut self, req: Request, t: Millis) -> RequestId {
        let id = self.lab.dispatch(req, t);
        self.dispatched.push(id);
        id
    }

    fn confirm(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String> {
        self.lab.confirm(id, by, t)
    }

    fn decline(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String> {
        self.lab.decline(id, by, t)
    }

    /// every execution that left ends; the reads it asked for stay in
    /// flight.
    fn settle(&mut self) -> Result<(), String> {
        let ids = &self.dispatched;
        self.lab.answer_while(|bench| executing(bench, ids))
    }

    fn project(&self) -> Result<Snapshot, String> {
        self.lab.project()
    }

    fn receipt(&self, id: RequestId) -> Option<Receipt> {
        self.lab.receipt(id)
    }
}

/// the task-84 race, recorded by the laboratory with the race back
/// (branch `task/80a-bug84`, never merged) and exported as a bug report:
/// the scripted agent unstages README.md and stages src/session.rs and
/// tests/login_test.rs, each act chained on the answer to the one before.
/// in the recording each staged file left `changes:staged` while its own
/// stage was executing. rerun with the reads in flight, the view keeps
/// every file the ide accepted; with the race back, the marks after
/// requests 2 and 3 say which file went missing.
#[test]
fn the_bug84_excerpt_keeps_every_staged_file_with_the_reads_in_flight() {
    let mut lab = match Lab::start() {
        Ok(lab) => lab,
        Err(e) => panic!("the fixture: {e}"),
    };
    let (session, marks) = fixture("bug84");
    let report = match rerun(&session, &marks, || Ok(InFlight::new(&mut lab))) {
        Ok(report) => report,
        Err(e) => panic!("bug84: the rerun could not be made: {e}"),
    };
    assert!(!report.diverged(), "{report}");
    println!("{report}");
    assert_eq!(report.requests.len(), 4);
    assert_eq!(report.marks.len(), 6);
}
