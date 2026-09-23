//! The workbench without a repository: what it projects, what the render
//! binds, and which state a commit depends on. The flows against a real
//! repository live in `crates/ide/tests`.

use engine::compositor::Compositor;
use sem::{Actor, Confirm, Request, Role, SemId, Snapshot, Value, Verdict, check};

use super::*;
use crate::views::multi_stack_view::{CommitEntry, Stack};
use crate::views::unassigned_view::{FileEntry, FileStatus};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("id")
}

fn entry(path: &str, status: FileStatus, staged: bool) -> FileEntry {
    FileEntry {
        path: path.into(),
        status,
        staged,
    }
}

/// `a.txt` staged, `b.txt` untracked, `c.rs` staged and changed again in
/// the worktree (two rows), one commit, the form open with a message.
fn bench() -> Workbench {
    let mut bench = Workbench::new(1280.0, 800.0);
    let ws = bench.workspace_mut();
    ws.unassigned.set_files(vec![
        entry("a.txt", FileStatus::Added, true),
        entry("b.txt", FileStatus::Untracked, false),
        entry("c.rs", FileStatus::Modified, true),
        entry("c.rs", FileStatus::Modified, false),
    ]);
    ws.stacks.set_stacks(vec![Stack {
        branch_name: "main".into(),
        is_active: true,
        commits: vec![CommitEntry {
            sha: "f00d".repeat(10),
            message: "initial".into(),
            author: "Test".into(),
            time_ago: "now".into(),
        }],
    }]);
    ws.branch_label = "main".into();
    ws.commit_form.show();
    for ch in "fix login".chars() {
        ws.commit_form.append_char(ch);
    }
    bench
}

fn value(snap: &Snapshot, at: &str) -> Value {
    snap.value(&at.parse().expect("state ref"))
        .cloned()
        .expect("projected")
}

fn reads(snap: &Snapshot, id: &str, action: &str) -> Vec<String> {
    let node = snap.get(&sid(id)).expect("node");
    let decl = node
        .actions
        .iter()
        .find(|a| a.name.as_str() == action)
        .expect("action");
    decl.reads.iter().map(ToString::to_string).collect()
}

#[test]
fn the_workbench_projects_the_workspace_model() {
    let snap = bench().snapshot().expect("the projection validates");
    let node = |id: &str| snap.get(&sid(id)).expect(id);

    assert_eq!(node("changes").role, Role::List);
    assert_eq!(
        value(&snap, "changes:files"),
        Value::List(vec!["a.txt".into(), "b.txt".into(), "c.rs".into()])
    );
    assert_eq!(
        value(&snap, "changes:staged"),
        Value::Set(["a.txt", "c.rs"].map(String::from).into())
    );

    let file = node("changes.file[c.rs]");
    assert_eq!(file.role, Role::ListItem);
    assert_eq!(file.parent, Some(sid("changes")));
    assert_eq!(file.state.get("staged"), Some(&Value::Bool(true)));
    assert_eq!(file.state.get("unstaged"), Some(&Value::Bool(true)));
    let names: Vec<&str> = file.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(
        names,
        ["stage", "unstage", "discard", "ignore", "show_diff"]
    );
    for action in ["stage", "unstage", "discard", "ignore"] {
        assert_eq!(
            reads(&snap, "changes.file[c.rs]", action),
            ["changes.file[c.rs]:staged", "changes.file[c.rs]:unstaged"],
            "{action} reads the state it changes"
        );
    }
    let discard = &file.actions[2];
    assert!(discard.policy.destructive);
    assert_eq!(discard.policy.confirm, Confirm::ForAgents);
    assert_eq!(
        reads(&snap, "changes.file[b.txt]", "show_diff"),
        ["diff:target"]
    );
    let untracked = node("changes.file[b.txt]");
    assert_eq!(
        untracked.state.get("status"),
        Some(&Value::from("untracked"))
    );
    assert!(untracked.actions[0].enabled, "an untracked file can stage");
    assert!(!untracked.actions[1].enabled, "nothing of it to unstage");

    assert_eq!(node("diff").role, Role::Region);
    assert_eq!(value(&snap, "diff:target"), Value::from(""));

    assert_eq!(node("commit.form").role, Role::Group);
    assert_eq!(value(&snap, "commit.form:open"), Value::Bool(true));
    assert_eq!(
        reads(&snap, "commit.form", "cancel"),
        ["commit.form:open", "commit.message:text"]
    );
    let message = node("commit.message");
    assert_eq!(message.role, Role::TextInput);
    assert!(message.constraints.editable && message.constraints.required);
    assert_eq!(
        value(&snap, "commit.message:text"),
        Value::from("fix login")
    );
    assert_eq!(
        reads(&snap, "commit.message", "set_value"),
        ["commit.message:text"]
    );
    let submit = node("commit.submit");
    assert_eq!(submit.role, Role::Button);
    assert!(submit.enabled);
    assert_eq!(
        reads(&snap, "commit.submit", "submit"),
        ["changes:staged", "commit.message:text"]
    );

    let sha = "f00d".repeat(10);
    let commit = node(&format!("stacks.commit[{sha}]"));
    assert_eq!(commit.parent, Some(sid("stacks")));
    assert_eq!(commit.state.get("summary"), Some(&Value::from("initial")));
    assert_eq!(value(&snap, "stacks:branch"), Value::from("main"));
}

