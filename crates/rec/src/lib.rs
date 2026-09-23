//! the flight recorder of a plev session (task-63,
//! docs/ADR/session-recording-replay-not-reexecution.md).
//!
//! a bug report should be an inspectable excerpt, not a video. the
//! recorder keeps a session as data that explains what happened: who
//! observed what (`Kind::Observed`), the state the app projected
//! (`Kind::Snapshot`, whole on a cadence and whenever nodes come, move or
//! reorder, `Kind::Changes` in between), who asked for which action on
//! which state and how it went, stage by stage (`Kind::Stage`, from a
//! `sem::ReceiptSink`), and a person's input with the control under it
//! (`Kind::Input`).
//!
//! it only reads. nothing it is given is changed, no clock is read (every
//! time is the caller's), and a secret never lands in it: the snapshot
//! already keeps only the length of a `secret` field, and the recorder
//! reduces to a length every parameter, observed value, delta, outcome
//! or error aimed at one.
//!
//! the read side is `Session`: events by time, the semantic state at any
//! instant, the receipts that touch a control. replay is the promise,
//! re-execution is not: a session holds no dispatcher and no app, so
//! reading it can never create a request or run an action.
//!
//! ```
//! use std::collections::BTreeSet;
//!
//! use rec::{Limits, Recorder};
//! use sem::{Actor, Dispatcher, Execution, Operable, Outcome, Request, RequestId, Role};
//! use sem::{Semantics, SnapshotBuilder, StateRef, Value};
//!
//! struct Git {
//!     staged: BTreeSet<String>,
//! }
//!
//! impl Semantics for Git {
//!     fn project(&self, b: &mut SnapshotBuilder) {
//!         b.node("changes", Role::List, "changes")
//!             .state("staged", self.staged.clone());
//!         b.node("commit.submit", Role::Button, "commit")
//!             .action("submit")
//!             .reads("changes:staged");
//!     }
//! }
//!
//! impl Operable for Git {
//!     fn execute(&mut self, _id: RequestId, _req: &Request) -> Execution {
//!         self.staged.clear();
//!         Execution::Done(Outcome::default())
//!     }
//! }
//!
//! fn look(git: &Git) -> sem::Snapshot {
//!     let mut b = SnapshotBuilder::new();
//!     git.project(&mut b);
//!     b.finish().expect("the projection is valid")
//! }
//!
//! let rec = Recorder::new(Limits::default());
//! let mut dispatcher = Dispatcher::new();
//! dispatcher.add_sink(Box::new(rec.sink()));
//! let mut git = Git { staged: BTreeSet::from(["a.rs".to_owned()]) };
//!
//! let seen = look(&git);
//! rec.snapshot(100, &seen);
//! let req = Request::observe(&seen, Actor::Human, "commit.submit", "submit", None)?;
//! dispatcher.dispatch(&mut git, req, 200);
//! rec.snapshot(200, &look(&git));
//!
//! let session = rec.snapshot_window();
//! let staged = StateRef::parse("changes:staged")?;
//! let before = session.state_at(150)?.ok_or("no state")?;
//! assert_eq!(before.value(&staged), Some(&Value::Set(BTreeSet::from(["a.rs".to_owned()]))));
//! let after = session.state_at(200)?.ok_or("no state")?;
//! assert_eq!(after.value(&staged), Some(&Value::Set(BTreeSet::new())));
//! assert_eq!(session.receipts_for(&"commit.submit".parse()?, 200).len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod error;
mod event;
mod limits;
mod recorder;
mod redact;
mod ring;
mod session;
mod state;
mod track;
mod weigh;

pub use error::ReadError;
pub use event::{Event, InputKind, Kind};
pub use limits::{Limits, Stats};
pub use recorder::{Recorder, Sink};
pub use session::{ReceiptAt, Session};
pub use state::State;
