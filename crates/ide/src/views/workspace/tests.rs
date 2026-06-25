//! Workspace tests, split by area:
//! - [`overlays`]: context menu / modal flow (stage, unstage, discard).
//! - [`scrolling`]: per-panel scroll, hit clamping and hover probes.
//! - [`panels`]: commit form and the non-destructive panel width clamp.
//!
//! Shared fixtures live here — tests never touch a real repo; the app feeds
//! the same structs from `git status` via `adapters`.

mod overlays;
mod panels;
mod scrolling;

use super::*;
use crate::views::unassigned_view::{FileEntry, FileStatus};
use engine::compositor::Compositor;
use engine::overlay::OverlayKind;

fn sample_files() -> Vec<FileEntry> {
    let entry = |path: &str, status, staged| FileEntry {
        path: path.into(),
        status,
        staged,
    };
    vec![
        entry("src/compositor.rs", FileStatus::Modified, false),
        entry("src/builder.rs", FileStatus::Modified, false),
        entry("src/scroll.rs", FileStatus::Added, true),
        entry("docs/notes.md", FileStatus::Untracked, false),
        entry("old/gone.rs", FileStatus::Deleted, false),
    ]
}

fn ws() -> (WorkspaceView, Compositor) {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.unassigned.set_files(sample_files());
    let mut c = Compositor::new();
    w.render(&mut c);
    (w, c)
}

fn first_file_center(w: &WorkspaceView) -> (f32, f32) {
    let r = &w.unassigned.hit_rects()[0];
    (r.0 + r.2 / 2.0, r.1 + r.3 / 2.0)
}

fn many_files(n: usize) -> Vec<FileEntry> {
    (0..n)
        .map(|i| FileEntry {
            path: format!("src/file_{i}.rs"),
            status: FileStatus::Modified,
            staged: false,
        })
        .collect()
}

fn sample_stacks(commits: usize) -> Vec<crate::views::multi_stack_view::Stack> {
    use crate::views::multi_stack_view::{CommitEntry, Stack};
    vec![Stack {
        branch_name: "main".into(),
        is_active: true,
        commits: (0..commits)
            .map(|i| CommitEntry {
                sha: format!("{i:07x}"),
                message: format!("commit {i}"),
                author: "Hoff".into(),
                time_ago: "now".into(),
            })
            .collect(),
    }]
}

fn sample_diff(lines: usize) -> Vec<crate::views::diff_view::DiffLine> {
    use crate::views::diff_view::{DiffLine, DiffLineKind};
    (0..lines)
        .map(|i| DiffLine {
            kind: DiffLineKind::Context,
            line_no_old: Some(i as u32 + 1),
            line_no_new: Some(i as u32 + 1),
            content: format!("line {i}"),
        })
        .collect()
}