#[test]
fn submit_is_disabled_until_form_message_and_index_allow_it() {
    let mut bench = bench();
    let enabled = |b: &Workbench| {
        let snap = b.snapshot().expect("snapshot");
        snap.get(&sid("commit.submit")).expect("submit").enabled
    };
    assert!(enabled(&bench));

    bench.workspace_mut().commit_form.hide();
    assert!(!enabled(&bench), "a closed form cannot submit");
    bench.workspace_mut().commit_form.show();
    for ch in "  ".chars() {
        bench.workspace_mut().commit_form.append_char(ch);
    }
    assert!(!enabled(&bench), "a blank message cannot submit");
    for ch in "x".chars() {
        bench.workspace_mut().commit_form.append_char(ch);
    }
    assert!(enabled(&bench));
    for file in &mut bench.workspace_mut().unassigned.files {
        file.staged = false;
    }
    assert!(!enabled(&bench), "nothing staged, nothing to commit");
}

#[test]
fn a_moved_cursor_moves_the_revision_and_never_stales_a_commit() {
    let mut bench = bench();
    let seen = bench.snapshot().expect("snapshot");
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    let req = Request::observe(&seen, agent, "commit.submit", "submit", None).expect("observe");

    assert!(bench.workspace_mut().unassigned.select(Some(2)));
    let now = bench.snapshot().expect("snapshot");
    assert_eq!(value(&now, "changes:selected"), Value::from("c.rs"));
    assert_ne!(now.revision(), seen.revision(), "the cursor is projected");
    assert_eq!(check(&req, &now), Verdict::Accepted);

    bench.workspace_mut().unassigned.files[2].staged = false;
    bench.workspace_mut().unassigned.files.remove(3);
    let moved = bench.snapshot().expect("snapshot");
    assert!(matches!(check(&req, &moved), Verdict::Rejected(_)));
}

#[test]
fn the_render_binds_the_ids_the_projection_declares() {
    let mut bench = bench();
    let mut c = Compositor::new();
    c.begin_frame();
    bench.workspace_mut().render(&mut c);
    let snap = bench.snapshot().expect("snapshot");

    let bound: Vec<String> = c.bindings().map(|(id, _)| id.to_string()).collect();
    for id in [
        "changes",
        "changes.file[a.txt]",
        "changes.file[b.txt]",
        "changes.file[c.rs]",
        "diff",
        "commit.form",
        "commit.message",
        "commit.submit",
        "stacks",
    ] {
        assert!(bound.iter().any(|b| b == id), "{id} is bound: {bound:?}");
    }
    for id in &bound {
        assert!(snap.get(&sid(id)).is_some(), "{id} is bound, not projected");
    }
    // The two rows of `c.rs` bind once, on the first.
    let rows = bench.workspace().unassigned.hit_rects().to_vec();
    let row = c.binding(&sid("changes.file[c.rs]")).expect("bound").rect;
    assert_eq!(row[1], rows[2].1);

    // The Commit button binds where it is drawn, over the form.
    let submit = c.binding(&sid("commit.submit")).expect("bound").rect;
    let (x, y) = (submit[0] + submit[2] / 2.0, submit[1] + submit[3] / 2.0);
    assert_eq!(c.hit(x, y), Some(&sid("commit.submit")));
}
