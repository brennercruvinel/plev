//! Threaded wrapper around [`GitRepo`] so UIs never block on git.
//!
//! Commands flow through an mpsc channel into a worker thread that owns the
//! `GitRepo`; results come back through a caller-provided callback. The
//! callback typically forwards into a `winit::EventLoopProxy` (or any other
//! wake-up mechanism) — this crate stays UI-agnostic on purpose.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, channel};
use std::thread::JoinHandle;

use crate::error::Result;
use crate::repo::GitRepo;
use crate::types::{Branch, Commit, CommitCheck, FileStatus, Hunk, NewCommit};

/// Requests the worker thread understands.
#[derive(Clone, Debug)]
pub enum GitCommand {
    Status,
    Log {
        limit: usize,
    },
    Branches,
    DiffWorkdir {
        path: String,
    },
    DiffCommit {
        sha: String,
    },
    Stage {
        path: String,
    },
    Unstage {
        path: String,
    },
    Discard {
        path: String,
    },
    Ignore {
        path: String,
    },
    /// Commits the index with `message` only if it stages exactly
    /// `expect_staged`, the set the requester decided on: the worker reads
    /// the real index right before `git commit` and answers
    /// `GitEvent::Committed` or, when the index moved, `CommitRejected`
    /// with the delta and no commit.
    CommitIf {
        message: String,
        expect_staged: BTreeSet<String>,
    },
    /// Convenience: emits `Status`, `Log` and `Branches` events in one go
    /// (used after mutations and by file watchers).
    Refresh {
        log_limit: usize,
    },
    Shutdown,
}

/// Results delivered to the event callback, one per command (mutations also
/// echo the path so the UI can reconcile optimistic updates).
#[derive(Debug)]
pub enum GitEvent {
    Status(Result<Vec<FileStatus>>),
    Log(Result<Vec<Commit>>),
    Branches(Result<Vec<Branch>>),
    DiffWorkdir {
        path: String,
        result: Result<Vec<Hunk>>,
    },
    DiffCommit {
        sha: String,
        result: Result<Vec<Hunk>>,
    },
    Staged {
        path: String,
        result: Result<()>,
    },
    Unstaged {
        path: String,
        result: Result<()>,
    },
    Discarded {
        path: String,
        result: Result<()>,
    },
    Ignored {
        path: String,
        result: Result<()>,
    },
    /// A `CommitIf` whose index matched: the new commit's sha and files,
    /// or why git failed.
    Committed(Result<NewCommit>),
    /// A `CommitIf` whose index did not match: nothing was committed.
    /// `added` are staged and were not expected, `removed` were expected
    /// and are not staged (both sorted).
    CommitRejected {
        added: Vec<String>,
        removed: Vec<String>,
    },
}

/// Handle to the git worker thread. Dropping it shuts the worker down.
pub struct GitClient {
    tx: Sender<GitCommand>,
    handle: Option<JoinHandle<()>>,
    workdir: PathBuf,
}

impl GitClient {
    /// Opens the repository containing `path` and spawns the worker.
    /// `on_event` is called from the worker thread for every finished
    /// command. Fails when there is no repository at `path` or the OS
    /// refuses the thread.
    pub fn spawn(
        path: impl AsRef<Path>,
        on_event: impl Fn(GitEvent) + Send + 'static,
    ) -> Result<Self> {
        let repo = GitRepo::open(path)?;
        let workdir = repo.workdir().to_path_buf();
        let (tx, rx) = channel::<GitCommand>();
        let handle = std::thread::Builder::new()
            .name("git-backend".into())
            .spawn(move || {
                while let Ok(command) = rx.recv() {
                    if matches!(command, GitCommand::Shutdown) {
                        break;
                    }
                    run_command(&repo, command, &on_event);
                }
            })?;
        Ok(Self {
            tx,
            handle: Some(handle),
            workdir,
        })
    }

    /// Working tree root of the opened repository (e.g. for window titles
    /// and file watchers).
    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    /// Queues a command; never blocks. Returns `false` if the worker is
    /// gone (only possible after shutdown).
    pub fn send(&self, command: GitCommand) -> bool {
        self.tx.send(command).is_ok()
    }
}

impl Drop for GitClient {
    fn drop(&mut self) {
        let _ = self.tx.send(GitCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_command(repo: &GitRepo, command: GitCommand, on_event: &impl Fn(GitEvent)) {
    match command {
        GitCommand::Status => on_event(GitEvent::Status(repo.status())),
        GitCommand::Log { limit } => on_event(GitEvent::Log(repo.log(limit))),
        GitCommand::Branches => on_event(GitEvent::Branches(repo.branches())),
        GitCommand::DiffWorkdir { path } => {
            let result = repo.diff_workdir(&path);
            on_event(GitEvent::DiffWorkdir { path, result });
        }
        GitCommand::DiffCommit { sha } => {
            let result = repo.diff_commit(&sha);
            on_event(GitEvent::DiffCommit { sha, result });
        }
        GitCommand::Stage { path } => {
            let result = repo.stage(&path);
            on_event(GitEvent::Staged { path, result });
        }
        GitCommand::Unstage { path } => {
            let result = repo.unstage(&path);
            on_event(GitEvent::Unstaged { path, result });
        }
        GitCommand::Discard { path } => {
            let result = repo.discard(&path);
            on_event(GitEvent::Discarded { path, result });
        }
        GitCommand::Ignore { path } => {
            let result = repo.ignore(&path);
            on_event(GitEvent::Ignored { path, result });
        }
        GitCommand::CommitIf {
            message,
            expect_staged,
        } => on_event(match repo.commit_if(&message, &expect_staged) {
            Ok(CommitCheck::Committed(commit)) => GitEvent::Committed(Ok(commit)),
            Ok(CommitCheck::Stale { added, removed }) => {
                GitEvent::CommitRejected { added, removed }
            }
            Err(e) => GitEvent::Committed(Err(e)),
        }),
        GitCommand::Refresh { log_limit } => {
            on_event(GitEvent::Status(repo.status()));
            on_event(GitEvent::Log(repo.log(log_limit)));
            on_event(GitEvent::Branches(repo.branches()));
        }
        GitCommand::Shutdown => unreachable!("handled by the worker loop"),
    }
}
