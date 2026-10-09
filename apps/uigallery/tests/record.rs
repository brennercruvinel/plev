//! The gallery's session recording, through the path its shell takes
//! (`showcase::app::record::Recording`, `ShowcaseView::serve_agent_recorded`),
//! with no window: the Silences page as it opens (seeded; nothing here
//! runs ffmpeg), a real agent port in a temporary directory. This thread
//! plays the shell as `app.rs` does with `PLEV_RECORD=1`: it serves the
//! port after its wake, ticks the page, and records every frame it
//! draws and the person's click. A spawned thread is the agent, speaking
//! the protocol; the person applies every pending cut between the
//! agent's two decisions.
//!
//! What it holds the recording to: the saved file opens, and holds the
//! agent's observations (with their scope and revision), every receipt
//! with its actor, the person's click with the control under it, the
//! frames, and where each control showed in them, the same rect the agent
//! was served; and the inspector reads from it why the agent's second
//! decision was refused (the person's click moved what it read). Without
//! the variable nothing is created. The file is kept at
//! `$CARGO_TARGET_TMPDIR/showcase-record-test.monster`, for
//! `plev-inspect`.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use engine::widgets::prelude::WidgetEvent;
use engine::drawing::compositor::Compositor;
use replay::{Basis, By, Ending, Inspector};
use record::{InputKind, Kind, Limits, Session};
use accord::{ActionName, Actor, Rejection, Revision, SemId, Stage, Value};
use uigallery::shell::window::record::{ENV, Recording};
use uigallery::views::ShowcaseView;
use socket::{Act, AgentPort, Client, Config, Observe, Reply, SnapshotReply, StageReply};

/// One turn of the shell's clock, a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

fn agent() -> Actor {
    Actor::Agent {
        name: "test-agent".into(),
    }
}

/// What the shell's thread does between two of the agent's messages.
enum Poke {
    /// A person clicks the middle of where `id` shows in the frame on
    /// screen: the press and the release recorded with the control under
    /// them.
    Click(&'static str),
}

type Pokes = Sender<(Poke, Sender<()>)>;

fn poke(pokes: &Pokes, what: Poke) {
    let (done, wait) = mpsc::channel();
    pokes.send((what, done)).expect("the shell listens");
    wait.recv().expect("the shell answers");
}

/// The recording shell around the gallery.
struct Shell<'a> {
    view: ShowcaseView,
    port: AgentPort,
    frame: Compositor,
    recording: &'a Recording,
}

impl Shell<'_> {
    /// A frame drawn, and recorded with the state it was drawn from.
    fn draw(&mut self) {
        self.view.render(&mut self.frame);
        self.recording.frame(&mut self.view, &self.frame);
    }

    /// The port served (an act records the state it left), the page
    /// ticked, a frame when either changed something.
    fn turn(&mut self) {
        let acted = self.view.serve_agent_recorded(
            &mut self.port,
            &self.frame,
            Some(self.recording.recorder()),
        );
        let moved = self.view.tick(TURN);
        if acted || moved {
            self.draw();
        }
    }

    fn poked(&mut self, what: Poke) {
        match what {
            Poke::Click(id) => {
                let [x, y, w, h] = self
                    .frame
                    .binding(&sid(id))
                    .and_then(|b| b.visible_rect())
                    .expect("the control shows");
                let (x, y) = (x + w / 2.0, y + h / 2.0);
                self.view.handle_event(&WidgetEvent::MouseMove { x, y });
                for (kind, event) in [
                    (InputKind::PointerDown, WidgetEvent::MouseDown { x, y }),
                    (InputKind::PointerUp, WidgetEvent::MouseUp { x, y }),
                ] {
                    self.recording
                        .input(&mut self.view, kind, Some([x, y]), &self.frame);
                    if self.view.handle_event(&event) {
                        self.recording.state(&mut self.view);
                    }
                }
                self.draw();
            }
        }
    }
}

