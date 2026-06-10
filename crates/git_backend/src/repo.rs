//! Synchronous git repository API.
//!
//! Split of responsibilities (ADR #5 of the technical plan):
//! - **gix** handles the read side that it makes easy and fast: opening /
//!   discovering the repository, walking history, listing references.
//! - **git CLI** handles worktree status, diffs and all mutations
//!   (stage/unstage/discard/commit). For these, gix either lacks a porcelain
//!   API or requires assembling plumbing crates by hand; the CLI's porcelain
//!   v2 / unified diff formats are explicitly stable for tooling. This is
//!   the same pragmatic route Zed takes.
//!
//! Every method is synchronous; the UI talks to [`GitClient`](crate::GitClient)
//! instead, which runs a `GitRepo` on a worker thread.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::diff::parse_unified;
use crate::error::{GitError, Result, repo_err};
use crate::status::parse_porcelain_v2;
use crate::types::{Branch, Commit, FileStatus, Hunk};

pub struct GitRepo {
    repo: gix::Repository,
    /// Working tree root — also the cwd for every CLI invocation.
    workdir: PathBuf,
}

impl GitRepo {
    /// Opens the repository containing `path` (any directory inside the
    /// working tree works, like `git` itself).
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let repo = gix::discover(path.as_ref()).map_err(repo_err)?;
        let workdir = repo.workdir().ok_or(GitError::Bare)?.to_path_buf();
        Ok(Self { repo, workdir })
    }

    /// Working tree root.
    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    // -- read side (gix) ----------------------------------------------------

    /// Last `limit` commits reachable from HEAD, newest first. Returns an
    /// empty list on a repository without commits yet.
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
            });
        }
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

    /// Commits the index with `message`; returns the new commit sha.
    pub fn commit(&self, message: &str) -> Result<String> {
        self.git(&["commit", "--message", message])?;
        let sha = self.git(&["rev-parse", "HEAD"])?;
        Ok(sha.trim().to_string())
    }

    // -- helpers ---------------------------------------------------------------

    fn head_exists(&self) -> bool {
        self.repo.head_id().is_ok()
    }

    /// `true` when the path exists in the index (i.e. not untracked).
    fn is_tracked(&self, path: &str) -> Result<bool> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.workdir)
            .args(["ls-files", "--error-unmatch", "--", path])
            .output()?;
        Ok(out.status.success())
    }

    fn synthesize_untracked_hunk(&self, path: &str) -> Result<Vec<Hunk>> {
        use crate::types::{DiffLine, DiffLineKind};
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

    /// Runs `git <args>` in the working tree, returning stdout on success.
    fn git(&self, args: &[&str]) -> Result<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.workdir)
            .args(args)
            .output()?;
        if !out.status.success() {
            return Err(GitError::Command {
                command: args.join(" "),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}
