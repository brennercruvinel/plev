//! The ephemeral interface (task-80 demo 4; the method, written before
//! this ran, is in
//! docs/tasks/task-80-proof-qualitatively-different-software/README.md):
//! the review of the silences born from an intent beside the fixed
//! screen, a person deciding a cut on it with a click, "I can't compare
//! these" refused by the rules, the review turned into the comparison in
//! place, and closed. Recorded with no window, the way the gallery's shell
//! records (`uigallery::shell::window::record::Recording`), and held to the criterion
//! of the method, read from the recording:
//!
//! - K1: the person's decision on the generated surface is a request on
//!   the app's cut, and the fixed screen holds the same decision;
//! - K2: for each cut, the controls of the review in view at 1200 by 800,
//!   generated against fixed, at the same frame;
//! - K3: the video frames in view to put the original beside the result;
//! - K4: the surface's nodes only between the propose and the close,
//!   nothing of it after, and every node of it from the materializer,
//!   never from a file of the app's screen;
//! - K5: what it cost (printed);
//! - K6: the refusal leaves the surface as it was;
//! - and the privacy the recording keeps: a person's input only as its
//!   kind and the control under it, no secret, no camera, no preference
//!   of accessibility as a state.
//!
//! The Silences page from its seed (cuts 1 and 2). Nothing here runs
//! ffmpeg: the frames the page and the surface show are synthetic, put in
//! the page's cache (`ShowcaseView::stage_silences_frames`) before every
//! tick, as the regression harness does. A real agent port in a
//! temporary directory. The file is kept at
//! `$CARGO_TARGET_TMPDIR/showcase-ephemeral-test.monster` for
//! `plev-inspect`.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use accord::{ActionName, Actor, Millis, Node, Role, SemId, Stage, Value};
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::image::{ImageHandle, load_image_rgba};
use engine::widgets::prelude::WidgetEvent;
use record::{InputKind, Kind, Limits, Session};
use socket::{Act, AgentPort, Client, Config, Observe, Reply, SnapshotReply, StageReply};
use uigallery::shell::window::record::Recording;
use uigallery::views::ShowcaseView;

/// The window of checkpoint b.
const W: f32 = 1200.0;
const H: f32 = 800.0;

/// One turn of the shell's clock, a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

/// The surface the rules open, and the pieces of a cut the review task
/// needs: what the cut is (its frame, its range, its decision) and the
/// two answers.
const SURFACE: &str = "proposed";
const PIECES: [&str; 5] = ["frame", "range", "decision", "apply_cut", "reject_cut"];

const REVIEW: &str = "the list with the video and the counts";
const CANT_COMPARE: &str = "I can't compare these";
const COMPARE: &str = "compare the original with the cuts";

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

fn of_surface(id: &str) -> bool {
    id == SURFACE || id.starts_with("proposed.")
}

/// A frame of the video, synthetic: a wash whose grey follows the
/// instant, so two instants are two pictures.
fn wash(ms: u64) -> Result<ImageHandle, String> {
    let (w, h) = (32u32, 18u32);
    let v = u8::try_from(40 + (ms / 100) % 200).unwrap_or(120);
    let pixels = std::iter::repeat_n([v, v, v, 255], (w * h) as usize)
        .flatten()
        .collect();
    load_image_rgba(w, h, pixels).map_err(|e| e.to_string())
}

/// Where a control showed in the frame on screen: its rect and the part
/// inside its clip (`None` when nothing of it shows); `None` when it is
/// not bound in that frame.
type Where = Vec<(String, Option<([f32; 4], Option<[f32; 4]>)>)>;

/// A settled frame measured: when it was drawn, on the recorder's clock,
/// and where each control asked for showed in it.
#[derive(Clone, Debug, Default)]
struct Seen {
    t: Millis,
    at: Where,
}

/// What the shell's thread does between two of the agent's messages.
enum Poke {
    /// A person clicks the middle of where the control shows.
    Click(String),
    /// Where each control shows in the frame on screen now.
    Measure(Vec<String>),
}

type Pokes = Sender<(Poke, Sender<Seen>)>;

fn poke(pokes: &Pokes, what: Poke) -> Seen {
    let (done, wait) = mpsc::channel();
    pokes.send((what, done)).expect("the shell listens");
    wait.recv().expect("the shell answers")
}

/// The recording shell around the gallery, with no window.
struct Shell<'a> {
    view: ShowcaseView,
    port: AgentPort,
    frame: Compositor,
    recording: &'a Recording,
}

