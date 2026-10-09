//! The Surface page through a real agent port in a temporary directory,
//! with no window, in the pattern of the gallery's agent tests
//! (`view/dispatch/agent.rs`): this thread is the shell's (it drains the
//! port through the view and draws a frame after an act), a spawned
//! thread is the agent.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use accord::{ActionName, Actor, SemId, StateRef, Value};
use engine::drawing::compositor::Compositor;
use socket::{Act, AgentPort, Client, Config, Observe, ProtoError, Reply, WireError};

use crate::views::ShowcaseView;

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

/// A private directory for the test's socket, removed after.
struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn frame(view: &mut ShowcaseView) -> Compositor {
    let mut c = Compositor::new();
    view.render(&mut c);
    c
}

/// Runs `agent` against the gallery on the Surface page, serving the port
/// the way the shell does: the wake, `serve_agent`, a new frame after an
/// act.
fn gallery<T: Send + 'static>(
    dir: &Path,
    agent: impl FnOnce(PathBuf) -> T + Send + 'static,
) -> (T, ShowcaseView) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("surface");
    assert_eq!(view.page_title(), "Surface");
    let mut c = frame(&mut view);
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
    let deadline = Instant::now() + Duration::from_secs(60);
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the agent did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(2));
        if view.serve_agent(&mut port, &c) {
            c = frame(&mut view);
        }
    }
    (agent.join().unwrap(), view)
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
) -> Result<Vec<&'static str>, WireError> {
    let mut act = Act::from_revision(sid(target), ActionName::parse(action).unwrap(), r);
    if let Some(p) = param {
        act = act.with_param(p);
    }
    Ok(c.act(act, |_| {})?
        .into_iter()
        .map(|s| s.stage.name())
        .collect())
}

fn scoped(c: &mut Client, scope: &str) -> socket::SnapshotReply {
    snapshot(
        c.observe(Observe {
            scope: Some(sid(scope)),
            bounds: true,
            ..Observe::default()
        })
        .unwrap(),
    )
}

/// The acceptance through the port: the agent sees the surface (its
/// nodes, where they sit), writes a todo in the surface's field and adds
/// it from what it observed of the surface alone, and changes the
/// filter; each act is the todo's capability, by the agent, and the
/// fixed todo shows it. An act whose capability reads state the surface
/// does not show needs that state observed: from the surface's scope
/// alone the port refuses it, naming the read.
#[test]
fn an_agent_observes_the_surface_and_operates_the_todo_through_it() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t69-{}-surface", std::process::id())));
    let (seen, view) = gallery(&dir.0, |path| {
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();

        let surface = scoped(&mut c, "todo_panel");
        let typed = act(
            &mut c,
            "todo_panel.input",
            "set_value",
            surface.revision,
            Some(Value::Text("from the agent".into())),
        );
        // the add reads the field as the surface shows it: observed again
        // after its own set_value, it rests on the text it wrote
        let again = scoped(&mut c, "todo_panel");
        let added = act(&mut c, "todo_panel.add", "activate", again.revision, None);
        // the filter's state is the todo's, not shown on the surface
        let unseen = act(
            &mut c,
            "todo_panel.filter.active",
            "activate",
            again.revision,
            None,
        );
        let whole = snapshot(c.observe(Observe::default()).unwrap());
        let filtered = act(
            &mut c,
            "todo_panel.filter.active",
            "activate",
            whole.revision,
            None,
        );
        (surface, typed, added, unseen, filtered)
    });
    let (surface, typed, added, unseen, filtered) = seen;

    let ids: Vec<&str> = surface.nodes.iter().map(|n| n.id.as_str()).collect();
    for id in [
        "todo_panel",
        "todo_panel.input",
        "todo_panel.add",
        "todo_panel.active",
    ] {
        assert!(ids.contains(&id), "{id} in {ids:?}");
    }
    let bounds = surface.bounds.expect("bounds");
    assert!(
        bounds
            .iter()
            .any(|b| b.id == sid("todo_panel.add") && b.rect[2] > 0.0)
    );

    let done = ["requested", "accepted", "executing", "completed"];
    assert_eq!(typed.unwrap(), done);
    assert_eq!(added.unwrap(), done);
    let Err(WireError::Refused(ProtoError::NotObserved { reads })) = unseen else {
        panic!("not observed: {unseen:?}");
    };
    assert_eq!(reads, [StateRef::parse("todo.filter:selected").unwrap()]);
    assert_eq!(filtered.unwrap(), done);

    // the fixed todo shows what the agent did on the surface
    let now = view.semantics().expect("the page projects");
    let item = now.nodes().values().find(|n| n.label == "from the agent");
    assert!(item.is_some(), "the new todo is in the fixed todo");
    let selected = StateRef::parse("todo.filter:selected").unwrap();
    assert_eq!(now.value(&selected), Some(&Value::Text("active".into())));
    // and the todo's dispatcher kept each act as the agent's, on the
    // todo's capabilities
    let receipt = view
        .panes
        .todo
        .receipt(accord::RequestId::parse("2").unwrap());
    let receipt = receipt.expect("the add");
    assert_eq!(
        receipt.request.actor,
        Actor::Agent {
            name: "claude".into()
        }
    );
    assert_eq!(receipt.request.target, sid("todo.input"));
    assert_eq!(receipt.request.action.as_str(), "submit");
}