/// Runs `agent` against `view` through a port in a temp dir, this thread
/// playing the recording shell.
fn shell<T: Send + 'static>(
    view: ShowcaseView,
    recording: &Recording,
    agent: impl FnOnce(PathBuf, Pokes) -> T + Send + 'static,
) -> (T, ShowcaseView) {
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let (tx, woke): (Sender<()>, Receiver<()>) = mpsc::channel();
    let config = Config {
        app: "showcase".into(),
        dir: dir.path().to_owned(),
        ring: 16,
    };
    let port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .expect("the port opens");
    let path = port.path().to_owned();
    let mut shell = Shell {
        view,
        port,
        frame: Compositor::new(),
        recording,
    };
    shell.draw();
    let (pokes, poked) = mpsc::channel::<(Poke, Sender<()>)>();
    let agent = thread::spawn(move || agent(path, pokes));
    let deadline = Instant::now() + Duration::from_secs(60);
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the agent did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(2));
        shell.turn();
        while let Ok((what, done)) = poked.try_recv() {
            shell.poked(what);
            let _ = done.send(());
        }
    }
    (agent.join().expect("the agent ran"), shell.view)
}

fn client(path: &Path) -> Client {
    let mut c = Client::connect(path).expect("the agent connects");
    c.set_timeout(Some(20_000)).expect("a timeout");
    c
}

fn snapshot(reply: Reply) -> SnapshotReply {
    match reply {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot: {other:?}"),
    }
}

fn act(c: &mut Client, target: &str, action: &str, at: Revision) -> Vec<StageReply> {
    let name = ActionName::parse(action).expect("an action");
    c.act(Act::from_revision(sid(target), name, at), |_| {})
        .expect("the act's stages")
}

/// What the agent saw and got, in order.
struct Story {
    seen: SnapshotReply,
    applied: Vec<StageReply>,
    stale: Vec<StageReply>,
    counts: SnapshotReply,
}

