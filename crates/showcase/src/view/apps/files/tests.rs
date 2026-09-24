//! The Files page as a capability (task-76, item 3): what it projects
//! (the dialog for a person, the drop for anyone, the files received), a
//! drop through the engine's event becoming a request by the person, the
//! button asking the fake dialog and showing what it picked, an agent
//! refused the dialog and allowed the drop (in process, and through the
//! real port), two viewports, the chrome's routes, and the golden of the
//! projection. No test opens a native dialog: every page here is over
//! the fake, and the gallery's page is over the fake in the crate's
//! tests by construction (`FilesPage::new`).

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use comps::prelude::{Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::input::FileDrop;
use engine::platform::files::{DialogKind, FakeDialogs, NoDialogs};
use engine::theme::Theme;
use sem::{
    ActionName, Actor, Rejection, Request, RequestId, Role, SemId, Stage, StateRef, Value,
    ValueKind,
};
use wire::{Act, AgentPort, Client, Config, Observe, Reply};

use super::FilesPage;
use super::semantics::{Ids, OPEN_FILE_ACTORS};
use crate::view::apps::app::TodoPage;
use crate::view::{EditKey, ShowcaseView};

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

fn wide() -> Rect {
    Rect::new(288.0, 118.0, 900.0, 682.0)
}

fn narrow() -> Rect {
    Rect::new(16.0, 118.0, 358.0, 682.0)
}

type Shared = Arc<Mutex<FakeDialogs>>;

/// The page over a shared fake: the test keeps the other handle.
fn setup(fake: FakeDialogs) -> (TodoPage, FilesPage, Theme, Shared) {
    let shared = Arc::new(Mutex::new(fake));
    let page = FilesPage::with(Box::new(shared.clone()));
    (TodoPage::new(), page, Theme::hoff_core(), shared)
}

fn frame(page: &mut FilesPage, content: Rect, theme: &Theme) -> Compositor {
    let mut c = Compositor::new();
    page.render(&mut c, content, theme);
    c
}

fn center(c: &Compositor, at: &str) -> (f32, f32) {
    let b = c
        .binding(&sid(at))
        .unwrap_or_else(|| panic!("{at} is bound"));
    let [x, y, w, h] = b.rect;
    (x + w / 2.0, y + h / 2.0)
}

fn click(
    page: &mut FilesPage,
    todo: &mut TodoPage,
    content: Rect,
    theme: &Theme,
    (x, y): (f32, f32),
) {
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
        WidgetEvent::MouseUp { x, y },
    ] {
        page.handle_event(&event, content, theme, todo, false);
    }
}

/// `files:received` as projected now.
fn received(page: &FilesPage) -> Vec<String> {
    let snap = page.snapshot().expect("the page projects");
    match snap.value(&StateRef::parse("files:received").unwrap()) {
        Some(Value::List(paths)) => paths.clone(),
        other => panic!("a list: {other:?}"),
    }
}

fn message(page: &FilesPage) -> String {
    let snap = page.snapshot().unwrap();
    match snap.value(&StateRef::parse("files.status:message").unwrap()) {
        Some(Value::Text(text)) => text.clone(),
        other => panic!("a text: {other:?}"),
    }
}

fn stage(todo: &TodoPage, id: u64) -> Stage {
    todo.receipt(RequestId::parse(&id.to_string()).unwrap())
        .unwrap_or_else(|| panic!("receipt {id}"))
        .stage()
        .unwrap()
        .clone()
}

fn dropped(paths: &[&str], position: Option<(f32, f32)>) -> FileDrop {
    FileDrop::Dropped {
        paths: paths.iter().map(|s| p(s)).collect(),
        position,
    }
}

