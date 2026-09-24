//! Integration tests against real temporary repositories (`git init` via
//! CLI, like the fixtures gix itself uses). Each test gets an isolated
//! `TempDir`, so they can run in parallel.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

use git::{CommitCheck, DiffLineKind, GitClient, GitCommand, GitEvent, GitRepo, StatusKind};
use tempfile::TempDir;

/// Creates a repo with a deterministic identity and one initial commit
/// containing `a.txt` and `src/lib.rs`.
fn repo_with_commit() -> (TempDir, GitRepo) {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "--initial-branch=main"]);
    git(dir.path(), &["config", "user.name", "Test Author"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    write(dir.path(), "a.txt", "line one\nline two\nline three\n");
    write(
        dir.path(),
        "src/lib.rs",
        "pub fn one() {}\npub fn two() {}\n",
    );
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "initial commit"]);
    let repo = GitRepo::open(dir.path()).unwrap();
    (dir, repo)
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(dir: &Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

// -- status -------------------------------------------------------------------

#[test]
fn status_detects_untracked_modified_and_staged() {
    let (dir, repo) = repo_with_commit();
    assert!(repo.status().unwrap().is_empty(), "fresh commit is clean");

    write(dir.path(), "new.txt", "hello\n");
    write(
        dir.path(),
        "a.txt",
        "line one CHANGED\nline two\nline three\n",
    );
    git(dir.path(), &["add", "a.txt"]);
    write(dir.path(), "src/lib.rs", "pub fn one() {}\n");

    let status = repo.status().unwrap();
    let find = |path: &str| status.iter().find(|s| s.path == path).unwrap();

    let untracked = find("new.txt");
    assert_eq!(untracked.status, StatusKind::Untracked);
    assert!(!untracked.staged);

    let staged = find("a.txt");
    assert_eq!(staged.status, StatusKind::Modified);
    assert!(staged.staged);

    let modified = find("src/lib.rs");
    assert_eq!(modified.status, StatusKind::Modified);
    assert!(!modified.staged);

    assert_eq!(status.len(), 3);
}

#[test]
fn stage_and_unstage_transition_entries() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "new.txt", "hello\n");

    repo.stage("new.txt").unwrap();
    let status = repo.status().unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].status, StatusKind::Added);
    assert!(status[0].staged);

    repo.unstage("new.txt").unwrap();
    let status = repo.status().unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].status, StatusKind::Untracked);
    assert!(!status[0].staged);
}

// -- commit & log ---------------------------------------------------------------

#[test]
fn commit_creates_sha_and_clears_status() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "new.txt", "hello\n");
    repo.stage("new.txt").unwrap();

    let commit = repo.commit("add new.txt").unwrap();
    assert_eq!(commit.files, ["new.txt"]);
    let sha = commit.sha;
    assert_eq!(sha.len(), 40, "full hex sha expected, got `{sha}`");
    assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(repo.status().unwrap().is_empty());

    let log = repo.log(10).unwrap();
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].sha, sha);
    assert_eq!(log[0].short_sha, sha[..7]);
    assert_eq!(log[0].message, "add new.txt");
    assert_eq!(log[0].author, "Test Author");
    assert!(log[0].time > 0);
    assert_eq!(log[1].message, "initial commit");
}

#[test]
fn log_respects_limit_and_handles_empty_repo() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "b.txt", "b\n");
    git(dir.path(), &["add", "b.txt"]);
    git(dir.path(), &["commit", "-m", "second"]);
    assert_eq!(repo.log(1).unwrap().len(), 1);
    assert_eq!(repo.log(1).unwrap()[0].message, "second");

    let empty = TempDir::new().unwrap();
    git(empty.path(), &["init"]);
    let repo = GitRepo::open(empty.path()).unwrap();
    assert!(
        repo.log(10).unwrap().is_empty(),
        "unborn HEAD yields no log"
    );
}

// -- the files of each commit in the log --------------------------------------------

