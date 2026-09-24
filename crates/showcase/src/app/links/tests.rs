//! The gallery's links over a bare main window and a fake source: a page
//! link goes through the dispatcher as the agent `link`; what is not open
//! to links is refused before any request, with the line shown; what the
//! list lets through is still checked as an agent's; a link that comes
//! asleep waits for the wake; the launch argument is a link and never the
//! page title; a recorded session keeps it as an agent's request, with no
//! input.

use std::sync::{Arc, Mutex};

use engine::platform::capture::{DeviceKind, FakeCapture};
use engine::platform::links::{FakeLinks, LaunchLinks, read};
use engine::platform::menu::locked;
use engine::window::Background;
use rec::{Kind, Limits};
use sem::{Actor, Receipt, ReceiptSink, Rejection, Stage, StateRef, Value};
use winit::window::WindowId;

use super::super::App;
use super::super::record::Recording;
use super::{OPEN, SCHEME, follow};
use crate::view::ShowcaseView;

/// Every stage the gallery's dispatcher records: who asked, what, which
/// stage.
type Seen = Arc<Mutex<Vec<(Actor, String, &'static str)>>>;

struct Stages(Seen);

impl ReceiptSink for Stages {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        locked(&self.0).push((
            receipt.request.actor.clone(),
            receipt.request.target.to_string(),
            stage.name(),
        ));
    }
}

fn link_actor() -> Actor {
    Actor::Agent {
        name: "link".to_owned(),
    }
}

/// The shell on `page` with a bare main window, a fake link source, and
/// an ear on the gallery's dispatcher.
fn shell(page: &str) -> (App, Arc<Mutex<FakeLinks>>, Seen) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page(page);
    view.resize(1200.0, 800.0, 1.0);
    let seen: Seen = Arc::default();
    view.add_sink(Box::new(Stages(seen.clone())));
    let mut app = App::with_bare_window(view);
    let fake = Arc::new(Mutex::new(FakeLinks::new(SCHEME)));
    app.set_links(Box::new(fake.clone()));
    frame(&mut app);
    (app, fake, seen)
}

/// A frame built for the main window: the pending ask taken.
fn frame(app: &mut App) {
    let host = app.windows.get_mut(WindowId::dummy()).unwrap();
    host.begin_frame();
    app.view.render(host.compositor_mut());
}

/// A frame was asked of the main window.
fn asked(app: &App) -> bool {
    app.windows.get(WindowId::dummy()).unwrap().redraw_asked()
}

fn open(fake: &Arc<Mutex<FakeLinks>>, url: &str) {
    locked(fake).open(url);
}

/// What the shell told on its own, as an agent reads it.
fn notice(app: &App) -> String {
    let gallery = app.view.gallery_semantics().expect("the gallery projects");
    match gallery.value(&StateRef::parse("shell:notice").unwrap()) {
        Some(Value::Text(text)) => text.clone(),
        other => panic!("shell:notice is {other:?}"),
    }
}

/// The toasts on screen, oldest first.
fn toasts(app: &App) -> Vec<String> {
    app.view
        .toasts
        .visible()
        .map(|t| t.message.clone())
        .collect()
}

#[test]
fn a_page_link_goes_to_the_page_through_the_dispatcher_as_the_agent_link() {
    let (mut app, fake, seen) = shell("Todo");
    assert!(!asked(&app), "at rest");
    open(&fake, "plev-showcase://shell.page/select?value=Files");
    app.links_arrived();
    assert_eq!(app.view.page_title(), "Files");
    assert_eq!(
        *locked(&seen),
        [
            (link_actor(), "shell.page".to_owned(), "requested"),
            (link_actor(), "shell.page".to_owned(), "accepted"),
            (link_actor(), "shell.page".to_owned(), "executing"),
            (link_actor(), "shell.page".to_owned(), "completed"),
        ],
        "one request, by the agent link, never the person"
    );
    assert!(asked(&app), "the page it went to is drawn");
    assert_eq!(notice(&app), "", "a link that ran tells nothing more");
    assert!(toasts(&app).is_empty());
    assert_eq!(locked(&fake).pending(), 0);

    // the theme, by its name; the title of a page in any ASCII case, as
    // the sidebar's names are matched
    open(&fake, "plev-showcase://shell.theme/select?value=dark");
    open(
        &fake,
        "plev-showcase://shell.page/select?value=Data%20Table",
    );
    app.links_arrived();
    assert_eq!(app.view.theme_name, "dark");
    assert_eq!(app.view.page_title(), "Data Table");
    assert!(
        locked(&seen)
            .iter()
            .all(|(actor, _, _)| *actor == link_actor())
    );
    // nothing waiting: a poll with nothing asks for nothing
    frame(&mut app);
    app.links_arrived();
    assert!(!asked(&app));
}