impl Shell<'_> {
    fn draw(&mut self) {
        self.view.render(&mut self.frame);
        self.recording.frame(&mut self.view, &self.frame);
    }

    /// The port served, the frames the page would ask ffmpeg for put in
    /// its cache, the page ticked, a frame when anything changed.
    fn turn(&mut self) -> bool {
        let acted = self.view.serve_agent_recorded(
            &mut self.port,
            &self.frame,
            Some(self.recording.recorder()),
        );
        self.view.stage_silences_frames(&mut wash);
        let moved = self.view.tick(TURN);
        if acted || moved {
            self.draw();
        }
        acted || moved
    }

    /// Turns until the page stops moving (a surface's entry or exit).
    fn settle(&mut self) {
        for _ in 0..240 {
            if !self.turn() {
                break;
            }
        }
    }

    fn poked(&mut self, what: Poke) -> Seen {
        self.settle();
        match what {
            Poke::Click(id) => {
                let [x, y, w, h] = self
                    .frame
                    .binding(&sid(&id))
                    .and_then(|b| b.visible_rect())
                    .unwrap_or_else(|| panic!("{id} shows"));
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
                Seen::default()
            }
            Poke::Measure(ids) => Seen {
                t: self.view.now(),
                at: ids
                    .into_iter()
                    .map(|id| {
                        let b = self.frame.binding(&sid(&id));
                        let at = b.map(|b| (b.rect, b.visible_rect()));
                        (id, at)
                    })
                    .collect(),
            },
        }
    }
}

/// Runs `agent` against `view` through a port in a temp dir, this thread
/// playing the recording shell; then the page settles (the surface's
/// exit), a frame for every change.
fn shell<T: Send + 'static>(
    view: ShowcaseView,
    recording: &Recording,
    agent: impl FnOnce(PathBuf, Pokes) -> T + Send + 'static,
) -> T {
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
    shell.view.stage_silences_frames(&mut wash);
    shell.draw();
    let (pokes, poked) = mpsc::channel::<(Poke, Sender<Seen>)>();
    let agent = thread::spawn(move || agent(path, pokes));
    let deadline = Instant::now() + Duration::from_secs(60);
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the agent did not finish");
        let _ = woke.recv_timeout(Duration::from_millis(2));
        shell.turn();
        while let Ok((what, done)) = poked.try_recv() {
            let answer = shell.poked(what);
            let _ = done.send(answer);
        }
    }
    let story = agent.join().expect("the agent ran");
    shell.settle();
    story
}

fn client(path: &Path) -> Client {
    let mut c = Client::connect(path).expect("the agent connects");
    c.set_timeout(Some(20_000)).expect("a timeout");
    c
}

fn look(c: &mut Client) -> SnapshotReply {
    match c.observe(Observe::default()).expect("an observation") {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot: {other:?}"),
    }
}

/// An act resting on an observation of the whole app made right before.
fn act(c: &mut Client, target: &str, action: &str, param: Option<&str>) -> Vec<StageReply> {
    let seen = look(c);
    let name = ActionName::parse(action).expect("an action");
    let mut act = Act::from_revision(sid(target), name, seen.revision);
    act.param = param.map(|p| Value::Text(p.to_owned()));
    c.act(act, |_| {}).expect("the act's stages")
}

fn last(stages: &[StageReply]) -> &Stage {
    &stages.last().expect("a stage").stage
}

/// The ids of every piece of the review task, generated and fixed.
fn review_ids() -> Vec<String> {
    let mut ids = Vec::new();
    for cut in [1, 2] {
        ids.push(format!("silences.cut[{cut}]"));
        for piece in PIECES {
            ids.push(format!("proposed.silences.cut[{cut}].{piece}"));
        }
    }
    ids
}

/// What the agent went through, in order.
struct Story {
    at_rest: Seen,
    review: Vec<StageReply>,
    opened: Vec<Node>,
    at_open: Seen,
    refused: Vec<StageReply>,
    compared: Vec<StageReply>,
    compared_nodes: Vec<Node>,
    at_compare: Seen,
    closed: Vec<StageReply>,
    after: Vec<Node>,
}

/// In view in a frame of `W` by `H`: something of it shows, inside it.
fn in_view(at: Option<&([f32; 4], Option<[f32; 4]>)>) -> bool {
    at.and_then(|(_, visible)| *visible)
        .is_some_and(|[x, y, w, h]| w > 0.0 && h > 0.0 && x + w <= W + 0.5 && y + h <= H + 0.5)
}

/// Whole in view: the rect the control draws is the part that shows.
fn whole(at: Option<&([f32; 4], Option<[f32; 4]>)>) -> bool {
    in_view(at) && at.is_some_and(|(rect, visible)| *visible == Some(*rect))
}

