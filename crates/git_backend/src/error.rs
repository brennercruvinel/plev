use std::fmt;

pub type Result<T> = std::result::Result<T, GitError>;

/// Errors produced by [`GitRepo`](crate::GitRepo) operations.
#[derive(Debug)]
pub enum GitError {
    /// Spawning or talking to the `git` CLI failed at the OS level.
    Io(std::io::Error),
    /// The `git` CLI ran but exited with a failure status.
    Command { command: String, stderr: String },
    /// gix-level failure (discovery, object decode, ref traversal, …).
    Repo(String),
    /// Output of a git command did not match the expected format.
    Parse(String),
    /// The repository has no working tree (bare repo).
    Bare,
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::Io(e) => write!(f, "git io error: {e}"),
            GitError::Command { command, stderr } => {
                write!(f, "`git {command}` failed: {}", stderr.trim())
            }
            GitError::Repo(message) => write!(f, "repository error: {message}"),
            GitError::Parse(message) => write!(f, "unexpected git output: {message}"),
            GitError::Bare => write!(f, "repository has no working tree"),
        }
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GitError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self {
        GitError::Io(e)
    }
}

/// Stringifies any gix error into [`GitError::Repo`]. gix has dozens of
/// per-operation error types; the UI only ever displays them.
pub(crate) fn repo_err(e: impl fmt::Display) -> GitError {
    GitError::Repo(e.to_string())
}
