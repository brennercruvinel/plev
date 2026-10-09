//! The Silences page through a real agent port in a temporary directory,
//! with no window, in the pattern of the gallery's agent tests
//! (`view/dispatch/agent.rs`): this thread is the shell's (it drains the
//! port through the view, ticks it and draws a frame after an act), a
//! spawned thread is the agent.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use engine::compositor::Compositor;
use sem::{ActionName, Actor, RequestId, SemId, StateRef, Value};
use showcase::model::silences::{PROBE_STDERR, parse};
use wire::{Act, AgentPort, Client, Config, Observe, Reply};

use super::super::worker::Worker;
use super::TURN;
use crate::view::ShowcaseView;

pub(super) fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

/// A private directory for the test's socket, removed after.
pub(super) struct Dir(pub PathBuf);

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

/// Runs `agent` against the gallery on the Silences page (the fake worker
/// answering the probe), serving the port the way the shell does: the
/// wake, `serve_agent`, a tick, a new frame after an act.
pub(super) fn gallery<T: Send + 'static>(
    dir: &Path,
    agent: impl FnOnce(PathBuf) -> T + Send + 'static,
) -> (T, ShowcaseView) {
    gallery_with(dir, |_| {}, agent)
}

/// `gallery`, with the view set up by `before` (as the person left it)
/// before the agent starts.
pub(super) fn gallery_with<T: Send + 'static>(
    dir: &Path,
    before: impl FnOnce(&mut ShowcaseView),
    agent: impl FnOnce(PathBuf) -> T + Send + 'static,
) -> (T, ShowcaseView) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("silences");
    assert_eq!(view.page_title(), "Silences");
    view.panes
        .silences
        .set_worker(Worker::fake(parse(PROBE_STDERR), false));
    before(&mut view);
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
        let acted = view.serve_agent(&mut port, &c);
        let moved = view.tick(TURN);
        if acted || moved {
            c = frame(&mut view);
        }
    }
    (agent.join().unwrap(), view)
}

pub(super) fn snapshot(reply: Reply) -> wire::SnapshotReply {
    match reply {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot: {other:?}"),
    }
}

pub(super) fn act(
    c: &mut Client,
    target: &str,
    action: &str,
    r: sem::Revision,
) -> Vec<&'static str> {
    let act = Act::from_revision(sid(target), ActionName::parse(action).unwrap(), r);
    c.act(act, |_| {})
        .unwrap()
        .into_iter()
        .map(|s| s.stage.name())
        .collect()
}

fn state<'a>(reply: &'a wire::SnapshotReply, id: &str, key: &str) -> Option<&'a Value> {
    reply
        .nodes
        .iter()
        .find(|n| n.id == sid(id))
        .and_then(|n| n.state.get(key))
}

/// The acceptance: the agent observes the silences (the cuts, where they
/// sit), applies a cut from what it saw, detects again (the request ends
/// later, when the worker answers, and the port hears it end), and the
/// page shows both.
#[test]
fn an_agent_observes_the_silences_and_applies_a_cut_through_the_port() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t69f-{}-silences", std::process::id())));
    let (seen, view) = gallery(&dir.0, |path| {
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();

        let scoped = snapshot(
            c.observe(Observe {
                scope: Some(sid("silences")),
                bounds: true,
                ..Observe::default()
            })
            .unwrap(),
        );
        let applied = act(&mut c, "silences.cut[1]", "apply_cut", scoped.revision);
        // the same decision again is disabled now: observed again, refused
        let again = snapshot(c.observe(Observe::default()).unwrap());
        let refused = c
            .act(
                Act::from_revision(
                    sid("silences.cut[1]"),
                    ActionName::parse("apply_cut").unwrap(),
                    again.revision,
                ),
                |_| {},
            )
            .map(|stages| {
                stages
                    .into_iter()
                    .map(|s| s.stage.name())
                    .collect::<Vec<_>>()
            });
        let detected = act(&mut c, "silences.detect", "detect_silence", again.revision);
        let after = snapshot(c.observe(Observe::default()).unwrap());
        (scoped, applied, refused, detected, after)
    });
    let (scoped, applied, refused, detected, after) = seen;

    let ids: Vec<&str> = scoped.nodes.iter().map(|n| n.id.as_str()).collect();
    for id in [
        "silences",
        "silences.video",
        "silences.threshold",
        "silences.detect",
        "silences.cut[1]",
        "silences.cut[2]",
        "silences.counts",
        "silences.apply_all",
    ] {
        assert!(ids.contains(&id), "{id} in {ids:?}");
    }
    let bounds = scoped.bounds.expect("bounds");
    assert!(
        bounds
            .iter()
            .any(|b| b.id == sid("silences.cut[1]") && b.rect[2] > 0.0)
    );

    let done = ["requested", "accepted", "executing", "completed"];
    assert_eq!(applied, done);
    match refused {
        Ok(stages) => assert_eq!(stages, ["requested", "rejected"]),
        Err(e) => panic!("the refusal is a receipt, not an error: {e:?}"),
    }
    // the detection started in the act and ended when the worker answered
    assert_eq!(detected, done);
    let ids: Vec<&str> = after.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(
        ids.contains(&"silences.cut[3]") && ids.contains(&"silences.cut[4]"),
        "{ids:?}"
    );
    assert!(matches!(
        state(&after, "silences.status", "busy"),
        Some(Value::Bool(_))
    ));

    // the frames that follow the detection end in the shell's ticks; then
    // the page shows what the agent did, and the gallery's dispatcher kept
    // the acts as the agent's
    let mut view = view;
    for _ in 0..20 {
        view.tick(TURN);
    }
    let now = view.semantics().expect("the page projects");
    assert_eq!(
        now.value(&StateRef::parse("silences.status:busy").unwrap()),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        now.value(&StateRef::parse("silences.counts:total").unwrap()),
        Some(&Value::Int(2))
    );
    let receipt = view
        .panes
        .todo
        .receipt(RequestId::parse("1").unwrap())
        .expect("the apply");
    assert_eq!(
        receipt.request.actor,
        Actor::Agent {
            name: "claude".into()
        }
    );
    assert_eq!(receipt.request.target, sid("silences.cut[1]"));
    assert_eq!(receipt.request.action.as_str(), "apply_cut");
}
