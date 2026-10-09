//! The workbench through a real agent port, with no window, served the
//! way the ide's shell serves it (`ide::agent::serve`): after the port's
//! wake, and again after every worker event the shell applies. A real
//! repository in a temp dir and the real git worker; this thread is the
//! shell, a spawned thread is the agent speaking the protocol.
//!
//! What it holds the lib's `Host` to: hello names the ide and the agent,
//! an observation carries the projection and where each row shows, a
//! stage completes only in the drain after the worker's answer is applied,
//! with the revision the app is at, and an act that rests on a state a
//! person changed is refused as stale, naming the person, to the agent
//! and on screen in the same words. An agent's request that waits for a
//! person opens the modal, and the person's answer goes through the
//! dispatcher; a decline reaches the agent at once, since the port's sink
//! hears the stage outside a drain and wakes the port (the P0 of task-92;
//! before it, a decline waited for the agent's next message).
//! The last tests run the `plev` binary itself as the agent, as an agent
//! in a shell runs it, and read what it printed (task-94).

#![cfg(unix)]

// shared with human_agent.rs: this binary uses the rig and a few helpers.
#[allow(dead_code)]
mod support;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use engine::widgets::overlay::OverlayKind;
use engine::drawing::compositor::Compositor;
use gitdrv::GitEvent;
use idestudio::interfaces::window::record::Recording;
use idestudio::capability::Workbench;
use accord::{Acted, ActionName, Actor, Change, Rejection, Revision, SemId, Stage, StateRef, Value};
use support::{Rig, set};
use socket::{
    Act, AgentPort, Ask, Client, Config, Observe, Reply, ServerMsg, SnapshotReply, StageReply,
    decode_server, encode_server,
};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

/// What woke the shell before one drain.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cause {
    /// The port's wake: an agent sent something.
    Port,
    /// A worker event, applied right before the drain.
    Worker(&'static str),
}

/// One drain of the port, as the shell ran it.
#[derive(Clone, Copy, Debug)]
struct Drain {
    cause: Cause,
    /// What `serve` answered: an act was dispatched.
    acted: bool,
    /// Stages sent by the port so far.
    stages_out: u64,
}

/// What the shell's thread does between two of the agent's messages.
enum Poke {
    /// Apply worker events until this many `Branches` (the last event of
    /// a refresh) came since the shell started: every refresh the
    /// agent's acts sent has landed, and nothing is left in flight.
    Settle(usize),
    /// A person unstages a file, as the file menu's "Unstage file" asks
    /// it: through the dispatcher, as `Actor::Human`.
    Unstage(&'static str),
    /// A person stages a file, through the dispatcher, as `Actor::Human`.
    Stage(&'static str),
    /// The person reads the confirmation modal on screen and clicks its
    /// confirm button (`confirm`) or its cancel button: the answer goes
    /// through the dispatcher as `Actor::Human`. What the modal said goes
    /// back through `said`. Like the ide's window after a person's input,
    /// the shell draws and does not serve the port.
    Answer { confirm: bool, said: Sender<Asked> },
}

/// What a confirmation modal says: its title, its body and the label of
/// its confirm button.
#[derive(Clone, Debug, PartialEq)]
struct Asked {
    title: String,
    body: String,
    confirm: String,
}

/// The confirmation modal on screen, from the top of the overlay stack.
fn modal_on_screen(bench: &Workbench) -> Option<Asked> {
    bench
        .workspace()
        .overlay_mgr
        .stack
        .iter()
        .rev()
        .find_map(|overlay| match &overlay.kind {
            OverlayKind::Modal {
                title,
                body,
                confirm,
                ..
            } => Some(Asked {
                title: title.clone(),
                body: body.clone(),
                confirm: confirm.clone(),
            }),
            _ => None,
        })
}

type Pokes = Sender<(Poke, Sender<()>)>;

/// Asks the shell to do `what` and waits until it is done.
fn poke(pokes: &Pokes, what: Poke) {
    let (done, wait) = mpsc::channel();
    pokes.send((what, done)).expect("the shell listens");
    wait.recv().expect("the shell answers");
}

fn kind(event: &GitEvent) -> &'static str {
    match event {
        GitEvent::Status(_) => "status",
        GitEvent::Log(_) => "log",
        GitEvent::Branches(_) => "branches",
        GitEvent::DiffWorkdir { .. } | GitEvent::DiffCommit { .. } => "diff",
        GitEvent::Staged { .. } => "staged",
        GitEvent::Unstaged { .. } => "unstaged",
        GitEvent::Committed(_) => "committed",
        GitEvent::CommitRejected { .. } => "commit_rejected",
        _ => "other",
    }
}

/// The frame as the window draws it: the workbench binds its ids.
fn frame(bench: &mut Workbench) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    bench.render(&mut c);
    c
}

/// The ide's shell around the rig's workbench, with no window: the port,
/// the frame on screen, and every drain it ran.
struct Shell<'a> {
    rig: &'a mut Rig,
    port: AgentPort,
    frame: Compositor,
    drains: Vec<Drain>,
    /// `Branches` events applied since the shell started.
    branches: usize,
    /// Whether the shell also serves the port right after a worker event,
    /// as the ide's window does; without it, only the port's wake serves.
    after_worker: bool,
    /// The session recording, when the shell records as the ide's window
    /// does with `PLEV_RECORD=1`: the observations served, the state after
    /// an act, a git result and a person's input.
    recording: Option<&'a Recording>,
}

