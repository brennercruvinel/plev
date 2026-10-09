//! The Claude asked from the Surface page (task-69 phase 3b-ii), against
//! the fake claude of `claude-model` (`tests/fake-claude`, a shell
//! script), never the real one: `propose` with `claude` chosen answers
//! `Started` and the request stays open while the page goes on taking
//! requests, keys and frames; the answer arrives through `wake`, the
//! request completes and the surface takes the form with what was typed
//! going on; a cancel kills the child and fails the request; without a
//! program the proposer offers the rules alone; and through the real
//! port an agent's act completes when the Claude answers, later.
//!
//! The one test that asks the real `claude` is ignored (a call of the
//! plan): `cargo test -p uigallery --lib surface_page::tests::claude::the_real
//! -- --ignored --nocapture`, with `SURFACE_PAGE_OUT=<dir>` for the png
//! and the projection.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use accord::{Actor, Rejection, Request, RequestId, Stage, StateRef, Value};
use claude::ClaudeCli;
use engine::drawing::compositor::Compositor;
use serde_json::json;
use socket::{Act, AgentPort, Client, Config, Observe, Reply};
use tempfile::TempDir;

use super::super::SurfacePage;
use super::super::asking::{Claude, POLL};
use super::super::offering::Said;
use super::{TURN, center, click, frame, setup, sid, text, value, wide};
use crate::views::ShowcaseView;
use crate::views::apps::todo::TodoPage;

const INTENT: &str = "the items as a list with the add field on top";

