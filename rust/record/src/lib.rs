//! the flight recorder of a plev session (task-63,
//! docs/decisoes/gravacao/0037-session-recording-replay-not-reexecution.md).
//!
//! a bug report should be an inspectable excerpt, not a video. the
//! recorder keeps a session as data that explains what happened: who
//! observed what (`Kind::Observed`), the state the app projected
//! (`Kind::Snapshot`, whole on a cadence and whenever nodes come, move or
//! reorder, `Kind::Changes` in between), who asked for which action on
//! which state and how it went, stage by stage (`Kind::Stage`, from a
//! `accord::ReceiptSink`), and a person's input with the control under it
//! (`Kind::Input`). beside that semantic track, the visual track keeps
//! what was on screen: every rendered frame, raised to the monster IR
//! and compressed by monster's delta discovery in chunks
//! (`Recorder::frame`). one ring holds both, bounded by time and by
//! bytes (`Limits`), and cuts only at a keyframe of each track.
//!
//! it only reads. nothing it is given is changed, no clock is read (every
//! time is the caller's), and a secret never lands in it: the snapshot
//! already keeps only the length of a `secret` field, and the recorder
//! reduces to a length every parameter, observed value, delta, outcome
//! or error aimed at one, with `accord::redactor`, the rule the agent port
//! applies too.
//!
//! the read side is `Session`: events by time, the semantic state at any
//! instant, the receipts that touch a control, the frame on screen at any
//! instant and where each bound control showed in it (the semantic track
//! reads without decoding a frame). replay
//! is the promise, re-execution is not: a session holds no dispatcher and
//! no app, so reading it can never create a request or run an action.
//!
//! ```
//! use std::collections::BTreeSet;
//!
//! use record::{Limits, Recorder};
//! use accord::{Actor, Dispatcher, Execution, Operable, Outcome, Request, RequestId, Role};
//! use accord::{Semantics, SnapshotBuilder, StateRef, Value};
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
//! fn look(git: &Git) -> accord::Snapshot {
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

mod bound;
mod error;
mod event;
mod format;
mod hook;
mod limits;
mod payload;
mod persist;
mod recorder;
mod ring;
pub mod scene;
mod session;
mod state;
mod track;
mod visuals;
mod weigh;

pub use bound::Bound;
pub use error::{FileError, FrameError, ReadError};
pub use event::{Event, InputKind, Kind};
pub use format::{ANIMATION_MAGIC, Build, FileKind, MAGIC, VERSION, VERSION_BINDINGS, probe};
pub use hook::install_panic_hook;
pub use limits::{Limits, Stats};
pub use persist::Persisted;
pub use recorder::{Recorder, Sink};
pub use session::{ReceiptAt, Session};
pub use state::State;