/// The camera, the print, the Files dialog, a todo removed, an id that
/// does not exist, a text that is not a link: refused before any request,
/// nothing runs, and the line says the link and why, on screen and in
/// `shell:notice`.
#[test]
fn what_is_not_open_to_links_is_refused_nothing_runs_and_the_person_is_told() {
    let cases = [
        (
            "Capture",
            "plev-showcase://capture.camera/toggle",
            "link refused: plev-showcase://capture.camera/toggle: capture.camera toggle is not \
             open to links",
        ),
        (
            "Todo",
            "plev-showcase://shell.print/activate",
            "link refused: plev-showcase://shell.print/activate: shell.print activate is not \
             open to links",
        ),
        (
            "Files",
            "plev-showcase://files.open/open_file",
            "link refused: plev-showcase://files.open/open_file: files.open open_file is not \
             open to links",
        ),
        (
            "Todo",
            "plev-showcase://todo.item[1]/remove",
            "link refused: plev-showcase://todo.item[1]/remove: todo.item[1] remove is not open \
             to links",
        ),
        (
            "Todo",
            "plev-showcase://nope.nothing/select?value=x",
            "link refused: plev-showcase://nope.nothing/select?value=x: nope.nothing select is \
             not open to links",
        ),
        (
            "Todo",
            "plev-showcase://shell.page/select?value=Files#top",
            "link refused: plev-showcase://shell.page/select?value=Files#top: a link has no \
             fragment (#)",
        ),
        (
            "Todo",
            "plev-showcase://shell.page/select?page=Files",
            "link refused: plev-showcase://shell.page/select?page=Files: a link takes value= \
             alone, not \"page\"",
        ),
    ];
    for (page, url, line) in cases {
        let (mut app, fake, seen) = shell(page);
        let camera = Arc::new(Mutex::new(FakeCapture::new()));
        app.view.set_capture(Box::new(camera.clone()));
        app.capture_follows();
        let todo_before = app.view.semantics();
        open(&fake, url);
        app.links_arrived();
        assert!(locked(&seen).is_empty(), "{url}: no request at all");
        assert_eq!(app.view.page_title(), page, "{url}");
        assert_eq!(locked(&camera).starts(DeviceKind::Camera), 0, "{url}");
        assert_eq!(
            app.view.semantics(),
            todo_before,
            "{url}: the page as it was"
        );
        assert_eq!(notice(&app), line);
        assert_eq!(toasts(&app), [line], "{url}: shown on screen");
        assert!(asked(&app), "{url}: a frame shows the line");
    }
}

/// Inside the list, the dispatcher still checks the link as an agent's:
/// a value out of the options is refused; a capability for a person alone
/// is `NotAuthorized` whatever list an app would write.
#[test]
fn what_the_list_lets_through_is_still_checked_as_an_agents() {
    let (mut app, fake, seen) = shell("Todo");
    open(&fake, "plev-showcase://shell.theme/select?value=nope");
    app.links_arrived();
    assert_eq!(app.view.theme_name, "core");
    let stages: Vec<&str> = locked(&seen).iter().map(|s| s.2).collect();
    assert_eq!(stages, ["requested", "rejected"]);
    assert_eq!(
        notice(&app),
        "link refused: plev-showcase://shell.theme/select?value=nope: \"nope\" is not one of the \
         options"
    );

    // a list that let a person's capabilities through: the policy holds
    let careless = [
        ("shell.print", "activate"),
        ("files.open", "open_file"),
        ("capture.camera", "toggle"),
    ];
    for (page, url, target) in [
        (
            "Todo",
            "plev-showcase://shell.print/activate",
            "shell.print",
        ),
        (
            "Files",
            "plev-showcase://files.open/open_file",
            "files.open",
        ),
        (
            "Capture",
            "plev-showcase://capture.camera/toggle",
            "capture.camera",
        ),
    ] {
        let (mut app, _, seen) = shell(page);
        let camera = Arc::new(Mutex::new(FakeCapture::new()));
        app.view.set_capture(Box::new(camera.clone()));
        let followed = follow(&mut app.view, read(url, SCHEME), &careless, &mut |_, _| {});
        let receipt = followed.receipt.expect("dispatched");
        assert_eq!(
            receipt.stage(),
            Some(&Stage::Rejected(Rejection::NotAuthorized {
                actor: link_actor()
            })),
            "{url}"
        );
        assert_eq!(
            locked(&seen).iter().map(|s| s.2).collect::<Vec<_>>(),
            ["requested", "rejected"],
            "{url}: refused before it ran"
        );
        assert!(locked(&seen).iter().all(|s| s.1 == target));
        assert_eq!(locked(&camera).starts(DeviceKind::Camera), 0);
        assert_eq!(
            followed.told.as_deref(),
            Some(format!("link refused: {url}: agent:link may not ask this").as_str())
        );
    }
}

