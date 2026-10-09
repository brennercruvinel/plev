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
//! Then the privacy promises the scene channel already kept, each tested
//! again against the click and the mask: the scene only when the person
//! shares it, a camera's frames never on the wire, a secret only as its
//! length (a live region's too), an action kept for a person refused, the
//! person's preference of accessibility never out by the port. (Who
//! watches, in the log and the recording: the shell's own tests.)
//!
//! No GPU: the scene is built into a plain compositor. Desktop only,
//! like the port.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

mod login;

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

    /// One turn of the shell: after the wake, the drain, and a new frame
    /// when an act was dispatched; what the person does at the window,
    /// and a new frame after it; each frame offered to the watches.
    fn turn(&mut self, person: &Receiver<(Job, mpsc::Sender<()>)>) {
        let woke = self.woke.recv_timeout(Duration::from_millis(2)).is_ok();
        if woke {
            if self.view.serve_agent(&mut self.port, &self.c) {
                settle(&mut self.view);
                self.view.render(&mut self.c);
            }
            self.offer();
        }
        while let Ok((job, done)) = person.try_recv() {
            job(&mut self.view);
            self.view.render(&mut self.c);
            self.offer();
            let _ = done.send(());
        }
    }

    fn offer(&mut self) {
        let head = Head {
            width: WIDTH,
            height: HEIGHT,
            scale: 1.0,
            clear: self.view.theme.colors.bg.to_linear_array(),
        };
        let view = &self.view;
        self.scenes.offer(&mut self.port, MAIN, &self.c, head, || {
            view.gallery_semantics().ok()
        });
    }

    /// Serves the port while `client` runs on a thread of its own, with
    /// a hand on the person at the window; what it returned.
    fn run<T: Send + 'static>(
        &mut self,
        client: impl FnOnce(PathBuf, Person) -> T + Send + 'static,
    ) -> T {
        let path = self.path();
        let (tx, person) = mpsc::channel();
        let side = thread::spawn(move || client(path, Person(tx)));
        let deadline = Instant::now() + Duration::from_secs(120);
        while !side.is_finished() {
            assert!(Instant::now() < deadline, "the client did not finish");
            self.turn(&person);
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

/// What the person at the window does, on the shell's thread.
type Job = Box<dyn FnOnce(&mut ShowcaseView) + Send>;

/// The person at the app's window, as the client's thread asks the
/// shell's: `does` returns once it was done and the frame it left
/// offered.
struct Person(mpsc::Sender<(Job, mpsc::Sender<()>)>);

impl Person {
    fn does(&self, job: impl FnOnce(&mut ShowcaseView) + Send + 'static) {
        let (done, wait) = mpsc::channel();
        self.0
            .send((Box::new(job), done))
            .expect("the shell listens");
        wait.recv().expect("the shell did it");
    }
}

/// The viewer's side: its watch and its hand, on the same name, and
/// everything the watch brought: each line as the app wrote it, each
/// binary part in them decoded.
struct Eye {
    watch: Client,
    viewer: Viewer,
    hand: Hand,
    lines: Vec<String>,
    parts: Vec<Vec<u8>>,
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
            lines: Vec::new(),
            parts: Vec::new(),
        };
        eye.frame();
        eye
    }

    /// The next frame of the watch, taken; then the hand looks at the
    /// app, as the viewer does after each frame.
    fn frame(&mut self) -> u64 {
        loop {
            let msg = self.watch.recv().expect("a frame comes");
            if let Some(seq) = self.take(msg) {
                self.hand.look().expect("the hand looks");
                return seq;
            }
        }
    }

    /// One line of the watch, kept and taken.
    fn take(&mut self, msg: wire::ServerMsg) -> Option<u64> {
        if let Reply::Error(e) = &msg.reply {
            panic!("the watch was refused: {e}");
        }
        self.lines.push(wire::encode_server(&msg));
        match &msg.reply {
            Reply::Scene(f) => {
                self.parts.extend(f.head.clone());
                for l in &f.layers {
                    self.parts.extend(l.body.clone());
                    self.parts.extend(l.patch.clone());
                }
            }
            Reply::SceneBlob(b) => self.parts.push(b.bytes.clone()),
            _ => {}
        }
        self.viewer.take(msg.reply).expect("the frame reads")
    }

    /// Every frame that comes within `ms`, taken, until none does.
    fn drain(&mut self, ms: u64) {
        self.watch.set_timeout(Some(ms)).expect("a timeout");
        while let Ok(msg) = self.watch.recv() {
            self.take(msg);
        }
        self.watch.set_timeout(Some(30_000)).expect("a timeout");
    }

    /// Whether `bytes` went out on the watch: in a binary part, or in a
    /// line as they are.
    fn carried(&self, bytes: &[u8]) -> bool {
        let text = String::from_utf8_lossy(bytes);
        self.parts
            .iter()
            .any(|p| p.windows(bytes.len()).any(|w| w == bytes))
            || self.lines.iter().any(|l| l.contains(text.as_ref()))
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
    let (pressed, drawn) = clicked.run(|path, _| {
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
    let (stage, revision) =
        acted.run(|path, _| agent_acts(&path, "claude", "todo.item[1]", "toggle"));
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
    let pressed = g.run(|path, _| Eye::open(&path).click("capture.camera"));
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
    let said = g.run(|path, _| {
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
    let (stale, again) = g.run(|path, _| {
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

// -- The promises the channel keeps, against the click and the mask --------
//
// The probe of phase 2, (d): each privacy promise that already held for
// the scene channel, tested again now that the mirror answers and masks.
// The one of an action kept for a person is
// `a_click_on_an_action_kept_for_a_person_is_refused_and_said` above;
// the recording's, in the shell (`app::mirror::tests`).

/// A person's request at the window, through the gallery's dispatcher;
/// it completes.
fn person_acts(view: &mut ShowcaseView, target: &str, action: &str, param: Option<&str>) {
    let seen = view.semantics().expect("the page projects");
    let param = param.map(|p| Value::Text(p.to_owned()));
    let req = sem::Request::observe(&seen, Actor::Human, target, action, param)
        .unwrap_or_else(|e| panic!("{target} {action}: {e}"));
    let receipt = view.act(req).expect("the page takes it");
    assert_eq!(
        receipt.stage().map(Stage::name),
        Some("completed"),
        "{target} {action}"
    );
    settle(view);
}

/// The scene is shown only when the person shares it (`PLEV_MIRROR=1`):
/// unshared, the watch is refused and says how to share, so the viewer
/// holds no binding and a click where a control is on the app's screen
/// finds nothing and sends nothing. Observing still works.
#[test]
fn unshared_the_mirror_has_nothing_to_click() {
    let mut g = Gallery::open("Todo", false);
    let [x, y, w, h] =
        g.c.binding(&sid("todo.item[1]"))
            .expect("the item on the app's screen")
            .rect;
    let (refusal, pressed) = g.run(move |path, _| {
        let mut watch = Client::connect(&path).expect("the watch connects");
        watch.set_timeout(Some(30_000)).expect("a timeout");
        watch.hello(NAME).expect("hello");
        let id = watch.watch_scene(MAIN).expect("the watch goes");
        let refusal = watch.recv_for(&id).expect("an answer");
        let mut hand = Hand::connect(&path, NAME).expect("the hand connects");
        hand.set_timeout(Some(30_000)).expect("a timeout");
        hand.look().expect("observing works without it");
        let viewer = Viewer::new();
        let pressed = hand.click(&viewer, x + w / 2.0, y + h / 2.0, |_, _| {});
        (refusal, pressed)
    });
    let Reply::Error(wire::ProtoError::SceneNotShared { reason }) = refusal else {
        panic!("refused as not shared: {refusal:?}");
    };
    assert!(reason.contains("PLEV_MIRROR=1"), "{reason}");
    assert_eq!(pressed, Pressed::Nothing);
    let t = g.port.traffic();
    assert_eq!((t.acts, t.observes_snapshot), (0, 1));
}

/// A camera's frames never cross: with the camera on and the frame
/// masked by the shell, a click on the camera's box presses nothing, a
/// click on its switch is refused, and none of the frames the camera
/// made while the mirror watched is in any byte of the watch.
#[test]
fn a_camera_s_frames_never_cross_whatever_the_mirror_clicks() {
    let mut g = Gallery::open("Capture", true);
    person_acts(&mut g.view, "capture.camera", "toggle", None);
    g.view.wake(1.0 / 60.0);
    g.view.render(&mut g.c);
    let (said, carried) = g.run(|path, person| {
        let mut eye = Eye::open(&path);
        let on_box = eye.click("capture.frame").said();
        let switch = eye.click("capture.camera").said();
        for _ in 0..4 {
            person.does(|view| {
                view.wake(1.0 / 60.0);
            });
        }
        eye.drain(300);
        // every color a frame of the fake takes (its frame number mod 200)
        let carried: Vec<u64> = (0..200)
            .filter(|n| eye.carried(&FakeCapture::pixel(*n).repeat(16)))
            .collect();
        ([on_box, switch], carried)
    });
    assert_eq!(
        said[0],
        "capture.frame has nothing a click presses: nothing went to the app"
    );
    assert!(
        said[1].contains("refused: only a person asks it"),
        "{}",
        said[1]
    );
    assert!(carried.is_empty(), "camera frames went out: {carried:?}");
    assert_eq!(g.state("capture.camera:on"), Some(Value::Bool(true)));
    assert_eq!(
        g.state("capture.frame:showing"),
        Some(Value::Bool(true)),
        "the app showed the camera's frames"
    );
    // and they were in the app's frame, a stream image with the fake's
    // pixels: what the watch never carried
    let live: Vec<Vec<u8>> =
        g.c.layers()
            .iter()
            .flat_map(|l| l.nodes())
            .filter_map(|n| match n {
                engine::compositor::SceneNode::Image { image, .. }
                    if engine::gpu::image_is_stream(image) =>
                {
                    engine::gpu::image_pixels(image)
                }
                _ => None,
            })
            .collect();
    assert!(
        live.iter()
            .any(|px| (0..200).any(|n| px.starts_with(&FakeCapture::pixel(n).repeat(16)))),
        "the camera's frame in the app's scene"
    );
}

/// The app of the login form, for the port: its projection, and nothing
/// that takes a request.
struct Login(sem::Snapshot);

impl wire::Host for Login {
    fn project(&self) -> Option<sem::Snapshot> {
        Some(self.0.clone())
    }

    fn dispatch(&mut self, _req: sem::Request) -> Option<sem::RequestId> {
        None
    }

    fn add_sink(&mut self, _sink: Box<dyn sem::ReceiptSink>) {}
}

/// `sem`'s redaction, which this phase extends to the scene: a password
/// field and a pin said by a status (a live region), drawn and marked
/// secret, reach the mirror as their length in the look its clicks rest
/// on, and as a box in the scene: neither is in any line of the watch or
/// of the look. A click on either presses nothing.
#[test]
fn a_secret_leaves_as_its_length_and_never_as_what_it_draws() {
    let secret = "hunter2-correct-horse";
    let mut c = Compositor::new();
    let now = login::form(&mut c, secret);
    let dir = tempfile::tempdir().expect("a temporary directory");
    let (tx, woke) = mpsc::channel();
    let config = Config {
        app: "login".into(),
        dir: dir.path().join("plev"),
        ring: 16,
    };
    let mut port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .expect("the port opens");
    port.share_scenes(true);
    let path = port.path().to_owned();
    let side = thread::spawn(move || {
        let mut eye = Eye::open(&path);
        let shown = eye.hand.shown().expect("a look").clone();
        let said = ["login.password", "login.pin"].map(|id| eye.click(id).said());
        eye.drain(200);
        let crossed = [secret, login::PIN, login::USER].map(|t| eye.carried(t.as_bytes()));
        (shown, said, crossed)
    });
    let (mut host, mut scenes) = (Login(now.clone()), Scenes::default());
    let head = Head {
        width: login::WIDTH,
        height: login::HEIGHT,
        scale: 1.0,
        clear: [0.0, 0.0, 0.0, 1.0],
    };
    let deadline = Instant::now() + Duration::from_secs(60);
    while !side.is_finished() {
        assert!(Instant::now() < deadline, "the viewer did not finish");
        if woke.recv_timeout(Duration::from_millis(2)).is_ok() {
            port.drain(&mut host);
            scenes.offer(&mut port, MAIN, &c, head, || Some(now.clone()));
        }
    }
    let (shown, said, crossed) = side.join().unwrap_or_else(|p| std::panic::resume_unwind(p));
    assert_eq!(
        crossed,
        [false, false, true],
        "the password, the pin, the user"
    );
    let value = |id: &str, key: &str| {
        let node = shown.nodes.iter().find(|n| n.id == sid(id)).expect(id);
        node.state.get(key).cloned()
    };
    let chars = |t: &str| Some(Value::Int(t.chars().count() as i64));
    assert_eq!(value("login.password", "text"), chars(secret));
    assert_eq!(value("login.pin", "message"), chars(login::PIN));
    let look = wire::encode_server(&wire::ServerMsg {
        id: None,
        reply: Reply::Snapshot(shown),
    });
    assert!(!look.contains(secret) && !look.contains(login::PIN));
    assert!(look.contains(login::USER));
    assert_eq!(
        said,
        [
            "login.password has nothing a click presses: nothing went to the app",
            "login.pin has nothing a click presses: nothing went to the app",
        ]
    );
}

/// The person's preference of accessibility never leaves by the port nor
/// reaches a model's prompt: on the Silences page, with the review open
/// and the band the person opened, the person turns larger text on. The
/// look the mirror's clicks rest on is the same, node for node and
/// revision, before and after (the page projects the same with and
/// without, and a proposer reads that projection); the mirror pressing
/// larger text is refused. What the scene shows of it, kept as it is:
/// the review drawn with the larger text, pixels the person shares.
#[test]
fn the_person_s_preference_never_leaves_by_the_port() {
    const LARGER: &str = "silences_surface.access.larger_text";
    let mut g = Gallery::open("Silences", true);
    g.view.stage_silences_frames(&mut |_ms| {
        engine::gpu::image::load_image_rgba(4, 4, vec![128; 64]).map_err(|e| e.to_string())
    });
    let rules = "the list with the video and the counts";
    person_acts(
        &mut g.view,
        "silences_surface.intent",
        "set_value",
        Some(rules),
    );
    person_acts(
        &mut g.view,
        "silences_surface.proposer",
        "select",
        Some("rules"),
    );
    person_acts(&mut g.view, "silences_surface.propose", "propose", None);
    person_acts(
        &mut g.view,
        "silences_surface.intent",
        "set_value",
        Some("accessibility"),
    );
    person_acts(&mut g.view, "silences_surface.propose", "propose", None);
    g.view.render(&mut g.c);
    let (before, after, drawn, pressed) = g.run(|path, person| {
        let mut eye = Eye::open(&path);
        let before = eye.hand.shown().cloned().expect("a look");
        let lines = eye.lines.len();
        person.does(|view| person_acts(view, LARGER, "activate", None));
        eye.frame();
        let drawn = eye.lines.len() > lines;
        let after = eye.hand.shown().cloned().expect("a look");
        let pressed = eye.click(LARGER);
        (before, after, drawn, pressed)
    });
    assert_eq!(before.revision, after.revision, "the same projection");
    assert_eq!(before.nodes, after.nodes);
    assert!(drawn, "the scene shows the larger text: pixels, shared");
    assert_eq!(
        pressed.stage(),
        Some(&Stage::Rejected(Rejection::NotAuthorized {
            actor: mirror()
        }))
    );
}
