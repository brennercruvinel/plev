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
//!
//! it only reads. the inspector borrows the session and nothing else; it
//! holds no app and takes nothing that could run an action, so reading a
//! session can never create a request (replay, never re-execution:
//! docs/ADR/session-recording-replay-not-reexecution.md). every model type
//! is `Serialize`.

mod describe;
mod index;
mod timeline;

pub use index::{Inspector, Stamp};
pub use timeline::{Count, Entry, Tally, Timeline, What};