#[test]
fn the_page_projects_the_dialog_for_a_person_and_the_drop_for_anyone() {
    assert!(Ids::new().is_ok(), "the constant ids parse");
    let (_, page, _, _) = setup(FakeDialogs::new());
    let snap = page.snapshot().unwrap();
    let node = |text: &str| snap.get(&sid(text)).unwrap();

    let files = node("files");
    assert_eq!(files.role, Role::Region);
    assert_eq!(files.state.get("received"), Some(&Value::List(vec![])));
    assert_eq!(files.state.get("count"), Some(&Value::Int(0)));
    assert!(!files.inert);

    let zone = node("files.drop");
    assert_eq!(zone.role, Role::Region);
    assert_eq!(zone.label, "drop files here");
    assert_eq!(zone.actions.len(), 1);
    let drop = &zone.actions[0];
    assert_eq!(drop.name.as_str(), "drop_files");
    assert_eq!(drop.param.map(|p| p.kind), Some(ValueKind::List));
    assert_eq!(drop.reads, [StateRef::parse("files:received").unwrap()]);
    assert!(drop.policy.actors.agent, "anyone drops files");
    assert!(drop.policy.actors.human);

    let open = node("files.open");
    assert_eq!(open.role, Role::Button);
    assert_eq!(open.label, "open file");
    assert!(open.focusable);
    assert_eq!(open.actions.len(), 1);
    let action = &open.actions[0];
    assert_eq!(action.name.as_str(), "open_file");
    assert_eq!(action.param, None);
    assert_eq!(action.reads, [StateRef::parse("files:received").unwrap()]);
    assert_eq!(action.policy.actors, OPEN_FILE_ACTORS);
    assert!(!action.policy.actors.agent, "a native dialog is a person's");
    assert!(action.policy.actors.human && action.policy.actors.assistive);
    assert!(action.policy.actors.test, "the test drives the fake");

    let status = node("files.status");
    assert_eq!(status.role, Role::Status);
    assert_eq!(
        status.state.get("message"),
        Some(&Value::from(
            "no files yet: drop some on the window, or open one"
        ))
    );
    assert_eq!(snap.focused(), None);
}

/// The engine's drop event becomes `drop_files` by the person, through
/// the gallery's dispatcher, and the list shows the files: with no
/// position (the window as a whole), over the zone, never outside it;
/// the hover lights the zone and a cancel ends it; an inert page refuses.
#[test]
fn a_drop_on_the_window_is_a_request_by_the_person_and_the_list_shows_it() {
    let (mut todo, mut page, theme, fake) = setup(FakeDialogs::new());
    let content = wide();

    let hover = FileDrop::Hovered {
        paths: vec![p("/a.urna")],
        position: None,
    };
    let r = page.handle_file_drop(&hover, content, &theme, &mut todo, false);
    assert!(r.handled && r.changed, "the zone lights up");
    let r = page.handle_file_drop(&hover, content, &theme, &mut todo, false);
    assert!(r.handled && !r.changed, "already lit");
    let r = page.handle_file_drop(&FileDrop::Cancelled, content, &theme, &mut todo, false);
    assert!(r.handled && r.changed, "the zone goes dark");
    assert!(received(&page).is_empty());

    // no position (what winit 0.30 sends): the window is the zone
    page.handle_file_drop(&hover, content, &theme, &mut todo, false);
    let r = page.handle_file_drop(
        &dropped(&["/a.urna", "/b.png"], None),
        content,
        &theme,
        &mut todo,
        false,
    );
    assert!(r.changed);
    assert_eq!(received(&page), ["/a.urna", "/b.png"]);
    assert_eq!(message(&page), "2 received now, 2 in all");
    let receipt = todo.receipt(RequestId::parse("1").unwrap()).unwrap();
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(receipt.request.target, sid("files.drop"));
    assert_eq!(receipt.request.action.as_str(), "drop_files");
    assert_eq!(
        receipt.request.param,
        Some(Value::List(vec!["/a.urna".into(), "/b.png".into()]))
    );
    let Stage::Completed(outcome) = stage(&todo, 1) else {
        panic!("{:?}", stage(&todo, 1));
    };
    assert_eq!(outcome.result.get("received"), Some(&Value::Int(2)));
    assert_eq!(outcome.result.get("count"), Some(&Value::Int(2)));

    // released outside the zone: not received, the hover ends
    page.handle_file_drop(&hover, content, &theme, &mut todo, false);
    let zone = page.zone_rect(content, &theme);
    let outside = (zone.x + zone.w / 2.0, zone.bottom() + theme.spacing.xl);
    let r = page.handle_file_drop(
        &dropped(&["/c.txt"], Some(outside)),
        content,
        &theme,
        &mut todo,
        false,
    );
    assert!(r.handled && r.changed, "the hover ended");
    assert_eq!(received(&page).len(), 2, "nothing received");
    assert!(todo.receipt(RequestId::parse("2").unwrap()).is_none());

    // released over the zone: received
    let inside = zone.center();
    let r = page.handle_file_drop(
        &dropped(&["/c.txt"], Some(inside)),
        content,
        &theme,
        &mut todo,
        false,
    );
    assert!(r.changed);
    assert_eq!(received(&page), ["/a.urna", "/b.png", "/c.txt"]);
    assert_eq!(message(&page), "1 received now, 3 in all");

    // an overlay holds the pointer: the page is inert, the drop refused
    let r = page.handle_file_drop(&dropped(&["/d"], None), content, &theme, &mut todo, true);
    assert!(r.handled);
    assert_eq!(received(&page).len(), 3);
    assert!(matches!(
        stage(&todo, 3),
        Stage::Rejected(Rejection::Inert { .. })
    ));

    // an empty drop fails, saying why
    let r = page.handle_file_drop(&dropped(&[], None), content, &theme, &mut todo, false);
    assert!(r.handled);
    assert!(
        matches!(stage(&todo, 4), Stage::Failed { error } if error == "the drop carries no file")
    );

    assert!(
        fake.lock().unwrap().asked().is_empty(),
        "a drop asks no dialog"
    );
    // the zone is what the page binds for the drop
    let c = frame(&mut page, content, &theme);
    let [x, y, w, h] = c.binding(&sid("files.drop")).unwrap().rect;
    assert_eq!(Rect::new(x, y, w, h), zone);
}