fn manifest(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> String {
    let path = manifest(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `reply` as `claude -p --output-format json` writes it.
fn result(reply: &str) -> String {
    json!({
        "type": "result", "subtype": "success", "is_error": false,
        "num_turns": 1, "total_cost_usd": 0.05, "result": reply
    })
    .to_string()
}

/// The fake in `mode`, answering the two-columns form of the todo's
/// surface (`good`; a refused answer first in `repair`), and a
/// `ClaudeCli` that runs it in a directory of its own.
fn fake(mode: &str) -> (TempDir, ClaudeCli) {
    let dir = tempfile::tempdir().unwrap();
    let columns = read("../../rust/screen/tests/fixtures/uigallery-todo-columns.ir.json");
    let wrong = read("../../rust/plevir/tests/fixtures/uigallery-todo.ir.json")
        .replace("todo.counter:active", "todo.counter:gone");
    for (name, text) in [
        ("mode", mode.to_owned()),
        ("good", result(&columns)),
        ("bad", result(&wrong)),
    ] {
        std::fs::write(dir.path().join(name), text).unwrap();
    }
    let mut cli = ClaudeCli::new();
    cli.program = manifest("../../rust/claude/tests/fakes/claude");
    cli.dir = dir.path().to_owned();
    (dir, cli)
}

fn left(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(name)).unwrap_or_default()
}

fn alive(pid: &str) -> bool {
    Command::new("kill")
        .args(["-0", pid])
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Waits for the fake to write its pid (`slow` mode).
fn child_pid(dir: &Path) -> String {
    let started = Instant::now();
    loop {
        let pid = left(dir, "pid").trim().to_owned();
        if !pid.is_empty() {
            return pid;
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the child started"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

/// A request on the page as it is now, by `actor`, through the page's
/// dispatcher.
fn dispatch(
    page: &mut SurfacePage,
    todo: &mut TodoPage,
    actor: Actor,
    target: &str,
    action: &str,
    param: Option<Value>,
) -> RequestId {
    let seen = page.snapshot(todo).expect("the page projects");
    let req = Request::observe(&seen, actor, &sid(target), action, param).expect("observed");
    page.page(todo).dispatch(req)
}

fn stage(todo: &TodoPage, id: RequestId) -> Stage {
    todo.receipt(id)
        .expect("kept")
        .stage()
        .expect("a stage")
        .clone()
}

fn type_in(page: &mut SurfacePage, todo: &mut TodoPage, s: &str) {
    for ch in s.chars() {
        assert!(page.handle_text(&ch.to_string(), todo), "{ch} typed");
    }
}

fn node_enabled(page: &SurfacePage, todo: &TodoPage, id: &str) -> bool {
    let snap = page.snapshot(todo).expect("projects");
    snap.get(&sid(id)).unwrap_or_else(|| panic!("{id}")).enabled
}

fn message(page: &SurfacePage, todo: &TodoPage) -> String {
    match value(page, todo, "surface_page.status:message") {
        Some(Value::Text(t)) => t,
        other => panic!("a message: {other:?}"),
    }
}

/// Wakes the page every `POLL` until `done` says so or `max` passes;
/// while it waits the wake asks for the next poll.
fn pump(
    page: &mut SurfacePage,
    todo: &mut TodoPage,
    max: Duration,
    done: impl Fn(&SurfacePage, &TodoPage) -> bool,
) {
    let started = Instant::now();
    while !done(page, todo) {
        assert!(started.elapsed() < max, "the wait ended in {max:?}");
        let wake = page.wake(TURN, todo);
        if page.proposing.waiting() {
            // the earliest deadline: the poll, or a caret's flip before it
            assert!(
                wake.next.is_some_and(|d| d <= POLL),
                "the page polls while it waits: {wake:?}"
            );
        }
        thread::sleep(POLL / 4);
    }
}

/// The proposer offers the rules and the Claude and chooses the Claude
/// when the program is there; `propose` then starts and the request
/// stays open: the status says it is asking (busy, the attempt, the
/// seconds), the propose button and its action are disabled (a second
/// propose is refused as disabled), the cancel is enabled, and the page
/// goes on: a toggle of the todo through it completes, a key in the
/// surface's field reaches the app, frames are drawn. A cancel by the
/// person kills the child and fails the request as cancelled; the
/// surface is as it was.
#[test]
fn asked_of_the_claude_the_request_starts_and_the_page_goes_on() {
    let (mut todo, mut page, theme) = setup();
    let (dir, mut cli) = fake("slow");
    cli.timeout = Duration::from_secs(20);
    page.proposing.set_claude(Claude::with(cli));
    let content = wide();

    let snap = page.snapshot(&todo).expect("projects");
    let proposer = snap
        .get(&sid("surface_page.proposer"))
        .expect("the proposer");
    assert_eq!(
        proposer.constraints.options,
        Some(vec!["rules".to_owned(), "claude".to_owned()])
    );
    assert_eq!(
        value(&page, &todo, "surface_page.proposer:selected"),
        text("claude")
    );
    assert!(!node_enabled(&page, &todo, "surface_page.cancel"));

    page.proposing.intent.set_text(INTENT);
    let asked = page.propose(&mut todo).expect("dispatched");
    assert_eq!(stage(&todo, asked), Stage::Executing);
    assert!(page.proposing.waiting());
    assert_eq!(
        value(&page, &todo, "surface_page.status:busy"),
        Some(Value::Bool(true))
    );
    assert!(
        message(&page, &todo).starts_with(&format!("asking the Claude for \"{INTENT}\": attempt")),
        "{}",
        message(&page, &todo)
    );
    assert!(!node_enabled(&page, &todo, "surface_page.propose"));
    assert!(node_enabled(&page, &todo, "surface_page.cancel"));

    // a second propose while it waits is refused: the action is disabled
    let again = dispatch(
        &mut page,
        &mut todo,
        Actor::Human,
        "surface_page.propose",
        "propose",
        None,
    );
    assert!(
        matches!(
            stage(&todo, again),
            Stage::Rejected(Rejection::Disabled { .. })
        ),
        "{:?}",
        stage(&todo, again)
    );

    // the app answers while the Claude thinks: a toggle through the page
    let toggled = dispatch(
        &mut page,
        &mut todo,
        Actor::Human,
        "todo.item[1]",
        "toggle",
        None,
    );
    assert!(matches!(stage(&todo, toggled), Stage::Completed(_)));
    assert_eq!(
        value(&page, &todo, "todo.item[1]:completed"),
        Some(Value::Bool(true))
    );
    // and a key in the surface's field
    let c = frame(&mut page, &mut todo, content, &theme);
    assert!(c.binding(&sid("surface_page.cancel")).is_some());
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "todo_panel.input"),
    );
    type_in(&mut page, &mut todo, "buy");
    assert_eq!(value(&page, &todo, "todo.input:text"), text("buy"));
    // the request is still open, the worker on its first attempt
    assert_eq!(stage(&todo, asked), Stage::Executing);
    let pid = child_pid(dir.path());
    assert!(alive(&pid), "the child {pid} runs");
    pump(
        &mut page,
        &mut todo,
        Duration::from_secs(5),
        |page, todo| value(page, todo, "surface_page.status:attempt") == Some(Value::Int(1)),
    );
    assert!(page.proposing.waiting());
    assert!(message(&page, &todo).contains("attempt 1 of 3"));

    // the person cancels
    let c = frame(&mut page, &mut todo, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "surface_page.cancel"),
    );
    assert!(!page.proposing.waiting());
    assert_eq!(
        stage(&todo, asked),
        Stage::Failed {
            error: "cancelled by human".to_owned()
        }
    );
    assert_eq!(
        page.proposing.said,
        Some(Said::Refused("cancelled by human".to_owned()))
    );
    assert_eq!(
        value(&page, &todo, "surface_page.status:busy"),
        Some(Value::Bool(false))
    );
    assert!(!node_enabled(&page, &todo, "surface_page.cancel"));
    assert!(node_enabled(&page, &todo, "surface_page.propose"));
    let started = Instant::now();
    while alive(&pid) {
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the child {pid} is killed"
        );
        thread::sleep(Duration::from_millis(10));
    }
    // the surface is as it was, the text typed in it too
    assert!(
        page.snapshot(&todo)
            .unwrap()
            .get(&sid("todo_panel.filter.all"))
            .is_some()
    );
    assert_eq!(value(&page, &todo, "todo.input:text"), text("buy"));
    // nothing is asked any more: a cancel is refused as disabled
    let idle = dispatch(
        &mut page,
        &mut todo,
        Actor::Human,
        "surface_page.cancel",
        "activate",
        None,
    );
    assert!(matches!(
        stage(&todo, idle),
        Stage::Rejected(Rejection::Disabled { .. })
    ));
}

/// The Claude answers (the fake: a refused first answer, repaired by
/// the second call): the answer arrives through `wake`, the request
/// completes with the surface, the attempts, the seconds and the cost,
/// the surface takes the two-columns form in place, and what was typed
/// in its field goes on in the same widget, the caret where it was. The
/// status says who proposed, in how long and for how much.
#[test]
fn the_claude_answers_and_the_surface_takes_the_form_with_what_was_typed() {
    let (mut todo, mut page, theme) = setup();
    let (dir, cli) = fake("repair");
    page.proposing.set_claude(Claude::with(cli));
    let content = wide();
    let c = frame(&mut page, &mut todo, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "todo_panel.input"),
    );
    type_in(&mut page, &mut todo, "buy");
    assert!(page.handle_edit_key(crate::views::EditKey::Left, &mut todo));
    assert!(
        page.snapshot(&todo)
            .unwrap()
            .get(&sid("todo_panel.counts"))
            .is_some()
    );

    page.proposing.intent.set_text(INTENT);
    let asked = page.propose(&mut todo).expect("dispatched");
    assert_eq!(stage(&todo, asked), Stage::Executing);
    pump(&mut page, &mut todo, Duration::from_secs(15), |page, _| {
        !page.proposing.waiting()
    });

    let Stage::Completed(outcome) = stage(&todo, asked) else {
        panic!("completed: {:?}", stage(&todo, asked));
    };
    assert_eq!(
        outcome.result.get("surface"),
        Some(&Value::Text("todo_panel".to_owned()))
    );
    assert_eq!(outcome.result.get("attempts"), Some(&Value::Int(2)));
    assert!(matches!(outcome.result.get("seconds"), Some(Value::Num(s)) if *s > 0.0));
    assert!(
        matches!(outcome.result.get("cost_usd"), Some(Value::Num(c)) if (*c - 0.1).abs() < 1e-9)
    );
    let Some(Said::Answered {
        intent,
        attempts,
        cost_usd,
        ..
    }) = page.proposing.said.clone()
    else {
        panic!("answered: {:?}", page.proposing.said);
    };
    assert_eq!(intent, INTENT);
    assert_eq!(attempts, 2);
    assert_eq!(cost_usd, Some(0.1));
    assert!(page.proposing.shown);
    let said = message(&page, &todo);
    assert!(
        said.starts_with(&format!("proposed by the Claude for \"{INTENT}\" in ")),
        "{said}"
    );
    assert!(said.ends_with("2 attempts, 0.100 USD"), "{said}");
    assert_eq!(
        value(&page, &todo, "surface_page.status:busy"),
        Some(Value::Bool(false))
    );
    assert_eq!(
        value(&page, &todo, "surface_page.status:attempt"),
        Some(Value::Int(2))
    );
    assert_eq!(
        value(&page, &todo, "surface_page.status:cost_usd"),
        Some(Value::Num(0.1))
    );
    // the two prompts the fake got: the intent, then the refusal
    assert!(left(dir.path(), "prompt.1").contains(INTENT));
    assert!(left(dir.path(), "prompt.2").contains("## attempt 1 was refused"));

    // the surface took the two-columns form in place
    let snap = page.snapshot(&todo).unwrap();
    assert!(snap.get(&sid("todo_panel.side")).is_some(), "two columns");
    assert!(snap.get(&sid("todo_panel.counts")).is_none());
    assert_eq!(page.id, Some(sid("todo_panel")));
    // what was typed goes on, in the same widget: the caret is one back
    assert_eq!(value(&page, &todo, "todo.input:text"), text("buy"));
    assert!(page.focus(&sid("todo_panel.input"), &mut todo, false));
    assert!(page.handle_text("X", &mut todo));
    assert_eq!(value(&page, &todo, "todo.input:text"), text("buXy"));
    // and the page draws the new form, the status with it
    let c = frame(&mut page, &mut todo, content, &theme);
    assert!(c.binding(&sid("todo_panel.side")).is_some());
    assert!(c.binding(&sid("surface_page.status")).is_some());
    // at rest, the wake asks for no poll
    let wake = page.wake(TURN, &mut todo);
    assert_ne!(wake.next, Some(POLL));
}

/// Without a program (the crate's tests never look for one) the proposer
/// offers the rules alone and chooses them: choosing the Claude is not
/// an option, the rules propose as before and the request ends at once,
/// and the cancel has nothing to cancel.
#[test]
fn without_the_program_the_proposer_offers_the_rules_alone() {
    let (mut todo, mut page, _) = setup();
    assert!(!page.proposing.claude.is_some());
    let snap = page.snapshot(&todo).expect("projects");
    let proposer = snap
        .get(&sid("surface_page.proposer"))
        .expect("the proposer");
    assert_eq!(proposer.role, accord::Role::Select);
    assert_eq!(proposer.constraints.options, Some(vec!["rules".to_owned()]));
    assert_eq!(
        value(&page, &todo, "surface_page.proposer:selected"),
        text("rules")
    );
    let chosen = dispatch(
        &mut page,
        &mut todo,
        Actor::Human,
        "surface_page.proposer",
        "select",
        Some(Value::Text("claude".into())),
    );
    assert!(
        matches!(
            stage(&todo, chosen),
            Stage::Rejected(Rejection::InvalidParam { .. })
        ),
        "{:?}",
        stage(&todo, chosen)
    );
    page.proposing
        .intent
        .set_text("only what is active and the add field");
    let asked = page.propose(&mut todo).expect("dispatched");
    assert!(matches!(stage(&todo, asked), Stage::Completed(_)));
    assert_eq!(
        page.proposing.said,
        Some(Said::Proposed(
            "only what is active and the add field".to_owned()
        ))
    );
    assert!(!page.proposing.waiting());
    assert_eq!(
        value(&page, &todo, "surface_page.status:busy"),
        Some(Value::Bool(false))
    );
    let cancel = dispatch(
        &mut page,
        &mut todo,
        Actor::Human,
        "surface_page.cancel",
        "activate",
        None,
    );
    assert!(matches!(
        stage(&todo, cancel),
        Stage::Rejected(Rejection::Disabled { .. })
    ));
}

/// A private directory for a test's socket, removed after.
struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn view_frame(view: &mut ShowcaseView) -> Compositor {
    let mut c = Compositor::new();
    view.render(&mut c);
    c
}

/// Runs `agent` against the gallery on the Surface page, the Claude of
/// `claude` set, serving the port the way the shell does and ticking the
/// view (the page's wake collects the Claude's answer), for at most
/// `patience`.
fn gallery<T: Send + 'static>(
    dir: &Path,
    claude: Claude,
    patience: Duration,
    agent: impl FnOnce(PathBuf) -> T + Send + 'static,
) -> (T, ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("surface");
    assert_eq!(view.page_title(), "Surface");
    view.panes.surface.proposing.set_claude(claude);
    let mut c = view_frame(&mut view);
    let (tx, woke): (Sender<()>, Receiver<()>) = mpsc::channel();
    let config = Config {
        app: "showcase".into(),
        dir: dir.to_owned(),
        ring: 16,
    };
    let mut port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .unwrap();
    let path = port.path().to_owned();
    let agent = thread::spawn(move || agent(path));
    let deadline = Instant::now() + patience;
    let mut last = Instant::now();
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the agent did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(2));
        let dt = last.elapsed().as_secs_f32();
        last = Instant::now();
        if view.tick(dt) {
            c = view_frame(&mut view);
        }
        if view.serve_agent(&mut port, &c) {
            c = view_frame(&mut view);
        }
    }
    (agent.join().unwrap(), view, c)
}

