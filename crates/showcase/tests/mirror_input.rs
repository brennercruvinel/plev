//! The mirror answers (task-79 phase 2), in process: the gallery served
//! on this thread through a real agent port, as the shell serves it (the
//! port's drain, a new frame after an act, each frame offered to the
//! watches), and the viewer on a thread of its own, with its watch and
//! its hand (`showcase::app::mirror::Hand`), each a connection under the
//! name "mirror". A click at the viewer is found in the frame's bindings
//! (`Viewer::hit`) and goes back as an act on that control, resting on the
//! hand's last look: the same receipts, refusals and staleness as any
//! agent's act, and nothing at all for a point on no control.
//!
//! No GPU: the scene is built into a plain compositor. Desktop only,
//! like the port.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use engine::compositor::Compositor;
use engine::platform::capture::FakeCapture;
use rec::scene::Head;
use sem::{Actor, Rejection, Revision, SemId, Stage, StateRef, Value};
use showcase::app::mirror::{Hand, MAIN, Pressed, Scenes, Viewer};
use showcase::view::ShowcaseView;
use wire::{Act, AgentPort, Client, Config, Observe, Reply};

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 800.0;

/// The name the viewer says `hello` with, on both its connections.
const NAME: &str = "mirror";

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

fn mirror() -> Actor {
    Actor::Agent {
        name: NAME.to_owned(),
    }
}

/// Ticks until nothing moves (a caret at rest counts as settled).
fn settle(view: &mut ShowcaseView) {
    let mut dt = 0.0;
    for _ in 0..240 {
        if !view.tick(dt) {
            return;
        }
        dt = 1.0 / 60.0;
    }
}

/// The gallery on one page, its port open in a directory of its own, the
/// scene shared as `PLEV_MIRROR=1` shares it (or not).
struct Gallery {
    view: ShowcaseView,
    c: Compositor,
    port: AgentPort,
    woke: Receiver<()>,
    scenes: Scenes,
    _dir: tempfile::TempDir,
}

impl Gallery {
    fn open(page: &str, shared: bool) -> Gallery {
        let mut view = ShowcaseView::new(WIDTH, HEIGHT);
        // the camera is the fake, whatever a test asks of the page
        view.set_capture(Box::new(Arc::new(Mutex::new(FakeCapture::new()))));
        view.set_capture_shown(true);
        view.jump_to_page(page);
        view.resize(WIDTH, HEIGHT, 1.0);
        settle(&mut view);
        let mut c = Compositor::new();
        view.render(&mut c);
        let dir = tempfile::tempdir().expect("a temporary directory");
        let (tx, woke) = mpsc::channel();
        let config = Config {
            app: "showcase".into(),
            dir: dir.path().join("plev"),
            ring: 16,
        };
        let mut port = AgentPort::open_with(config, move || {
            let _ = tx.send(());
        })
        .expect("the port opens");
        port.share_scenes(shared);
        Gallery {
            view,
            c,
            port,
            woke,
            scenes: Scenes::default(),
            _dir: dir,
        }
    }

    fn path(&self) -> PathBuf {
        self.port.path().to_owned()
    }

    /// One turn of the shell: after the wake, the drain; a new frame when
    /// an act was dispatched; the frame on screen offered to the watches.
    fn turn(&mut self) {
        if self.woke.recv_timeout(Duration::from_millis(2)).is_err() {
            return;
        }
        if self.view.serve_agent(&mut self.port, &self.c) {
            settle(&mut self.view);
            self.view.render(&mut self.c);
        }
        let head = Head {
            width: WIDTH,
            height: HEIGHT,
            scale: 1.0,
            clear: self.view.theme.colors.bg.to_linear_array(),
        };
        self.scenes.offer(&mut self.port, MAIN, &self.c, head);
    }

