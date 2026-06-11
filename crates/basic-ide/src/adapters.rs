//! Maps `git_backend` data into basicIDE view models.
//!
//! The views own plain structs (`FileEntry`, `Stack`, `DiffLine`) and never
//! see gix or git_backend types; this module is the single conversion point,
//! so swapping the backend touches nothing else.

use crate::views::diff_view::{DiffLine, DiffLineKind};
use crate::views::multi_stack_view::{CommitEntry, Stack};
use crate::views::unassigned_view::{FileEntry, FileStatus};

/// Converts a status listing into file rows, preserving git's path order.
pub fn file_entries(status: &[git_backend::FileStatus]) -> Vec<FileEntry> {
    status
        .iter()
        .map(|s| FileEntry {
            path: s.path.clone(),
            status: status_kind(s.status),
            staged: s.staged,
        })
        .collect()
}

fn status_kind(kind: git_backend::StatusKind) -> FileStatus {
    match kind {
        git_backend::StatusKind::Added => FileStatus::Added,
        git_backend::StatusKind::Modified => FileStatus::Modified,
        git_backend::StatusKind::Deleted => FileStatus::Deleted,
        git_backend::StatusKind::Renamed => FileStatus::Renamed,
        git_backend::StatusKind::Untracked => FileStatus::Untracked,
    }
}

/// Builds the stacks panel: the checked-out branch carries the HEAD log,
/// other local branches appear as headers (their logs are not walked —
/// `git_backend::GitRepo::log` follows HEAD only).
pub fn stacks(branches: &[git_backend::Branch], log: &[git_backend::Commit]) -> Vec<Stack> {
    let now = unix_now();
    let mut stacks: Vec<Stack> = Vec::with_capacity(branches.len().max(1));
    for branch in branches {
        let commits = if branch.is_head {
            log.iter().map(|c| commit_entry(c, now)).collect()
        } else {
            Vec::new()
        };
        stacks.push(Stack {
            branch_name: branch.name.clone(),
            is_active: branch.is_head,
            commits,
        });
    }
    // Detached HEAD (or unborn branch list): still show the log.
    if !stacks.iter().any(|s| s.is_active) && !log.is_empty() {
        stacks.insert(
            0,
            Stack {
                branch_name: "(detached HEAD)".into(),
                is_active: true,
                commits: log.iter().map(|c| commit_entry(c, now)).collect(),
            },
        );
    }
    // Active stack first so the relevant commits are visible without scrolling.
    stacks.sort_by_key(|s| !s.is_active);
    stacks
}

fn commit_entry(commit: &git_backend::Commit, now: i64) -> CommitEntry {
    CommitEntry {
        sha: commit.sha.clone(),
        message: commit.message.clone(),
        author: commit.author.clone(),
        time_ago: time_ago(now, commit.time),
    }
}

/// Flattens diff hunks into renderable lines (hunk headers become
/// `DiffLineKind::Header` rows).
pub fn diff_lines(hunks: &[git_backend::Hunk]) -> Vec<DiffLine> {
    let mut lines = Vec::new();
    for hunk in hunks {
        lines.push(DiffLine {
            kind: DiffLineKind::Header,
            line_no_old: None,
            line_no_new: None,
            content: hunk.header.clone(),
        });
        for line in &hunk.lines {
            lines.push(DiffLine {
                kind: match line.kind {
                    git_backend::DiffLineKind::Context => DiffLineKind::Context,
                    git_backend::DiffLineKind::Add => DiffLineKind::Added,
                    git_backend::DiffLineKind::Remove => DiffLineKind::Removed,
                },
                line_no_old: line.old_no,
                line_no_new: line.new_no,
                content: line.content.clone(),
            });
        }
    }
    lines
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Coarse relative timestamp ("2m ago", "3d ago") for commit rows.
pub fn time_ago(now: i64, then: i64) -> String {
    let delta = (now - then).max(0);
    match delta {
        0..60 => "just now".into(),
        60..3600 => format!("{}m ago", delta / 60),
        3600..86_400 => format!("{}h ago", delta / 3600),
        86_400..2_592_000 => format!("{}d ago", delta / 86_400),
        2_592_000..31_536_000 => format!("{}mo ago", delta / 2_592_000),
        _ => format!("{}y ago", delta / 31_536_000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_ago_buckets() {
        assert_eq!(time_ago(1000, 990), "just now");
        assert_eq!(time_ago(1000, 1000 - 120), "2m ago");
        assert_eq!(time_ago(100_000, 100_000 - 7200), "2h ago");
        assert_eq!(time_ago(1_000_000, 1_000_000 - 86_400 * 3), "3d ago");
        assert_eq!(
            time_ago(100_000_000, 100_000_000 - 2_592_000 * 2),
            "2mo ago"
        );
        assert_eq!(time_ago(2_000_000_000, 0), "63y ago");
        // Clock skew must not underflow.
        assert_eq!(time_ago(100, 500), "just now");
    }

    #[test]
    fn stacks_put_active_branch_first_with_log() {
        let branches = vec![
            git_backend::Branch {
                name: "feature/x".into(),
                is_head: false,
            },
            git_backend::Branch {
                name: "main".into(),
                is_head: true,
            },
        ];
        let log = vec![git_backend::Commit {
            sha: "a".repeat(40),
            short_sha: "aaaaaaa".into(),
            message: "subject".into(),
            author: "Author".into(),
            time: 0,
        }];
        let stacks = stacks(&branches, &log);
        assert_eq!(stacks.len(), 2);
        assert_eq!(stacks[0].branch_name, "main");
        assert!(stacks[0].is_active);
        assert_eq!(stacks[0].commits.len(), 1);
        assert_eq!(stacks[0].commits[0].message, "subject");
        assert!(stacks[1].commits.is_empty());
    }

    #[test]
    fn diff_lines_flatten_hunks_with_headers() {
        let hunks = vec![git_backend::Hunk {
            header: "@@ -1,2 +1,2 @@".into(),
            lines: vec![
                git_backend::DiffLine {
                    kind: git_backend::DiffLineKind::Remove,
                    content: "old".into(),
                    old_no: Some(1),
                    new_no: None,
                },
                git_backend::DiffLine {
                    kind: git_backend::DiffLineKind::Add,
                    content: "new".into(),
                    old_no: None,
                    new_no: Some(1),
                },
            ],
        }];
        let lines = diff_lines(&hunks);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].kind, DiffLineKind::Header);
        assert_eq!(lines[1].kind, DiffLineKind::Removed);
        assert_eq!(lines[1].line_no_old, Some(1));
        assert_eq!(lines[2].kind, DiffLineKind::Added);
        assert_eq!(lines[2].content, "new");
    }
}