fn names(reply: &socket::SnapshotReply) -> Vec<String> {
    reply.nodes.iter().map(|n| n.id.to_string()).collect()
}

/// The acceptance of phase 3a through the port, no window: the agent
/// writes a todo's text in the surface's field, writes an intent in the
/// page's intent field, and proposes. The surface takes the proposed
/// form in place (the same id, only the active filter and count, the
/// entry), and the text the agent typed goes on in the field that stayed.
/// An intent the rules do not understand fails with the reason, which the
/// page's status shows. Every act is the agent's, in the todo's
/// dispatcher, and the proposal's author is the agent.
#[test]
fn an_agent_proposes_through_the_port_and_what_was_typed_goes_on() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t69c-{}-propose", std::process::id())));
    let (seen, view) = gallery(&dir.0, |path| {
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();

        let page = snapshot(c.observe(Observe::default()).unwrap());
        let typed = act(
            &mut c,
            "todo_panel.input",
            "set_value",
            page.revision,
            Some(Value::Text("milk".into())),
        );
        let page = snapshot(c.observe(Observe::default()).unwrap());
        let wrote = act(
            &mut c,
            "surface_page.intent",
            "set_value",
            page.revision,
            Some(Value::Text("only what is active and the add field".into())),
        );
        // the propose reads the intent: it rests on the text observed
        let page = snapshot(c.observe(Observe::default()).unwrap());
        let proposed = act(
            &mut c,
            "surface_page.propose",
            "propose",
            page.revision,
            None,
        );
        let after = snapshot(c.observe(Observe::default()).unwrap());

        let wrong = act(
            &mut c,
            "surface_page.intent",
            "set_value",
            after.revision,
            Some(Value::Text("make it purple".into())),
        );
        let page = snapshot(c.observe(Observe::default()).unwrap());
        let refused = act(
            &mut c,
            "surface_page.propose",
            "propose",
            page.revision,
            None,
        );
        let last = snapshot(c.observe(Observe::default()).unwrap());
        (page, typed, wrote, proposed, after, wrong, refused, last)
    });
    let (before, typed, wrote, proposed, after, wrong, refused, last) = seen;
    let done = ["requested", "accepted", "executing", "completed"];
    assert_eq!(typed.unwrap(), done);
    assert_eq!(wrote.unwrap(), done);
    assert_eq!(proposed.unwrap(), done);
    assert_eq!(wrong.unwrap(), done);
    assert_eq!(
        refused.unwrap(),
        ["requested", "accepted", "executing", "failed"]
    );

    // the page's own nodes, observed like any other
    let ids = names(&before);
    for id in [
        "surface_page",
        "surface_page.intent",
        "surface_page.propose",
    ] {
        assert!(ids.contains(&id.to_owned()), "{id} in {ids:?}");
    }
    // the proposed form, in place
    let ids = names(&after);
    for (id, kept) in [
        ("todo_panel", true),
        ("todo_panel.input", true),
        ("todo_panel.add", true),
        ("todo_panel.filter.active", true),
        ("todo_panel.active", true),
        ("todo_panel.filter.all", false),
        ("todo_panel.done", false),
    ] {
        assert_eq!(ids.contains(&id.to_owned()), kept, "{id} in {ids:?}");
    }
    let field = after
        .nodes
        .iter()
        .find(|n| n.id == sid("todo_panel.input"))
        .expect("the field");
    let key = accord::StateKey::parse("text").unwrap();
    assert_eq!(field.state.get(&key), Some(&Value::Text("milk".into())));
    // the refusal leaves the form and says why
    assert_eq!(names(&last), ids);
    let status = last
        .nodes
        .iter()
        .find(|n| n.id == sid("surface_page.status"))
        .expect("the status");
    let message = accord::StateKey::parse("message").unwrap();
    let Some(Value::Text(why)) = status.state.get(&message) else {
        panic!("a message: {status:?}");
    };
    assert!(why.contains("\"purple\" is not a word of the app"), "{why}");

    // the todo's dispatcher kept the proposal as the agent's
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let receipt = view
        .panes
        .todo
        .receipt(accord::RequestId::parse("3").unwrap());
    let receipt = receipt.expect("the proposal");
    assert_eq!(receipt.request.actor, agent);
    assert_eq!(receipt.request.target, sid("surface_page.propose"));
    assert_eq!(receipt.request.action.as_str(), "propose");
    let snap = view.semantics().expect("the page projects");
    assert_eq!(
        snap.value(&StateRef::parse("todo.input:text").unwrap()),
        Some(&Value::Text("milk".into()))
    );
}