/// The button asks the dialog (the fake here) through `open_file` by the
/// person: what it picks is received, a cancel is a completed request
/// that changed nothing, a target without dialogs fails the request and
/// the status says so. The keyboard reaches the button: Tab, Enter,
/// Escape.
#[test]
fn the_button_asks_the_dialog_and_shows_what_it_picked() {
    let (mut todo, mut page, theme, fake) = setup(
        FakeDialogs::new()
            .will_pick("/home/me/notes.md")
            .will_cancel()
            .will_refuse(),
    );
    let content = wide();
    let c = frame(&mut page, content, &theme);
    let button = center(&c, "files.open");

    click(&mut page, &mut todo, content, &theme, button);
    assert_eq!(received(&page), ["/home/me/notes.md"]);
    assert_eq!(message(&page), "1 received now, 1 in all");
    let receipt = todo.receipt(RequestId::parse("1").unwrap()).unwrap();
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(receipt.request.target, sid("files.open"));
    assert_eq!(receipt.request.action.as_str(), "open_file");
    let Stage::Completed(outcome) = stage(&todo, 1) else {
        panic!("{:?}", stage(&todo, 1));
    };
    assert_eq!(
        outcome.result.get("path"),
        Some(&Value::from("/home/me/notes.md"))
    );

    click(&mut page, &mut todo, content, &theme, button);
    assert_eq!(received(&page).len(), 1, "cancelled: nothing received");
    assert_eq!(message(&page), "the dialog was cancelled");
    let Stage::Completed(outcome) = stage(&todo, 2) else {
        panic!("{:?}", stage(&todo, 2));
    };
    assert_eq!(outcome.result.get("cancelled"), Some(&Value::Bool(true)));

    // Tab takes the focus to the button, Enter presses it
    assert!(page.handle_edit_key(EditKey::Tab));
    assert_eq!(page.snapshot().unwrap().focused(), Some(&sid("files.open")));
    assert!(page.handle_enter(&mut todo, false));
    assert!(matches!(
        stage(&todo, 3),
        Stage::Failed { error } if error == "pick_file: no native file dialog on this target"
    ));
    assert_eq!(message(&page), "no file dialog on this target");
    assert!(page.handle_escape(), "escape lets the focus go");
    assert_eq!(page.snapshot().unwrap().focused(), None);
    assert!(!page.handle_escape(), "nothing to let go");
    assert!(!page.handle_enter(&mut todo, false), "no focus, no press");

    let asked = fake.lock().unwrap().take_asked();
    assert_eq!(asked.len(), 3);
    for ask in &asked {
        assert_eq!(ask.kind, DialogKind::PickFile);
        assert_eq!(ask.request.title.as_deref(), Some("Open a file"));
        assert!(ask.request.filters.is_empty());
    }

    // a target without any dialog (wasm, mobile): the same failure
    page.set_dialogs(Box::new(NoDialogs));
    click(&mut page, &mut todo, content, &theme, button);
    assert!(matches!(stage(&todo, 4), Stage::Failed { .. }));
    assert_eq!(received(&page).len(), 1);
}

