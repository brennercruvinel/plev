//! What the workspace means, projected from the views as they are now.
//!
//! - `changes` (list): `files` (the changed paths, in list order),
//!   `staged` (the paths the index holds changes for) and `selected` (the
//!   highlighted row's path, `""` without one: navigation, which no action
//!   reads).
//! - `changes.file[<path>]` (list item): `status`, `staged`, `unstaged`;
//!   `stage`, `unstage`, `discard` (destructive, an agent's request waits
//!   for a person) and `ignore` read the file's `staged` and `unstaged`,
//!   the state they change; `show_diff` reads `diff:target`.
//! - `diff` (region): `target` (the id whose diff is shown), the first
//!   [`DIFF_LINES`] `lines` (each with the diff's `+`, `-` or space in
//!   front, a hunk's header as it is), the kind of each of them in
//!   `kinds` (`context`, `added`, `removed`, `header`: the names of
//!   `plevir::LineKind`, task-97) and the `count` of every line.
//! - `commit.form` (group): `open`; `open` reads it, `cancel` reads it and
//!   the message it clears.
//! - `commit.message` (text input): `text`, required; `set_value` reads it.
//! - `commit.submit` (button): enabled while the form is open, the message
//!   is not blank and something is staged; `submit` reads
//!   `changes:staged` and `commit.message:text`, and nothing else, so a
//!   moved cursor or a new diff never makes a commit stale.
//! - `stacks` (list): `branch`; `stacks.commit[<sha>]` (list item):
//!   `summary`, `author`; `show_diff` reads `diff:target`. The
//!   [`FILES_PROJECTED`] newest commits of the checked-out stack, and the
//!   last commit this workbench made wherever it is, also carry `files`
//!   (the paths the commit changes, in git's order); older ones do not.
//! - `notices` (region): one `notices[<id>]` per toast on screen (status,
//!   or alert for a refusal that names a change), named by its message,
//!   with `dismiss` (the comps toast helper).
//! - with the generated surface on (`crate::generated`, task-96), after
//!   all of the above: the surface open, and the intent line under
//!   `ide_surface`. [`Workbench::app_snapshot`] is the workspace alone.

use std::collections::{BTreeSet, HashSet};

use accord::{Confirm, Constraints, Policy, Role, Semantics, SnapshotBuilder, Value};
use plevir::LineKind;

use super::Workbench;
use crate::capability::ids::Ids;
use crate::interfaces::panels::diff::DiffLineKind;
use crate::interfaces::panels::layout::WorkspaceView;

/// How many diff lines the projection carries: an agent reads a bounded
/// diff, `count` says how many there are.
pub const DIFF_LINES: usize = 200;

/// How many of the newest commits of the checked-out stack carry their
/// `files`: enough for an agent to confirm what it just committed and
/// see what landed right before, without every observation paying for
/// the whole log's paths.
pub const FILES_PROJECTED: usize = 10;

impl Semantics for Workbench {
    /// The workspace, then the intent line and the generated surface when
    /// they are on (`crate::generated`): what every actor observes and
    /// the dispatcher checks against.
    fn project(&self, b: &mut SnapshotBuilder) {
        self.project_app(b);
        if let Some(generated) = &self.generated {
            generated.project(b);
        }
    }
}

impl Workbench {
    /// The workspace alone: what [`Workbench::app_snapshot`] gives, the
    /// registry a generated surface binds.
    pub(crate) fn project_app(&self, b: &mut SnapshotBuilder) {
        let ws = &self.workspace;
        let Some(ids) = &ws.ids else {
            return;
        };
        changes(ws, ids, b);
        diff(ws, ids, b);
        commit(ws, ids, b);
        let made = self.git.as_ref().and_then(|git| git.made());
        stacks(ws, ids, made, b);
        ws.toasts.project(b, &ids.notices, "notices", None);
        if ws.commit_form.visible && ws.commit_form.field().is_focused() {
            b.focus(&ids.message);
        }
    }
}

