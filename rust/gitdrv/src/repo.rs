//! Synchronous git repository API.
//!
//! Split of responsibilities (ADR #5 of the technical plan):
//! - **gix** handles the read side that it makes easy and fast: opening /
//!   discovering the repository, walking history (with the paths each
//!   commit changes, a tree diff), listing references.
//! - **git CLI** handles worktree status, diffs and all mutations
//!   (stage/unstage/discard/commit). For these, gix either lacks a porcelain
//!   API or requires assembling plumbing crates by hand; the CLI's porcelain
//!   v2 / unified diff formats are explicitly stable for tooling. This is
//!   the same pragmatic route Zed takes.
//!
//! Every method is synchronous; the UI talks to [`GitClient`](crate::GitClient)
//! instead, which runs a `GitRepo` on a worker thread.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;

use gix::diff::tree::recorder::Change;
use gix::objs::FindExt;

use crate::diff::parse_unified;
use crate::error::{GitError, Result, repo_err};
use crate::status::parse_porcelain_v2;
use crate::domain::{Branch, Commit, CommitCheck, FileStatus, Hunk, NewCommit};

pub struct GitRepo {
    repo: gix::Repository,
    /// Working tree root — also the cwd for every CLI invocation.
    workdir: PathBuf,
    /// The paths each commit changes, read once per commit
    /// ([`commit_files`](Self::commit_files)), and the buffers that read
    /// them.
    files: RefCell<CommitFiles>,
}