impl Shell<'_> {
    /// The port's wake: serve it, draw when an act ran.
    fn woke(&mut self) {
        let acted = self.serve();
        self.drained(Cause::Port, acted);
        if acted {
            self.frame = frame(&mut self.rig.bench);
        }
    }

    /// A worker event: apply it, serve the port (the stages it ended go
    /// out), draw.
    fn worker(&mut self, event: GitEvent) {
        let cause = Cause::Worker(kind(&event));
        self.branches += usize::from(matches!(event, GitEvent::Branches(_)));
        self.rig.bench.apply(event);
        if self.after_worker {
            let acted = self.serve();
            self.drained(cause, acted);
        }
        self.record();
        self.frame = frame(&mut self.rig.bench);
    }

    /// One drain of the port, recorded when the shell records.
    fn serve(&mut self) -> bool {
        idestudio::capability::agent::serve_recorded(
            &mut self.port,
            &mut self.rig.bench,
            &self.frame,
            self.recording,
        )
    }

    /// The state now, when the shell records.
    fn record(&self) {
        if let Some(recording) = self.recording {
            recording.state(&self.rig.bench);
        }
    }

    fn drained(&mut self, cause: Cause, acted: bool) {
        self.drains.push(Drain {
            cause,
            acted,
            stages_out: self.port.traffic().stages_out,
        });
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
                self.rig.ask(Actor::Human, &target, "unstage", None);
                self.record();
                self.frame = frame(&mut self.rig.bench);
            }
            Poke::Stage(path) => {
                let target = format!("changes.file[{path}]");
                self.rig.ask(Actor::Human, &target, "stage", None);
                self.record();
                self.frame = frame(&mut self.rig.bench);
            }
            Poke::Answer { confirm, said } => {
                let asked = modal_on_screen(&self.rig.bench).expect("a modal asks the person");
                let _ = said.send(asked);
                self.frame = frame(&mut self.rig.bench);
                let (yes, no) = self.rig.bench.workspace().modal_buttons().expect("drawn");
                let [x, y, w, h] = if confirm { yes } else { no };
                assert!(self.rig.bench.click(x + w / 2.0, y + h / 2.0));
                self.record();
                self.frame = frame(&mut self.rig.bench);
            }
        }
    }
}

/// Runs `agent` against the workbench of `rig` through a port in a temp
/// dir, this thread playing the ide's shell (`Shell`). Returns what the
/// agent returned and every drain.
fn shell<T: Send + 'static>(
    rig: &mut Rig,
    agent: impl FnOnce(PathBuf, Pokes) -> T + Send + 'static,
) -> (T, Vec<Drain>) {
    run_shell(rig, true, None, agent)
}

/// [`shell`], with a shell that serves the port only after its wake:
/// what reaches the agent after a worker's answer or a person's input
/// goes out because the port woke the loop itself.
fn shell_on_wake<T: Send + 'static>(
    rig: &mut Rig,
    agent: impl FnOnce(PathBuf, Pokes) -> T + Send + 'static,
) -> (T, Vec<Drain>) {
    run_shell(rig, false, None, agent)
}

fn run_shell<'a, T: Send + 'static>(
    rig: &'a mut Rig,
    after_worker: bool,
    recording: Option<&'a Recording>,
    agent: impl FnOnce(PathBuf, Pokes) -> T + Send + 'static,
) -> (T, Vec<Drain>) {
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
        frame: frame(&mut rig.bench),
        rig,
        port,
        drains: Vec::new(),
        branches: 0,
        after_worker,
        recording,
    };
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
    (agent.join().expect("the agent ran"), shell.drains)
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
    act_with(c, target, action, None, at)
}

fn act_with(
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

/// The revision the last stage of an act carries.
fn landed(stages: &[StageReply]) -> Revision {
    stages
        .last()
        .and_then(|s| s.revision)
        .expect("the last stage carries the revision")
}

fn names(stages: &[StageReply]) -> Vec<&'static str> {
    stages.iter().map(|s| s.stage.name()).collect()
}

/// A value a snapshot reply holds.
fn value(seen: &SnapshotReply, node: &str, key: &str) -> Option<Value> {
    let id = sid(node);
    let at = StateRef::parse(&format!("{node}:{key}")).expect("a state ref");
    seen.nodes
        .iter()
        .find(|n| n.id == id)
        .and_then(|n| n.state.get(&at.key).cloned())
}

