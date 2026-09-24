//! The workbench without a repository: what it projects, what the render
//! binds, and which state a commit depends on. The flows against a real
//! repository live in `crates/ide/tests`.

use std::cell::RefCell;
use std::rc::Rc;

use engine::compositor::Compositor;
use sem::{
    Actor, Confirm, Mark, Receipt, ReceiptSink, Rejection, Request, Role, SemId, Snapshot, Stage,
    Value, Verdict, check,
};

use super::*;
use crate::views::multi_stack_view::{CommitEntry, Stack};
use crate::views::unassigned_view::{FileEntry, FileStatus};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("id")
}

/// Every receipt as it stood at its last stage, in the order they ended
/// or last moved.
#[derive(Clone, Default)]
struct Log(Rc<RefCell<Vec<Receipt>>>);

impl ReceiptSink for Log {
    fn on_stage(&mut self, receipt: &Receipt, _stage: &Stage) {
        let mut all = self.0.borrow_mut();
        all.retain(|r| r.id != receipt.id);
        all.push(receipt.clone());
    }
}

impl Log {
    fn last(&self) -> Receipt {
        self.0.borrow().last().cloned().expect("a receipt")
    }
}

fn tester() -> Actor {
    Actor::Test {
        name: "workbench".into(),
    }
}

/// An `Actor::Test` request over the workbench as it is now; its receipt.
fn ask(bench: &mut Workbench, target: &str, action: &str, param: Option<Value>) -> Receipt {
    let seen = bench.snapshot().expect("snapshot");
    let req = Request::observe(&seen, tester(), target, action, param).expect("observe");
    let id = bench.dispatch(req);
    bench.receipt(id).cloned().expect("receipt kept")
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
            files: vec!["README".into(), "src/main.rs".into()],
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
    assert_eq!(
        commit.state.get("files"),
        Some(&Value::List(vec!["README".into(), "src/main.rs".into()])),
        "the paths the commit changes, as a list in git's order"
    );
    assert_eq!(
        reads(&snap, &format!("stacks.commit[{sha}]"), "show_diff"),
        ["diff:target"]
    );
    assert_eq!(value(&snap, "stacks:branch"), Value::from("main"));
}

#[test]
fn only_the_newest_commits_of_the_checked_out_stack_carry_their_files() {
    let mut bench = bench();
    let commit = |n: usize| CommitEntry {
        sha: format!("{n:040x}"),
        message: format!("commit {n}"),
        author: "Test".into(),
        time_ago: "now".into(),
        files: vec![format!("f{n}")],
    };
    let total = FILES_PROJECTED + 2;
    bench.workspace_mut().stacks.set_stacks(vec![
        Stack {
            branch_name: "main".into(),
            is_active: true,
            commits: (0..total).map(commit).collect(),
        },
        Stack {
            branch_name: "feature/x".into(),
            is_active: false,
            commits: vec![commit(total)],
        },
    ]);
    let snap = bench.snapshot().expect("the projection validates");
    for n in 0..=total {
        let node = snap
            .get(&sid(&format!("stacks.commit[{n:040x}]")))
            .expect("every commit is projected");
        let files = node.state.get("files");
        if n < FILES_PROJECTED {
            assert_eq!(
                files,
                Some(&Value::List(vec![format!("f{n}")])),
                "commit {n}"
            );
        } else {
            assert_eq!(
                files, None,
                "commit {n} is past the newest {FILES_PROJECTED}"
            );
        }
        assert!(
            node.state.get("summary").is_some(),
            "commit {n} keeps its summary"
        );
    }
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

/// The center of `id` in the frame just drawn.
fn center(c: &Compositor, id: &str) -> (f32, f32) {
    let r = c.binding(&sid(id)).expect("bound").rect;
    (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0)
}

fn frame(bench: &mut Workbench) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    bench.render(&mut c);
    c
}