#[test]
fn the_recorded_session_holds_the_agent_the_person_the_frames_and_where_each_control_showed() {
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let mut recording = Recording::new(dir.path().join("sessions"), Limits::default());
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("silences");
    assert_eq!(view.page_title(), "Silences");
    view.resize(1200.0, 800.0, 1.0);
    recording.attach(&mut view);

    let (story, view) = shell(view, &recording, |path, pokes| {
        let mut c = client(&path);
        c.hello("test-agent").expect("welcome");
        let seen = snapshot(
            c.observe(Observe {
                bounds: true,
                ..Observe::default()
            })
            .expect("an observation"),
        );
        let applied = act(&mut c, "silences.cut[1]", "apply_cut", seen.revision);
        poke(&pokes, Poke::Click("silences.apply_all"));
        // the second cut was pending when the agent looked; the person
        // applied it since
        let stale = act(&mut c, "silences.cut[2]", "reject_cut", seen.revision);
        let counts = snapshot(
            c.observe(Observe {
                scope: Some(sid("silences.counts")),
                ..Observe::default()
            })
            .expect("the counts"),
        );
        Story {
            seen,
            applied,
            stale,
            counts,
        }
    });
    assert!(matches!(
        story.applied.last().map(|s| &s.stage),
        Some(Stage::Completed(_))
    ));
    let Some(Stage::Rejected(Rejection::Stale { .. })) = story.stale.last().map(|s| &s.stage)
    else {
        panic!("refused as stale: {:?}", story.stale);
    };
    let now = view.semantics().expect("the page projects");
    assert_eq!(
        now.value(&"silences.counts:applied".parse().unwrap()),
        Some(&Value::Int(2)),
        "the agent applied one, the person the other"
    );

    // saved as the shell saves it, and read back as plev-inspect reads it.
    let (path, written) = recording.save().expect("the window is saved");
    assert_eq!(
        path,
        dir.path()
            .join("sessions")
            .join(format!("showcase-{}-0.monster", std::process::id()))
    );
    assert!(written.bindings_bytes > 0, "{written:?}");
    let session = Session::open(&path).expect("the session opens");
    let kept = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("showcase-record-test.monster");
    std::fs::copy(&path, &kept).expect("kept for plev-inspect");
    let (_, end) = session.span().expect("events");

    // who observed what: the agent, at each revision it was served, with
    // the scope it asked for.
    let observed: Vec<(Option<SemId>, Revision)> = session
        .events()
        .iter()
        .filter_map(|e| match &e.kind {
            Kind::Observed {
                actor,
                scope,
                revision,
            } if *actor == agent() => Some((scope.clone(), *revision)),
            _ => None,
        })
        .collect();
    for (scope, revision) in [
        (None, story.seen.revision),
        (Some(sid("silences.counts")), story.counts.revision),
    ] {
        assert!(
            observed.contains(&(scope.clone(), revision)),
            "{scope:?} at {revision}: {observed:?}"
        );
    }

    // every receipt: the agent's two decisions and the person's click.
    let receipts = session.receipts(end);
    let by = |actor: &Actor| {
        receipts
            .iter()
            .filter(|r| r.receipt.request.actor == *actor)
            .count()
    };
    assert_eq!((by(&agent()), by(&Actor::Human)), (2, 1), "{receipts:#?}");
    assert!(receipts.iter().all(|r| r.whole && r.receipt.is_finished()));
    let human = receipts
        .iter()
        .find(|r| r.receipt.request.actor == Actor::Human)
        .expect("the person's click");
    assert_eq!(human.receipt.request.target, sid("silences.apply_all"));
    assert_eq!(human.receipt.request.action.as_str(), "apply_all");

    // the person's click, with the control under it in the frame on
    // screen: the press and the release, never a character.
    let inputs: Vec<(InputKind, Option<SemId>)> = session
        .events()
        .iter()
        .filter_map(|e| match &e.kind {
            Kind::Input { kind, target, .. } => Some((*kind, target.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        inputs,
        [
            (InputKind::PointerDown, Some(sid("silences.apply_all"))),
            (InputKind::PointerUp, Some(sid("silences.apply_all"))),
        ]
    );

    // the frames, and where each control showed in them: at the agent's
    // first look, the rect it was served.
    let times = session.frame_times();
    assert!(times.len() >= 3, "{times:?}");
    let scene = session
        .scene_at(end)
        .expect("a frame reads")
        .expect("a frame");
    assert!(!scene.is_empty());
    let first_look = session
        .events()
        .iter()
        .find(|e| matches!(&e.kind, Kind::Observed { revision, .. } if *revision == story.seen.revision))
        .map(|e| e.t)
        .expect("the first look is recorded");
    let served = story.seen.bounds.as_ref().expect("bounds asked for");
    assert!(!served.is_empty());
    for bound in served {
        assert_eq!(
            session.binding_at(&bound.id, first_look),
            Some(bound.rect),
            "{} at the first look",
            bound.id
        );
    }
    let [x, y, w, h] = session
        .binding_at(&sid("silences.cut[1]"), first_look)
        .expect("the first cut shows");
    assert_eq!(
        session.hit_at(first_look, x + w / 2.0, y + h / 2.0),
        Some(sid("silences.cut[1]"))
    );

    // the inspector reads why the reject was refused: it rested on the
    // agent's first look, and the person's click moved the decision it
    // read.
    let inspector = Inspector::new(&session);
    let stale_id = story.stale[0].request;
    let origin = receipts
        .iter()
        .find(|r| r.receipt.id == stale_id && r.receipt.request.actor == agent())
        .map(|r| r.origin)
        .expect("the refused reject is recorded");
    let why = inspector
        .why(origin, stale_id)
        .expect("it reads")
        .expect("the request is in the session");
    let Basis::Observed { revision, .. } = &why.basis else {
        panic!("the first look is the basis: {:?}", why.basis);
    };
    assert_eq!(*revision, story.seen.revision);
    let decision = why
        .between
        .reads
        .iter()
        .find(|r| r.at.to_string() == "silences.cut[2]:decision")
        .expect("reject_cut reads the decision");
    let [moved] = decision.moves.as_slice() else {
        panic!("one move: {:?}", decision.moves);
    };
    let By::Receipt { cause } = &moved.by else {
        panic!("one receipt moved it: {:?}", moved.by);
    };
    assert_eq!(
        (&cause.actor, cause.target.as_str(), cause.action.as_str()),
        (&Actor::Human, "silences.apply_all", "apply_all")
    );
    assert!(
        why.between
            .inputs
            .iter()
            .any(|i| i.target.as_ref() == Some(&sid("silences.apply_all"))),
        "the click is in the stretch: {:?}",
        why.between.inputs
    );
    assert!(matches!(why.ending, Ending::Rejected { .. }));

    // without the variable nothing is created.
    if std::env::var_os(ENV).is_none() {
        assert!(Recording::from_env().is_none());
    }
}