fn snapshot(reply: Reply) -> socket::SnapshotReply {
    match reply {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot: {other:?}"),
    }
}

fn act(
    c: &mut Client,
    target: &str,
    action: &str,
    r: accord::Revision,
    param: Option<Value>,
) -> Vec<&'static str> {
    let mut act = Act::from_revision(sid(target), accord::ActionName::parse(action).unwrap(), r);
    if let Some(p) = param {
        act = act.with_param(p);
    }
    c.act(act, |_| {})
        .unwrap_or_else(|e| panic!("{target} {action}: {e}"))
        .into_iter()
        .map(|s| s.stage.name())
        .collect()
}

/// What an agent does through the port: chooses the Claude on the
/// proposer, writes `intent`, proposes, and reads the page after. The
/// act of the proposal returns when the request ends, with every stage.
fn agent_proposes(
    path: PathBuf,
    intent: &str,
    timeout_ms: u64,
) -> (Vec<&'static str>, socket::SnapshotReply) {
    let mut c = Client::connect(&path).unwrap();
    c.set_timeout(Some(timeout_ms)).unwrap();
    c.hello("claude").unwrap();
    let page = snapshot(c.observe(Observe::default()).unwrap());
    let chose = act(
        &mut c,
        "surface_page.proposer",
        "select",
        page.revision,
        Some(Value::Text("claude".into())),
    );
    assert_eq!(chose, ["requested", "accepted", "executing", "completed"]);
    let page = snapshot(c.observe(Observe::default()).unwrap());
    let wrote = act(
        &mut c,
        "surface_page.intent",
        "set_value",
        page.revision,
        Some(Value::Text(intent.into())),
    );
    assert_eq!(wrote, ["requested", "accepted", "executing", "completed"]);
    let page = snapshot(c.observe(Observe::default()).unwrap());
    let proposed = act(
        &mut c,
        "surface_page.propose",
        "propose",
        page.revision,
        None,
    );
    let after = snapshot(c.observe(Observe::default()).unwrap());
    (proposed, after)
}

