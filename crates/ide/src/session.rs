//! The git side of a workspace: sends the commands the workbench's
//! `execute` decides on to the worker thread and applies the worker's
//! `GitEvent`s back onto the views. The workbench owns it, so the ide
//! binary and the hosts that embed the workspace (the showcase's Git tab)
//! share one copy of the mapping.

use git::{Branch, Commit, GitClient, GitCommand, GitEvent};

use crate::adapters;
use crate::views::workspace::WorkspaceView;

/// How many commits the stacks panel loads per refresh.
pub const LOG_LIMIT: usize = 100;

/// What the diff panel is currently waiting for; stale diff events (the
/// user clicked elsewhere meanwhile) are dropped.
#[derive(PartialEq)]
enum DiffTarget {
    File(String),
    Commit(String),
}

pub struct GitSession {
    git: GitClient,
    diff_target: Option<DiffTarget>,
    /// Last log/branches payloads — the stacks panel needs both.
    log_cache: Vec<Commit>,
    branch_cache: Vec<Branch>,
}

impl GitSession {
    pub fn new(git: GitClient) -> Self {
        Self {
            git,
            diff_target: None,
            log_cache: Vec::new(),
            branch_cache: Vec::new(),
        }
    }

    pub fn client(&self) -> &GitClient {
        &self.git
    }

    /// Repository name for titles/headers (the worktree's directory name).
    pub fn repo_label(&self) -> String {
        self.git
            .workdir()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.git.workdir().display().to_string())
    }

    /// Reload status, log and branches (results stream back as events).
    pub fn refresh(&self) {
        self.git.send(GitCommand::Refresh {
            log_limit: LOG_LIMIT,
        });
    }

    /// Queues `command` on the worker. `false` when the worker is gone.
    pub fn send(&self, command: GitCommand) -> bool {
        self.git.send(command)
    }

    /// Fetches the diff of `path`; the panel takes only that answer.
    pub fn diff_file(&mut self, path: &str) -> bool {
        self.diff_target = Some(DiffTarget::File(path.to_owned()));
        self.send(GitCommand::DiffWorkdir {
            path: path.to_owned(),
        })
    }

    /// Fetches the diff of commit `sha`; the panel takes only that answer.
    pub fn diff_commit(&mut self, sha: &str) -> bool {
        self.diff_target = Some(DiffTarget::Commit(sha.to_owned()));
        self.send(GitCommand::DiffCommit {
            sha: sha.to_owned(),
        })
    }

    /// The panel shows nothing any more: a diff still on its way is
    /// dropped when it arrives.
    pub fn forget_diff(&mut self) {
        self.diff_target = None;
    }

    /// Applies a result from the git worker to the views.
    pub fn apply(&mut self, workspace: &mut WorkspaceView, event: GitEvent) {
        match event {
            GitEvent::Status(Ok(status)) => {
                workspace
                    .unassigned
                    .set_files(adapters::file_entries(&status));
            }
            GitEvent::Log(Ok(log)) => {
                self.log_cache = log;
                self.rebuild_stacks(workspace);
            }
            GitEvent::Branches(Ok(branches)) => {
                self.branch_cache = branches;
                workspace.branch_label = self
                    .branch_cache
                    .iter()
                    .find(|b| b.is_head)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                self.rebuild_stacks(workspace);
            }
            GitEvent::DiffWorkdir { path, result } => {
                if self.diff_target == Some(DiffTarget::File(path)) {
                    match result {
                        Ok(hunks) => workspace.diff.set_lines(adapters::diff_lines(&hunks)),
                        Err(e) => log::error!("diff failed: {e}"),
                    }
                }
            }
            GitEvent::DiffCommit { sha, result } => {
                if self.diff_target == Some(DiffTarget::Commit(sha)) {
                    match result {
                        Ok(hunks) => workspace.diff.set_lines(adapters::diff_lines(&hunks)),
                        Err(e) => log::error!("commit diff failed: {e}"),
                    }
                }
            }
            // Mutations: refresh everything afterwards so the optimistic
            // view updates reconcile with what git actually did (also on
            // error, to roll wrong guesses back).
            GitEvent::Staged { path, result }
            | GitEvent::Unstaged { path, result }
            | GitEvent::Discarded { path, result }
            | GitEvent::Ignored { path, result } => {
                if let Err(e) = result {
                    log::error!("git operation on {path} failed: {e}");
                }
                self.refresh();
            }
            GitEvent::Committed(result) => {
                match result {
                    Ok(commit) => log::info!("committed {} {:?}", commit.sha, commit.files),
                    Err(e) => log::error!("commit failed: {e}"),
                }
                self.refresh();
            }
            // The index moved under the view: reload it.
            GitEvent::CommitRejected { added, removed } => {
                log::warn!("commit refused: the index staged {added:?} and not {removed:?}");
                self.refresh();
            }
            GitEvent::Status(Err(e)) => log::error!("status failed: {e}"),
            GitEvent::Log(Err(e)) => log::error!("log failed: {e}"),
            GitEvent::Branches(Err(e)) => log::error!("branches failed: {e}"),
        }
    }

    fn rebuild_stacks(&self, workspace: &mut WorkspaceView) {
        workspace
            .stacks
            .set_stacks(adapters::stacks(&self.branch_cache, &self.log_cache));
    }
}