    /// Serves the port while `client` runs on a thread of its own; what
    /// it returned.
    fn run<T: Send + 'static>(&mut self, client: impl FnOnce(PathBuf) -> T + Send + 'static) -> T {
        let path = self.path();
        let side = thread::spawn(move || client(path));
        let deadline = Instant::now() + Duration::from_secs(120);
        while !side.is_finished() {
            assert!(Instant::now() < deadline, "the client did not finish");
            self.turn();
        }
        match side.join() {
            Ok(out) => out,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    fn state(&self, at: &str) -> Option<Value> {
        let r: StateRef = at.parse().expect("a state ref");
        let now = self.view.gallery_semantics().expect("the gallery projects");
        now.get(&r.node)?.state.get(&r.key).cloned()
    }
}

/// The viewer's side: its watch and its hand, on the same name.
struct Eye {
    watch: Client,
    viewer: Viewer,
    hand: Hand,
}

impl Eye {
    fn open(path: &Path) -> Eye {
        let mut watch = Client::connect(path).expect("the watch connects");
        watch.set_timeout(Some(30_000)).expect("a timeout");
        watch.hello(NAME).expect("the viewer names itself");
        watch.watch_scene(MAIN).expect("the watch goes");
        let mut hand = Hand::connect(path, NAME).expect("the hand connects");
        hand.set_timeout(Some(30_000)).expect("a timeout");
        let mut eye = Eye {
            watch,
            viewer: Viewer::new(),
            hand,
        };
        eye.frame();
        eye
    }

    /// The next frame of the watch, taken; then the hand looks at the
    /// app, as the viewer does after each frame.
    fn frame(&mut self) -> u64 {
        loop {
            let msg = self.watch.recv().expect("a frame comes");
            if let Reply::Error(e) = &msg.reply {
                panic!("the watch was refused: {e}");
            }
            if let Some(seq) = self.viewer.take(msg.reply).expect("the frame reads") {
                self.hand.look().expect("the hand looks");
                return seq;
            }
        }
    }

    /// The center of `id` in the last frame.
    fn center(&self, id: &str) -> (f32, f32) {
        let b = self
            .viewer
            .bounds()
            .iter()
            .find(|b| b.id.as_str() == id)
            .unwrap_or_else(|| panic!("{id} is bound in the frame"));
        let [x, y, w, h] = b.rect;
        (x + w / 2.0, y + h / 2.0)
    }

    fn click(&mut self, id: &str) -> Pressed {
        let (x, y) = self.center(id);
        assert_eq!(
            self.viewer.hit(x, y),
            Some(&sid(id)),
            "{id} under its center"
        );
        self.hand.click(&self.viewer, x, y, |_, _| {})
    }
}

/// An agent on its own connection: observes the whole app, then acts
/// on `target` from that revision; the terminal stage.
fn agent_acts(path: &Path, name: &str, target: &str, action: &str) -> (Stage, Option<Revision>) {
    let mut c = Client::connect(path).expect("the agent connects");
    c.set_timeout(Some(30_000)).expect("a timeout");
    c.hello(name).expect("hello");
    let Reply::Snapshot(seen) = c.observe(Observe::default()).expect("an observation") else {
        panic!("a snapshot");
    };
    let act = Act::from_revision(
        sid(target),
        action.parse().expect("an action"),
        seen.revision,
    );
    let stages = c.act(act, |_| {}).expect("the act's stages");
    let last = stages.last().expect("a terminal stage");
    (last.stage.clone(), last.revision)
}

/// The acceptance of the input: a click on a Todo item at the mirror
/// toggles it through the app's dispatcher, as the agent "mirror", and
/// the app ends in the state (the same revision, the same projection) an
/// agent's act on that item leaves in a gallery of its own. The frame
/// that comes after it is the one the app drew.
#[test]
fn a_click_at_the_mirror_reaches_the_state_an_agent_act_reaches() {
    let mut clicked = Gallery::open("Todo", true);
    assert_eq!(
        clicked.state("todo.item[1]:completed"),
        Some(Value::Bool(false))
    );
    let (pressed, drawn) = clicked.run(|path| {
        let mut eye = Eye::open(&path);
        let pressed = eye.click("todo.item[1]");
        eye.frame();
        let mut c = Compositor::new();
        eye.viewer.draw(&mut c, false);
        (pressed, nodes(&c))
    });
    assert_eq!(nodes(&clicked.c), drawn, "the frame after the click");
    let Pressed::Acted {
        target,
        action,
        stages,
    } = &pressed
    else {
        panic!("the act went: {pressed:?}");
    };
    assert_eq!(
        (target.as_str(), action.as_str()),
        ("todo.item[1]", "toggle")
    );
    let asked = stages[0].asked.as_ref().expect("the request as dispatched");
    assert_eq!(asked.actor, mirror(), "an agent named mirror");
    let names: Vec<&str> = stages.iter().map(|s| s.stage.name()).collect();
    assert_eq!(names, ["requested", "accepted", "executing", "completed"]);
    assert_eq!(pressed.said(), "todo.item[1]: toggle done");
    assert_eq!(
        clicked.state("todo.item[1]:completed"),
        Some(Value::Bool(true))
    );

    // an agent's act on the same item, in a gallery of its own
    let mut acted = Gallery::open("Todo", true);
    let (stage, revision) = acted.run(|path| agent_acts(&path, "claude", "todo.item[1]", "toggle"));
    assert_eq!(stage.name(), "completed");
    assert_eq!(stages.last().and_then(|s| s.revision), revision);
    assert_eq!(
        clicked.view.gallery_semantics().ok(),
        acted.view.gallery_semantics().ok(),
        "the same state"
    );
}

/// The layers a compositor holds, as nodes: two frames are the same
/// scene when these are equal.
fn nodes(c: &Compositor) -> Vec<String> {
    c.layers()
        .iter()
        .filter(|l| !l.nodes().is_empty())
        .map(|l| format!("{} {:?}", l.z_order, l.nodes()))
        .collect()
}

/// The camera's switch is kept for a person (`PERSON_ONLY`): the
/// mirror's click on it is an agent's act, refused `not_authorized` by
/// the dispatcher before anything runs, and the viewer says so. The
/// camera stays off.
#[test]
fn a_click_on_an_action_kept_for_a_person_is_refused_and_said() {
    let mut g = Gallery::open("Capture", true);
    let pressed = g.run(|path| Eye::open(&path).click("capture.camera"));
    assert_eq!(
        pressed.stage(),
        Some(&Stage::Rejected(Rejection::NotAuthorized {
            actor: mirror()
        }))
    );
    assert_eq!(
        pressed.said(),
        "capture.camera: toggle refused: only a person asks it, and the mirror acts as \
         agent:mirror"
    );
    assert_eq!(g.state("capture.camera:on"), Some(Value::Bool(false)));
}

/// A point on no control sends nothing to the app; nor does a control
/// with nothing a click presses (the Todo region), nor one whose press
/// is off (the empty field's submit, which a screen reader does not
/// offer either). The app took no act.
#[test]
fn a_point_on_no_control_sends_nothing() {
    let mut g = Gallery::open("Todo", true);
    let said = g.run(|path| {
        let mut eye = Eye::open(&path);
        let empty = (0..HEIGHT as u32)
            .step_by(10)
            .flat_map(|y| {
                (0..WIDTH as u32)
                    .step_by(10)
                    .map(move |x| (x as f32, y as f32))
            })
            .find(|(x, y)| eye.viewer.hit(*x, *y).is_none())
            .expect("a point on no control");
        let nothing = eye.hand.click(&eye.viewer, empty.0, empty.1, |_, _| {});
        assert_eq!(nothing, Pressed::Nothing);
        let region = eye.hand.press(&sid("todo"), |_, _| {});
        let field = eye.click("todo.input");
        [nothing, region, field].map(|p| p.said())
    });
    assert_eq!(
        said,
        [
            "no control here: nothing went to the app",
            "todo has nothing a click presses: nothing went to the app",
            "todo.input: submit is off: nothing went to the app",
        ]
    );
    assert_eq!(g.port.traffic().acts, 0, "no act reached the app");
}

/// A scene the app moved on from: the viewer looked, another agent then
/// toggled the item, and the viewer, one look behind, clicks it. The act
/// rests on what the mirror was shown and comes back stale, with what
/// moved and who acted, as for any agent; nothing ran. After the next
/// frame and look, the same click goes through.
#[test]
fn an_old_scene_is_refused_as_stale() {
    let mut g = Gallery::open("Todo", true);
    let (stale, again) = g.run(|path| {
        let mut eye = Eye::open(&path);
        let (stage, _) = agent_acts(&path, "claude", "todo.item[1]", "toggle");
        assert_eq!(stage.name(), "completed");
        let stale = eye.click("todo.item[1]");
        eye.frame();
        let again = eye.click("todo.item[1]");
        (stale, again)
    });
    let Some(Stage::Rejected(Rejection::Stale { changes, acted })) = stale.stage() else {
        panic!("stale: {stale:?}");
    };
    assert_eq!(changes.len(), 1);
    let who: Vec<String> = acted.iter().map(|a| a.actor.to_string()).collect();
    assert_eq!(who, ["agent:claude"]);
    assert_eq!(
        stale.said(),
        "todo.item[1]: toggle refused: the mirror was behind the app \
         (todo.item[1]:completed moved, agent:claude acted since); look again"
    );
    assert_eq!(again.stage().map(Stage::name), Some("completed"));
    assert_eq!(
        g.state("todo.item[1]:completed"),
        Some(Value::Bool(false)),
        "claude's toggle, then the mirror's"
    );
}