/// Runs `git <args>` in `dir`; its stdout.
fn stdout(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8(out.stdout).unwrap()
}

/// The paths git itself says commit `sha` changes: against its first
/// parent (a root commit against the empty tree), renames not detected.
/// The definition of `Commit::files`, by the CLI.
fn changed(dir: &Path, sha: &str) -> Vec<String> {
    let args = [
        "diff-tree",
        "--no-commit-id",
        "--name-only",
        "-r",
        "-z",
        "--no-renames",
        "--root",
        "--diff-merges=first-parent",
        sha,
    ];
    stdout(dir, &args)
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn log_lists_the_files_each_commit_changes_in_gits_order() {
    let (dir, repo) = repo_with_commit();
    // A modification, a file in a new directory whose name sorts after a
    // file of the same stem, a path with a space, a deletion.
    write(dir.path(), "a.txt", "changed\n");
    write(dir.path(), "a/inner.txt", "inner\n");
    write(dir.path(), "b c.txt", "spaced\n");
    git(dir.path(), &["rm", "--quiet", "src/lib.rs"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "second"]);

    let log = repo.log(10).unwrap();
    assert_eq!(log.len(), 2);
    assert_eq!(
        log[0].files,
        ["a.txt", "a/inner.txt", "b c.txt", "src/lib.rs"]
    );
    assert_eq!(log[0].files, changed(dir.path(), &log[0].sha));
    // The root commit, against the empty tree.
    assert_eq!(log[1].files, ["a.txt", "src/lib.rs"]);
    assert_eq!(log[1].files, changed(dir.path(), &log[1].sha));
    for commit in &log {
        assert_eq!(repo.commit_files(&commit.sha).unwrap(), commit.files);
    }
}

#[test]
fn the_files_of_a_merge_are_what_it_brings_to_its_first_parent() {
    let (dir, repo) = repo_with_commit();
    git(dir.path(), &["checkout", "--quiet", "-b", "feature"]);
    write(dir.path(), "f.txt", "feature\n");
    git(dir.path(), &["add", "f.txt"]);
    git(dir.path(), &["commit", "-m", "on feature"]);
    git(dir.path(), &["checkout", "--quiet", "main"]);
    write(dir.path(), "a.txt", "on main\n");
    git(dir.path(), &["commit", "--quiet", "-am", "on main"]);
    git(
        dir.path(),
        &["merge", "--quiet", "--no-ff", "--no-edit", "feature"],
    );

    let log = repo.log(10).unwrap();
    let merge = &log[0];
    assert_eq!(merge.sha, head(dir.path()));
    assert_eq!(merge.files, ["f.txt"], "what the branch brought to main");
    assert_eq!(merge.files, changed(dir.path(), &merge.sha));
    assert_eq!(repo.commit_files("HEAD").unwrap(), merge.files);
}

#[test]
fn a_moved_file_counts_at_both_paths_in_the_commit_and_in_the_log() {
    let (dir, repo) = repo_with_commit();
    git(dir.path(), &["config", "diff.renames", "true"]);
    git(dir.path(), &["mv", "a.txt", "moved.txt"]);
    // The index shows the move as one rename, at its new path.
    assert_eq!(repo.staged_paths().unwrap(), set(&["moved.txt"]));

    let check = repo.commit_if("move", &set(&["moved.txt"])).unwrap();
    let CommitCheck::Committed(commit) = check else {
        panic!("expected a commit, got {check:?}");
    };
    assert_eq!(commit.files, ["a.txt", "moved.txt"]);
    assert_eq!(commit.files, changed(dir.path(), &commit.sha));
    assert_eq!(repo.log(1).unwrap()[0].files, commit.files);
}

#[test]
fn a_second_log_reads_no_tree() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "b.txt", "b\n");
    git(dir.path(), &["add", "b.txt"]);
    git(dir.path(), &["commit", "-m", "second"]);
    let first = repo.log(10).unwrap();
    assert_eq!(first[0].files, ["b.txt"]);

    // Every tree leaves the object database: a log that reads one fails.
    let objects = stdout(
        dir.path(),
        &["cat-file", "--batch-check", "--batch-all-objects"],
    );
    let trees: Vec<&str> = objects
        .lines()
        .filter_map(|line| {
            let mut fields = line.split(' ');
            let id = fields.next()?;
            (fields.next() == Some("tree")).then_some(id)
        })
        .collect();
    assert!(trees.len() >= 2, "the two commits' trees: {objects}");
    for tree in trees {
        let (fan, rest) = tree.split_at(2);
        std::fs::remove_file(dir.path().join(".git/objects").join(fan).join(rest)).unwrap();
    }
    let fresh = GitRepo::open(dir.path()).unwrap();
    assert!(
        fresh.log(10).is_err(),
        "without the trees a first log fails"
    );

    assert_eq!(
        repo.log(10).unwrap(),
        first,
        "the files came from the cache"
    );
}

