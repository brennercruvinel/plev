//! The ide's session recording, through the path its shell takes
//! (`ide::record::Recording`, `ide::agent::serve_recorded`), with no
//! window: a real repository in a temp dir, the real git worker, a real
//! agent port. This thread plays the shell as `main.rs` does with
//! `PLEV_RECORD=1`: it serves the port after its wake and after every
//! worker event, records the state after a git result and after a
//! person's act, and records every frame it draws. A spawned thread is the
//! agent, speaking the protocol; the person takes a file out of the index
//! between the agent's decision and its submit.
//!
//! What it holds the recording to: the saved file opens, and holds the
//! agent's observations (with their scope and revision), every receipt,
//! the person's click with the control under it, the frames, and where
//! each row showed in them, the same rect the agent was served; and the
//! inspector reads from it why the submit was refused (the person's
//! unstage moved what the agent decided on) and how the recovery ended.
//! The file is kept at `$CARGO_TARGET_TMPDIR/ide-record-test.monster`,
//! for `plev-inspect`.

#![cfg(unix)]

// shared with the other ide tests: this binary uses the rig and a helper.
#[allow(dead_code)]
mod support;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use engine::drawing::compositor::Compositor;
use gitdrv::GitEvent;
use idestudio::interfaces::window::record::Recording;
use idestudio::capability::Workbench;
use replay::{Basis, By, Ending, Inspector};
use record::{InputKind, Kind, Limits, Session};
use accord::{ActionName, Actor, Rejection, Revision, SemId, Stage, Value};
use support::Rig;
use socket::{Act, AgentPort, Client, Config, Observe, Reply, SnapshotReply, StageReply};

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
    /// Apply worker events until this many `Branches` came since the
    /// shell started: every refresh the agent's acts sent has landed.
    Settle(usize),
    /// A person clicks the row of a file and takes it out of the index,
    /// as the file menu's "Unstage file" asks it: the click recorded with
    /// the row under it, the unstage through the dispatcher as
    /// `Actor::Human`.
    Unstage(&'static str),
}

type Pokes = Sender<(Poke, Sender<()>)>;

fn poke(pokes: &Pokes, what: Poke) {
    let (done, wait) = mpsc::channel();
    pokes.send((what, done)).expect("the shell listens");
    wait.recv().expect("the shell answers");
}

/// The frame as the window draws it: the workbench binds its ids.
fn frame(bench: &mut Workbench) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    bench.render(&mut c);
    c
}

/// The recording shell around the rig's workbench.
struct Shell<'a> {
    rig: &'a mut Rig,
    port: AgentPort,
    frame: Compositor,
    recording: &'a Recording,
    branches: usize,
}

impl Shell<'_> {
    /// A frame drawn, and recorded with the state it was drawn from.
    fn draw(&mut self) {
        self.frame = frame(&mut self.rig.bench);
        self.recording.frame(&self.rig.bench, &self.frame);
    }

    fn serve(&mut self) -> bool {
        idestudio::capability::agent::serve_recorded(
            &mut self.port,
            &mut self.rig.bench,
            &self.frame,
            Some(self.recording),
        )
    }

    /// The port's wake: serve it, draw when an act ran.
    fn woke(&mut self) {
        if self.serve() {
            self.draw();
        }
    }

    /// A worker event: apply it, serve the port, record the state, draw.
    fn worker(&mut self, event: GitEvent) {
        self.branches += usize::from(matches!(event, GitEvent::Branches(_)));
        self.rig.bench.apply(event);
        self.serve();
        self.recording.state(&self.rig.bench);
        self.draw();
    }

    fn poked(&mut self, what: Poke) {
        match what {
            Poke::Settle(n) => {
                while self.branches < n {
                    let event = self.rig.event(support::WAIT).expect("a refresh in time");
                    self.worker(event);
                }
            }
            Poke::Unstage(path) => {
                let target = format!("changes.file[{path}]");
                let [x, y, w, h] = self
                    .frame
                    .binding(&sid(&target))
                    .and_then(|b| b.visible_rect())
                    .expect("the row shows");
                let at = [x + w / 2.0, y + h / 2.0];
                self.recording.input(
                    &self.rig.bench,
                    InputKind::PointerDown,
                    Some(at),
                    &self.frame,
                );
                self.rig.ask(Actor::Human, &target, "unstage", None);
                self.recording.state(&self.rig.bench);
                self.draw();
            }
        }
    }
}