#[test]
fn keys_typing_and_test_requests_reach_the_same_form() {
    let empty = |bench: &mut Workbench| {
        bench
            .workspace_mut()
            .unassigned
            .set_files(vec![entry("a.txt", FileStatus::Added, true)]);
    };
    // A person: the `c` key, then typing into the field.
    let mut person = Workbench::new(1280.0, 800.0);
    empty(&mut person);
    assert!(person.toggle_commit_form());
    for ch in "fix login".chars() {
        person.workspace_mut().commit_form.append_char(ch);
    }
    let typed = person.snapshot().expect("snapshot");

    // A test: the same capabilities by request.
    let mut test = Workbench::new(1280.0, 800.0);
    empty(&mut test);
    let done = |r: Receipt| assert!(matches!(r.stage(), Some(Stage::Completed(_))), "{r:?}");
    done(ask(&mut test, "commit.form", "open", None));
    done(ask(
        &mut test,
        "commit.message",
        "set_value",
        Some("fix login".into()),
    ));
    let asked = test.snapshot().expect("snapshot");
    assert_eq!(asked, typed, "same form, same meaning");
    assert_eq!(asked.revision(), typed.revision());

    // Cancel clears the message it reads, by key and by request alike.
    assert!(person.cancel_commit_form());
    done(ask(&mut test, "commit.form", "cancel", None));
    assert!(!person.workspace().commit_form.visible);
    assert_eq!(person.workspace().commit_form.message(), "");
    assert_eq!(
        person.snapshot().expect("snapshot"),
        test.snapshot().expect("snapshot")
    );
}

#[test]
fn a_click_on_a_row_asks_its_diff_as_the_person() {
    let mut bench = bench();
    let log = Log::default();
    bench.add_sink(Box::new(log.clone()));
    let c = frame(&mut bench);

    let (x, y) = center(&c, "changes.file[b.txt]");
    assert!(bench.click(x, y));
    let receipt = log.last();
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(receipt.request.target, sid("changes.file[b.txt]"));
    assert_eq!(receipt.request.action.as_str(), "show_diff");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    let snap = bench.snapshot().expect("snapshot");
    assert_eq!(
        value(&snap, "diff:target"),
        Value::from("changes.file[b.txt]")
    );
    assert_eq!(value(&snap, "changes:selected"), Value::from("b.txt"));
}

#[test]
fn a_click_on_a_disabled_commit_is_refused_as_disabled() {
    let mut bench = bench();
    bench.workspace_mut().commit_form.take_message();
    let log = Log::default();
    bench.add_sink(Box::new(log.clone()));
    let c = frame(&mut bench);

    let (x, y) = center(&c, "commit.submit");
    assert!(bench.click(x, y));
    let receipt = log.last();
    assert_eq!(receipt.request.actor, Actor::Human);
    assert!(
        matches!(
            receipt.stage(),
            Some(Stage::Rejected(Rejection::Disabled { target, .. })) if *target == sid("commit.submit")
        ),
        "{receipt:?}"
    );
    assert_eq!(
        notices(&bench),
        [(
            Role::Status,
            "Refused: the commit message is empty".to_owned()
        )]
    );
}

/// The toasts on screen, as projected: role and message.
fn notices(bench: &Workbench) -> Vec<(Role, String)> {
    let snap = bench.snapshot().expect("snapshot");
    snap.children(&sid("notices"))
        .map(|n| (n.role, n.label.clone()))
        .collect()
}

/// Lets the toasts spring in (they are drawn and clickable once shown).
fn settle_toasts(bench: &mut Workbench) {
    for _ in 0..60 {
        bench.tick(1.0 / 60.0);
    }
}

#[test]
fn a_stale_agent_request_says_what_changed_and_a_click_dismisses_it() {
    let mut bench = bench();
    let seen = bench.snapshot().expect("snapshot");
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    let req = Request::observe(&seen, agent, "commit.submit", "submit", None).expect("observe");
    for ch in " flow".chars() {
        bench.workspace_mut().commit_form.append_char(ch);
    }
    let id = bench.dispatch(req);
    assert!(matches!(
        bench.receipt(id).and_then(Receipt::stage),
        Some(Stage::Rejected(Rejection::Stale { .. }))
    ));
    assert_eq!(
        notices(&bench),
        [(
            Role::Alert,
            "Refused test-agent's submit: the commit message changed".to_owned()
        )]
    );

    // The person clicks the toast away, through the dispatcher.
    settle_toasts(&mut bench);
    let log = Log::default();
    bench.add_sink(Box::new(log.clone()));
    let c = frame(&mut bench);
    let (x, y) = center(&c, "notices[0]");
    assert_eq!(c.hit(x, y), Some(&sid("notices[0]")));
    assert!(bench.click(x, y));
    let receipt = log.last();
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(receipt.request.action.as_str(), "dismiss");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert!(
        notices(&bench).is_empty(),
        "a leaving toast is not projected"
    );
    assert_eq!(
        bench.workspace().commit_form.message(),
        "fix login flow",
        "the click on the toast reached nothing under it"
    );
}