// -- conditional commit -----------------------------------------------------------

fn set(paths: &[&str]) -> BTreeSet<String> {
    paths.iter().map(|p| p.to_string()).collect()
}

fn head(dir: &Path) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
fn commit_if_commits_when_the_index_stages_what_was_expected() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "a.txt", "changed\n");
    write(dir.path(), "dir/b c.txt", "spaced\n");
    write(dir.path(), "left.txt", "not staged\n");
    git(dir.path(), &["add", "a.txt", "dir/b c.txt"]);
    assert_eq!(repo.staged_paths().unwrap(), set(&["a.txt", "dir/b c.txt"]));

    let check = repo
        .commit_if("two files", &set(&["a.txt", "dir/b c.txt"]))
        .unwrap();
    let CommitCheck::Committed(commit) = check else {
        panic!("expected a commit, got {check:?}");
    };
    assert_eq!(commit.sha, head(dir.path()));
    assert_eq!(commit.files, ["a.txt", "dir/b c.txt"]);
    assert_eq!(repo.commit_files(&commit.sha).unwrap(), commit.files);
    assert_eq!(repo.log(1).unwrap()[0].message, "two files");
    // The unstaged file stays out of the commit and in the worktree.
    let status = repo.status().unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].path, "left.txt");
}

#[test]
fn commit_if_refuses_with_the_delta_when_the_index_moved() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "A", "a\n");
    write(dir.path(), "B", "b\n");
    write(dir.path(), "C", "c\n");
    git(dir.path(), &["add", "A", "C"]);
    let before = head(dir.path());

    // Decided on {A, B}; the index holds {A, C}.
    let check = repo.commit_if("fix login", &set(&["A", "B"])).unwrap();
    assert_eq!(
        check,
        CommitCheck::Stale {
            added: vec!["C".into()],
            removed: vec!["B".into()],
        }
    );
    assert_eq!(head(dir.path()), before, "nothing was committed");
    assert_eq!(repo.staged_paths().unwrap(), set(&["A", "C"]));

    // An empty expectation against a staged index is stale too.
    let check = repo.commit_if("nothing", &BTreeSet::new()).unwrap();
    assert!(matches!(check, CommitCheck::Stale { removed, .. } if removed.is_empty()));
    assert_eq!(head(dir.path()), before);
}

#[test]
fn commit_if_on_an_unborn_head_lists_the_root_commit_files() {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "--initial-branch=main"]);
    git(dir.path(), &["config", "user.name", "Test Author"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    write(dir.path(), "first.txt", "1\n");
    git(dir.path(), &["add", "first.txt"]);
    let repo = GitRepo::open(dir.path()).unwrap();

    let check = repo.commit_if("root", &set(&["first.txt"])).unwrap();
    let CommitCheck::Committed(commit) = check else {
        panic!("expected a commit, got {check:?}");
    };
    assert_eq!(commit.files, ["first.txt"]);
    assert_eq!(commit.sha, head(dir.path()));
}

// -- branches ---------------------------------------------------------------------

#[test]
fn branches_and_current_branch() {
    let (dir, repo) = repo_with_commit();
    git(dir.path(), &["branch", "feature/x"]);

    assert_eq!(repo.current_branch().unwrap().as_deref(), Some("main"));
    let branches = repo.branches().unwrap();
    let names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"main"));
    assert!(names.contains(&"feature/x"));
    for branch in &branches {
        assert_eq!(branch.is_head, branch.name == "main");
    }
}