/// An agent may drop files (the same capability as the person's drop)
/// and is refused the dialog before anything runs: the fake is never
/// asked. A test actor is allowed (over the fake).
#[test]
fn an_agent_drops_files_and_is_refused_the_dialog() {
    let (mut todo, mut page, theme, fake) = setup(FakeDialogs::new().will_pick("/never"));
    let agent = Actor::Agent {
        name: "claude".to_owned(),
    };
    let seen = page.snapshot().unwrap();
    let req = Request::observe(&seen, agent.clone(), "files.open", "open_file", None).unwrap();
    let receipt = page.act(req, &mut todo, false).expect("a receipt");
    assert!(
        matches!(receipt.stage(), Some(Stage::Rejected(Rejection::NotAuthorized { actor })) if *actor == agent),
        "{:?}",
        receipt.stage()
    );
    assert!(fake.lock().unwrap().asked().is_empty(), "never asked");
    assert!(received(&page).is_empty());

    let seen = page.snapshot().unwrap();
    let req = Request::observe(
        &seen,
        agent.clone(),
        "files.drop",
        "drop_files",
        Some(Value::List(vec!["/agent/dropped.csv".into()])),
    )
    .unwrap();
    let receipt = page.act(req, &mut todo, false).unwrap();
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(received(&page), ["/agent/dropped.csv"]);
    assert_eq!(receipt.request.actor, agent);

    // stale: the agent observed the list before the drop landed
    let req = Request::observe(
        &seen,
        agent,
        "files.drop",
        "drop_files",
        Some(Value::List(vec!["/late".into()])),
    )
    .unwrap();
    let receipt = page.act(req, &mut todo, false).unwrap();
    assert!(matches!(
        receipt.stage(),
        Some(Stage::Rejected(Rejection::Stale { .. }))
    ));
    assert_eq!(received(&page).len(), 1);

    // a test actor may ask; the fake answers
    let seen = page.snapshot().unwrap();
    let tester = Actor::Test {
        name: "files".to_owned(),
    };
    let req = Request::observe(&seen, tester, "files.open", "open_file", None).unwrap();
    let receipt = page.act(req, &mut todo, false).unwrap();
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(received(&page), ["/agent/dropped.csv", "/never"]);
    let _ = theme;
}

/// The page renders and binds at a wide and a narrow content rect, with
/// and without files: the zone and the button inside the card, the card
/// growing with the files, the height the chrome scrolls by.
#[test]
fn two_viewports_render_and_bind_the_page() {
    let (mut todo, mut page, theme, _) = setup(FakeDialogs::new());
    for content in [wide(), narrow()] {
        let empty_h = page.content_height(content, &theme);
        let c = frame(&mut page, content, &theme);
        let rect = |at: &str| {
            let [x, y, w, h] = c.binding(&sid(at)).unwrap().rect;
            Rect::new(x, y, w, h)
        };
        let card = rect("files");
        let (zone, open, status) = (rect("files.drop"), rect("files.open"), rect("files.status"));
        assert!(
            card.w <= content.w && card.x >= content.x,
            "{card:?} in {content:?}"
        );
        assert_eq!(card.h, empty_h, "the height is the card's");
        for (name, r) in [("zone", zone), ("open", open), ("status", status)] {
            assert!(
                r.x >= card.x && r.right() <= card.right() + 0.5,
                "{name} {r:?} in {card:?}"
            );
            assert!(
                r.y >= card.y && r.bottom() <= card.bottom() + 0.5,
                "{name} {r:?} in {card:?}"
            );
        }
        assert!(zone.bottom() <= open.y, "the button is under the zone");
        assert!(status.x >= open.right(), "the status is beside the button");

        page.handle_file_drop(
            &dropped(&["/one", "/two", "/three"], None),
            content,
            &theme,
            &mut todo,
            false,
        );
        let full_h = page.content_height(content, &theme);
        assert!(full_h > empty_h, "three files grow the card");
        let c = frame(&mut page, content, &theme);
        let [_, _, _, h] = c.binding(&sid("files")).unwrap().rect;
        assert_eq!(h, full_h);
        // back to empty for the next viewport
        page.received.clear();
        page.said = super::Said::Nothing;
    }
}