fn state_of(reply: &socket::SnapshotReply, id: &str, key: &str) -> Option<Value> {
    let node = reply.nodes.iter().find(|n| n.id == sid(id))?;
    node.state
        .get(&accord::StateKey::parse(key).unwrap())
        .cloned()
}

/// Through the real port, no window: an agent chooses the Claude (the
/// fake, answering at once) on the proposer, writes the intent and
/// proposes; its act returns when the request completes, later, through
/// the page's wake; the surface has the proposed form, the status says
/// who proposed, and the receipt is the agent's in the todo's
/// dispatcher, completed with the surface and the cost.
#[test]
fn an_agent_asks_the_claude_through_the_port_and_the_act_completes_later() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t69e-{}-claude", std::process::id())));
    let (_fake, cli) = fake("ok");
    // the fake starts with the rules chosen: the agent chooses the Claude
    let (seen, view, _) = gallery(&dir.0, Claude::with(cli), Duration::from_secs(60), |path| {
        agent_proposes(path, "the two columns", 20_000)
    });
    let (proposed, after) = seen;
    assert_eq!(
        proposed,
        ["requested", "accepted", "executing", "completed"]
    );
    let ids: Vec<String> = after.nodes.iter().map(|n| n.id.to_string()).collect();
    assert!(ids.contains(&"todo_panel.side".to_owned()), "{ids:?}");
    assert!(!ids.contains(&"todo_panel.counts".to_owned()), "{ids:?}");
    let Some(Value::Text(said)) = state_of(&after, "surface_page.status", "message") else {
        panic!("a message");
    };
    assert!(
        said.starts_with("proposed by the Claude for \"the two columns\" in "),
        "{said}"
    );
    assert_eq!(
        state_of(&after, "surface_page.status", "busy"),
        Some(Value::Bool(false))
    );
    assert_eq!(
        state_of(&after, "surface_page.status", "cost_usd"),
        Some(Value::Num(0.05))
    );
    // the receipt: the agent's proposal, completed later, in the todo's dispatcher
    let receipt = view.panes.todo.receipt(RequestId::parse("3").unwrap());
    let receipt = receipt.expect("the proposal");
    assert_eq!(
        receipt.request.actor,
        Actor::Agent {
            name: "claude".into()
        }
    );
    assert_eq!(receipt.request.target, sid("surface_page.propose"));
    let Some(Stage::Completed(outcome)) = receipt.stage() else {
        panic!("completed: {:?}", receipt.stages);
    };
    assert_eq!(
        outcome.result.get("surface"),
        Some(&Value::Text("todo_panel".to_owned()))
    );
    assert_eq!(outcome.result.get("attempts"), Some(&Value::Int(1)));
    let names: Vec<&str> = receipt.stages.iter().map(|(s, _)| s.name()).collect();
    assert_eq!(names, ["requested", "accepted", "executing", "completed"]);
    let (_, executing) = receipt.stages[2];
    let (_, completed) = receipt.stages[3];
    assert!(completed >= executing);
}

