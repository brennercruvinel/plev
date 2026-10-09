//! The git side of a workspace: sends the commands the workbench's
//! `execute` decides on to the worker thread and applies the worker's
//! `GitEvent`s back onto the views. The workbench owns it, so the ide
//! binary and the hosts that embed the workspace (the showcase's Git tab)
//! share one copy of the mapping.
//!
//! The view shows a mutation as soon as it leaves for the worker, and the
//! refresh its answer sends reconciles it with git. A status read before
//! a mutation the view already shows is older than the view, and applying
//! it would undo the mutation until the next refresh. The worker runs
//! commands one at a time in the order they were sent, so the session
//! knows which answers were read before the last mutation without the
//! worker saying so: it counts the reads it asked for and, when a
//! mutation leaves, marks every read still owed as read before it
//! ([`Reads`]). Those answers are dropped; the refresh the mutation's own
//! answer sends is read after it, and after any change made outside.
//!
//! A commit is shown when the worker says it exists, in the same step
//! that ends its receipt ([`GitSession::show_commit`]): its files leave
//! the staged rows and it tops the log the stacks are built from. Every
//! read owed when the commit left was marked old by it and is dropped,
//! and every read asked after it is read after it, so no answer takes the
//! commit back; the refresh its answer sends replaces the guess with git.

use std::collections::HashSet;

use gitdrv::{Branch, Commit, GitClient, GitCommand, GitEvent, NewCommit};

use crate::repository::adapters;
use crate::interfaces::panels::layout::WorkspaceView;

/// How many commits the stacks panel loads per refresh.
pub const LOG_LIMIT: usize = 100;

/// What the diff panel is currently waiting for; stale diff events (the
/// user clicked elsewhere meanwhile) are dropped.
#[derive(PartialEq)]
enum DiffTarget {
    File(String),
    Commit(String),
}

/// The answers of one read (status, log or branches) the worker still
/// owes. It answers in the order it was asked, so the oldest `stale` of
/// the `owed` ones were read before a mutation that changes what they
/// read, which the view already shows.
#[derive(Debug, Default, PartialEq)]
struct Owed {
    owed: usize,
    stale: usize,
}

impl Owed {
    fn asked(&mut self) {
        self.owed += 1;
    }

    /// A mutation that changes this read left behind every answer owed.
    fn outdated(&mut self) {
        self.stale = self.owed;
    }

    /// One answer arrived: `false` when it was read before a mutation
    /// the view already shows. An answer nobody asked for through the
    /// session has nothing to compare with and is taken.
    fn fresh(&mut self) -> bool {
        let Some(left) = self.owed.checked_sub(1) else {
            return true;
        };
        self.owed = left;
        match self.stale.checked_sub(1) {
            Some(left) => {
                self.stale = left;
                false
            }
            None => true,
        }
    }
}

/// The three reads the views are built from, and which answers of each
/// are still owed. A stage, an unstage, a discard or an ignore changes
/// what the status reads; a commit changes the status, the log and the
/// branches (a first commit creates the branch).
#[derive(Debug, Default)]
struct Reads {
    status: Owed,
    log: Owed,
    branches: Owed,
}

impl Reads {
    /// `command` is leaving for the worker.
    fn sent(&mut self, command: &GitCommand) {
        match command {
            GitCommand::Refresh { .. } => {
                self.status.asked();
                self.log.asked();
                self.branches.asked();
            }
            GitCommand::Status => self.status.asked(),
            GitCommand::Log { .. } => self.log.asked(),
            GitCommand::Branches => self.branches.asked(),
            GitCommand::Stage { .. }
            | GitCommand::Unstage { .. }
            | GitCommand::Discard { .. }
            | GitCommand::Ignore { .. } => self.status.outdated(),
            GitCommand::CommitIf { .. } => {
                self.status.outdated();
                self.log.outdated();
                self.branches.outdated();
            }
            GitCommand::DiffWorkdir { .. }
            | GitCommand::DiffCommit { .. }
            | GitCommand::Shutdown => {}
        }
    }

    /// `event` arrived: `false` when it is a read taken before a mutation
    /// the view already shows.
    fn fresh(&mut self, event: &GitEvent) -> bool {
        match event {
            GitEvent::Status(_) => self.status.fresh(),
            GitEvent::Log(_) => self.log.fresh(),
            GitEvent::Branches(_) => self.branches.fresh(),
            _ => true,
        }
    }
}

pub struct GitSession {
    git: GitClient,
    diff_target: Option<DiffTarget>,
    /// Last log/branches payloads — the stacks panel needs both.
    log_cache: Vec<Commit>,
    branch_cache: Vec<Branch>,
    /// The reads still owed, and which of them are older than the view.
    reads: Reads,
    /// The last commit this session made (`show_commit`).
    made: Option<String>,
}

impl GitSession {
    pub fn new(git: GitClient) -> Self {
        Self {
            git,
            diff_target: None,
            log_cache: Vec::new(),
            branch_cache: Vec::new(),
            reads: Reads::default(),
            made: None,
        }
    }

