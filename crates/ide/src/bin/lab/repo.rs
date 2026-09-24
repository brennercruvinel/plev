//! the repository of the experiment: a real git repository in a temporary
//! directory with one commit and six changes, part of them in the index,
//! mixing the files of the task with files that are not. the same for
//! every arm, so the arms compare.
//!
//! the shape follows `crates/ide/tests/support/mod.rs` (`Rig::new`): real
//! git, a temp dir, `git(args)`, `head`, `head_files`. the tests panic on
//! a failure; a binary reports it, so the helpers return `Result`.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

/// the files the task asks to commit, in path order.
pub const TASK_FILES: [&str; 3] = ["src/login.rs", "src/session.rs", "tests/login_test.rs"];

/// the message the task asks for.
pub const TASK_MESSAGE: &str = "fix login session timeout";

/// the task as the agent reads it.
pub const TASK: &str = "commit only the login fix: src/login.rs, src/session.rs, \
                        tests/login_test.rs, message: fix login session timeout";

/// the six changes of the working tree: the path, whether it starts in
/// the index, and whether the initial commit already tracks it. the
/// repository is built from this list, and the test reads the porcelain
/// status back against it.
pub const CHANGES: [(&str, bool, bool); 6] = [
    ("src/login.rs", true, true),
    ("src/session.rs", false, true),
    ("tests/login_test.rs", false, false),
    ("README.md", true, true),
    ("notes.txt", false, false),
    ("Cargo.toml", false, true),
];

pub struct Repo {
    dir: TempDir,
}

impl Repo {
    /// builds the repository: the initial commit holds `src/login.rs`,
    /// `src/session.rs`, `README.md` and `Cargo.toml`; then every file of
    /// `CHANGES` is written (modified or new) and `src/login.rs` and
    /// `README.md` are added to the index.
    pub fn build() -> Result<Repo, String> {
        let dir = TempDir::new().map_err(|e| format!("a temp dir: {e}"))?;
        let repo = Repo { dir };
        let at = repo.path();
        repo.git(&["init", "-q", "--initial-branch=main"])?;
        repo.git(&["config", "user.name", "Lab Author"])?;
        repo.git(&["config", "user.email", "lab@example.com"])?;
        repo.git(&["config", "commit.gpgsign", "false"])?;
        for (path, content) in BEFORE {
            write(at, path, content)?;
        }
        repo.add(&tracked())?;
        repo.git(&["commit", "-q", "--message", "login form and session store"])?;
        for (path, content) in AFTER {
            write(at, path, content)?;
        }
        repo.add(&staged_at_start())?;
        Ok(repo)
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// `git add -- <paths>`.
    fn add(&self, paths: &[&str]) -> Result<(), String> {
        let mut args = vec!["add", "--"];
        args.extend_from_slice(paths);
        self.git(&args).map(|_| ())
    }

    /// runs `git` in the repository; its stdout, or git's stderr as the
    /// error.
    pub fn git(&self, args: &[&str]) -> Result<String, String> {
        git(self.path(), args)
    }

    pub fn head(&self) -> Result<String, String> {
        self.git(&["rev-parse", "HEAD"])
            .map(|s| s.trim().to_owned())
    }

    /// the paths the last commit changes, as git lists them against its
    /// first parent (a root commit against the empty tree), a renamed
    /// file at both paths: the definition of a commit's files in the
    /// receipt and in the stacks (`git::Commit::files`), read here by the
    /// cli, apart from the workbench.
    pub fn head_files(&self) -> Result<Vec<String>, String> {
        Ok(self
            .git(&HEAD_FILES)?
            .split('\0')
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .collect())
    }

    pub fn head_subject(&self) -> Result<String, String> {
        self.git(&["log", "-1", "--format=%s"])
            .map(|s| s.trim().to_owned())
    }

    pub fn log_oneline(&self) -> Result<String, String> {
        self.git(&["log", "--oneline"])
    }

    /// `git status --porcelain`, listing every untracked file rather
    /// than the directory that holds them, which is what the ide's own
    /// status reads (`--untracked-files=all`) and what `CHANGES`
    /// declares.
    pub fn status(&self) -> Result<String, String> {
        self.git(&["status", "--porcelain", "--untracked-files=all"])
    }
}

/// the paths the initial commit tracks.
fn tracked() -> Vec<&'static str> {
    CHANGES
        .iter()
        .filter(|(_, _, tracked)| *tracked)
        .map(|(path, _, _)| *path)
        .collect()
}

