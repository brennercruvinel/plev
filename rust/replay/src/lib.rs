//! the session inspector (task-64 phase a,
//! docs/ADR/the-inspector-reads-a-session-never-runs-it.md).
//!
//! a recorded session (`rec::Session`, a `.monster` whose magic is `MONS`)
//! holds who observed what, the state the app projected at every step, and
//! every stage of every receipt, each with its time. the inspector reads
//! it and answers the questions of the thesis about any request or
//! control: which control it was, its state, what came before, which
//! action, who started it, the precondition, what changed, the result.
//!
//! - `Inspector::timeline`: every event, summarized, by kind and by actor,
//!   with its time from the start of the session.
//! - `Inspector::why`: the causal chain of one request: the observation it
//!   rested on, the request with the values it read, what moved in those
//!   values between the observation and the request and which receipt ran
//!   then, the stages, the verdict with its delta, and the result.
//! - `Inspector::control`: a control at an instant, and every request that
//!   touched it up to then, each with its `why`.
//! - `Inspector::summary`: the build, the span, the actors, every request
//!   with how it ended, and every refusal with its `why`.
//!
//! who moved a value is a correlation by time, never a declaration: the
//! contract declares what an action reads, not what it writes. a change is
//! put on the receipts that ran (reached executing, completed or failed)
//! between the recorded state before it and the change itself; none is a
//! change from outside the app, and two or more are ambiguous and all
//! shown (see `By`).
//!
//! it only reads. the inspector borrows the session and nothing else; it
//! holds no app and takes nothing that could run an action, so reading a
//! session can never create a request (replay, never re-execution:
//! docs/ADR/session-recording-replay-not-reexecution.md). `text` renders
//! every answer as plain english, the way the `plev-inspect` binary prints
//! it; every model type is also `Serialize`, for its `--json`.

mod control;
mod describe;
mod index;
pub mod prose;
mod summary;
mod timeline;
mod why;

pub use control::Control;
pub use index::{Inspector, Stamp};
pub use summary::{Line, Summary};
pub use timeline::{Count, Entry, Tally, Timeline, What};
pub use why::{
    Asked, Basis, Between, By, Cause, Ending, InputAt, Move, Read, ReadMoves, StageAt, Unrecorded,
    Why,
};