/// Runs `agent` against the rig's workbench through a port in a temp dir,
/// this thread playing the recording shell.
fn shell<T: Send + 'static>(
    rig: &mut Rig,
    recording: &Recording,
    agent: impl FnOnce(PathBuf, Pokes) -> T + Send + 'static,
) -> T {
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let (tx, woke): (Sender<()>, Receiver<()>) = mpsc::channel();
    let config = Config {
        app: idestudio::capability::agent::APP.into(),
        dir: dir.path().to_owned(),
        ring: 16,
    };
    let port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .expect("the port opens");
    let path = port.path().to_owned();
    let mut shell = Shell {
        frame: Compositor::new(),
        rig,
        port,
        recording,
        branches: 0,
    };
    shell.draw();
    let (pokes, poked) = mpsc::channel::<(Poke, Sender<()>)>();
    let agent = thread::spawn(move || agent(path, pokes));
    let deadline = Instant::now() + Duration::from_secs(60);
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the agent did not finish");
        if woke.recv_timeout(Duration::from_millis(2)).is_ok() {
            shell.woke();
        }
        while let Some(event) = shell.rig.event(Duration::ZERO) {
            shell.worker(event);
        }
        while let Ok((what, done)) = poked.try_recv() {
            shell.poked(what);
            let _ = done.send(());
        }
    }
    agent.join().expect("the agent ran")
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

fn act(
    c: &mut Client,
    target: &str,
    action: &str,
    param: Option<Value>,
    at: Revision,
) -> Vec<StageReply> {
    let name = ActionName::parse(action).expect("an action");
    let mut act = Act::from_revision(sid(target), name, at);
    if let Some(param) = param {
        act = act.with_param(param);
    }
    c.act(act, |_| {}).expect("the act's stages")
}

fn landed(stages: &[StageReply]) -> Revision {
    stages
        .last()
        .and_then(|s| s.revision)
        .expect("the last stage carries the revision")
}

/// What the agent saw and got, in order.
struct Story {
    seen: SnapshotReply,
    decided: SnapshotReply,
    stale: Vec<StageReply>,
    notices: SnapshotReply,
    recovered: Vec<StageReply>,
}