/// the paths the index holds when the experiment starts.
fn staged_at_start() -> Vec<&'static str> {
    CHANGES
        .iter()
        .filter(|(_, staged, _)| *staged)
        .map(|(path, _, _)| *path)
        .collect()
}

/// true when `files` is exactly the task's set, in any order.
pub fn is_the_task(files: &[String]) -> bool {
    let mut got: Vec<&str> = files.iter().map(String::as_str).collect();
    got.sort_unstable();
    got == TASK_FILES
}

pub fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git {args:?}: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn write(dir: &Path, rel: &str, content: &str) -> Result<(), String> {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))
}

/// `git diff-tree` for HEAD's files: `--root` lists a root commit against
/// the empty tree, `--diff-merges=first-parent` a merge against its first
/// parent, `--no-renames` a rename at both paths.
const HEAD_FILES: [&str; 9] = [
    "diff-tree",
    "--no-commit-id",
    "--name-only",
    "-r",
    "-z",
    "--no-renames",
    "--root",
    "--diff-merges=first-parent",
    "HEAD",
];

/// the tree at the initial commit.
const BEFORE: [(&str, &str); 4] = [
    (
        "src/login.rs",
        "pub struct Login {\n    pub user: String,\n}\n\nimpl Login {\n    pub fn timeout_secs(&self) -> u64 {\n        30\n    }\n}\n",
    ),
    (
        "src/session.rs",
        "pub struct Session {\n    pub started_at: u64,\n}\n\nimpl Session {\n    pub fn expired(&self, now: u64) -> bool {\n        now - self.started_at > 30\n    }\n}\n",
    ),
    ("README.md", "# demo\n\na login form and a session store.\n"),
    (
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    ),
];

/// the working tree at the start of the experiment: the login fix in
/// three files, and three other changes that are not part of it.
const AFTER: [(&str, &str); 6] = [
    (
        "src/login.rs",
        "pub struct Login {\n    pub user: String,\n}\n\nimpl Login {\n    /// the session timeout the server enforces, in seconds.\n    pub fn timeout_secs(&self) -> u64 {\n        1800\n    }\n}\n",
    ),
    (
        "src/session.rs",
        "pub struct Session {\n    pub started_at: u64,\n    pub timeout_secs: u64,\n}\n\nimpl Session {\n    pub fn expired(&self, now: u64) -> bool {\n        now.saturating_sub(self.started_at) > self.timeout_secs\n    }\n}\n",
    ),
    (
        "tests/login_test.rs",
        "#[test]\nfn a_session_lasts_thirty_minutes() {\n    let login = demo::Login { user: \"ana\".into() };\n    assert_eq!(login.timeout_secs(), 1800);\n}\n",
    ),
    (
        "README.md",
        "# demo\n\na login form and a session store.\n\n## running\n\ncargo run\n",
    ),
    ("notes.txt", "ideas for the next sprint\n- dark theme\n"),
    (
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.2.0\"\nedition = \"2024\"\n",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_repository_starts_as_the_protocol_says() {
        let repo = Repo::build().unwrap();
        let status = repo.status().unwrap();
        let mut lines: Vec<&str> = status.lines().collect();
        lines.sort_unstable();
        // one porcelain line per change, as CHANGES declares it: the
        // index side for a staged one, the worktree side otherwise, and
        // `??` for a file git does not track yet.
        let mut want: Vec<String> = CHANGES
            .iter()
            .map(|(path, staged, tracked)| match (staged, tracked) {
                (true, _) => format!("M  {path}"),
                (false, true) => format!(" M {path}"),
                (false, false) => format!("?? {path}"),
            })
            .collect();
        want.sort();
        assert_eq!(lines, want);
        assert_eq!(
            repo.git(&["rev-list", "--count", "HEAD"]).unwrap().trim(),
            "1"
        );
        assert!(!is_the_task(&repo.head_files().unwrap()));
        let mut task: Vec<String> = TASK_FILES.iter().rev().map(|s| (*s).to_owned()).collect();
        assert!(is_the_task(&task));
        task.push("README.md".into());
        assert!(!is_the_task(&task));
    }
}