    /// The worker's handle, for what it knows (its workdir). Commands go
    /// through [`GitSession::send`], which counts the reads still owed.
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
    pub fn refresh(&mut self) {
        self.send(GitCommand::Refresh {
            log_limit: LOG_LIMIT,
        });
    }

    /// Queues `command` on the worker. `false` when the worker is gone.
    /// Every command goes through here, so the session knows which reads
    /// are older than the mutations the view shows.
    pub fn send(&mut self, command: GitCommand) -> bool {
        self.reads.sent(&command);
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

    /// Shows `commit`, which the worker just made with `message`, before
    /// the refresh its answer sends reads it back.
    ///
    /// Each committed path loses its staged row: a path with only that row
    /// leaves the list, a path also changed outside the index keeps its
    /// unstaged row. A path the view does not show as staged (moved since
    /// the commit left) is left as it is, for the refresh to settle. The
    /// commit tops the log: the message's first line as its summary, the
    /// author of the commit HEAD was until now (none on a first commit),
    /// `files` as the worker listed them.
    pub(crate) fn show_commit(
        &mut self,
        workspace: &mut WorkspaceView,
        commit: &NewCommit,
        message: &str,
    ) {
        self.made = Some(commit.sha.clone());
        let committed: HashSet<&str> = commit.files.iter().map(String::as_str).collect();
        let rows = workspace
            .unassigned
            .files
            .iter()
            .filter(|row| !(row.staged && committed.contains(row.path.as_str())))
            .cloned()
            .collect();
        workspace.unassigned.set_files(rows);

        if !self.log_cache.iter().any(|c| c.sha == commit.sha) {
            let author = self
                .log_cache
                .first()
                .map(|head| head.author.clone())
                .unwrap_or_default();
            let summary = message.trim().lines().next().unwrap_or_default().trim();
            self.log_cache.insert(
                0,
                Commit {
                    sha: commit.sha.clone(),
                    short_sha: commit.sha.chars().take(7).collect(),
                    message: summary.to_owned(),
                    author,
                    time: adapters::unix_now(),
                    files: commit.files.clone(),
                },
            );
            self.log_cache.truncate(LOG_LIMIT);
        }
        self.rebuild_stacks(workspace);
    }

    /// The sha of the last commit this session made, if any.
    pub(crate) fn made(&self) -> Option<&str> {
        self.made.as_deref()
    }

    /// Applies a result from the git worker to the views. A status, log
    /// or branches read before a mutation the view already shows is
    /// dropped: the refresh that mutation's answer sends replaces it.
    pub fn apply(&mut self, workspace: &mut WorkspaceView, event: GitEvent) {
        if !self.reads.fresh(&event) {
            log::debug!("ide: dropped a read older than the view: {event:?}");
            return;
        }
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
            // The commit is already shown (`show_commit`, when the receipt
            // ended); the refresh reads it back from git.
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn refresh() -> GitCommand {
        GitCommand::Refresh {
            log_limit: LOG_LIMIT,
        }
    }

    /// The three answers of one refresh, in the worker's order: which
    /// of them the session takes.
    fn answers(reads: &mut Reads) -> [bool; 3] {
        [
            reads.fresh(&GitEvent::Status(Ok(Vec::new()))),
            reads.fresh(&GitEvent::Log(Ok(Vec::new()))),
            reads.fresh(&GitEvent::Branches(Ok(Vec::new()))),
        ]
    }

    #[test]
    fn a_stage_outdates_the_status_owed_and_not_the_log_or_the_branches() {
        let mut reads = Reads::default();
        reads.sent(&refresh());
        reads.sent(&GitCommand::Stage { path: "B".into() });
        reads.sent(&refresh());
        assert_eq!(answers(&mut reads), [false, true, true], "read before B");
        assert_eq!(answers(&mut reads), [true, true, true], "read after B");
        assert_eq!(reads.status, Owed::default());
    }

    #[test]
    fn a_commit_outdates_the_status_the_log_and_the_branches_owed() {
        let mut reads = Reads::default();
        reads.sent(&refresh());
        reads.sent(&refresh());
        reads.sent(&GitCommand::CommitIf {
            message: "m".into(),
            expect_staged: BTreeSet::from(["A".to_owned()]),
        });
        reads.sent(&refresh());
        for _ in 0..2 {
            assert_eq!(answers(&mut reads), [false; 3], "read before the commit");
        }
        assert_eq!(answers(&mut reads), [true; 3], "read after it");
    }

    #[test]
    fn reads_asked_after_the_last_mutation_and_reads_nobody_asked_for_are_taken() {
        let mut reads = Reads::default();
        reads.sent(&GitCommand::Discard { path: "A".into() });
        reads.sent(&GitCommand::DiffWorkdir { path: "B".into() });
        reads.sent(&GitCommand::Status);
        assert!(reads.fresh(&GitEvent::Status(Ok(Vec::new()))));
        assert_eq!(answers(&mut reads), [true; 3], "nothing owed");
        assert_eq!(reads.status, Owed::default());
    }
}
