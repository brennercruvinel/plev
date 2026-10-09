//! The intent line under the fixed views (task-96, phase b): with no
//! surface open the workspace has the whole width of the window, the one
//! it had before the line existed, and gives the line only a strip of its
//! height along the bottom; a surface opens in a column at the right, and
//! the width comes back when it closes. The line stays in reach as it was:
//! the person's click and keys, and an agent through the port, which also
//! sees where each control of the line shows. The proposer's panel opens
//! upward, inside the window, since there is no room under the strip.
//!
//! A real repository in a temp dir and the real git worker; in the port's
//! test this thread is the ide's shell and a spawned thread the agent.

#![cfg(unix)]

// shared with the other ide tests: this binary uses part of the rig.
#[allow(dead_code)]
mod support;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use engine::drawing::compositor::Compositor;
use accord::{ActionName, Actor, SemId, Stage, Value};
use support::{Rig, completed};
use socket::{Act, AgentPort, Client, Config, Observe, Reply, SnapshotReply, StageReply};

/// The window of the rig.
const W: f32 = 1280.0;
const H: f32 = 800.0;

/// The line's controls, as the frame binds them.
const LINE: [&str; 6] = [
    "ide_surface",
    "ide_surface.intent",
    "ide_surface.proposer",
    "ide_surface.propose",
    "ide_surface.close",
    "ide_surface.status",
];

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

/// The workbench drawn into a fresh frame, as the window draws it.
fn frame(rig: &mut Rig) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    rig.bench.render(&mut c);
    c
}

/// Where `control` is drawn in `c`.
fn rect(c: &Compositor, control: &str) -> [f32; 4] {
    c.binding(&id(control))
        .unwrap_or_else(|| panic!("{control} is drawn"))
        .rect
}

/// `r` is inside a window `w` by `h`, from `top` down.
fn inside(r: [f32; 4], top: f32, (w, h): (f32, f32)) -> bool {
    let [x, y, rw, rh] = r;
    x >= 0.0 && y >= top - 0.5 && x + rw <= w + 0.5 && y + rh <= h + 0.5
}

/// The person clicks in the middle of `control` as `c` draws it.
fn click(rig: &mut Rig, c: &Compositor, control: &str) -> bool {
    let [x, y, w, h] = rect(c, control);
    rig.bench.click(x + w / 2.0, y + h / 2.0)
}

#[test]
fn with_no_surface_open_the_fixed_views_have_the_width_they_had_before_the_line() {
    let mut without = Rig::new(&["a.txt"], &[]);
    let mut with = Rig::new(&["a.txt"], &[]);
    with.bench.host_surfaces_with(None);
    let (before, now) = (frame(&mut without), frame(&mut with));
    let ws = with.bench.workspace();
    assert_eq!(ws.vw, W, "the window's width, as before task-96");
    assert_eq!(without.bench.workspace().vw, W);
    let strip = H - ws.vh;
    assert!(
        strip > 0.0 && strip <= H / 10.0,
        "the line takes a strip of the height: {strip}"
    );
    for view in ["changes", "stacks", "diff"] {
        let (a, b) = (rect(&before, view), rect(&now, view));
        assert_eq!(
            (a[0], a[1], a[2]),
            (b[0], b[1], b[2]),
            "{view} where it was, as wide"
        );
        assert!(b[1] + b[3] <= ws.vh + 0.5, "{view} ends above the strip");
    }
    // each repository has its own first commit, with a sha of its own
    let card = |c: &Compositor| {
        c.bindings()
            .find(|(k, _)| k.as_str().starts_with("stacks.commit["))
            .map(|(_, b)| b.rect)
            .expect("a commit's card")
    };
    assert_eq!(
        card(&before)[2],
        card(&now)[2],
        "a commit's card is as wide as it was"
    );
    for control in LINE {
        let r = rect(&now, control);
        assert!(inside(r, ws.vh, (W, H)), "{control} in the strip: {r:?}");
    }
}

#[test]
fn a_surface_opens_in_a_column_and_gives_the_width_back_when_it_closes() {
    let mut rig = Rig::new(&["a.txt", "b.txt"], &["a.txt"]);
    rig.bench.host_surfaces_with(None);
    let intent = Some("the items".into());
    rig.ask(Actor::Human, "ide_surface.intent", "set_value", intent);
    let asked = rig.ask(Actor::Human, "ide_surface.propose", "propose", None);
    assert!(completed(&rig.receipt(asked)).is_some());
    let surface = rig.bench.surface_open().expect("open").to_string();
    let c = frame(&mut rig);
    let ws = (rig.bench.workspace().vw, rig.bench.workspace().vh);
    assert!(
        (ws.0 - W / 2.0).abs() <= 16.0,
        "the column takes half: {ws:?}"
    );
    let prefix = format!("{surface}.");
    let shown: Vec<[f32; 4]> = c
        .bindings()
        .filter(|(k, _)| k.as_str().starts_with(&prefix))
        .filter_map(|(_, b)| b.visible_rect())
        .collect();
    assert!(!shown.is_empty(), "the surface is drawn");
    for [x, y, w, h] in shown {
        assert!(x >= ws.0 && y >= 0.0, "in the column: {x} {y}");
        assert!(x + w <= W + 0.5 && y + h <= ws.1 + 0.5, "above the strip");
    }
    for control in LINE {
        assert!(inside(rect(&c, control), ws.1, (W, H)), "{control}");
    }
    assert!(
        click(&mut rig, &c, "ide_surface.close"),
        "the person closes it"
    );
    frame(&mut rig);
    assert_eq!(rig.bench.surface_open(), None);
    assert_eq!(rig.bench.workspace().vw, W, "the width comes back");
    assert_eq!(rig.bench.workspace().vh, ws.1, "the strip stays");
}