// -- diff -------------------------------------------------------------------------

#[test]
fn diff_workdir_of_modified_file_has_expected_hunks() {
    let (dir, repo) = repo_with_commit();
    write(
        dir.path(),
        "a.txt",
        "line one\nline 2 CHANGED\nline three\n",
    );

    let hunks = repo.diff_workdir("a.txt").unwrap();
    assert_eq!(hunks.len(), 1);
    let hunk = &hunks[0];
    assert!(
        hunk.header.starts_with("@@ -1,3 +1,3 @@"),
        "{}",
        hunk.header
    );

    let removed: Vec<_> = hunk
        .lines
        .iter()
        .filter(|l| l.kind == DiffLineKind::Remove)
        .collect();
    let added: Vec<_> = hunk
        .lines
        .iter()
        .filter(|l| l.kind == DiffLineKind::Add)
        .collect();
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].content, "line two");
    assert_eq!(removed[0].old_no, Some(2));
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].content, "line 2 CHANGED");
    assert_eq!(added[0].new_no, Some(2));
}

#[test]
fn diff_workdir_includes_staged_changes() {
    let (dir, repo) = repo_with_commit();
    write(
        dir.path(),
        "a.txt",
        "line one\nline two\nline three\nline four\n",
    );
    git(dir.path(), &["add", "a.txt"]);

    // Staged-only change must still appear (diff is against HEAD).
    let hunks = repo.diff_workdir("a.txt").unwrap();
    assert_eq!(hunks.len(), 1);
    let added: Vec<_> = hunks[0]
        .lines
        .iter()
        .filter(|l| l.kind == DiffLineKind::Add)
        .collect();
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].content, "line four");
}

#[test]
fn diff_workdir_of_untracked_file_synthesizes_added_hunk() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "new.txt", "alpha\nbeta\n");

    let hunks = repo.diff_workdir("new.txt").unwrap();
    assert_eq!(hunks.len(), 1);
    assert_eq!(hunks[0].lines.len(), 2);
    assert!(
        hunks[0]
            .lines
            .iter()
            .all(|l| l.kind == DiffLineKind::Add && l.old_no.is_none())
    );
    assert_eq!(hunks[0].lines[0].content, "alpha");
    assert_eq!(hunks[0].lines[0].new_no, Some(1));
    assert_eq!(hunks[0].lines[1].new_no, Some(2));
}

#[test]
fn diff_commit_shows_changes_of_that_commit() {
    let (dir, repo) = repo_with_commit();
    write(
        dir.path(),
        "a.txt",
        "line one\nline two\nline three\nline four\n",
    );
    write(dir.path(), "c.txt", "c content\n");
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "touch two files"]);
    let sha = repo.log(1).unwrap()[0].sha.clone();

    let hunks = repo.diff_commit(&sha).unwrap();
    assert_eq!(hunks.len(), 2, "one hunk per touched file");
    // Multi-file diff prefixes headers with the path.
    assert!(hunks.iter().any(|h| h.header.starts_with("a.txt: @@")));
    assert!(hunks.iter().any(|h| h.header.starts_with("c.txt: @@")));

    // Root commits diff against the empty tree.
    let root_sha = repo.log(10).unwrap().last().unwrap().sha.clone();
    let root_hunks = repo.diff_commit(&root_sha).unwrap();
    assert!(!root_hunks.is_empty());
    assert!(
        root_hunks
            .iter()
            .flat_map(|h| &h.lines)
            .all(|l| l.kind == DiffLineKind::Add)
    );
}