fn changes(ws: &WorkspaceView, ids: &Ids, b: &mut SnapshotBuilder) {
    let changes = ws.unassigned.changes();
    let files: Vec<String> = changes.iter().map(|c| c.path.clone()).collect();
    let staged: BTreeSet<String> = changes
        .iter()
        .filter(|c| c.staged)
        .map(|c| c.path.clone())
        .collect();
    b.node(&ids.changes, Role::List, "changes")
        .state("files", files)
        .state("staged", staged)
        .state("selected", ws.unassigned.selected_path().unwrap_or(""));
    let target = format!("{}:target", ids.diff);
    let destructive = Policy {
        confirm: Confirm::ForAgents,
        destructive: true,
        ..Policy::default()
    };
    for change in &changes {
        let id = ids.file(&change.path);
        let staged = format!("{id}:staged");
        let unstaged = format!("{id}:unstaged");
        let mut node = b.node(&id, Role::ListItem, change.path.as_str());
        node.parent(&ids.changes)
            .state("status", change.status.name())
            .state("staged", change.staged)
            .state("unstaged", change.unstaged);
        node.action("stage")
            .enabled(change.unstaged)
            .reads(staged.as_str())
            .reads(unstaged.as_str());
        node.action("unstage")
            .enabled(change.staged)
            .reads(staged.as_str())
            .reads(unstaged.as_str());
        node.action("discard")
            .reads(staged.as_str())
            .reads(unstaged.as_str())
            .policy(destructive);
        node.action("ignore")
            .reads(staged.as_str())
            .reads(unstaged.as_str());
        node.action("show_diff").reads(target.as_str());
    }
}

fn diff(ws: &WorkspaceView, ids: &Ids, b: &mut SnapshotBuilder) {
    let view = &ws.diff;
    let shown = || view.lines.iter().take(DIFF_LINES);
    let lines: Vec<String> = shown()
        .map(|line| {
            let mark = match line.kind {
                DiffLineKind::Header => return line.content.clone(),
                DiffLineKind::Added => '+',
                DiffLineKind::Removed => '-',
                DiffLineKind::Context => ' ',
            };
            format!("{mark}{}", line.content)
        })
        .collect();
    // the kind of each line by name: a header's first letter is its
    // file's, so the mark alone cannot say it
    let kinds: Vec<String> = shown()
        .map(|line| line_kind(line.kind).as_str().to_owned())
        .collect();
    let target = view.target.as_ref().map_or("", |id| id.as_str());
    b.node(&ids.diff, Role::Region, "diff")
        .state("target", target)
        .state("lines", lines)
        .state("kinds", kinds)
        .state("count", count(view.lines.len()));
}

/// The kind of a line of the diff panel by the name a `code` shows it by.
fn line_kind(kind: DiffLineKind) -> LineKind {
    match kind {
        DiffLineKind::Context => LineKind::Context,
        DiffLineKind::Added => LineKind::Added,
        DiffLineKind::Removed => LineKind::Removed,
        DiffLineKind::Header => LineKind::Header,
    }
}

fn commit(ws: &WorkspaceView, ids: &Ids, b: &mut SnapshotBuilder) {
    let form = &ws.commit_form;
    let open = format!("{}:open", ids.form);
    let text = format!("{}:text", ids.message);
    let mut node = b.node(&ids.form, Role::Group, "commit");
    node.state("open", form.visible);
    node.action("open")
        .enabled(!form.visible)
        .reads(open.as_str());
    node.action("cancel")
        .enabled(form.visible)
        .reads(open.as_str())
        .reads(text.as_str());
    form.field()
        .project(b, &ids.message, "commit message", Some(&ids.form))
        .enabled(form.visible)
        .constraints(Constraints {
            editable: true,
            required: true,
            ..Constraints::default()
        });
    b.node(&ids.submit, Role::Button, form.submit_label())
        .parent(&ids.form)
        .enabled(ws.can_submit())
        .action("submit")
        .reads(format!("{}:staged", ids.changes))
        .reads(text.as_str());
}

/// `made` is the last commit this workbench made, which keeps its
/// `files` however many commits landed after it.
fn stacks(ws: &WorkspaceView, ids: &Ids, made: Option<&str>, b: &mut SnapshotBuilder) {
    b.node(&ids.stacks, Role::List, "stacks")
        .state("branch", ws.branch_label.as_str());
    let target = format!("{}:target", ids.diff);
    let mut seen = HashSet::new();
    let commits = ws.stacks.stacks.iter().flat_map(|stack| {
        stack
            .commits
            .iter()
            .enumerate()
            .map(move |(at, commit)| (stack.is_active && at < FILES_PROJECTED, commit))
    });
    for (recent, commit) in commits {
        if !seen.insert(commit.sha.as_str()) {
            continue;
        }
        // An interactive node needs a label; a commit may have an empty
        // subject.
        let label = if commit.message.trim().is_empty() {
            commit.sha.get(..7).unwrap_or(&commit.sha)
        } else {
            commit.message.as_str()
        };
        let mut node = b.node(ids.commit(&commit.sha), Role::ListItem, label);
        node.parent(&ids.stacks)
            .state("summary", commit.message.as_str())
            .state("author", commit.author.as_str());
        if recent || made == Some(commit.sha.as_str()) {
            node.state("files", commit.files.clone());
        }
        node.action("show_diff").reads(target.as_str());
    }
}

fn count(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}
