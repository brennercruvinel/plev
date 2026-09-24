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
//! and on screen in the same words.

#![cfg(unix)]

// shared with human_agent.rs: this binary uses the rig and a few helpers.
#[allow(dead_code)]
mod support;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use engine::compositor::Compositor;
use git::GitEvent;
use ide::workbench::Workbench;
use sem::{Acted, ActionName, Actor, Change, Rejection, Revision, SemId, Stage, StateRef, Value};
use support::{Rig, set};
use wire::{Act, AgentPort, Ask, Client, Config, Observe, Reply, SnapshotReply, StageReply};

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
}

impl Shell<'_> {
    /// The port's wake: serve it, draw when an act ran.
    fn woke(&mut self) {
        let acted = ide::agent::serve(&mut self.port, &mut self.rig.bench, &self.frame);
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
        let acted = ide::agent::serve(&mut self.port, &mut self.rig.bench, &self.frame);
        self.drained(cause, acted);
        self.frame = frame(&mut self.rig.bench);
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
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let (tx, woke): (Sender<()>, Receiver<()>) = mpsc::channel();
    let config = Config {
        app: ide::agent::APP.into(),
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
        // it recovers on the revision the refusal carried: what is staged
        // now, the person's choice.
        let recovered = act(&mut c, "commit.submit", "submit", landed(&stale));
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