/// The person reaches the line as before: a click in the field gives it
/// the keyboard, the keys write the intent, Enter proposes as her, Escape
/// lets the keyboard go.
#[test]
fn the_person_reaches_the_line_by_the_pointer_and_the_keys() {
    let mut rig = Rig::new(&["a.txt", "b.txt"], &["a.txt"]);
    rig.bench.host_surfaces_with(None);
    let c = frame(&mut rig);
    assert!(!rig.bench.has_keyboard());
    assert!(
        click(&mut rig, &c, "ide_surface.intent"),
        "the field takes it"
    );
    assert!(rig.bench.has_keyboard(), "the keys are the field's");
    for ch in "the items".chars() {
        assert!(rig.bench.type_text(&ch.to_string()));
    }
    assert_eq!(
        rig.value("ide_surface.intent:text"),
        Value::Text("the items".into())
    );
    assert!(rig.bench.enter(), "Enter proposes");
    assert!(rig.bench.surface_open().is_some(), "a surface opened");
    let message = rig.value("ide_surface.status:message");
    assert_eq!(
        message,
        Value::Text("proposed by the rules for \"the items\"".into())
    );
    assert!(rig.bench.escape(), "Escape lets the keyboard go");
    assert!(!rig.bench.has_keyboard());
}

/// The person's clipboard keys in the intent field (task-74): select all
/// and copy, cut, paste on one line, on a `LocalClipboard`; with no field
/// focused nothing takes them.
#[test]
fn the_intent_field_takes_the_clipboard_keys() {
    use engine::widgets::form::ClipKey;
    use engine::runtime::platform::clipboard::{ClipboardProvider, LocalClipboard};

    let mut rig = Rig::new(&["a.txt"], &[]);
    rig.bench.host_surfaces_with(None);
    let mut board = LocalClipboard::new();
    board.set_text("held");
    assert!(
        !rig.bench.clip(ClipKey::Paste, &mut board),
        "nobody has the keys"
    );
    let c = frame(&mut rig);
    assert!(click(&mut rig, &c, "ide_surface.intent"));
    for ch in "the items".chars() {
        assert!(rig.bench.type_text(&ch.to_string()));
    }
    assert!(rig.bench.clip(ClipKey::SelectAll, &mut board));
    assert!(rig.bench.clip(ClipKey::Copy, &mut board));
    assert_eq!(board.get_text().as_deref(), Some("the items"));
    assert!(rig.bench.clip(ClipKey::Cut, &mut board));
    assert_eq!(rig.value("ide_surface.intent:text"), Value::Text("".into()));
    board.set_text("staged\nfiles");
    assert!(rig.bench.clip(ClipKey::Paste, &mut board));
    assert_eq!(
        rig.value("ide_surface.intent:text"),
        Value::Text("staged files".into())
    );
}

/// The proposer's panel opens upward from the strip, inside the window,
/// and an option there is chosen by the person's click.
#[test]
fn the_proposers_panel_opens_upward_inside_the_window() {
    let mut rig = Rig::new(&["a.txt"], &[]);
    rig.bench.host_surfaces_with(None);
    let c = frame(&mut rig);
    let top = rig.bench.workspace().vh;
    let control = rect(&c, "ide_surface.proposer");
    assert!(click(&mut rig, &c, "ide_surface.proposer"), "it opens");
    assert_eq!(
        rig.value("ide_surface.proposer:expanded"),
        Value::Bool(true)
    );
    let open = frame(&mut rig);
    let option = rect(&open, "ide_surface.proposer[0]");
    assert!(
        option[1] >= 0.0 && option[1] + option[3] <= control[1],
        "the option above the control: {option:?} {control:?}"
    );
    assert!(option[1] + option[3] <= top + 0.5, "over the workspace");
    assert!(click(&mut rig, &open, "ide_surface.proposer[0]"));
    assert_eq!(
        rig.value("ide_surface.proposer:expanded"),
        Value::Bool(false),
        "chosen, it closes"
    );
    assert_eq!(
        rig.value("ide_surface.proposer:selected"),
        Value::Text("rules".into())
    );
}