// -- discard & ignore -------------------------------------------------------------

#[test]
fn discard_restores_content_from_head() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "a.txt", "TRASHED\n");
    repo.discard("a.txt").unwrap();
    let content = std::fs::read_to_string(dir.path().join("a.txt")).unwrap();
    assert_eq!(content, "line one\nline two\nline three\n");
    assert!(repo.status().unwrap().is_empty());
}

#[test]
fn discard_untracked_file_removes_it() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "junk.txt", "junk\n");
    repo.discard("junk.txt").unwrap();
    assert!(!dir.path().join("junk.txt").exists());
    assert!(repo.status().unwrap().is_empty());
}

#[test]
fn ignore_appends_to_gitignore_and_hides_file() {
    let (dir, repo) = repo_with_commit();
    write(dir.path(), "build.log", "noise\n");
    assert_eq!(repo.status().unwrap().len(), 1);

    repo.ignore("build.log").unwrap();
    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert_eq!(gitignore, "build.log\n");

    // The ignored file disappears; the new .gitignore itself shows up.
    let status = repo.status().unwrap();
    assert!(status.iter().all(|s| s.path != "build.log"));
    assert!(status.iter().any(|s| s.path == ".gitignore"));

    // Appending preserves existing entries on their own lines.
    repo.ignore("dist/").unwrap();
    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert_eq!(gitignore, "build.log\ndist/\n");
}

// -- threaded client -------------------------------------------------------------

#[test]
fn git_client_round_trips_commands_off_thread() {
    let (dir, _repo) = repo_with_commit();
    write(dir.path(), "new.txt", "hello\n");

    let (tx, rx) = channel();
    let client = GitClient::spawn(dir.path(), move |event| {
        tx.send(event).unwrap();
    })
    .unwrap();

    client.send(GitCommand::Status);
    let event = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let GitEvent::Status(Ok(status)) = event else {
        panic!("expected Status event, got {event:?}");
    };
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].path, "new.txt");

    client.send(GitCommand::Stage {
        path: "new.txt".into(),
    });
    let event = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(matches!(event, GitEvent::Staged { result: Ok(()), .. }));

    // Decided on a set the index does not hold: refused with the delta.
    client.send(GitCommand::CommitIf {
        message: "via client".into(),
        expect_staged: set(&["other.txt"]),
    });
    let event = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let GitEvent::CommitRejected { added, removed } = event else {
        panic!("expected CommitRejected event, got {event:?}");
    };
    assert_eq!(
        (added, removed),
        (vec!["new.txt".into()], vec!["other.txt".into()])
    );

    client.send(GitCommand::CommitIf {
        message: "via client".into(),
        expect_staged: set(&["new.txt"]),
    });
    let event = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let GitEvent::Committed(Ok(commit)) = event else {
        panic!("expected Committed event, got {event:?}");
    };
    assert_eq!(commit.sha.len(), 40);
    assert_eq!(commit.files, ["new.txt"]);

    // Refresh fans out three events.
    client.send(GitCommand::Refresh { log_limit: 5 });
    let mut kinds = Vec::new();
    for _ in 0..3 {
        let event = rx.recv_timeout(Duration::from_secs(10)).unwrap();
        kinds.push(match event {
            GitEvent::Status(r) => {
                assert!(r.unwrap().is_empty());
                "status"
            }
            GitEvent::Log(r) => {
                assert_eq!(r.unwrap().len(), 2);
                "log"
            }
            GitEvent::Branches(r) => {
                assert!(r.unwrap().iter().any(|b| b.is_head));
                "branches"
            }
            other => panic!("unexpected event {other:?}"),
        });
    }
    kinds.sort();
    assert_eq!(kinds, ["branches", "log", "status"]);

    drop(client); // joins the worker without hanging
}

#[test]
fn git_client_spawn_fails_outside_a_repository() {
    let dir = TempDir::new().unwrap();
    let result = GitClient::spawn(dir.path(), |_| {});
    assert!(result.is_err());
}