#[test]
fn a_link_that_comes_asleep_waits_for_the_wake() {
    let (mut app, fake, seen) = shell("Todo");
    app.sleep();
    assert!(Background::asleep());
    open(&fake, "plev-showcase://shell.page/select?value=Files");
    for _ in 0..3 {
        app.links_arrived();
    }
    assert_eq!(app.view.page_title(), "Todo", "not applied asleep");
    assert_eq!(locked(&fake).pending(), 1, "it waits in its source");
    assert!(locked(&seen).is_empty());
    assert!(!asked(&app), "no frame asleep");

    // the wake, then the poll the shell makes after it (`resumed`)
    app.wake();
    app.links_arrived();
    assert_eq!(app.view.page_title(), "Files");
    assert_eq!(locked(&fake).pending(), 0);
    assert!(locked(&seen).iter().any(|s| s.2 == "completed"));
    assert!(asked(&app));
}

/// A link among the launch arguments is not the page title the first
/// argument otherwise is, nor any other; the launch source delivers it
/// once the window is up, refused visibly when it is malformed.
#[test]
fn the_launch_argument_is_a_link_and_never_the_page_title() {
    let first = ShowcaseView::new(1200.0, 800.0).page_title();
    let link = "plev-showcase://shell.page/select?value=Files";
    let args = |list: &[&str]| -> Vec<String> { list.iter().map(|a| a.to_string()).collect() };

    let view = App::launch_view(args(&[link]).into_iter());
    assert_eq!(view.page_title(), first, "the link is no page title");
    let view = App::launch_view(args(&["Todo", link, "dark"]).into_iter());
    assert_eq!(view.page_title(), "Todo");
    assert_eq!(view.theme_name, "dark", "the link took no argument's place");

    let launched = args(&["Todo", link, "plev-showcase://shell.page"]);
    let mut view = App::launch_view(launched.clone().into_iter());
    view.resize(1200.0, 800.0, 1.0);
    let mut app = App::with_bare_window(view);
    app.set_links(Box::new(LaunchLinks::from_args(SCHEME, launched)));
    app.links_arrived();
    assert_eq!(app.view.page_title(), "Files");
    assert_eq!(
        notice(&app),
        "link refused: plev-showcase://shell.page: the link names no action"
    );
    // once
    frame(&mut app);
    app.links_arrived();
    assert!(!asked(&app));
}

/// The recording keeps the link as it keeps an agent: its look at the
/// gallery, then every stage of its request, by `agent:link`; no input,
/// since nothing was clicked or typed.
#[test]
fn a_recorded_session_keeps_the_link_as_an_agents_request_and_no_input() {
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let (mut app, fake, _) = shell("Todo");
    app.record(Some(Recording::new(dir.path(), Limits::default())));
    open(&fake, "plev-showcase://shell.page/select?value=Files");
    app.links_arrived();
    let session = app
        .recording
        .as_ref()
        .expect("recording")
        .recorder()
        .snapshot_window();
    let kinds: Vec<&str> = session
        .events()
        .iter()
        .map(|e| e.kind.name())
        .filter(|k| *k != "snapshot" && *k != "changes")
        .collect();
    assert_eq!(
        kinds,
        ["observed", "stage", "stage", "stage", "stage"],
        "{kinds:?}"
    );
    for event in session.events() {
        match &event.kind {
            Kind::Observed { actor, scope, .. } => {
                assert_eq!(*actor, link_actor());
                assert_eq!(*scope, None, "the whole gallery");
            }
            Kind::Stage { request, .. } => {
                assert_eq!(request.actor, link_actor());
                assert_eq!(request.target.as_str(), "shell.page");
            }
            Kind::Input { .. } => panic!("an input was made up for a link"),
            Kind::Snapshot { .. } | Kind::Changes(_) => {}
        }
    }
}

#[test]
fn the_open_list_is_the_page_and_the_theme_and_nothing_else() {
    assert_eq!(OPEN, [("shell.page", "select"), ("shell.theme", "select")]);
    let gallery = {
        let (app, _, _) = shell("Todo");
        app.view.gallery_semantics().expect("the gallery projects")
    };
    for (id, action) in OPEN {
        let node = gallery.get(&id.parse().expect("an id")).expect("projected");
        let decl = node
            .actions
            .iter()
            .find(|a| a.name.as_str() == *action)
            .expect("declared");
        assert!(decl.policy.actors.agent, "{id}: open to an agent");
        assert!(!decl.policy.destructive, "{id}: nothing destructive");
        assert_eq!(decl.policy.confirm, sem::Confirm::Never);
    }
}