#[test]
fn an_agent_stages_a_file_and_the_last_stage_leaves_after_the_worker() {
    let mut rig = Rig::new(&["A", "B"], &[]);
    let ((welcome, seen, staged, since), drains) = shell(&mut rig, |path, _| {
        let mut c = client(&path);
        let welcome = c
            .call(Ask::Hello {
                name: "test-agent".into(),
            })
            .expect("welcome");
        let seen = snapshot(
            c.observe(Observe {
                bounds: true,
                ..Observe::default()
            })
            .expect("an observation"),
        );
        let staged = act(&mut c, "changes.file[A]", "stage", seen.revision);
        let since = c
            .observe(Observe {
                since: Some(landed(&staged)),
                scope: Some(sid("changes.file[A]")),
                ..Observe::default()
            })
            .expect("an observation since the stage");
        (welcome, seen, staged, since)
    });

    let agent = Actor::Agent {
        name: "test-agent".into(),
    };
    assert_eq!(
        welcome,
        Reply::Welcome {
            app: "ide".into(),
            actor: agent.clone(),
        }
    );

    // the projection, and where the row shows in the frame on screen.
    assert_eq!(
        value(&seen, "changes.file[A]", "unstaged"),
        Some(Value::Bool(true))
    );
    assert_eq!(
        value(&seen, "changes.file[A]", "staged"),
        Some(Value::Bool(false))
    );
    let bounds = seen.bounds.expect("bounds asked for");
    let row = bounds
        .iter()
        .find(|b| b.id == sid("changes.file[A]"))
        .expect("the row is bound");
    assert!(row.rect[2] > 0.0 && row.rect[3] > 0.0, "{row:?}");
    for b in &bounds {
        let [x, y, w, h] = b.rect;
        assert!(
            x >= 0.0 && y >= 0.0 && x + w <= 1280.0 && y + h <= 800.0,
            "inside the window: {b:?}"
        );
        assert!(
            seen.nodes.iter().any(|n| n.id == b.id),
            "bound and projected: {b:?}"
        );
    }

    // every stage, the revision only on the last.
    assert_eq!(
        names(&staged),
        ["requested", "accepted", "executing", "completed"]
    );
    assert!(staged[..3].iter().all(|s| s.revision.is_none()));
    let asked = staged[0].asked.as_ref().expect("requested carries asked");
    assert_eq!(asked.actor, agent);

    // the drain that ran the act sent three stages; the fourth went out
    // in the drain the shell ran right after applying the worker's
    // answer, and in no drain before it.
    let at_act = drains
        .iter()
        .position(|d| d.acted)
        .expect("a drain ran the act");
    assert_eq!(drains[at_act].cause, Cause::Port);
    assert_eq!(drains[at_act].stages_out, 3, "{drains:?}");
    let at_end = drains
        .iter()
        .position(|d| d.stages_out == 4)
        .expect("the last stage went out");
    assert_eq!(drains[at_end].cause, Cause::Worker("staged"), "{drains:?}");
    assert!(at_end > at_act);
    assert_eq!(
        drains.iter().filter(|d| d.acted).count(),
        1,
        "hello and the observations dispatched nothing: {drains:?}"
    );

    // the revision the last stage carried is held: `since` answers a delta
    // from it, and the file is staged, in the workbench and in git.
    let Reply::Delta(delta) = since else {
        panic!("a delta from the stage's revision: {since:?}");
    };
    assert_eq!(delta.base, landed(&staged));
    assert_eq!(rig.value("changes.file[A]:staged"), Value::Bool(true));
    assert_eq!(
        rig.git(&["diff", "--cached", "--name-only"]).trim(),
        "A",
        "git staged A"
    );
}

#[test]
fn a_person_unstaging_between_the_agents_look_and_its_submit_is_named_in_both_places() {
    let mut rig = Rig::new(&["A", "B"], &[]);
    let ((stale, notices, recovered), _) = shell(&mut rig, |path, pokes| {
        let mut c = client(&path);
        c.hello("test-agent").expect("welcome");
        let seen = snapshot(c.observe(Observe::default()).expect("an observation"));
        let a = act(&mut c, "changes.file[A]", "stage", seen.revision);
        let b = act(&mut c, "changes.file[B]", "stage", landed(&a));
        let open = act(&mut c, "commit.form", "open", landed(&b));
        let text = Some(Value::Text("two files".into()));
        act_with(&mut c, "commit.message", "set_value", text, landed(&open));
        // the refreshes the two stages sent land (one each), so nothing
        // moves the workspace between the decision and the person.
        poke(&pokes, Poke::Settle(2));
        let decided = snapshot(c.observe(Observe::default()).expect("the decision"));
        // the agent decided on {A, B}; meanwhile the person takes B out.
        poke(&pokes, Poke::Unstage("B"));
        let stale = act(&mut c, "commit.submit", "submit", decided.revision);
        let notices = snapshot(
            c.observe(Observe {
                scope: Some(sid("notices")),
                ..Observe::default()
            })
            .expect("the notices"),
        );
        // it recovers: it looks again and commits what is staged now, the
        // person's choice, deciding on what it was shown.
        let now = snapshot(c.observe(Observe::default()).expect("a new look"));
        let recovered = act(&mut c, "commit.submit", "submit", now.revision);
        (stale, notices, recovered)
    });

    assert_eq!(names(&stale), ["requested", "rejected"]);
    let Stage::Rejected(Rejection::Stale { changes, acted }) = &stale[1].stage else {
        panic!("stale: {stale:?}");
    };
    assert_eq!(
        changes,
        &[Change::StateChanged {
            at: StateRef::parse("changes:staged").expect("a state ref"),
            before: Some(set(&["A", "B"])),
            after: Some(set(&["A"])),
            added: Vec::new(),
            removed: vec!["B".into()],
        }]
    );
    let [
        Acted {
            actor,
            target,
            action,
            ..
        },
    ] = acted.as_slice()
    else {
        panic!("one actor: {acted:?}");
    };
    assert_eq!(
        (actor, target, action.as_str()),
        (&Actor::Human, &sid("changes.file[B]"), "unstage")
    );

    // what the agent was told is what the screen says.
    let said: Vec<&str> = notices
        .nodes
        .iter()
        .filter(|n| n.id != sid("notices"))
        .map(|n| n.label.as_str())
        .collect();
    assert_eq!(
        said,
        ["Refused test-agent's submit: B left the staged files (the person unstaged B)"]
    );
    assert_eq!(rig.notices(), said);

    assert_eq!(
        names(&recovered),
        ["requested", "accepted", "executing", "completed"]
    );
    // one commit over the first: the refused submit committed nothing.
    assert_eq!(rig.commits(), 2);
    assert_eq!(rig.head_files(), ["A"]);
    assert_eq!(rig.git(&["log", "-1", "--format=%s"]).trim(), "two files");
}

