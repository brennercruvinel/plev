//! Plain data types shared by the sync API ([`GitRepo`](crate::GitRepo)) and
//! the threaded client ([`GitClient`](crate::GitClient)). UI crates map these
//! into their own view models; nothing here knows about gix or rendering.

/// Kind of change a file underwent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
}

/// One changed path in the working tree or index.
///
/// A path that has both staged and unstaged changes yields *two* entries
/// (one with `staged: true`, one with `staged: false`), mirroring how git
/// itself reports the X and Y columns of `status --porcelain=v2`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileStatus {
    pub path: String,
    pub status: StatusKind,
    pub staged: bool,
}

/// A commit as shown in log views.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    /// Full hex object id.
    pub sha: String,
    /// First 7 hex chars of `sha`.
    pub short_sha: String,
    /// Subject line (first line of the message).
    pub message: String,
    pub author: String,
    /// Commit time in seconds since the Unix epoch.
    pub time: i64,
}

/// Kind of a single diff line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Add,
    Remove,
}

/// One line of a unified diff hunk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    /// Line content without the leading `+`/`-`/` ` marker.
    pub content: String,
    /// Line number on the old side (`None` for added lines).
    pub old_no: Option<u32>,
    /// Line number on the new side (`None` for removed lines).
    pub new_no: Option<u32>,
}

/// A unified diff hunk. For multi-file diffs (e.g. a whole commit) the
/// header is prefixed with the file path so consumers can render flat lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hunk {
    /// The `@@ -a,b +c,d @@ …` line, prefixed with `path: ` when the diff
    /// spans more than one file.
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// A local branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    /// `true` when HEAD currently points at this branch.
    pub is_head: bool,
}
