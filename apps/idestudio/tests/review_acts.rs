//! Every act on the review of the ide (task-96) goes through the
//! workbench's one dispatcher: a person's click on the surface is her
//! request, with her receipt, for the capability the surface's button
//! stands for; an agent through the port acts on the same button by the
//! contract (it observes the surface, acts on its node, and the workbench
//! routes it to the capability); and an act that rests on what moved is
//! refused stale like any other, naming the person who moved it.
//!
//! A real repository (the laboratory's review) and the real git worker;
//! this thread is the ide's shell (it serves the port after its wake and
//! after every worker event, draws after every act), a spawned thread is
//! the agent speaking the protocol.

#![cfg(unix)]

// shared with the other ide tests: this binary uses part of the rig.
#[allow(dead_code)]
mod support;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use accord::{
    ActionName, Actor, Change, Receipt, ReceiptSink, Rejection, RequestId, SemId, Stage, StateRef,
    Value,
};
use engine::drawing::compositor::Compositor;
use gitdrv::GitEvent;
use socket::{Act, AgentPort, Client, Config, Observe, Reply, SnapshotReply, StageReply};
use support::{Rig, completed};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

/// Every receipt the dispatcher kept, as its last stage left it.
#[derive(Clone, Default)]
struct Receipts(Arc<Mutex<Vec<Receipt>>>);

impl ReceiptSink for Receipts {
    fn on_stage(&mut self, receipt: &Receipt, _stage: &Stage) {
        let mut all = self.0.lock().expect("the receipts");
        match all.iter_mut().find(|r| r.id == receipt.id) {
            Some(kept) => *kept = receipt.clone(),
            None => all.push(receipt.clone()),
        }
    }
}

impl Receipts {
    fn get(&self, id: RequestId) -> Receipt {
        let all = self.0.lock().expect("the receipts");
        all.iter().find(|r| r.id == id).cloned().expect("a receipt")
    }

    fn all(&self) -> Vec<Receipt> {
        self.0.lock().expect("the receipts").clone()
    }
}

/// The workbench drawn into a fresh frame, as the window draws it.
fn frame(rig: &mut Rig) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    rig.bench.render(&mut c);
    c
}

/// The commits of the stacks, newest first.
fn commits(rig: &Rig) -> Vec<SemId> {
    let app = rig.bench.app_snapshot().expect("validates");
    app.children(&id("stacks")).map(|n| n.id.clone()).collect()
}

/// The surface's button that shows the diff of `commit`.
fn button(commit: &SemId) -> SemId {
    id(&format!("proposed.{commit}.show_diff"))
}

/// The review of the three newest commits, open, by the person.
fn reviewed() -> (Rig, Receipts) {
    let mut rig = Rig::review();
    rig.bench.host_surfaces();
    let receipts = Receipts::default();
    rig.bench.add_sink(Box::new(receipts.clone()));
    let intent = Some("review these three commits".into());
    rig.ask(Actor::Human, "ide_surface.intent", "set_value", intent);
    let asked = rig.ask(Actor::Human, "ide_surface.propose", "propose", None);
    assert!(completed(&rig.receipt(asked)).is_some());
    (rig, receipts)
}

/// The person clicks in the middle of `control` as the frame on screen
/// shows it: through the workbench, as the window's click goes.
fn click(rig: &mut Rig, control: &SemId) {
    let c = frame(rig);
    let binding = c
        .binding(control)
        .unwrap_or_else(|| panic!("{control} drawn"));
    let [x, y, w, h] = binding.visible_rect().expect("on screen");
    assert!(rig.bench.click(x + w / 2.0, y + h / 2.0), "the click took");
}

#[test]
fn a_click_on_the_surface_is_the_persons_request_for_the_capability() {
    let (mut rig, receipts) = reviewed();
    let commits = commits(&rig);
    let before = receipts.all().len();
    click(&mut rig, &button(&commits[1]));
    let all = receipts.all();
    assert_eq!(all.len(), before + 1, "one request for one click");
    let receipt = all.last().expect("the click's").clone();
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(
        receipt.request.target, commits[1],
        "the capability, not the button"
    );
    assert_eq!(receipt.request.action.as_str(), "show_diff");
    let target: StateRef = "diff:target".parse().unwrap();
    assert_eq!(
        receipt.request.observed.values,
        [(target, Some(Value::Text(String::new())))],
        "observed on the workspace's own state, as the surface showed it"
    );
    assert!(completed(&receipt).is_some(), "{receipt:?}");
    assert_eq!(
        rig.value("diff:target"),
        Value::Text(commits[1].to_string())
    );
    rig.until(|e| matches!(e, GitEvent::DiffCommit { .. }));
    let lines = rig.value("diff:lines");
    assert!(matches!(&lines, Value::List(l) if !l.is_empty()));
    assert_eq!(
        rig.value("proposed.diff.lines:lines"),
        lines,
        "the surface shows the diff the dispatcher asked for"
    );
    assert_eq!(
        rig.value("proposed.diff.lines:kinds"),
        rig.value("diff:kinds"),
        "in its code, with the kind of each line (task-97)"
    );
    for (n, commit) in commits.iter().take(3).enumerate() {
        click(&mut rig, &button(commit));
        rig.until(|e| matches!(e, GitEvent::DiffCommit { .. }));
        assert_eq!(
            rig.value("diff:target"),
            Value::Text(commit.to_string()),
            "{n}"
        );
    }
    assert!(
        receipts
            .all()
            .iter()
            .all(|r| r.request.actor == Actor::Human),
        "every act here is the person's"
    );
}

