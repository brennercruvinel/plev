//! What the Claude is shown of the repository (task-96): what the review
//! needs, never the workspace whole. The prompt of `offers::Llm` carries
//! every node of the snapshot it is given, with its values
//! (`offers::prompt`), so the workbench gives it a snapshot of its own:
//! the commits the review names, as the rules read the intent, each with
//! its message, its files and the action that shows its diff; and the diff
//! region, with its lines and their kinds only while it shows one of
//! those commits.
//!
//! Left out, whatever the workspace shows: the working tree (its changed
//! files, a file's diff), every other commit, the authors, the branch, a
//! commit message being written, the notices, the path of the repository.

use accord::{Role, SemErrors, SemId, Snapshot, SnapshotBuilder, Value};
use plevir::{Ir, IrNode};

use crate::capability::ids::Ids;

/// The commits `ir` binds (a state it shows or an action it asks), in
/// the order it first binds them, each once.
pub(crate) fn commits(ir: &Ir, ids: &Ids) -> Vec<SemId> {
    let mut out: Vec<SemId> = Vec::new();
    let mut stack: Vec<&IrNode> = vec![&ir.surface.root];
    while let Some(node) = stack.pop() {
        let bound = node
            .bind
            .state
            .values()
            .map(|at| &at.node)
            .chain(node.bind.actions.values().map(|cap| &cap.target));
        for id in bound {
            if ids.sha_of(id).is_some() && !out.contains(id) {
                out.push(id.clone());
            }
        }
        if let Some(repeat) = &node.repeat {
            stack.push(&repeat.item);
        }
        stack.extend(node.children.iter().rev());
    }
    out
}

/// The snapshot the Claude is shown for `commits` over `app`.
pub(crate) fn scoped(app: &Snapshot, commits: &[SemId], ids: &Ids) -> Result<Snapshot, SemErrors> {
    let mut b = SnapshotBuilder::new();
    b.node(&ids.stacks, Role::List, "stacks");
    let target = format!("{}:target", ids.diff);
    for id in commits {
        let Some(node) = app.get(id) else {
            continue;
        };
        let mut n = b.node(id, Role::ListItem, node.label.as_str());
        n.parent(&ids.stacks);
        for (key, value) in &node.state {
            if matches!(key.as_str(), "summary" | "files") {
                n.state(key.as_str(), value.clone());
            }
        }
        if node.actions.iter().any(|a| a.name.as_str() == "show_diff") {
            n.action("show_diff").reads(target.as_str());
        }
    }
    let at = |key: &str| format!("{}:{key}", ids.diff).parse().ok();
    let value = |key: &str| at(key).and_then(|at| app.value(&at)).cloned();
    let shows = match value("target") {
        Some(Value::Text(t)) => commits.iter().any(|c| c.as_str() == t),
        _ => false,
    };
    let mut diff = b.node(&ids.diff, Role::Region, "diff");
    let lines = (value("lines"), value("kinds"));
    match (shows, value("target"), lines, value("count")) {
        (true, Some(t), (Some(lines), Some(kinds)), Some(count)) => {
            diff.state("target", t)
                .state("lines", lines)
                .state("kinds", kinds)
                .state("count", count);
        }
        _ => {
            diff.state("target", "")
                .state("lines", Vec::<String>::new())
                .state("kinds", Vec::<String>::new())
                .state("count", 0i64);
        }
    }
    b.finish()
}