/// The chrome's routes: the gallery on the Files page projects it, hands
/// it the engine's drop event, routes an act and a focus, and draws it;
/// another page takes no files.
#[test]
fn the_gallery_routes_the_page() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("todo");
    assert!(
        !view.handle_file_drop(&dropped(&["/x"], None)),
        "the todo page takes no files"
    );
    view.jump_to_page("files");
    assert_eq!(view.page_title(), "Files");
    let snap = view.semantics().expect("the page projects");
    assert!(snap.get(&sid("files.open")).is_some());

    assert!(view.handle_file_drop(&FileDrop::Hovered {
        paths: vec![p("/x")],
        position: None,
    }));
    assert!(view.handle_file_drop(&dropped(&["/x", "/y"], None)));
    let snap = view.semantics().unwrap();
    assert_eq!(
        snap.value(&StateRef::parse("files:count").unwrap()),
        Some(&Value::Int(2))
    );

    let req = Request::observe(
        &snap,
        Actor::Assistive,
        "files.drop",
        "drop_files",
        Some(Value::List(vec!["/z".into()])),
    )
    .unwrap();
    let receipt = view.act(req).expect("the page takes it");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(
        view.semantics()
            .unwrap()
            .value(&StateRef::parse("files:received").unwrap()),
        Some(&Value::List(vec!["/x".into(), "/y".into(), "/z".into()]))
    );

    assert!(view.assist_focus(&sid("files.open")));
    assert_eq!(
        view.semantics().unwrap().focused(),
        Some(&sid("files.open"))
    );
    assert!(view.handle_edit_key(EditKey::Tab), "tab lets the focus go");
    assert_eq!(view.semantics().unwrap().focused(), None);
    assert!(view.handle_edit_key(EditKey::Tab));
    assert!(view.close_top_overlay(), "escape lets the focus go");
    assert!(!view.handle_edit_key(EditKey::Backspace), "nothing to edit");

    // the gallery's page is over the fake in the crate's tests: enter on
    // the focused button asks it, and it cancels
    assert!(view.handle_edit_key(EditKey::Tab));
    assert!(view.handle_enter());
    assert_eq!(
        view.semantics()
            .unwrap()
            .value(&StateRef::parse("files.status:message").unwrap()),
        Some(&Value::from("the dialog was cancelled"))
    );

    let mut c = Compositor::new();
    view.render(&mut c);
    for id in ["files", "files.drop", "files.open", "files.status"] {
        assert!(c.binding(&sid(id)).is_some(), "{id} is bound");
    }
    assert!(!view.tick(TURN), "at rest the page asks for nothing");
}

/// The page's projection is the golden of `crates/uir/fixtures`, the
/// registry a generated surface would verify against: the page as it
/// opens, empty, settled as the headless staging settles it.
/// `UIR_BLESS=1` writes it again.
#[test]
fn the_page_projection_is_the_uir_golden() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("files");
    let mut c = Compositor::new();
    view.render(&mut c);
    let mut steps = 0;
    while view.tick(TURN) {
        view.render(&mut c);
        steps += 1;
        assert!(steps < 120, "at rest the page asks for nothing");
    }
    let snap = view.panes.files.snapshot().expect("the page projects");
    let mut text = crate::app::semantics_json(&snap, false).expect("json prints");
    text.push('\n');
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uir/fixtures/showcase-files.snapshot.json");
    if std::env::var_os("UIR_BLESS").is_some() {
        std::fs::write(&path, text).expect("the golden writes");
        return;
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        kept == text,
        "the projection moved away from the golden; regenerate it with \
         UIR_BLESS=1 cargo test -p showcase --lib files::tests::the_page_projection"
    );
}

// -- through the real port ---------------------------------------------------

/// A private directory for the test's socket, removed after.
struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn gallery_frame(view: &mut ShowcaseView) -> Compositor {
    let mut c = Compositor::new();
    view.render(&mut c);
    c
}