fn seen<'a>(seen: &'a Seen, id: &str) -> Option<&'a ([f32; 4], Option<[f32; 4]>)> {
    seen.at
        .iter()
        .find(|(i, _)| i == id)
        .and_then(|(_, at)| at.as_ref())
}

/// The time of the stage `name` in a request's recorded receipt.
fn stage_at(stages: &[(Stage, Millis)], name: &str) -> Millis {
    stages
        .iter()
        .find(|(s, _)| s.name() == name)
        .map(|(_, t)| *t)
        .unwrap_or_else(|| panic!("a {name} stage: {stages:?}"))
}

#[test]
fn the_review_is_born_from_an_intent_becomes_the_comparison_and_leaves_nothing() {
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let mut recording = Recording::new(dir.path().join("sessions"), Limits::default());
    let mut view = ShowcaseView::new(W, H);
    view.jump_to_page("silences");
    assert_eq!(view.page_title(), "Silences");
    view.resize(W, H, 1.0);
    recording.attach(&mut view);

    let story = shell(view, &recording, |path, pokes| {
        let mut c = client(&path);
        c.hello("demo80").expect("welcome");
        // the fixed screen alone, at the whole width, before any surface
        let at_rest = poke(&pokes, Poke::Measure(review_ids()));
        act(&mut c, "silences_surface.proposer", "select", Some("rules"));
        act(&mut c, "silences_surface.intent", "set_value", Some(REVIEW));
        let review = act(&mut c, "silences_surface.propose", "propose", None);
        let opened = look(&mut c).nodes;
        let at_open = poke(&pokes, Poke::Measure(review_ids()));
        // the person decides the first cut on the generated surface
        poke(
            &pokes,
            Poke::Click("proposed.silences.cut[1].apply_cut".to_owned()),
        );
        act(
            &mut c,
            "silences_surface.intent",
            "set_value",
            Some(CANT_COMPARE),
        );
        let refused = act(&mut c, "silences_surface.propose", "propose", None);
        act(
            &mut c,
            "silences_surface.intent",
            "set_value",
            Some(COMPARE),
        );
        let compared = act(&mut c, "silences_surface.propose", "propose", None);
        let compared_nodes = look(&mut c).nodes;
        let mut media: Vec<String> = compared_nodes
            .iter()
            .filter(|n| of_surface(n.id.as_str()) && n.id.as_str().contains("video"))
            .map(|n| n.id.as_str().to_owned())
            .collect();
        media.push("silences.video".to_owned());
        let at_compare = poke(&pokes, Poke::Measure(media));
        let closed = act(&mut c, "silences_surface.close", "close", None);
        let after = look(&mut c).nodes;
        Story {
            at_rest,
            review,
            opened,
            at_open,
            refused,
            compared,
            compared_nodes,
            at_compare,
            closed,
            after,
        }
    });

    // the agent's side: opened, refused with why, compared in place, closed.
    let Stage::Completed(opened) = last(&story.review) else {
        panic!("the review opens: {:?}", story.review);
    };
    assert_eq!(
        opened.result.get("surface"),
        Some(&Value::Text(SURFACE.into()))
    );
    let Stage::Failed { error: why } = last(&story.refused) else {
        panic!(
            "the rules refuse to compare on the Silences: {:?}",
            story.refused
        );
    };
    assert!(
        why.starts_with("nothing here can be swapped for a table or a chart to compare"),
        "{why}"
    );
    let Stage::Completed(compared) = last(&story.compared) else {
        panic!("the comparison: {:?}", story.compared);
    };
    assert_eq!(
        compared.result.get("surface"),
        Some(&Value::Text(SURFACE.into())),
        "the same surface, in place"
    );
    assert!(
        story
            .compared_nodes
            .iter()
            .any(|n| n.id.as_str() == "proposed.silences.video_views"),
        "the views side by side"
    );
    assert!(matches!(last(&story.closed), Stage::Completed(_)));
    assert!(story.opened.iter().any(|n| of_surface(n.id.as_str())));
    assert!(
        !story.after.iter().any(|n| of_surface(n.id.as_str())),
        "nothing of the surface in the app after the close"
    );

    // saved as the shell saves it, and read back as plev-inspect reads it.
    let (path, written) = recording.save().expect("the window is saved");
    let session = Session::open(&path).expect("the session opens");
    let kept = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("showcase-ephemeral-test.monster");
    std::fs::copy(&path, &kept).expect("kept for plev-inspect");
    let (start, end) = session.span().expect("events");
    let receipts = session.receipts(end);
    let proposes: Vec<_> = receipts
        .iter()
        .filter(|r| r.receipt.request.action.as_str() == "propose")
        .collect();
    assert_eq!(proposes.len(), 3, "the review, the refusal, the comparison");
    let t_open = stage_at(&proposes[0].receipt.stages, "completed");
    let t_asked = stage_at(&proposes[0].receipt.stages, "requested");
    let t_refused = stage_at(&proposes[1].receipt.stages, "failed");
    let t_compared = stage_at(&proposes[2].receipt.stages, "completed");
    let close = receipts
        .iter()
        .find(|r| r.receipt.request.action.as_str() == "close")
        .expect("the close");
    let t_closed = stage_at(&close.receipt.stages, "completed");

    // K1: the person's click on the generated surface is her request on
    // the app's cut, and the fixed screen holds the decision.
    let person = receipts
        .iter()
        .find(|r| r.receipt.request.actor == Actor::Human)
        .expect("the person's decision");
    assert_eq!(person.receipt.request.target, sid("silences.cut[1]"));
    assert_eq!(person.receipt.request.action.as_str(), "apply_cut");
    assert!(matches!(person.receipt.stage(), Some(Stage::Completed(_))));
    let state_end = session.state_at(end).expect("reads").expect("a state");
    assert_eq!(
        state_end.value(&"silences.cut[1]:decision".parse().expect("a ref")),
        Some(&Value::Text("apply".into())),
        "the fixed screen holds the person's decision"
    );

    // K2: the review in view at 1200 by 800, generated against fixed, in
    // the frame the review was measured on; the fixed screen alone, at
    // the whole width before any surface, as a second control. a fixed
    // row counts its five pieces when it shows whole (the row is one
    // binding), a generated piece when something of it shows.
    let fixed_count = |at: &Seen, cut: u32| {
        if whole(seen(at, &format!("silences.cut[{cut}]"))) {
            PIECES.len()
        } else {
            0
        }
    };
    for cut in [1, 2] {
        let generated = PIECES
            .iter()
            .filter(|p| {
                in_view(seen(
                    &story.at_open,
                    &format!("proposed.silences.cut[{cut}].{p}"),
                ))
            })
            .count();
        let beside = fixed_count(&story.at_open, cut);
        let alone = fixed_count(&story.at_rest, cut);
        println!(
            "K2 cut {cut}: generated {generated} of {n} in view; the fixed beside it {beside} of {n} \
             (row {:?}); the fixed alone {alone} of {n} (row {:?})",
            seen(&story.at_open, &format!("silences.cut[{cut}]")),
            seen(&story.at_rest, &format!("silences.cut[{cut}]")),
            n = PIECES.len(),
        );
        assert!(
            generated >= beside,
            "cut {cut}: the generated shows {generated}, the fixed beside it {beside}"
        );
    }
    // the recorded frames hold what the live ones showed.
    for measured in [&story.at_rest, &story.at_open, &story.at_compare] {
        let bound = session.bindings_at(measured.t).expect("bindings");
        for (id, at) in &measured.at {
            let recorded = bound.iter().find(|b| b.id.as_str() == id).map(|b| b.rect);
            let live = at.as_ref().and_then(|(_, v)| *v);
            assert_eq!(recorded, live, "{id} at +{} ms", measured.t - start);
        }
    }

    // K3: the frames in view to put the original beside the result.
    let generated_frames = story
        .at_compare
        .at
        .iter()
        .filter(|(id, at)| {
            of_surface(id)
                && in_view(at.as_ref())
                && story
                    .compared_nodes
                    .iter()
                    .any(|n| n.id.as_str() == id && matches!(n.role, Role::Image | Role::Media))
        })
        .count();
    let fixed_frames = usize::from(in_view(seen(&story.at_compare, "silences.video")));
    println!("K3: generated {generated_frames} frames in view, the fixed {fixed_frames}");
    println!("K3 at the comparison: {:?}", story.at_compare);
    assert!(generated_frames >= fixed_frames);

    // K4: the surface only between the propose and the close, in the
    // state and in the frames; every node of it from the materializer.
    // the recorded states, event by event: absent, then present in one
    // stretch that starts with the propose and ends with the close, then
    // absent to the end.
    let mut stretch: Vec<(Millis, bool)> = Vec::new();
    for event in session.events() {
        let state = session.state_at(event.t).expect("reads").expect("a state");
        let has = state.nodes().iter().any(|n| of_surface(n.id.as_str()));
        if stretch.last().is_none_or(|(_, was)| *was != has) {
            stretch.push((event.t, has));
        }
    }
    let flips: Vec<(Millis, bool)> = stretch.iter().map(|(t, h)| (t - start, *h)).collect();
    println!(
        "K4: the surface in the recorded state {flips:?} (ms from the start, present or not); \
         asked +{}, opened +{}, closed +{}",
        t_asked - start,
        t_open - start,
        t_closed - start
    );
    let [(_, false), (born, true), (gone, false)] = stretch[..] else {
        panic!("one stretch with the surface: {flips:?}");
    };
    // the state with the surface is recorded after the drain that ran
    // the propose, before anything else happens on the page.
    let next_stage = session
        .events()
        .iter()
        .find(|e| e.t > t_open && matches!(e.kind, Kind::Stage { .. }))
        .map(|e| e.t)
        .expect("a stage after the review opened");
    assert!(
        born >= t_asked && born <= next_stage,
        "born at +{}",
        born - start
    );
    assert!(gone >= t_closed, "gone at +{}", gone - start);
    for t in session.frame_times() {
        let bound = session.bindings_at(t).expect("bindings");
        let has = bound.iter().any(|b| of_surface(b.id.as_str()));
        if t < t_asked || t > t_closed {
            assert!(!has, "a binding of the surface at +{} ms", t - start);
        }
    }
    let last_frame = session.frame_times().last().copied().expect("frames");
    assert!(last_frame > t_closed, "a frame after the close");
    let at_open = session.state_at(born).expect("reads").expect("a state");
    let mut sources = (0, 0);
    for node in at_open.nodes() {
        let Some(source) = &node.source else {
            continue;
        };
        if of_surface(node.id.as_str()) {
            assert!(
                source.file.starts_with("rust/screen/src/"),
                "{}: {}",
                node.id,
                source.file
            );
            sources.0 += 1;
        } else if node.id.as_str().starts_with("silences.") {
            assert!(
                source
                    .file
                    .starts_with("apps/uigallery/src/views/apps/silences"),
                "{}: {}",
                node.id,
                source.file
            );
            sources.1 += 1;
        }
    }
    let of_it = at_open
        .nodes()
        .iter()
        .filter(|n| of_surface(n.id.as_str()))
        .count();
    println!(
        "K4: {} of the surface's {of_it} nodes carry a source, all in rust/screen/src; \
         {} of the fixed screen's, all in apps/uigallery/src/view/apps/silences",
        sources.0, sources.1
    );
    assert!(sources.0 > 0 && sources.1 > 0, "{sources:?}");

    // K6: the refusal leaves the surface as it was.
    let surface_at = |t: Millis| {
        let state = session.state_at(t).expect("reads").expect("a state");
        state
            .nodes()
            .iter()
            .filter(|n| of_surface(n.id.as_str()))
            .map(|n| (n.id.clone(), n.state.clone()))
            .collect::<Vec<_>>()
    };
    let t_refusal_asked = stage_at(&proposes[1].receipt.stages, "requested");
    assert_eq!(surface_at(t_refusal_asked), surface_at(t_refused));

    // privacy: a person's input only as its kind and the control under
    // it; no secret, no camera, no preference of accessibility as a state.
    let inputs: Vec<(InputKind, Option<SemId>)> = session
        .events()
        .iter()
        .filter_map(|e| match &e.kind {
            Kind::Input { kind, target, .. } => Some((*kind, target.clone())),
            _ => None,
        })
        .collect();
    let button = Some(sid("proposed.silences.cut[1].apply_cut"));
    assert_eq!(
        inputs,
        [
            (InputKind::PointerDown, button.clone()),
            (InputKind::PointerUp, button)
        ]
    );
    let mut nodes_seen = 0;
    for event in session.events() {
        let state = session.state_at(event.t).expect("reads").expect("a state");
        for node in state.nodes() {
            nodes_seen += 1;
            assert!(!node.constraints.secret, "{} is secret", node.id);
            let named = format!("{} {}", node.id, node.label).to_lowercase();
            assert!(!named.contains("camera"), "{named}");
            if named.contains("accessibility") {
                assert!(node.state.is_empty(), "{} keeps a state", node.id);
            }
        }
    }

    // K5: what it cost.
    println!(
        "K5: propose to review {} ms by the rules ({:?} nodes); comparison {} ms; \
         the surface open {} ms; recording {} bytes (events {}, chunks {}, bindings {}), \
         {} events, {} frames; {} nodes read for privacy",
        t_open - t_asked,
        opened.result.get("nodes"),
        t_compared - stage_at(&proposes[2].receipt.stages, "requested"),
        t_closed - t_open,
        written.bytes,
        written.events_bytes,
        written.chunks_bytes,
        written.bindings_bytes,
        written.events,
        session.frame_times().len(),
        nodes_seen
    );
    println!("kept at {}", kept.display());
}
