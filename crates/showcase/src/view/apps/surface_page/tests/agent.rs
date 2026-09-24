//! The Surface page through a real agent port in a temporary directory,
//! with no window, in the pattern of the gallery's agent tests
//! (`view/dispatch/agent.rs`): this thread is the shell's (it drains the
//! port through the view and draws a frame after an act), a spawned
//! thread is the agent.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use engine::compositor::Compositor;
use sem::{ActionName, Actor, SemId, StateRef, Value};
use wire::{Act, AgentPort, Client, Config, Observe, ProtoError, Reply, WireError};

use crate::view::ShowcaseView;

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

fn snapshot(reply: Reply) -> wire::SnapshotReply {
    match reply {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot: {other:?}"),
    }
}

fn act(
    c: &mut Client,
    target: &str,
    action: &str,
    r: sem::Revision,
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

fn scoped(c: &mut Client, scope: &str) -> wire::SnapshotReply {
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
    let receipt = view.panes.todo.receipt(sem::RequestId::parse("2").unwrap());
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