impl GitRepo {
    /// Opens the repository containing `path` (any directory inside the
    /// working tree works, like `git` itself).
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let repo = gix::discover(path.as_ref()).map_err(repo_err)?;
        let workdir = repo.workdir().ok_or(GitError::Bare)?.to_path_buf();
        Ok(Self {
            repo,
            workdir,
            files: RefCell::default(),
        })
    }

    /// Working tree root.
    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    // -- read side (gix) ----------------------------------------------------

    /// Last `limit` commits reachable from HEAD, newest first, each with the
    /// paths it changes ([`commit_files`](Self::commit_files)). Returns an
    /// empty list on a repository without commits yet.
    ///
    /// The cache of those paths keeps the four times `limit` commits asked
    /// for most recently, so a log that moves by one commit reads one tree
    /// diff.
    pub fn log(&self, limit: usize) -> Result<Vec<Commit>> {
        let Ok(head_id) = self.repo.head_id() else {
            return Ok(Vec::new()); // unborn HEAD (fresh `git init`)
        };
        let walk = self
            .repo
            .rev_walk(Some(head_id.detach()))
            .all()
            .map_err(repo_err)?;

        let mut commits = Vec::new();
        for info in walk.take(limit) {
            let info = info.map_err(repo_err)?;
            let commit = info.object().map_err(repo_err)?;
            let files = self.files_of(info.id)?;
            let sha = commit.id().to_hex().to_string();
            let message = commit.message().map_err(repo_err)?.summary().to_string();
            let author = commit.author().map_err(repo_err)?;
            let time = commit.time().map_err(repo_err)?;
            commits.push(Commit {
                short_sha: sha.chars().take(7).collect(),
                sha,
                message,
                author: author.name.to_string(),
                time: time.seconds,
                files,
            });
        }
        self.files.borrow_mut().cache.trim(limit.saturating_mul(4));
        Ok(commits)
    }

    /// Local branches, with `is_head` set on the checked-out one.
    pub fn branches(&self) -> Result<Vec<Branch>> {
        let head = self.current_branch()?;
        let platform = self.repo.references().map_err(repo_err)?;
        let mut branches = Vec::new();
        for reference in platform.local_branches().map_err(repo_err)? {
            let reference = reference.map_err(repo_err)?;
            let name = reference.name().shorten().to_string();
            branches.push(Branch {
                is_head: head.as_deref() == Some(name.as_str()),
                name,
            });
        }
        Ok(branches)
    }

    /// Name of the branch HEAD points at, or `None` when detached.
    pub fn current_branch(&self) -> Result<Option<String>> {
        let name = self.repo.head_name().map_err(repo_err)?;
        Ok(name.map(|n| n.shorten().to_string()))
    }

    // -- status & diff (git CLI) ---------------------------------------------

    /// Working tree + index status. See [`parse_porcelain_v2`] for the
    /// staged/unstaged split semantics.
    pub fn status(&self) -> Result<Vec<FileStatus>> {
        let raw = self.git(&["status", "--porcelain=v2", "--untracked-files=all", "-z"])?;
        parse_porcelain_v2(&raw)
    }

    /// Diff of one path against HEAD (staged + unstaged changes combined).
    ///
    /// Untracked files have no blob to diff against, so their full content
    /// is synthesized as a single all-added hunk — exactly what a user
    /// expects "the diff of a new file" to look like.
    pub fn diff_workdir(&self, path: &str) -> Result<Vec<Hunk>> {
        let raw = if self.head_exists() {
            self.git(&["diff", "HEAD", "--", path])?
        } else {
            // No commits yet: diff index vs worktree is all that exists.
            self.git(&["diff", "--", path])?
        };
        if !raw.trim().is_empty() {
            return parse_unified(&raw);
        }
        if self.is_tracked(path)? {
            return Ok(Vec::new()); // tracked and unchanged
        }
        self.synthesize_untracked_hunk(path)
    }

    /// Changes introduced by a commit (diff against its first parent; for a
    /// root commit, against the empty tree).
    pub fn diff_commit(&self, sha: &str) -> Result<Vec<Hunk>> {
        // `git show` handles the root-commit case for free.
        let raw = self.git(&["show", "--format=", "--patch", sha])?;
        parse_unified(&raw)
    }

    // -- mutations (git CLI) --------------------------------------------------

    /// Stages one path (`git add`). Also covers untracked and deleted files.
    pub fn stage(&self, path: &str) -> Result<()> {
        self.git(&["add", "--", path]).map(drop)
    }

    /// Removes one path from the index, keeping worktree changes.
    pub fn unstage(&self, path: &str) -> Result<()> {
        if self.head_exists() {
            self.git(&["restore", "--staged", "--", path]).map(drop)
        } else {
            // No HEAD to restore from (repo without commits): drop the
            // index entry instead, which is the only meaning "unstage" can
            // have here.
            self.git(&["rm", "--cached", "--force", "--quiet", "--", path])
                .map(drop)
        }
    }

    /// Discards all changes to one path, restoring index and worktree from
    /// HEAD. An untracked file has nothing in HEAD to restore, so discard
    /// deletes it (matching the behavior of GitButler/Zed).
    pub fn discard(&self, path: &str) -> Result<()> {
        if self.is_tracked(path)? {
            self.git(&["checkout", "HEAD", "--", path]).map(drop)
        } else {
            std::fs::remove_file(self.workdir.join(path))?;
            Ok(())
        }
    }

    /// Appends `path` to the repository root `.gitignore` (created if
    /// missing), ensuring it lands on its own line.
    pub fn ignore(&self, path: &str) -> Result<()> {
        let gitignore = self.workdir.join(".gitignore");
        let mut content = match std::fs::read_to_string(&gitignore) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e.into()),
        };
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(path);
        content.push('\n');
        std::fs::write(&gitignore, content)?;
        Ok(())
    }

    /// Commits the index with `message`; returns the new commit's sha and
    /// the paths it changes ([`commit_files`](Self::commit_files)).
    pub fn commit(&self, message: &str) -> Result<NewCommit> {
        self.git(&["commit", "--message", message])?;
        let sha = self.git(&["rev-parse", "HEAD"])?.trim().to_string();
        let files = self.commit_files(&sha)?;
        Ok(NewCommit { sha, files })
    }

    /// Commits the index with `message` only when the paths it stages are
    /// exactly `expect`, the staged set the caller decided on. The index
    /// is read (through `status`, the same source the staged set shown to
    /// a person comes from) right before `git commit`; when it differs,
    /// nothing is committed and the answer is the delta.
    ///
    /// Between this read and `git commit` another process can still move
    /// the index: git holds `index.lock` only inside its own process. That
    /// window is outside this check.
    pub fn commit_if(&self, message: &str, expect: &BTreeSet<String>) -> Result<CommitCheck> {
        let staged = self.staged_paths()?;
        if &staged != expect {
            return Ok(CommitCheck::Stale {
                added: staged.difference(expect).cloned().collect(),
                removed: expect.difference(&staged).cloned().collect(),
            });
        }
        self.commit(message).map(CommitCheck::Committed)
    }

    /// The paths the index holds changes for: the staged rows of
    /// [`status`](Self::status).
    pub fn staged_paths(&self) -> Result<BTreeSet<String>> {
        Ok(self
            .status()?
            .into_iter()
            .filter(|s| s.staged)
            .map(|s| s.path)
            .collect())
    }

    /// The paths commit `sha` changes (see [`Commit::files`]): its tree
    /// against its first parent's, a root commit's against the empty tree,
    /// in git's order, a renamed file at both paths. The same paths as
    /// `git diff-tree -r --no-renames --root --diff-merges=first-parent`,
    /// read in process with gix, never by a `git` process. `sha` is any
    /// revision naming a commit.
    ///
    /// A commit never changes, so its paths are read once and kept by
    /// commit id: the receipt of a commit and the log that shows it later
    /// read the same entry.
    pub fn commit_files(&self, sha: &str) -> Result<Vec<String>> {
        let id = self.repo.rev_parse_single(sha).map_err(repo_err)?;
        self.files_of(id.detach())
    }

    /// The paths commit `id` changes, from the cache or read once.
    fn files_of(&self, id: gix::ObjectId) -> Result<Vec<String>> {
        let mut files = self.files.borrow_mut();
        if let Some(hit) = files.cache.get(&id) {
            return Ok(hit);
        }
        let read = files.diff.files(&self.repo, &id)?;
        files.cache.put(id, read.clone());
        Ok(read)
    }

    // -- helpers ---------------------------------------------------------------

    fn head_exists(&self) -> bool {
        self.repo.head_id().is_ok()
    }

    /// `true` when the path exists in the index (i.e. not untracked).
    fn is_tracked(&self, path: &str) -> Result<bool> {
        let out = self
            .command()
            .args(["ls-files", "--error-unmatch", "--", path])
            .output()?;
        Ok(out.status.success())
    }

    fn synthesize_untracked_hunk(&self, path: &str) -> Result<Vec<Hunk>> {
        use crate::domain::{DiffLine, DiffLineKind};
        let content = std::fs::read_to_string(self.workdir.join(path))?;
        let lines: Vec<DiffLine> = content
            .lines()
            .enumerate()
            .map(|(i, line)| DiffLine {
                kind: DiffLineKind::Add,
                content: line.to_string(),
                old_no: None,
                new_no: Some(i as u32 + 1),
            })
            .collect();
        if lines.is_empty() {
            return Ok(Vec::new());
        }
        Ok(vec![Hunk {
            header: format!("@@ -0,0 +1,{} @@ (untracked)", lines.len()),
            lines,
        }])
    }

    /// A `git` process in the working tree; every invocation starts here.
    /// `GIT_OPTIONAL_LOCKS=0` (the environment form of
    /// `--no-optional-locks`) keeps a read from writing: without it
    /// `status` takes `.git/index.lock` to store the stat data it
    /// refreshed, and a watcher on the repository (the ide's) sees that
    /// write, refreshes, runs `status` again, and never stops. Mutations
    /// still take the locks they need; only the optional ones go.
    fn command(&self) -> Command {
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(&self.workdir)
            .env("GIT_OPTIONAL_LOCKS", "0");
        command
    }

    /// Runs `git <args>` in the working tree, returning stdout on success.
    fn git(&self, args: &[&str]) -> Result<String> {
        let out = self.command().args(args).output()?;
        if !out.status.success() {
            return Err(GitError::Command {
                command: args.join(" "),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// What [`GitRepo::commit_files`] keeps between calls.
#[derive(Default)]
struct CommitFiles {
    cache: FilesCache,
    diff: TreeDiff,
}

/// The paths each commit changes, by commit id. A commit never changes, so
/// an entry never goes stale; the cache only forgets the commits asked for
/// least recently, to stay bounded.
#[derive(Default)]
struct FilesCache {
    /// The paths, and when they were last asked for.
    entries: HashMap<gix::ObjectId, (Vec<String>, u64)>,
    /// Counts every get and put: a larger stamp was asked for later.
    clock: u64,
}

impl FilesCache {
    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    fn get(&mut self, id: &gix::ObjectId) -> Option<Vec<String>> {
        let now = self.tick();
        let (files, used) = self.entries.get_mut(id)?;
        *used = now;
        Some(files.clone())
    }

    fn put(&mut self, id: gix::ObjectId, files: Vec<String>) {
        let now = self.tick();
        self.entries.insert(id, (files, now));
    }

    /// Keeps the `cap` commits asked for most recently.
    fn trim(&mut self, cap: usize) {
        let extra = self.entries.len().saturating_sub(cap);
        if extra == 0 {
            return;
        }
        if cap == 0 {
            self.entries.clear();
            return;
        }
        // Stamps are unique, so exactly `cap` are at or above the one at
        // index `extra` once sorted.
        let mut stamps: Vec<u64> = self.entries.values().map(|(_, used)| *used).collect();
        let (_, oldest_kept, _) = stamps.select_nth_unstable(extra);
        let oldest_kept = *oldest_kept;
        self.entries.retain(|_, (_, used)| *used >= oldest_kept);
    }
}

/// Reads the paths a commit changes: its tree against its first parent's
/// (a root commit's against the empty tree), with the buffers kept from
/// one commit to the next.
#[derive(Default)]
struct TreeDiff {
    state: gix::diff::tree::State,
    commit: Vec<u8>,
    parent: Vec<u8>,
    ours: Vec<u8>,
    theirs: Vec<u8>,
}

impl TreeDiff {
    /// Every path whose entry differs between the two trees, sorted as
    /// git sorts them (byte order of the full path). No rename detection:
    /// a moved file counts at both paths.
    fn files(&mut self, repo: &gix::Repository, id: &gix::oid) -> Result<Vec<String>> {
        let kind = repo.object_hash();
        let objects = &repo.objects;
        // The tree comes first in a commit, then its parents.
        let mut commit = objects
            .find_commit_iter(id, &mut self.commit)
            .map_err(repo_err)?;
        let tree = commit.tree_id().map_err(repo_err)?;
        let first_parent = commit.parent_ids().next();
        let before = match first_parent {
            Some(parent) => {
                let mut parent = objects
                    .find_commit_iter(&parent, &mut self.parent)
                    .map_err(repo_err)?;
                let tree = parent.tree_id().map_err(repo_err)?;
                objects
                    .find_tree_iter(&tree, &mut self.theirs)
                    .map_err(repo_err)?
            }
            None => gix::objs::TreeRefIter::from_bytes(&[], kind),
        };
        let after = objects
            .find_tree_iter(&tree, &mut self.ours)
            .map_err(repo_err)?;
        let mut recorder = gix::diff::tree::Recorder::default();
        gix::diff::tree(before, after, &mut self.state, objects, &mut recorder)
            .map_err(repo_err)?;
        let mut files: Vec<String> = recorder
            .records
            .into_iter()
            .filter_map(|change| match change {
                Change::Addition {
                    entry_mode, path, ..
                }
                | Change::Deletion {
                    entry_mode, path, ..
                }
                | Change::Modification {
                    entry_mode, path, ..
                } => (!entry_mode.is_tree()).then(|| path.to_string()),
            })
            .collect();
        files.sort_unstable();
        files.dedup();
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u8) -> gix::ObjectId {
        gix::ObjectId::from_bytes_or_panic(&[n; 20])
    }

    #[test]
    fn the_files_cache_keeps_the_commits_asked_for_most_recently() {
        let mut cache = FilesCache::default();
        for n in 1..=5 {
            cache.put(id(n), vec![format!("f{n}")]);
        }
        // 1 and 2 are asked for again, after 3, 4 and 5 went in.
        assert_eq!(cache.get(&id(1)), Some(vec!["f1".to_owned()]));
        assert!(cache.get(&id(2)).is_some());
        cache.trim(3);
        assert_eq!(cache.entries.len(), 3);
        for (n, kept) in [(1, true), (2, true), (3, false), (4, false), (5, true)] {
            assert_eq!(cache.get(&id(n)).is_some(), kept, "commit {n}");
        }
        // Under the cap nothing goes.
        cache.trim(3);
        assert_eq!(cache.entries.len(), 3);
        cache.trim(0);
        assert!(cache.entries.is_empty());
    }
}