/// A person edits the message through the dispatcher on the state the
/// agent saw: the refusal of the agent's submit names who did what.
#[test]
fn a_stale_agent_request_names_the_person_who_acted() {
    let mut bench = bench();
    let seen = bench.snapshot().expect("snapshot");
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    let req = Request::observe(&seen, agent, "commit.submit", "submit", None).expect("observe");
    let edit = Request::observe(
        &seen,
        Actor::Human,
        "commit.message",
        "set_value",
        Some(Value::Text("fix login flow".into())),
    )
    .expect("observe");
    let edited = bench.dispatch(edit);
    assert!(matches!(
        bench.receipt(edited).and_then(Receipt::stage),
        Some(Stage::Completed(_))
    ));
    let id = bench.dispatch(req);
    let Some(Stage::Rejected(Rejection::Stale { acted, .. })) =
        bench.receipt(id).and_then(Receipt::stage)
    else {
        panic!("stale: {:?}", bench.receipt(id));
    };
    assert_eq!(acted.len(), 1);
    assert_eq!((acted[0].id, &acted[0].actor), (edited, &Actor::Human));
    assert_eq!(
        notices(&bench),
        [(
            Role::Alert,
            "Refused test-agent's submit: the commit message changed \
             (the person edited the commit message)"
                .to_owned()
        )]
    );
}

/// Several actions since the agent looked, by two actors: the toast
/// names them in the order they ran, one clause per actor's run, the
/// actions of a run in one series.
#[test]
fn a_stale_refusal_toast_names_each_actors_run_in_order() {
    let mut bench = bench();
    let seen = bench.snapshot().expect("snapshot");
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    let req = Request::observe(&seen, agent, "commit.submit", "submit", None).expect("observe");
    let edit = Request::observe(
        &seen,
        Actor::Human,
        "commit.message",
        "set_value",
        Some(Value::Text("fix login flow".into())),
    )
    .expect("observe");
    bench.dispatch(edit);
    let person = Actor::Human;
    let other = Actor::Agent {
        name: "reviewer".into(),
    };
    for (actor, file) in [(person, "b.txt"), (other, "a.txt")] {
        let now = bench.snapshot().expect("snapshot");
        let target = format!("changes.file[{file}]");
        let show =
            Request::observe(&now, actor, target.as_str(), "show_diff", None).expect("observe");
        let id = bench.dispatch(show);
        assert!(matches!(
            bench.receipt(id).and_then(Receipt::stage),
            Some(Stage::Completed(_))
        ));
    }
    let id = bench.dispatch(req);
    let Some(Stage::Rejected(Rejection::Stale { acted, .. })) =
        bench.receipt(id).and_then(Receipt::stage)
    else {
        panic!("stale: {:?}", bench.receipt(id));
    };
    assert_eq!(acted.len(), 3);
    assert_eq!(
        notices(&bench),
        [(
            Role::Alert,
            "Refused test-agent's submit: the commit message changed \
             (the person edited the commit message and opened the diff of b.txt; \
             reviewer opened the diff of a.txt)"
                .to_owned()
        )]
    );
}

/// Knows when `actor` looked: at the mark it was added at, as the agent
/// port tells the dispatcher for the requests it dispatches (task-91).
struct LookedWhenAdded {
    actor: Actor,
    at: Option<Mark>,
}

impl ReceiptSink for LookedWhenAdded {
    fn on_stage(&mut self, _receipt: &Receipt, _stage: &Stage) {}

    fn attached(&mut self, mark: Mark) {
        self.at = Some(mark);
    }

    fn looked(&self, receipt: &Receipt) -> Option<Mark> {
        self.at.filter(|_| receipt.request.actor == self.actor)
    }
}