/// Runs `agent` against the gallery on the Files page, serving the port
/// the way the shell does (the pattern of the gallery's agent tests):
/// the wake, `serve_agent`, a tick, a new frame after an act.
fn gallery<T: Send + 'static>(
    dir: &Path,
    agent: impl FnOnce(PathBuf) -> T + Send + 'static,
) -> (T, ShowcaseView) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("files");
    assert_eq!(view.page_title(), "Files");
    let mut c = gallery_frame(&mut view);
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
            c = gallery_frame(&mut view);
        }
    }
    (agent.join().unwrap(), view)
}

fn snapshot(reply: Reply) -> wire::SnapshotReply {
    match reply {
        Reply::Snapshot(s) => s,
        other => panic!("a snapshot: {other:?}"),
    }
}

fn state<'a>(reply: &'a wire::SnapshotReply, id: &str, key: &str) -> Option<&'a Value> {
    reply
        .nodes
        .iter()
        .find(|n| n.id == sid(id))
        .and_then(|n| n.state.get(key))
}

/// The acceptance through the port: the agent observes the files (the
/// list, the zone and the button with where they sit), is refused the
/// dialog (a receipt, rejected, not an error), drops files from what it
/// saw, and the page shows them as the agent's act.
#[test]
fn an_agent_through_the_port_reads_the_files_and_is_refused_the_dialog() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t76-{}-files", std::process::id())));
    let (seen, view) = gallery(&dir.0, |path| {
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();
        let first = snapshot(
            c.observe(Observe {
                scope: Some(sid("files")),
                bounds: true,
                ..Observe::default()
            })
            .unwrap(),
        );
        let refused = c
            .act(
                Act::from_revision(
                    sid("files.open"),
                    ActionName::parse("open_file").unwrap(),
                    first.revision,
                ),
                |_| {},
            )
            .map(|stages| {
                stages
                    .into_iter()
                    .map(|s| s.stage.name())
                    .collect::<Vec<_>>()
            });
        let dropped = c
            .act(
                Act::from_revision(
                    sid("files.drop"),
                    ActionName::parse("drop_files").unwrap(),
                    first.revision,
                )
                .with_param(Value::List(vec!["/from/the/agent.wav".into()])),
                |_| {},
            )
            .unwrap()
            .into_iter()
            .map(|s| s.stage.name())
            .collect::<Vec<_>>();
        let after = snapshot(c.observe(Observe::default()).unwrap());
        (first, refused, dropped, after)
    });
    let (first, refused, dropped, after) = seen;

    let ids: Vec<&str> = first.nodes.iter().map(|n| n.id.as_str()).collect();
    for id in ["files", "files.drop", "files.open", "files.status"] {
        assert!(ids.contains(&id), "{id} in {ids:?}");
    }
    assert_eq!(state(&first, "files", "count"), Some(&Value::Int(0)));
    let bounds = first.bounds.expect("bounds");
    assert!(
        bounds
            .iter()
            .any(|b| b.id == sid("files.drop") && b.rect[2] > 0.0)
    );
    assert!(bounds.iter().any(|b| b.id == sid("files.open")));

    match refused {
        Ok(stages) => assert_eq!(stages, ["requested", "rejected"]),
        Err(e) => panic!("the refusal is a receipt, not an error: {e:?}"),
    }
    assert_eq!(dropped, ["requested", "accepted", "executing", "completed"]);
    assert_eq!(
        state(&after, "files", "received"),
        Some(&Value::List(vec!["/from/the/agent.wav".into()]))
    );

    // the gallery's dispatcher kept both as the agent's
    let refusal = view
        .panes
        .todo
        .receipt(RequestId::parse("1").unwrap())
        .expect("the refused open");
    assert_eq!(refusal.request.action.as_str(), "open_file");
    assert!(matches!(
        refusal.stage(),
        Some(Stage::Rejected(Rejection::NotAuthorized { actor: Actor::Agent { name } })) if name == "claude"
    ));
    let drop = view
        .panes
        .todo
        .receipt(RequestId::parse("2").unwrap())
        .expect("the drop");
    assert_eq!(
        drop.request.actor,
        Actor::Agent {
            name: "claude".into()
        }
    );
    assert_eq!(drop.request.target, sid("files.drop"));
}