/// In a narrower window the line wraps inside its strip and the
/// workspace still has the whole width: the message beside the controls
/// on a wide window, under them on a middle one, and the controls under
/// the field on a narrow one.
#[test]
fn the_line_wraps_in_its_strip_and_the_workspace_keeps_the_width() {
    let mut rig = Rig::new(&["a.txt"], &[]);
    rig.bench.host_surfaces_with(None);
    let intent = Some("zzz".into());
    rig.ask(Actor::Human, "ide_surface.intent", "set_value", intent);
    rig.ask(Actor::Human, "ide_surface.propose", "propose", None);
    for (w, form) in [(W, "beside"), (800.0, "under"), (480.0, "stacked")] {
        rig.bench.resize(w, H);
        let c = frame(&mut rig);
        let ws = rig.bench.workspace();
        assert_eq!(ws.vw, w, "{form}: the whole width");
        let top = ws.vh;
        for control in LINE {
            let r = rect(&c, control);
            assert!(inside(r, top, (w, H)), "{form}: {control} {r:?}");
        }
        let field = rect(&c, "ide_surface.intent");
        let proposer = rect(&c, "ide_surface.proposer");
        let message = rect(&c, "ide_surface.status");
        let below = |r: [f32; 4]| r[1] >= field[1] + field[3];
        match form {
            "beside" => {
                assert!(!below(proposer) && !below(message), "one line");
                assert!(message[0] >= proposer[0] + proposer[2], "after them");
            }
            "under" => assert!(!below(proposer) && below(message), "{message:?}"),
            _ => assert!(below(proposer) && below(message), "{proposer:?}"),
        }
    }
}

/// Runs `agent` against `rig`'s workbench through a port in a temp dir,
/// this thread playing the ide's shell: it serves the port after its wake
/// and after every worker event, and draws after every act. What the
/// agent returned.
fn shell<T: Send + 'static>(rig: &mut Rig, agent: impl FnOnce(PathBuf) -> T + Send + 'static) -> T {
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
    let agent = thread::spawn(move || agent(path));
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
    }
    agent.join().expect("the agent ran")
}

fn look(c: &mut Client) -> SnapshotReply {
    let observe = Observe {
        bounds: true,
        ..Observe::default()
    };
    match c.observe(observe).expect("an observation") {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot: {other:?}"),
    }
}

/// The agent acts `action` on `target`, with `param`, from what it saw.
fn act(c: &mut Client, target: &str, action: &str, param: Option<Value>) -> Vec<StageReply> {
    let seen = look(c);
    let action = ActionName::parse(action).expect("an action");
    let mut act = Act::from_revision(id(target), action, seen.revision);
    if let Some(p) = param {
        act = act.with_param(p);
    }
    c.act(act, |_| {}).expect("the act's stages")
}

/// An agent through the port sees the line where it is drawn, in the
/// strip under the workspace, and writes, proposes and closes through it,
/// each act its own request through the one dispatcher.
#[test]
fn an_agent_reaches_the_line_through_the_port() {
    let mut rig = Rig::new(&["a.txt", "b.txt"], &["a.txt"]);
    rig.bench.host_surfaces_with(None);
    frame(&mut rig);
    let top = rig.bench.workspace().vh;
    let (seen, stages, opened) = shell(&mut rig, |path| {
        let mut c = Client::connect(&path).expect("the agent connects");
        c.set_timeout(Some(20_000)).expect("a timeout");
        c.hello("line-agent").expect("welcome");
        let seen = look(&mut c);
        let mut stages = Vec::new();
        stages.push(act(
            &mut c,
            "ide_surface.intent",
            "set_value",
            Some(Value::Text("the items".into())),
        ));
        stages.push(act(&mut c, "ide_surface.propose", "propose", None));
        let opened = look(&mut c)
            .nodes
            .iter()
            .any(|n| n.id.as_str() == "ide_surface.close" && n.enabled);
        stages.push(act(&mut c, "ide_surface.close", "close", None));
        (seen, stages, opened)
    });
    let bounds = seen.bounds.expect("the observation says where");
    for control in LINE {
        let b = bounds
            .iter()
            .find(|b| b.id.as_str() == control)
            .unwrap_or_else(|| panic!("{control} in the bounds"));
        assert!(inside(b.rect, top, (W, H)), "{control}: {:?}", b.rect);
    }
    for (n, act) in stages.iter().enumerate() {
        let last = act.last().expect("a stage");
        assert!(matches!(last.stage, Stage::Completed(_)), "{n}: {act:?}");
    }
    assert!(opened, "the agent's proposal opened a surface");
    frame(&mut rig);
    assert_eq!(rig.bench.surface_open(), None, "and its close closed it");
    assert_eq!(rig.bench.workspace().vw, W);
}