/// The act of `action` on `target`, resting on `at`.
fn act_on(target: &str, action: &str, at: Revision) -> Act {
    Act::from_revision(
        sid(target),
        ActionName::parse(action).expect("an action"),
        at,
    )
}

/// The probe of task-92, the path end to end: an agent asks through the
/// port for a `discard`, which waits for a person (`Confirm::ForAgents`);
/// the ide opens its modal; the person clicks Discard, which confirms
/// through the dispatcher as `Actor::Human`; the discard runs. The shell
/// serves the port only after its wake: the stages after the person's
/// answer and the worker's reach the agent because the port woke the loop
/// when its sink heard them (the P0 of task-92).
#[test]
fn an_agent_discard_through_the_port_waits_for_the_person_who_confirms_it() {
    let mut rig = Rig::new(&["README", "new.txt"], &[]);
    let readme = rig.dir.path().join("README");
    let ((stages, asked), drains) = shell_on_wake(&mut rig, |path, pokes| {
        let mut c = client(&path);
        c.hello("test-agent").expect("welcome");
        let seen = snapshot(c.observe(Observe::default()).expect("an observation"));
        let (said, heard) = mpsc::channel();
        let act = act_on("changes.file[README]", "discard", seen.revision);
        let stages = c
            .act(act, |s| {
                if s.stage == Stage::AwaitingConfirmation {
                    let said = said.clone();
                    poke(
                        &pokes,
                        Poke::Answer {
                            confirm: true,
                            said,
                        },
                    );
                }
            })
            .expect("the act's stages");
        (stages, heard.recv().expect("what the modal said"))
    });

    assert_eq!(
        names(&stages),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    assert_eq!(stages[2].stage, Stage::Confirmed { by: Actor::Human });
    assert!(
        drains.iter().all(|d| d.cause == Cause::Port),
        "only the port's wake served: {drains:?}"
    );
    // what the person read: the text is born in `Workbench::confirmation`
    // (crates/ide/src/workbench/notice.rs), from the request's actor,
    // action and target.
    assert_eq!(
        asked,
        Asked {
            title: "Discard changes?".into(),
            body: "test-agent asks to discard all changes to README. This cannot be undone.".into(),
            confirm: "Discard".into(),
        }
    );
    assert_eq!(
        std::fs::read_to_string(&readme).expect("README"),
        "readme\n",
        "the discard ran"
    );
}

/// The probe of task-92, the decline: the person clicks Cancel, which
/// declines through the dispatcher as `Actor::Human`, and nothing runs.
/// Before the P0 the stage waited in the port until the agent's next
/// message, since a decline sends nothing to the worker and the shell
/// serves nothing after a person's input (`crates/ide/src/main.rs`,
/// `serve_agent`); a `plev act` that only waits never got it. Now the
/// port's sink hears it and wakes the loop, and the agent gets it without
/// sending anything more.
#[test]
fn a_decline_reaches_the_agent_with_no_message_from_it() {
    let mut rig = Rig::new(&["README", "new.txt"], &[]);
    let new = rig.dir.path().join("new.txt");
    let ((stages, asked, sent), drains) = shell_on_wake(&mut rig, |path, pokes| {
        let mut c = client(&path);
        c.hello("test-agent").expect("welcome");
        let seen = snapshot(c.observe(Observe::default()).expect("an observation"));
        let (said, heard) = mpsc::channel();
        let act = act_on("changes.file[new.txt]", "discard", seen.revision);
        let mut sent = None;
        let stages = c
            .act(act, |s| {
                if s.stage == Stage::AwaitingConfirmation {
                    let said = said.clone();
                    poke(
                        &pokes,
                        Poke::Answer {
                            confirm: false,
                            said,
                        },
                    );
                    sent = Some(Instant::now());
                }
            })
            .expect("the act's stages, the decline among them");
        let sent = sent.map(|at| at.elapsed());
        (stages, heard.recv().expect("what the modal said"), sent)
    });

    assert_eq!(
        names(&stages),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert_eq!(
        stages[2].stage,
        Stage::Rejected(Rejection::Declined { by: Actor::Human })
    );
    assert!(stages[2].revision.is_some(), "a terminal stage");
    // the act sent nothing after itself: the decline went out in a drain
    // of the port's own wake.
    assert!(drains.iter().all(|d| d.cause == Cause::Port), "{drains:?}");
    assert!(
        sent.is_some_and(|waited| waited < Duration::from_secs(5)),
        "the decline came without a message: {sent:?}"
    );
    assert_eq!(
        asked,
        Asked {
            title: "Discard changes?".into(),
            body: "test-agent asks to discard all changes to new.txt. This cannot be undone."
                .into(),
            confirm: "Discard".into(),
        }
    );
    assert_eq!(
        std::fs::read_to_string(&new).expect("new.txt"),
        "new.txt\n",
        "nothing ran"
    );
    assert_eq!(
        rig.notices(),
        ["Declined: discard of new.txt by test-agent did not run"]
    );
}

/// The three files of the task of the third round (task-62).
const TASK: [&str; 3] = ["src/lexer.rs", "src/parser.rs", "src/session.rs"];

/// What the agent asks the person in the A2''b scenario.
const QUESTION: &str = "recolocar src/session.rs?";

/// What the agent of the A2''b scenario went through.
struct Asking {
    /// The submit on what it decided, after the person took
    /// src/session.rs out.
    stale: Vec<StageReply>,
    /// The stage of src/session.rs it asked the person about.
    asked: Vec<StageReply>,
    /// What the modal said.
    modal: Asked,
    /// Its submit after the person's answer, when it went on.
    submit: Option<Vec<StageReply>>,
}

/// The A2''b scenario of the third round (task-62), with the question it
/// lacked (task-92): the agent stages the three files of the task and
/// writes the message; the person takes src/session.rs out before the
/// submit; the submit is refused as stale, naming the person. Instead of
/// staging the file over the person, or stopping, the agent asks: a stage
/// of src/session.rs with a question. The person answers in the modal
/// (`confirm`), through the dispatcher as `Actor::Human`. Confirmed, the
/// agent observes and commits; declined, it stops.
fn a2b(rig: &mut Rig, recording: Option<&Recording>, confirm: bool) -> Asking {
    let (story, _) = run_shell(rig, true, recording, move |path, pokes| {
        let mut c = client(&path);
        c.hello("test-agent").expect("welcome");
        let seen = snapshot(c.observe(Observe::default()).expect("an observation"));
        let mut at = seen.revision;
        for file in TASK {
            at = landed(&act(&mut c, &format!("changes.file[{file}]"), "stage", at));
        }
        let open = act(&mut c, "commit.form", "open", at);
        let text = Some(Value::Text("the three files".into()));
        act_with(&mut c, "commit.message", "set_value", text, landed(&open));
        poke(&pokes, Poke::Settle(3));
        let decided = snapshot(c.observe(Observe::default()).expect("the decision"));
        poke(&pokes, Poke::Unstage("src/session.rs"));
        let stale = act(&mut c, "commit.submit", "submit", decided.revision);

        let now = snapshot(c.observe(Observe::default()).expect("a new look"));
        let (said, heard) = mpsc::channel();
        let ask = act_on("changes.file[src/session.rs]", "stage", now.revision).with_ask(QUESTION);
        let asked = c
            .act(ask, |s| {
                if s.stage == Stage::AwaitingConfirmation {
                    let said = said.clone();
                    poke(&pokes, Poke::Answer { confirm, said });
                }
            })
            .expect("the asked stage's stages");
        let modal = heard.recv().expect("what the modal said");
        let went_on = matches!(asked.last().map(|s| &s.stage), Some(Stage::Completed(_)));
        let submit = went_on.then(|| {
            poke(&pokes, Poke::Settle(5));
            let after = snapshot(c.observe(Observe::default()).expect("a look after"));
            act(&mut c, "commit.submit", "submit", after.revision)
        });
        Asking {
            stale,
            asked,
            modal,
            submit,
        }
    });
    story
}

/// The stale refusal of the A2''b submit names the person's unstage.
fn refused_by_the_person(stale: &[StageReply]) {
    assert_eq!(names(stale), ["requested", "rejected"]);
    let Stage::Rejected(Rejection::Stale { acted, .. }) = &stale[1].stage else {
        panic!("stale: {stale:?}");
    };
    assert!(
        acted.iter().any(|a| a.actor == Actor::Human
            && a.target == sid("changes.file[src/session.rs]")
            && a.action.as_str() == "unstage"),
        "{acted:?}"
    );
}

/// The modal of the A2''b question.
fn asked_modal() -> Asked {
    Asked {
        title: "Question from test-agent".into(),
        body: format!(
            "test-agent asks the person: \"{QUESTION}\" If allowed, it will stage src/session.rs."
        ),
        confirm: "Allow".into(),
    }
}

/// task-92, the acceptance: the A2''b scenario, the person confirms. The
/// stage runs, and the agent commits the three files. The session is
/// recorded, and the inspector's why of the asked stage shows the question
/// and who answered; the file is kept for `plev-inspect` at
/// `$CARGO_TARGET_TMPDIR/ide-ask-test.monster`.
#[test]
fn the_agent_asks_the_person_who_confirms_and_the_three_files_are_committed() {
    let mut rig = Rig::new(&TASK, &[]);
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let mut recording = Recording::new(dir.path().join("sessions"), record::Limits::default());
    recording.attach(&mut rig.bench);
    let story = a2b(&mut rig, Some(&recording), true);

    refused_by_the_person(&story.stale);
    assert_eq!(
        names(&story.asked),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    let asked = story.asked[0].asked.as_ref().expect("asked");
    assert_eq!(asked.ask.as_deref(), Some(QUESTION));
    assert_eq!(story.asked[2].stage, Stage::Confirmed { by: Actor::Human });
    assert_eq!(story.modal, asked_modal());
    let submit = story.submit.expect("the agent went on");
    assert_eq!(
        names(&submit),
        ["requested", "accepted", "executing", "completed"]
    );
    assert_eq!(rig.commits(), 2);
    assert_eq!(rig.head_files(), TASK);
    assert_eq!(
        rig.git(&["log", "-1", "--format=%s"]).trim(),
        "the three files"
    );

    // the recording, read back as plev-inspect reads it.
    let (path, _) = recording.save().expect("the window is saved");
    let session = record::Session::open(&path).expect("the session opens");
    let kept = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ide-ask-test.monster");
    std::fs::copy(&path, &kept).expect("kept for plev-inspect");
    let id = story.asked[0].request;
    let origin = session
        .events()
        .iter()
        .find_map(|e| match &e.kind {
            record::Kind::Stage { origin, id: at, .. } if *at == id => Some(*origin),
            _ => None,
        })
        .expect("the asked stage is recorded");
    let why = replay::Inspector::new(&session)
        .why(origin, id)
        .expect("the session reads")
        .expect("the request is in the window");
    assert_eq!(why.request.ask.as_deref(), Some(QUESTION));
    let text = replay::prose::why(&why);
    for line in [
        format!("asked         agent:test-agent asked a person first: \"{QUESTION}\""),
        "answered      human confirmed it, at".to_owned(),
    ] {
        assert!(text.contains(&line), "{line:?} in:\n{text}");
    }
}

/// task-92, the acceptance: the same scenario, the person declines. The
/// agent gets `declined` through the port and stops: nothing more is
/// staged, nothing is committed, and the screen says what was declined.
#[test]
fn the_agent_asks_the_person_who_declines_and_it_stops() {
    let mut rig = Rig::new(&TASK, &[]);
    let story = a2b(&mut rig, None, false);

    refused_by_the_person(&story.stale);
    assert_eq!(
        names(&story.asked),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert_eq!(
        story.asked[2].stage,
        Stage::Rejected(Rejection::Declined { by: Actor::Human })
    );
    assert_eq!(story.modal, asked_modal());
    assert!(story.submit.is_none(), "the agent stopped");
    assert_eq!(rig.commits(), 1, "nothing committed");
    assert_eq!(
        rig.git(&["diff", "--cached", "--name-only"]).trim(),
        "src/lexer.rs\nsrc/parser.rs",
        "src/session.rs stays out, as the person left it"
    );
    assert!(
        rig.notices()
            .contains(&"Declined: stage of src/session.rs by test-agent did not run".to_owned()),
        "{:?}",
        rig.notices()
    );
}

/// task-92, what the P0 covers besides a decline, with a shell that
/// serves the port only after its wake: a question confirmed on an action
/// that ends at once, with no git behind it (`open`), and one the second
/// check refuses at the confirmation, since the person staged the file
/// meanwhile. Both endings reach the agent with no message from it.
#[test]
fn what_ends_at_once_or_at_the_second_check_reaches_the_agent() {
    let mut rig = Rig::new(&["A"], &[]);
    let ((open, stale), drains) = shell_on_wake(&mut rig, |path, pokes| {
        let mut c = client(&path);
        c.hello("test-agent").expect("welcome");
        let seen = snapshot(c.observe(Observe::default()).expect("an observation"));
        let (said, _heard) = mpsc::channel();
        let ask = act_on("commit.form", "open", seen.revision).with_ask("open the commit form?");
        let open = c
            .act(ask, |s| {
                if s.stage == Stage::AwaitingConfirmation {
                    let said = said.clone();
                    poke(
                        &pokes,
                        Poke::Answer {
                            confirm: true,
                            said,
                        },
                    );
                }
            })
            .expect("the open's stages");
        let now = snapshot(c.observe(Observe::default()).expect("a new look"));
        let ask = act_on("changes.file[A]", "stage", now.revision).with_ask("stage A?");
        let stale = c
            .act(ask, |s| {
                if s.stage == Stage::AwaitingConfirmation {
                    poke(&pokes, Poke::Stage("A"));
                    let said = said.clone();
                    poke(
                        &pokes,
                        Poke::Answer {
                            confirm: true,
                            said,
                        },
                    );
                }
            })
            .expect("the stage's stages");
        (open, stale)
    });

    assert_eq!(
        names(&open),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    assert_eq!(
        names(&stale),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    let Stage::Rejected(Rejection::Stale { acted, .. }) = &stale[2].stage else {
        panic!("refused at the second check: {stale:?}");
    };
    assert!(
        acted
            .iter()
            .any(|a| a.actor == Actor::Human && a.action.as_str() == "stage"),
        "{acted:?}"
    );
    assert!(drains.iter().all(|d| d.cause == Cause::Port), "{drains:?}");
    assert_eq!(rig.value("commit.form:open"), Value::Bool(true));
}

/// The `plev` binary of the wire package, in the directory above this
/// test binary's (`target/<profile>/plev`, the test in `deps`): cargo
/// names a binary only to the tests of its own package. `cargo test -p
/// wire -p ide`, or the workspace, builds it; one older than its source
/// would answer for another `plev`, so it is refused.
fn plev_bin() -> PathBuf {
    let exe = std::env::current_exe().expect("this test's path");
    let bin = exe
        .parent()
        .and_then(Path::parent)
        .expect("target/<profile>/deps")
        .join("plev");
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rust/socket/src/bin/plev.rs");
    let modified = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    assert!(
        modified(&bin) >= modified(&source),
        "no plev at {}, or one older than its source: build it with the wire package \
         (cargo test -p wire -p ide)",
        bin.display()
    );
    bin
}

/// What one `plev` run gave the agent: its exit code, and its stdout and
/// stderr as they came.
struct Printed {
    code: Option<i32>,
    out: String,
    err: String,
}

impl Printed {
    /// The messages on stdout, one JSON line each.
    fn messages(&self) -> Vec<ServerMsg> {
        self.out
            .lines()
            .map(|l| decode_server(l).unwrap_or_else(|e| panic!("{l}: {e:?}")))
            .collect()
    }

    /// The stages on stdout.
    fn stages(&self) -> Vec<StageReply> {
        self.messages()
            .into_iter()
            .map(|m| match m.reply {
                Reply::Stage(stage) => stage,
                other => panic!("a stage: {other:?}"),
            })
            .collect()
    }

    /// The revision of an observation, or of the last stage of an act
    /// that completed, as the next command takes it.
    fn revision(&self) -> String {
        assert_eq!(self.code, Some(0), "{}{}", self.out, self.err);
        let revision = match self.messages().pop().map(|m| m.reply) {
            Some(Reply::Snapshot(s)) => Some(s.revision),
            Some(Reply::Stage(s)) => s.revision,
            other => panic!("an observation or a stage: {other:?}"),
        };
        revision.expect("a revision").to_string()
    }
}

/// `plev <args>` on the port at `path`, as the test's agent runs it
/// from a shell: the same name for every run, so they are one agent.
fn plev_command(path: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(plev_bin());
    cmd.args(args)
        .arg("--app")
        .arg(path)
        .args(["--name", "test-agent"]);
    cmd
}

/// What a finished `plev` printed, as text.
fn printed(output: Output) -> Printed {
    Printed {
        code: output.status.code(),
        out: String::from_utf8(output.stdout).expect("utf-8 on stdout"),
        err: String::from_utf8(output.stderr).expect("utf-8 on stderr"),
    }
}

/// Runs `plev <args>` on the port at `path` (`plev_command`).
fn plev(path: &Path, args: &[&str]) -> Printed {
    printed(plev_command(path, args).output().expect("plev runs"))
}

/// Runs `plev <args>` for an act that asks the person: when its
/// `awaiting_confirmation` comes out, the person answers in the modal
/// (`confirm`), while `plev` waits.
fn plev_asking(path: &Path, args: &[&str], pokes: &Pokes, confirm: bool) -> Printed {
    let mut child = plev_command(path, args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("plev runs");
    let stdout = child.stdout.take().expect("piped");
    let mut out = Vec::new();
    for line in BufReader::new(stdout).lines() {
        let line = line.expect("a line of stdout");
        let asks = matches!(
            decode_server(&line).map(|m| m.reply),
            Ok(Reply::Stage(StageReply {
                stage: Stage::AwaitingConfirmation,
                ..
            }))
        );
        out.extend([line.as_bytes(), b"\n"].concat());
        if asks {
            let (said, _) = mpsc::channel();
            poke(pokes, Poke::Answer { confirm, said });
        }
    }
    let mut output = child.wait_with_output().expect("plev ends");
    output.stdout = out;
    printed(output)
}

/// The repository of the fourth round (task-62): the three files of the
/// task, `src/login.rs` already in the index, and three that are not the
/// task's, `README.md` in the index.
const ROUND: [&str; 6] = [
    "src/login.rs",
    "src/session.rs",
    "tests/login_test.rs",
    "README.md",
    "notes.txt",
    "Cargo.toml",
];

/// The message of the fourth round's task, as `--param` takes it.
const MESSAGE: &str = r#""fix login session timeout""#;

/// What the agent a `plev` binary got in the fourth round (`fourth_round`).
struct Round {
    /// Its submit, refused as stale after the person took a file out.
    submit: Printed,
    /// Its stage of that file again, asking the person first.
    asked: Printed,
}

/// The fourth round of task-62 through the real port, the agent a `plev`
/// binary run as an agent runs it: it observes, stages the two files of
/// the task left out, unstages `README.md`, opens the form and writes the
/// message, every act resting on the revision the one before it landed
/// on; it observes to decide, the person takes `src/session.rs` out
/// (`unstage-before-submit`), and `plev act commit.submit submit
/// --from-revision` names the revision of that decision. Then it does
/// what the fifth round measures: it looks again and stages the file with
/// a question, and the person answers in the modal, allowing it when
/// `confirm`, declining it otherwise.
fn fourth_round(rig: &mut Rig, confirm: bool) -> Round {
    let (round, _) = shell(rig, move |path, pokes| {
        let mut at = plev(&path, &["observe"]).revision();
        let steps = [
            ("changes.file[src/session.rs]", "stage", None),
            ("changes.file[tests/login_test.rs]", "stage", None),
            ("changes.file[README.md]", "unstage", None),
            ("commit.form", "open", None),
            ("commit.message", "set_value", Some(MESSAGE)),
        ];
        for (target, action, param) in steps {
            let mut args = vec!["act", target, action, "--from-revision", &at];
            if let Some(param) = param {
                args.extend(["--param", param]);
            }
            at = plev(&path, &args).revision();
        }
        poke(&pokes, Poke::Settle(3));
        let decided = plev(&path, &["observe"]).revision();
        poke(&pokes, Poke::Unstage("src/session.rs"));
        let submit = ["act", "commit.submit", "submit", "--from-revision"];
        let submit = plev(&path, &[&submit[..], &[&decided]].concat());

        let now = plev(&path, &["observe"]).revision();
        let stage = ["act", "changes.file[src/session.rs]", "stage"];
        let ask = ["--from-revision", &now, "--ask", ASKED];
        let asked = plev_asking(&path, &[&stage[..], &ask].concat(), &pokes, confirm);
        Round { submit, asked }
    });
    round
}

/// What the agent of `fourth_round` asks the person.
const ASKED: &str = "the person took src/session.rs out; the task needs it: stage it again?";

/// The probe of task-94, and what it asked for: what an agent that runs
/// `plev` gets when its submit is refused as stale after the person took
/// a file out. stdout is what it was before task-94: the two stages, one
/// canonical JSON line each and nothing else, the refusal's `acted`
/// naming the person's unstage (request 6, as in the fourth round); the
/// exit is 1. stderr, empty before, is one line that names the person's
/// act and points at `--ask`. The agent asks as the line says; the person
/// declines, and after the decline nothing points at asking again.
/// `--nocapture` prints both streams of both acts.
#[test]
fn a_plev_submit_refused_after_the_person_unstaged_points_at_the_question() {
    let mut rig = Rig::new(&ROUND, &["src/login.rs", "README.md"]);
    let Round { submit, asked } = fourth_round(&mut rig, false);
    for run in [&submit, &asked] {
        println!("stdout:\n{}stderr:\n{}", run.out, run.err);
    }

    assert_eq!(submit.code, Some(1), "rejected");
    let messages = submit.messages();
    let canonical: String = messages
        .iter()
        .map(|m| format!("{}\n", encode_server(m)))
        .collect();
    assert_eq!(submit.out, canonical, "the stages and nothing else");
    let stages = submit.stages();
    assert_eq!(names(&stages), ["requested", "rejected"]);
    let Stage::Rejected(Rejection::Stale { changes, acted }) = &stages[1].stage else {
        panic!("stale: {stages:?}");
    };
    assert_eq!(
        changes
            .iter()
            .map(|c| match c {
                Change::StateChanged { at, removed, .. } => (at.to_string(), removed.clone()),
                other => panic!("a state change: {other:?}"),
            })
            .collect::<Vec<_>>(),
        [("changes:staged".to_owned(), vec!["src/session.rs".into()])]
    );
    let named: Vec<(String, &Actor, String, &str)> = acted
        .iter()
        .map(|a| {
            let id = a.id.to_string();
            (id, &a.actor, a.target.to_string(), a.action.as_str())
        })
        .collect();
    assert_eq!(
        named,
        [(
            "6".to_owned(),
            &Actor::Human,
            "changes.file[src/session.rs]".to_owned(),
            "unstage"
        )]
    );
    assert_eq!(
        submit.err,
        "plev: a person acted since this agent looked (request 6, human: unstage \
         changes.file[src/session.rs]); to act over that, ask the person first: add --ask \
         \"<why>\" to the act (one line, at most 280 characters)\n"
    );

    // the agent asks; the person declines; nothing more on stderr.
    assert_eq!(asked.code, Some(1), "declined");
    let stages = asked.stages();
    assert_eq!(
        names(&stages),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert_eq!(
        stages[0].asked.as_ref().and_then(|a| a.ask.as_deref()),
        Some(ASKED)
    );
    assert_eq!(
        stages[2].stage,
        Stage::Rejected(Rejection::Declined { by: Actor::Human })
    );
    assert_eq!(asked.err, "", "the person answered");
    assert_eq!(rig.commits(), 1, "nothing committed");
    assert_eq!(
        rig.git(&["diff", "--cached", "--name-only"]).trim(),
        "src/login.rs\ntests/login_test.rs",
        "src/session.rs stays out, as the person left it"
    );
}

/// The same round, the person allows: the stage runs, and after a stage
/// that completed nothing points at asking.
#[test]
fn a_question_the_person_allows_runs_and_prints_no_line() {
    let mut rig = Rig::new(&ROUND, &["src/login.rs", "README.md"]);
    let Round { submit, asked } = fourth_round(&mut rig, true);
    assert!(
        submit.err.contains("ask the person first"),
        "{}",
        submit.err
    );
    assert_eq!(asked.code, Some(0), "{}", asked.err);
    assert_eq!(
        names(&asked.stages()),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    assert_eq!(asked.err, "", "it asked, and it ran");
    assert_eq!(
        rig.value("changes.file[src/session.rs]:staged"),
        Value::Bool(true)
    );
}