/// The proof with the real `claude` (one call of the plan, ignored):
/// an agent through the port asks for "the items as a list with the add
/// field on top", the act completes when the Claude answers, the surface
/// is born in place, and the page is drawn headless after: with
/// `SURFACE_PAGE_OUT=<dir>`, `surface-claude.png` and `surface-claude.json`.
#[test]
#[ignore = "asks the real claude: a call of the plan, seconds to minutes"]
fn the_real_claude_proposes_through_the_port() {
    use engine::drawing::gpu::RenderConfig;
    use engine::drawing::headless::{Renderer, Viewport};

    let cli = ClaudeCli::new();
    assert!(
        cli.exists(),
        "no claude on this machine ({})",
        cli.program.display()
    );
    let dir = Dir(std::env::temp_dir().join(format!("plev-t69e-{}-real", std::process::id())));
    let started = Instant::now();
    let (seen, mut view, mut c) = gallery(
        &dir.0,
        Claude::with(cli),
        Duration::from_secs(600),
        |path| agent_proposes(path, INTENT, 590_000),
    );
    let took = started.elapsed();
    let (proposed, after) = seen;
    println!("the act's stages: {proposed:?}, in {took:?}");
    let said = state_of(&after, "surface_page.status", "message");
    println!("the status: {said:?}");
    println!(
        "attempt {:?}, seconds {:?}, cost {:?}",
        state_of(&after, "surface_page.status", "attempt"),
        state_of(&after, "surface_page.status", "seconds"),
        state_of(&after, "surface_page.status", "cost_usd")
    );
    let receipt = view.panes.todo.receipt(RequestId::parse("3").unwrap());
    println!("the receipt: {:?}", receipt.map(|r| &r.stages));
    let ids: Vec<String> = after.nodes.iter().map(|n| n.id.to_string()).collect();
    println!("the page after: {ids:#?}");
    assert_eq!(
        proposed,
        ["requested", "accepted", "executing", "completed"]
    );
    assert!(
        ids.iter().any(|id| id.starts_with("todo_panel.item[")),
        "{ids:?}"
    );

    // settled, drawn, and with a GPU its pixels
    let mut steps = 0;
    while view.tick(TURN) {
        view.render(&mut c);
        steps += 1;
        assert!(steps < 600, "the page never settles");
    }
    view.render(&mut c);
    let snap = view.semantics().expect("the page projects");
    let viewport = Viewport {
        width: 1200.0,
        height: 800.0,
        scale: 1.0,
    };
    let Ok(mut r) = Renderer::new(viewport, RenderConfig::default()) else {
        eprintln!("no GPU adapter: the scene and the projection only");
        return;
    };
    let clear = view.theme.colors.bg.to_linear_array();
    let frame = r.render(&mut c, clear).expect("the frame draws");
    if let Some(out) = std::env::var_os("SURFACE_PAGE_OUT") {
        let path = PathBuf::from(out).join("surface-claude");
        frame
            .write_png(path.with_extension("png"))
            .expect("the png writes");
        let json = crate::shell::semantics_json(&snap, false).expect("the json");
        std::fs::write(path.with_extension("json"), json).expect("the json writes");
        println!("wrote {}.png and .json", path.display());
    }
    let _ = StateRef::parse("todo:items");
}