/// A change through the port (task-70 phase 2): the agent writes "too
/// much" in the intent and proposes, the same two acts as a proposal; the
/// rules hide the counters (context: they bind no action) and tighten
/// the spacing, as the agent's change. "undo" takes it back the same
/// way. The protocol is the proposal's: no action is new.
#[test]
fn an_agent_asks_for_a_change_through_the_port_as_it_proposes() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t70b-{}-surface", std::process::id())));
    let (seen, view) = gallery(&dir.0, |path| {
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();
        let ask = |c: &mut Client, intent: &str| {
            let now = snapshot(c.observe(Observe::default()).unwrap());
            let wrote = act(
                c,
                "surface_page.intent",
                "set_value",
                now.revision,
                Some(Value::Text(intent.into())),
            );
            let now = snapshot(c.observe(Observe::default()).unwrap());
            let asked = act(c, "surface_page.propose", "propose", now.revision, None);
            let after = snapshot(c.observe(Observe::default()).unwrap());
            (wrote.unwrap(), asked.unwrap(), names(&after))
        };
        let less = ask(&mut c, "too much");
        (less, ask(&mut c, "undo"))
    });
    let ((wrote, asked, after_less), (_, undone, after_undo)) = seen;
    let done = ["requested", "accepted", "executing", "completed"];
    assert_eq!(wrote, done);
    assert_eq!(asked, done);
    assert_eq!(undone, done);
    assert!(!after_less.contains(&"todo_panel.active".to_owned()));
    assert!(
        after_less.contains(&"todo.counter".to_owned()),
        "the todo's own"
    );
    assert!(after_undo.contains(&"todo_panel.active".to_owned()));
    // the change was the agent's: its request, completed with the steps
    let receipt = view
        .panes
        .todo
        .receipt(accord::RequestId::parse("2").unwrap());
    let receipt = receipt.expect("the change");
    assert_eq!(
        receipt.request.actor,
        Actor::Agent {
            name: "claude".into()
        }
    );
    assert_eq!(receipt.request.target, sid("surface_page.propose"));
    let Some(accord::Stage::Completed(outcome)) = receipt.stage() else {
        panic!("{:?}", receipt.stage());
    };
    assert_eq!(outcome.result.get("steps"), Some(&Value::Int(2)));
    let seen = view.semantics().expect("the page projects");
    let status = seen
        .get(&sid("surface_page.status"))
        .expect("the status")
        .state
        .get("message")
        .cloned();
    assert_eq!(
        status,
        Some(Value::Text("undone: the last change, 2 steps".into()))
    );
}