#[test]
fn the_recorded_session_holds_the_agent_the_person_the_frames_and_where_each_row_showed() {
    let mut rig = Rig::new(&["A", "B"], &[]);
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let mut recording = Recording::new(dir.path().join("sessions"), Limits::default());
    recording.attach(&mut rig.bench);

    let story = shell(&mut rig, &recording, |path, pokes| {
        let mut c = client(&path);
        c.hello("test-agent").expect("welcome");
        let seen = snapshot(
            c.observe(Observe {
                bounds: true,
                ..Observe::default()
            })
            .expect("an observation"),
        );
        let a = act(&mut c, "changes.file[A]", "stage", None, seen.revision);
        let b = act(&mut c, "changes.file[B]", "stage", None, landed(&a));
        let open = act(&mut c, "commit.form", "open", None, landed(&b));
        let text = Some(Value::Text("two files".into()));
        act(&mut c, "commit.message", "set_value", text, landed(&open));
        poke(&pokes, Poke::Settle(2));
        let decided = snapshot(c.observe(Observe::default()).expect("the decision"));
        poke(&pokes, Poke::Unstage("B"));
        let stale = act(&mut c, "commit.submit", "submit", None, decided.revision);
        let notices = snapshot(
            c.observe(Observe {
                scope: Some(sid("notices")),
                ..Observe::default()
            })
            .expect("the notices"),
        );
        let now = snapshot(c.observe(Observe::default()).expect("a new look"));
        let recovered = act(&mut c, "commit.submit", "submit", None, now.revision);
        Story {
            seen,
            decided,
            stale,
            notices,
            recovered,
        }
    });
    let Stage::Rejected(Rejection::Stale { .. }) = &story.stale[1].stage else {
        panic!("refused as stale: {:?}", story.stale);
    };
    assert!(matches!(
        story.recovered.last().map(|s| &s.stage),
        Some(Stage::Completed(_))
    ));

    // saved as the shell saves it, and read back as plev-inspect reads it.
    let (path, written) = recording.save().expect("the window is saved");
    assert_eq!(
        path,
        dir.path()
            .join("sessions")
            .join(format!("ide-{}-0.monster", std::process::id()))
    );
    assert!(written.bindings_bytes > 0, "{written:?}");
    let session = Session::open(&path).expect("the session opens");
    let kept = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ide-record-test.monster");
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
        (None, story.decided.revision),
        (Some(sid("notices")), story.notices.revision),
    ] {
        assert!(
            observed.contains(&(scope.clone(), revision)),
            "{scope:?} at {revision}: {observed:?}"
        );
    }

    // every receipt: the agent's six acts and the person's unstage.
    let receipts = session.receipts(end);
    let by = |actor: &Actor| {
        receipts
            .iter()
            .filter(|r| r.receipt.request.actor == *actor)
            .count()
    };
    assert_eq!((by(&agent()), by(&Actor::Human)), (6, 1), "{receipts:#?}");
    assert!(receipts.iter().all(|r| r.whole && r.receipt.is_finished()));

    // the person's click, with the row under it in the frame on screen.
    let clicked = session.events().iter().find_map(|e| match &e.kind {
        Kind::Input {
            kind: InputKind::PointerDown,
            target,
            ..
        } => Some(target.clone()),
        _ => None,
    });
    assert_eq!(clicked, Some(Some(sid("changes.file[B]"))));

    // the frames, and where each row showed in them: at the agent's first
    // look, the rect it was served.
    let times = session.frame_times();
    assert!(times.len() > 5, "{times:?}");
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
    for bound in served {
        assert_eq!(
            session.binding_at(&bound.id, first_look),
            Some(bound.rect),
            "{} at the first look",
            bound.id
        );
    }
    let row_a = session
        .binding_at(&sid("changes.file[A]"), first_look)
        .expect("row A shows");
    let [x, y, w, h] = row_a;
    assert_eq!(
        session.hit_at(first_look, x + w / 2.0, y + h / 2.0),
        Some(sid("changes.file[A]"))
    );

    // the inspector reads why the submit was refused: it rested on the
    // agent's decision, and the person's unstage moved what it read.
    let inspector = Inspector::new(&session);
    let stale_id = story.stale[0].request;
    let origin = receipts
        .iter()
        .find(|r| r.receipt.id == stale_id && r.receipt.request.actor == agent())
        .map(|r| r.origin)
        .expect("the refused submit is recorded");
    let why = inspector
        .why(origin, stale_id)
        .expect("it reads")
        .expect("the request is in the session");
    let Basis::Observed { revision, .. } = &why.basis else {
        panic!("the decision is the basis: {:?}", why.basis);
    };
    assert_eq!(*revision, story.decided.revision);
    let staged = why
        .between
        .reads
        .iter()
        .find(|r| r.at.to_string() == "changes:staged")
        .expect("submit reads changes:staged");
    let [moved] = staged.moves.as_slice() else {
        panic!("one move: {:?}", staged.moves);
    };
    let By::Receipt { cause } = &moved.by else {
        panic!("one receipt moved it: {:?}", moved.by);
    };
    assert_eq!(
        (&cause.actor, cause.target.as_str(), cause.action.as_str()),
        (&Actor::Human, "changes.file[B]", "unstage")
    );
    assert!(
        why.between
            .inputs
            .iter()
            .any(|i| i.target == Some(sid("changes.file[B]"))),
        "{:?}",
        why.between.inputs
    );
    assert!(matches!(why.ending, Ending::Rejected { .. }));

    let recovered = inspector
        .why(origin, story.recovered[0].request)
        .expect("it reads")
        .expect("the recovery is in the session");
    let Ending::Completed { outcome, .. } = &recovered.ending else {
        panic!("the recovery completed: {:?}", recovered.ending);
    };
    let head = rig.head();
    assert_eq!(
        outcome.result.get("commit"),
        Some(&Value::Text(head.clone())),
        "the sha git has"
    );
}
