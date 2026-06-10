//! Threaded wrapper around [`GitRepo`] so UIs never block on git.
//!
//! Commands flow through an mpsc channel into a worker thread that owns the
//! `GitRepo`; results come back through a caller-provided callback. The
//! callback typically forwards into a `winit::EventLoopProxy` (or any other
//! wake-up mechanism) — this crate stays UI-agnostic on purpose.

use std::path::Path;
use std::sync::mpsc::{Sender, channel};
use std::thread::JoinHandle;

use crate::error::Result;
use crate::repo::GitRepo;
use crate::types::{Branch, Commit, FileStatus, Hunk};

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
    Commit {
        message: String,
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
    Committed(Result<String>),
}

/// Handle to the git worker thread. Dropping it shuts the worker down.
pub struct GitClient {
    tx: Sender<GitCommand>,
    handle: Option<JoinHandle<()>>,
}

impl GitClient {
    /// Opens the repository containing `path` and spawns the worker.
    /// `on_event` is called from the worker thread for every finished
    /// command.
    pub fn spawn(
        path: impl AsRef<Path>,
        on_event: impl Fn(GitEvent) + Send + 'static,
    ) -> Result<Self> {
        let repo = GitRepo::open(path)?;
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
            })
            .expect("spawn git worker thread");
        Ok(Self {
            tx,
            handle: Some(handle),
        })
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
        GitCommand::Commit { message } => {
            on_event(GitEvent::Committed(repo.commit(&message)));
        }
        GitCommand::Refresh { log_limit } => {
            on_event(GitEvent::Status(repo.status()));
            on_event(GitEvent::Log(repo.log(log_limit)));
            on_event(GitEvent::Branches(repo.branches()));
        }
        GitCommand::Shutdown => unreachable!("handled by the worker loop"),
    }
}