/// What the person does on the shell's thread between two of the agent's
/// messages: a click on the surface's button of a commit's diff.
type Pokes = Sender<(SemId, Sender<()>)>;

/// Runs `agent` against `rig`'s workbench through a port in a temp dir,
/// this thread playing the ide's shell. What the agent returned.
fn shell<T: Send + 'static>(
    rig: &mut Rig,
    agent: impl FnOnce(PathBuf, Pokes) -> T + Send + 'static,
) -> T {
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let (tx, woke): (Sender<()>, Receiver<()>) = mpsc::channel();
    let config = Config {
        app: idestudio::capability::agent::APP.into(),
        dir: dir.path().to_owned(),
        ring: 16,
    };
    let mut port = AgentPort::open_with(config, move || {
        let _ = tx.send(());
    })
    .expect("the port opens");
    let path = port.path().to_owned();
    let mut shown = frame(rig);
    let (pokes, poked) = mpsc::channel::<(SemId, Sender<()>)>();
    let agent = thread::spawn(move || agent(path, pokes));
    let deadline = Instant::now() + Duration::from_secs(60);
    while !agent.is_finished() {
        assert!(Instant::now() < deadline, "the agent did not finish");
        if woke.recv_timeout(Duration::from_millis(2)).is_ok()
            && idestudio::capability::agent::serve(&mut port, &mut rig.bench, &shown)
        {
            shown = frame(rig);
        }
        while let Some(event) = rig.event(Duration::ZERO) {
            rig.bench.apply(event);
            idestudio::capability::agent::serve(&mut port, &mut rig.bench, &shown);
            shown = frame(rig);
        }
        while let Ok((control, done)) = poked.try_recv() {
            click(rig, &control);
            shown = frame(rig);
            let _ = done.send(());
        }
    }
    agent.join().expect("the agent ran")
}

fn snapshot(reply: Reply) -> SnapshotReply {
    match reply {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot: {other:?}"),
    }
}

/// The agent acts on the surface's `control` from the revision it saw.
fn press(c: &mut Client, control: &SemId, seen: &SnapshotReply) -> Vec<StageReply> {
    let action = ActionName::parse("activate").expect("an action");
    let act = Act::from_revision(control.clone(), action, seen.revision);
    c.act(act, |_| {}).expect("the act's stages")
}

#[test]
fn an_agent_acts_on_the_same_surface_and_a_stale_act_is_refused_naming_the_person() {
    let (mut rig, receipts) = reviewed();
    let commits = commits(&rig);
    let (newest, second) = (button(&commits[0]), button(&commits[1]));
    let (pressed, retried) = {
        let (newest, second) = (newest.clone(), second.clone());
        shell(&mut rig, move |path, pokes| {
            let mut c = Client::connect(&path).expect("the agent connects");
            c.set_timeout(Some(20_000)).expect("a timeout");
            c.hello("review-agent").expect("welcome");
            let seen = snapshot(c.observe(Observe::default()).expect("an observation"));
            assert!(
                seen.nodes.iter().any(|n| n.id == newest),
                "the agent observes the surface"
            );
            // the person opens the second commit's diff through the
            // surface, after the agent looked
            let (done, wait) = mpsc::channel();
            pokes.send((second, done)).expect("the shell listens");
            wait.recv().expect("the person clicked");
            let pressed = press(&mut c, &newest, &seen);
            let again = snapshot(c.observe(Observe::default()).expect("a new look"));
            let retried = press(&mut c, &newest, &again);
            (pressed, retried)
        })
    };
    let agent = Actor::Agent {
        name: "review-agent".into(),
    };

    let refused = pressed.last().expect("a stage");
    let Stage::Rejected(Rejection::Stale { changes, acted }) = &refused.stage else {
        panic!("stale: {pressed:?}");
    };
    let target: StateRef = "diff:target".parse().unwrap();
    assert_eq!(
        changes,
        &[Change::StateChanged {
            at: target,
            before: Some(Value::Text(String::new())),
            after: Some(Value::Text(commits[1].to_string())),
            added: Vec::new(),
            removed: Vec::new(),
        }],
        "the delta names the state that moved"
    );
    assert_eq!(acted.len(), 1, "{acted:?}");
    assert_eq!(acted[0].actor, Actor::Human, "who moved it");
    let stale = receipts.get(refused.request);
    assert_eq!(stale.request.actor, agent);
    assert_eq!(stale.request.target, commits[0], "routed to the capability");
    assert_eq!(stale.request.action.as_str(), "show_diff");

    let done = retried.last().expect("a stage");
    assert!(matches!(done.stage, Stage::Completed(_)), "{retried:?}");
    let receipt = receipts.get(done.request);
    assert_eq!(receipt.request.actor, agent);
    assert_eq!(receipt.request.target, commits[0]);
    assert_eq!(
        rig.value("diff:target"),
        Value::Text(commits[0].to_string())
    );
    let people: Vec<String> = receipts
        .all()
        .iter()
        .filter(|r| r.request.actor == Actor::Human)
        .map(|r| format!("{} {}", r.request.action, r.request.target))
        .collect();
    assert_eq!(
        people,
        [
            "set_value ide_surface.intent".to_owned(),
            "propose ide_surface.propose".to_owned(),
            format!("show_diff {}", commits[1]),
        ],
        "the person's write, propose and click, each hers"
    );
    assert_eq!(
        acted[0].id,
        receipts.all()[2].id,
        "the click the refusal names"
    );
}