/// With the look, the agent's own act after it is named with the others,
/// as the agent itself, never "you" (task-91): the agent looks, the
/// person opens a diff, the agent rewrites the message, and its submit on
/// the look is refused by its own write.
#[test]
fn a_stale_refusal_toast_names_the_agents_own_act_as_itself() {
    let mut bench = bench();
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    let seen = bench.snapshot().expect("snapshot");
    bench.add_sink(Box::new(LookedWhenAdded {
        actor: agent.clone(),
        at: None,
    }));
    let req =
        Request::observe(&seen, agent.clone(), "commit.submit", "submit", None).expect("observe");
    let show = Request::observe(
        &seen,
        Actor::Human,
        "changes.file[b.txt]",
        "show_diff",
        None,
    )
    .expect("observe");
    bench.dispatch(show);
    let now = bench.snapshot().expect("snapshot");
    let text = Some(Value::Text("fix login flow".into()));
    let write =
        Request::observe(&now, agent, "commit.message", "set_value", text).expect("observe");
    bench.dispatch(write);
    let id = bench.dispatch(req);
    let Some(Stage::Rejected(Rejection::Stale { acted, .. })) =
        bench.receipt(id).and_then(Receipt::stage)
    else {
        panic!("stale: {:?}", bench.receipt(id));
    };
    assert_eq!(acted.len(), 2);
    assert_eq!(
        notices(&bench),
        [(
            Role::Alert,
            "Refused test-agent's submit: the commit message changed \
             (the person opened the diff of b.txt; \
             test-agent itself edited the commit message)"
                .to_owned()
        )]
    );
}

#[test]
fn an_agent_discard_opens_the_modal_and_cancel_declines_it() {
    let mut bench = bench();
    let seen = bench.snapshot().expect("snapshot");
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    let req =
        Request::observe(&seen, agent, "changes.file[b.txt]", "discard", None).expect("observe");
    let id = bench.dispatch(req);
    assert!(bench.workspace().modal_open(), "a person is asked");

    let c = frame(&mut bench);
    // The modal holds the screen: no file menu under it.
    let (fx, fy) = center(&c, "changes.file[a.txt]");
    assert!(!bench.right_click(fx, fy));
    let (_, cancel) = bench.workspace().modal_buttons().expect("drawn");
    assert!(bench.click(cancel[0] + cancel[2] / 2.0, cancel[1] + cancel[3] / 2.0));

    assert!(!bench.workspace().modal_open());
    assert_eq!(
        notices(&bench),
        [(
            Role::Status,
            "Declined: discard of b.txt by test-agent did not run".to_owned()
        )]
    );
    assert_eq!(
        bench.receipt(id).and_then(Receipt::stage),
        Some(&Stage::Rejected(Rejection::Declined { by: Actor::Human }))
    );
    assert!(listed(&bench, "b.txt"), "nothing ran");
}

/// Whether `path` is still a row of the changes list.
fn listed(bench: &Workbench, path: &str) -> bool {
    bench
        .workspace()
        .unassigned
        .files
        .iter()
        .any(|f| f.path == path)
}

/// The agent asks to discard each file in turn; the ids, oldest first.
fn agent_discards(bench: &mut Workbench, files: &[&str]) -> Vec<RequestId> {
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    files
        .iter()
        .map(|file| {
            let seen = bench.snapshot().expect("snapshot");
            let target = format!("changes.file[{file}]");
            let req = Request::observe(&seen, agent.clone(), target.as_str(), "discard", None)
                .expect("observe");
            bench.dispatch(req)
        })
        .collect()
}

fn declined_by(bench: &Workbench, id: RequestId) -> Option<Actor> {
    match bench.receipt(id).and_then(Receipt::stage) {
        Some(Stage::Rejected(Rejection::Declined { by })) => Some(by.clone()),
        _ => None,
    }
}

#[test]
fn escape_and_a_click_outside_the_modal_decline_and_the_next_is_asked() {
    let mut bench = bench();
    let ids = agent_discards(&mut bench, &["a.txt", "b.txt"]);
    assert!(bench.workspace().modal_open());

    // Escape declines the first; the second is asked right away.
    assert!(bench.close_top_overlay());
    assert_eq!(declined_by(&bench, ids[0]), Some(Actor::Human));
    assert_eq!(
        bench.receipt(ids[1]).and_then(Receipt::stage),
        Some(&Stage::AwaitingConfirmation)
    );
    assert!(bench.workspace().modal_open(), "the second is asked now");

    // A click outside the dialog (clear of the toasts) declines it.
    let _ = frame(&mut bench);
    assert!(bench.click(4.0, 4.0));
    assert_eq!(declined_by(&bench, ids[1]), Some(Actor::Human));
    assert!(!bench.workspace().modal_open());

    let said: Vec<String> = notices(&bench).into_iter().map(|(_, m)| m).collect();
    assert_eq!(
        said,
        [
            "Declined: discard of a.txt by test-agent did not run",
            "Declined: discard of b.txt by test-agent did not run",
        ]
    );
    assert!(listed(&bench, "a.txt") && listed(&bench, "b.txt"));
}

