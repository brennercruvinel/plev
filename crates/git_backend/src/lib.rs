//! Git backend for plev applications.
//!
//! Two layers:
//! - [`GitRepo`]: clean synchronous API (gix for reads, git CLI for
//!   status/diff/mutations — see ADR notes in `repo.rs`).
//! - [`GitClient`]: runs a `GitRepo` on a worker thread; commands in via
//!   channel, results out via callback, so a UI thread never blocks.
//!
//! No UI dependencies — this crate is testable against real temporary
//! repositories (see `tests/real_repo.rs`).

mod client;
mod diff;
mod error;
mod repo;
mod status;
mod types;

pub use client::{GitClient, GitCommand, GitEvent};
pub use error::{GitError, Result};
pub use repo::GitRepo;
pub use types::{Branch, Commit, DiffLine, DiffLineKind, FileStatus, Hunk, StatusKind};