#[test]
fn a_screen_reader_declines_through_the_workbench_and_a_test_cannot() {
    let mut bench = bench();
    let ids = agent_discards(&mut bench, &["b.txt"]);
    let id = ids[0];
    assert!(bench.workspace().modal_open());

    assert!(
        matches!(
            bench.decline(id, tester()),
            Err(SemError::ConfirmByMachine { .. })
        ),
        "a test is no person"
    );
    assert!(bench.workspace().modal_open(), "still asked");
    assert_eq!(
        bench.receipt(id).and_then(Receipt::stage),
        Some(&Stage::AwaitingConfirmation)
    );

    bench
        .decline(id, Actor::Assistive)
        .expect("a person declines through assistive technology");
    assert_eq!(declined_by(&bench, id), Some(Actor::Assistive));
    assert!(!bench.workspace().modal_open(), "its modal closed");
    assert_eq!(notices(&bench).len(), 1);
    assert!(listed(&bench, "b.txt"));
    assert!(matches!(
        bench.decline(id, Actor::Human),
        Err(SemError::IllegalTransition { .. })
    ));
}

#[test]
fn requests_waiting_for_a_person_are_asked_one_at_a_time() {
    let mut bench = bench();
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    let mut ids = Vec::new();
    for file in ["changes.file[a.txt]", "changes.file[b.txt]"] {
        let seen = bench.snapshot().expect("snapshot");
        let req = Request::observe(&seen, agent.clone(), file, "discard", None).expect("observe");
        ids.push(bench.dispatch(req));
    }
    assert!(bench.workspace().modal_open());

    // Confirmed through the api: its modal closes and the next one opens.
    bench
        .confirm(ids[0], Actor::Human)
        .expect("a person confirms");
    assert!(matches!(
        bench.receipt(ids[0]).and_then(Receipt::stage),
        Some(Stage::Failed { .. })
    ));
    assert!(bench.workspace().modal_open(), "the second is asked now");
    let _ = frame(&mut bench);
    let (confirm, _) = bench.workspace().modal_buttons().expect("drawn");
    assert!(bench.click(confirm[0] + 2.0, confirm[1] + 2.0));
    assert!(!bench.workspace().modal_open());
    let second = bench.receipt(ids[1]).expect("kept");
    assert!(
        matches!(&second.stages[2].0, Stage::Confirmed { by: Actor::Human }),
        "{second:?}"
    );
}

#[test]
fn without_a_repository_git_capabilities_fail_and_change_nothing() {
    let mut bench = bench();
    let before = bench.snapshot().expect("snapshot");
    let receipt = ask(&mut bench, "changes.file[b.txt]", "stage", None);
    assert!(
        matches!(receipt.stage(), Some(Stage::Failed { error }) if error == "no repository is open"),
        "{receipt:?}"
    );
    let receipt = ask(&mut bench, "commit.submit", "submit", None);
    assert!(matches!(receipt.stage(), Some(Stage::Failed { .. })));
    // Nothing moved but the two toasts saying so.
    let after = bench.snapshot().expect("snapshot");
    for (id, node) in before.nodes() {
        assert_eq!(after.get(id), Some(node), "{id}");
    }
    assert_eq!(
        notices(&bench),
        [
            (
                Role::Alert,
                "stage b.txt failed: no repository is open".to_owned()
            ),
            (
                Role::Alert,
                "submit failed: no repository is open".to_owned()
            ),
        ]
    );
}

#[test]
fn an_agent_discard_waits_and_a_test_cannot_confirm_it() {
    let mut bench = bench();
    let seen = bench.snapshot().expect("snapshot");
    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    let req =
        Request::observe(&seen, agent, "changes.file[b.txt]", "discard", None).expect("observe");
    let id = bench.dispatch(req);
    assert_eq!(
        bench.receipt(id).and_then(Receipt::stage),
        Some(&Stage::AwaitingConfirmation)
    );
    assert!(bench.confirm(id, tester()).is_err(), "a test is no person");
    assert!(
        bench
            .workspace()
            .unassigned
            .files
            .iter()
            .any(|f| f.path == "b.txt"),
        "nothing ran"
    );
}
