//! The shell as a capability (task-76 item 4), without a window: what it
//! projects and who may ask, the keys as requests with receipts, an agent
//! switching the theme and the page and refused the window, a stale
//! pick refused as a button's would be, the gallery projection beside
//! the page's, and a menu click running the page's capability.

use engine::platform::menu::Clicked;
use sem::{Actor, Rejection, Request, RequestId, Role, SemId, Stage, Value};

use super::PERSON_ONLY;
use crate::view::foundations::theme::THEMES;
use crate::view::{PAGES, ShowcaseView};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

fn view(page: &str) -> ShowcaseView {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page(page);
    view.resize(1200.0, 800.0, 1.0);
    view
}

fn receipt(view: &ShowcaseView, n: u64) -> sem::Receipt {
    view.panes
        .todo
        .receipt(RequestId::parse(&n.to_string()).unwrap())
        .cloned()
        .unwrap_or_else(|| panic!("receipt {n} is kept"))
}

fn agent() -> Actor {
    Actor::Agent {
        name: "claude".into(),
    }
}

fn text(value: &str) -> Option<Value> {
    Some(Value::Text(value.to_owned()))
}

fn selected(view: &ShowcaseView, at: &str) -> Value {
    view.gallery_semantics()
        .unwrap()
        .value(&at.parse().unwrap())
        .cloned()
        .expect("projected")
}

#[test]
fn the_shell_projects_its_capabilities_and_who_may_ask() {
    let mut view = view("Todo");
    let shell = view.shell_semantics().expect("the shell projects");
    let ids: Vec<&str> = shell.nodes().keys().map(SemId::as_str).collect();
    assert_eq!(
        ids,
        [
            "shell",
            "shell.page",
            "shell.theme",
            "shell.detach",
            "shell.show",
            "shell.print"
        ]
    );

    let root = shell.get(&sid("shell")).unwrap();
    assert_eq!((root.role, root.label.as_str()), (Role::Region, "plev"));
    assert_eq!(
        root.state["notice"],
        Value::from(""),
        "nothing told by notification yet"
    );

    let page = shell.get(&sid("shell.page")).unwrap();
    assert_eq!(page.role, Role::Select);
    assert_eq!(page.state["selected"], Value::Text("Todo".into()));
    let titles: Vec<String> = PAGES.iter().map(|p| p.title.to_owned()).collect();
    assert_eq!(page.constraints.options.as_deref(), Some(titles.as_slice()));
    assert!(page.constraints.editable);
    assert!(!page.focusable, "nothing on screen takes the focus for it");
    let select = &page.actions[0];
    assert_eq!(select.name.as_str(), "select");
    assert_eq!(select.param.map(|p| p.kind), Some(sem::ValueKind::Text));
    assert_eq!(select.reads, ["shell.page:selected".parse().unwrap()]);
    assert_eq!(select.policy, sem::Policy::default(), "anyone picks a page");

    let theme = shell.get(&sid("shell.theme")).unwrap();
    assert_eq!(theme.role, Role::Select);
    assert_eq!(theme.state["selected"], Value::Text("core".into()));
    let names: Vec<String> = THEMES.iter().map(|t| (*t).to_owned()).collect();
    assert_eq!(theme.constraints.options.as_deref(), Some(names.as_slice()));
    assert_eq!(theme.actions[0].policy, sem::Policy::default());

    let detach = shell.get(&sid("shell.detach")).unwrap();
    assert_eq!(detach.role, Role::Toggle);
    assert_eq!(detach.state["on"], Value::Bool(false));
    assert!(
        detach.enabled,
        "the todo page can show in a window of its own"
    );
    let toggle = &detach.actions[0];
    assert_eq!(toggle.name.as_str(), "toggle");
    assert_eq!(toggle.reads, ["shell.detach:on".parse().unwrap()]);
    assert_eq!(toggle.policy.actors, PERSON_ONLY);
    assert!(!toggle.policy.actors.agent);

    // the window forward (the tray icon's click, a notification's button):
    // a person's, on the desktop
    let show = shell.get(&sid("shell.show")).unwrap();
    assert_eq!(show.role, Role::Button);
    assert_eq!(show.label, "show plev");
    assert!(show.enabled, "the desktop has windows to bring forward");
    assert!(!show.focusable);
    let activate = &show.actions[0];
    assert_eq!(activate.name.as_str(), "activate");
    assert_eq!(activate.param, None);
    assert_eq!(activate.policy.actors, PERSON_ONLY);

    // what the shell told by notification is projected for an agent
    view.set_notice("2 silences found");
    assert_eq!(
        view.shell_semantics()
            .unwrap()
            .get(&sid("shell"))
            .unwrap()
            .state["notice"],
        Value::from("2 silences found")
    );
    view.set_notice("");

    // the save shows while the session is recorded, for a person only
    view.set_recording(true);
    let shell = view.shell_semantics().unwrap();
    let save = shell
        .get(&sid("shell.save_session"))
        .expect("projected now");
    assert_eq!(save.role, Role::Button);
    assert_eq!(save.actions[0].name.as_str(), "activate");
    assert_eq!(save.actions[0].policy.actors, PERSON_ONLY);
    view.set_recording(false);
    assert!(
        view.shell_semantics()
            .unwrap()
            .get(&sid("shell.save_session"))
            .is_none()
    );

    // a placeholder page has nothing to show in a window of its own
    if let Some(placeholder) = PAGES.iter().find(|p| p.pane().is_none()) {
        view.jump_to_page(placeholder.title);
        let shell = view.shell_semantics().unwrap();
        assert!(!shell.get(&sid("shell.detach")).unwrap().enabled);
        assert_eq!(
            shell.get(&sid("shell.page")).unwrap().state["selected"],
            Value::Text(placeholder.title.to_owned())
        );
    }
}

#[test]
fn the_keys_ask_through_the_dispatcher_and_leave_receipts() {
    let mut view = view("Todo");
    assert!(view.handle_key("t"));
    assert_eq!(view.theme_name, "hoff");
    let first = receipt(&view, 1);
    assert_eq!(first.request.actor, Actor::Human);
    assert_eq!(first.request.target, sid("shell.theme"));
    assert_eq!(first.request.action.as_str(), "select");
    assert_eq!(first.request.param, text("hoff"));
    let Some(Stage::Completed(outcome)) = first.stage() else {
        panic!("completed: {:?}", first.stage());
    };
    assert_eq!(outcome.result["theme"], Value::Text("hoff".into()));

    assert!(!view.take_detach_request(), "nothing asked yet");
    assert!(view.handle_key("d"));
    let second = receipt(&view, 2);
    assert_eq!(second.request.target, sid("shell.detach"));
    let Some(Stage::Completed(outcome)) = second.stage() else {
        panic!("completed: {:?}", second.stage());
    };
    assert_eq!(outcome.result["on"], Value::Bool(true));
    assert!(view.take_detach_request(), "the shell takes the ask once");
    assert!(!view.take_detach_request());

    assert!(view.handle_key("3"));
    assert_eq!(view.page_title(), PAGES[2].title);
    let third = receipt(&view, 3);
    assert_eq!(third.request.target, sid("shell.page"));
    assert_eq!(third.request.param, text(PAGES[2].title));

    // the save: only while the session is recorded
    assert!(!view.request_save());
    assert!(!view.take_save_request());
    view.set_recording(true);
    assert!(view.request_save());
    assert!(view.take_save_request());
    assert!(!view.take_save_request());
    let fourth = receipt(&view, 4);
    assert_eq!(fourth.request.target, sid("shell.save_session"));
    assert!(matches!(fourth.stage(), Some(Stage::Completed(_))));

    // the window forward: asked as the tray and a notification ask it,
    // taken by the shell once
    assert!(!view.take_show_request(), "nothing asked yet");
    let show = Clicked {
        sem: sid("shell.show"),
        action: "activate".parse().unwrap(),
        param: None,
    };
    assert!(!view.menu_click(&show), "showing moves no projection");
    let fifth = receipt(&view, 5);
    assert_eq!(fifth.request.actor, Actor::Human);
    assert_eq!(fifth.request.target, sid("shell.show"));
    assert!(matches!(fifth.stage(), Some(Stage::Completed(_))));
    assert!(view.take_show_request());
    assert!(!view.take_show_request());
}

#[test]
fn the_theme_cards_and_the_sidebar_ask_the_same_way() {
    use comps::prelude::WidgetEvent;
    let mut view = view("Theme");
    let content = view.page_rect();
    // a point on a card that is not the current theme, found by asking the
    // page itself (its geometry is its own)
    let mut on_card = None;
    let (mut y, bottom) = (content.y, content.y + 400.0);
    'scan: while y < bottom {
        let mut x = content.x;
        while x < content.x + content.w {
            let (_, picked) = view
                .panes
                .themes
                .handle_event(&WidgetEvent::MouseDown { x, y }, content);
            if let Some(name) = picked.filter(|n| *n != view.theme_name) {
                on_card = Some((x, y, name));
                break 'scan;
            }
            x += 8.0;
        }
        y += 8.0;
    }
    let (x, y, name) = on_card.expect("a card of another theme shows");
    assert!(view.handle_event(&WidgetEvent::MouseDown { x, y }));
    view.handle_event(&WidgetEvent::MouseUp { x, y });
    assert_eq!(view.theme_name, name);
    let first = receipt(&view, 1);
    assert_eq!(first.request.target, sid("shell.theme"));
    assert_eq!(first.request.param, text(name));
    assert_eq!(first.request.actor, Actor::Human);

    // a sidebar link is `shell.page`
    let rect = view
        .shell
        .sidebar
        .rect(&view.theme, view.width, view.height)
        .expect("a sidebar");
    let mode = view.shell.sidebar.mode(&view.theme, view.width);
    let links = view.shell.sidebar.link_rects(rect, &view.theme, mode);
    let (x, y) = links[3].center();
    assert!(view.handle_event(&WidgetEvent::MouseDown { x, y }));
    view.handle_event(&WidgetEvent::MouseUp { x, y });
    assert_eq!(view.page_title(), PAGES[3].title);
    let second = receipt(&view, 2);
    assert_eq!(second.request.target, sid("shell.page"));
    assert_eq!(second.request.param, text(PAGES[3].title));
}

#[test]
fn an_agent_switches_the_theme_and_the_page_and_is_refused_the_window() {
    let mut view = view("Todo");
    let gallery = view.gallery_semantics().unwrap();

    let req = Request::observe(&gallery, agent(), "shell.theme", "select", text("dark")).unwrap();
    let receipt = view.act(req).expect("the shell takes it");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(view.theme_name, "dark");
    assert_eq!(
        selected(&view, "shell.theme:selected"),
        Value::Text("dark".into())
    );

    let gallery = view.gallery_semantics().unwrap();
    let req = Request::observe(&gallery, agent(), "shell.page", "select", text("Card")).unwrap();
    let receipt = view.act(req).unwrap();
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(view.page_title(), "Card");

    // a page that is not one of the options never reaches execute
    let gallery = view.gallery_semantics().unwrap();
    let req = Request::observe(&gallery, agent(), "shell.page", "select", text("Nowhere")).unwrap();
    let receipt = view.act(req).unwrap();
    assert!(
        matches!(
            receipt.stage(),
            Some(Stage::Rejected(Rejection::InvalidParam { .. }))
        ),
        "{:?}",
        receipt.stage()
    );
    assert_eq!(view.page_title(), "Card");

    // the window is a person's, forward or in a window of its own
    let req = Request::observe(&gallery, agent(), "shell.detach", "toggle", None).unwrap();
    let receipt = view.act(req).unwrap();
    assert_eq!(
        receipt.stage(),
        Some(&Stage::Rejected(Rejection::NotAuthorized {
            actor: agent()
        }))
    );
    assert!(!view.take_detach_request());
    let req = Request::observe(&gallery, agent(), "shell.show", "activate", None).unwrap();
    let receipt = view.act(req).unwrap();
    assert!(matches!(
        receipt.stage(),
        Some(Stage::Rejected(Rejection::NotAuthorized { .. }))
    ));
    assert!(!view.take_show_request());

    // a screen reader's person and a test may
    let req = Request::observe(&gallery, Actor::Assistive, "shell.detach", "toggle", None).unwrap();
    assert!(matches!(
        view.act(req).unwrap().stage(),
        Some(Stage::Completed(_))
    ));
    assert!(view.take_detach_request());
    let test = Actor::Test { name: "t".into() };
    let req = Request::observe(&gallery, test, "shell.detach", "toggle", None).unwrap();
    assert!(matches!(
        view.act(req).unwrap().stage(),
        Some(Stage::Completed(_))
    ));
    assert!(view.take_detach_request());
}

#[test]
fn a_stale_theme_pick_is_refused_as_a_button_would_be() {
    let mut view = view("Todo");
    let seen = view.gallery_semantics().unwrap();
    assert!(view.handle_key("t"), "the person moved the theme meanwhile");
    let req = Request::observe(&seen, agent(), "shell.theme", "select", text("dark")).unwrap();
    let receipt = view.act(req).unwrap();
    let Some(Stage::Rejected(Rejection::Stale { changes, .. })) = receipt.stage() else {
        panic!("stale: {:?}", receipt.stage());
    };
    assert_eq!(changes.len(), 1);
    assert!(matches!(
        &changes[0],
        sem::Change::StateChanged { at, before: Some(Value::Text(b)), after: Some(Value::Text(a)), .. }
            if *at == "shell.theme:selected".parse().unwrap() && b == "core" && a == "hoff"
    ));
    assert_eq!(view.theme_name, "hoff", "nothing ran");
}

#[test]
fn the_gallery_carries_the_shell_and_the_page_and_the_page_projection_stays_alone() {
    let mut view = view("Todo");
    let page = view.semantics().expect("the todo projects");
    assert!(
        page.get(&sid("shell")).is_none(),
        "the page's projection is the page's"
    );
    let gallery = view.gallery_semantics().unwrap();
    assert_eq!(gallery.len(), page.len() + 6);
    for (id, node) in page.nodes() {
        assert_eq!(
            gallery.get(id),
            Some(node),
            "{id}: the same node, source aside"
        );
    }
    let roots: Vec<&str> = gallery.roots().map(|n| n.id.as_str()).collect();
    assert_eq!(roots, ["shell", "todo"]);
    assert!(!view.is_shell(&sid("todo")));
    assert!(view.is_shell(&sid("shell.theme")));

    // a page without a projection leaves the shell alone
    view.jump_to_page("Card");
    assert!(view.semantics().is_none());
    let gallery = view.gallery_semantics().unwrap();
    let ids: Vec<&str> = gallery.nodes().keys().map(SemId::as_str).collect();
    assert_eq!(
        ids,
        [
            "shell",
            "shell.page",
            "shell.theme",
            "shell.detach",
            "shell.show",
            "shell.print"
        ]
    );
}

/// Catches up until the print's thread answered, or five seconds.
fn print_settled(view: &mut ShowcaseView) -> bool {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < until {
        if view.settle_print() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    false
}

/// The frame on paper is a person's, and needs a destination: without
/// one the button is disabled and the projection says so; a printer's
/// destinations, looked for off the loop, enable it and name where a
/// print goes; an agent is refused; a printer without a system leaves it
/// disabled. Nothing is printed here (the job is the integration test's,
/// with a GPU).
#[test]
fn the_print_is_a_persons_and_needs_a_destination() {
    use std::sync::{Arc, Mutex};

    use engine::platform::menu::locked;
    use engine::platform::print::{FakePrinter, NoPrinter};

    use super::ShellTarget;

    let mut view = view("Todo");
    let shell = view.shell_semantics().unwrap();
    let print = shell.get(&sid("shell.print")).unwrap();
    assert_eq!(print.role, Role::Button);
    assert_eq!(print.label, "print this page");
    assert!(!print.enabled, "no destination yet");
    assert!(!print.focusable);
    assert_eq!(print.state["destinations"], Value::List(vec![]));
    assert_eq!(print.state["destination"], Value::from(""));
    assert_eq!(print.state["job"], Value::from(""));
    assert_eq!(print.state["status"], Value::from(""));
    let activate = &print.actions[0];
    assert_eq!(activate.name.as_str(), "activate");
    assert_eq!(activate.param, None);
    assert_eq!(activate.policy.actors, PERSON_ONLY);
    assert_eq!(
        shell.get(&sid("shell")).unwrap().state["print"],
        Value::from("no destination")
    );

    // a person asks anyway: refused as the disabled button is, nothing runs
    let receipt = view
        .ask_shell_receipt(ShellTarget::Print, "activate", None)
        .expect("a request");
    assert!(
        matches!(
            receipt.stage(),
            Some(Stage::Rejected(Rejection::Disabled { .. }))
        ),
        "{:?}",
        receipt.stage()
    );
    assert!(!view.request_print());
    assert!(!view.settle_print(), "nothing to settle");

    // a printer that knows two destinations: looked for off the loop,
    // projected when they land, the default the one a print goes to
    let fake = Arc::new(Mutex::new(
        FakePrinter::new()
            .destination("Home", false)
            .destination("Office", true),
    ));
    view.set_printer(Box::new(fake.clone()));
    assert!(print_settled(&mut view), "the destinations land");
    let shell = view.shell_semantics().unwrap();
    let print = shell.get(&sid("shell.print")).unwrap();
    assert!(print.enabled);
    assert_eq!(
        print.state["destinations"],
        Value::List(vec!["Home".to_owned(), "Office".to_owned()])
    );
    assert_eq!(print.state["destination"], Value::from("Office"));
    assert_eq!(
        shell.get(&sid("shell")).unwrap().state["print"],
        Value::from("ready on Office")
    );

    // an agent is refused: printing is a person's
    let gallery = view.gallery_semantics().unwrap();
    let req = Request::observe(&gallery, agent(), "shell.print", "activate", None).unwrap();
    let receipt = view.act(req).unwrap();
    assert_eq!(
        receipt.stage(),
        Some(&Stage::Rejected(Rejection::NotAuthorized {
            actor: agent()
        }))
    );
    assert!(locked(&fake).printed().is_empty(), "nothing printed");
    assert!(!view.settle_print(), "nothing runs");

    // a printer without a system at all: no destination, said so
    view.set_printer(Box::new(NoPrinter));
    assert!(print_settled(&mut view));
    let shell = view.shell_semantics().unwrap();
    assert!(!shell.get(&sid("shell.print")).unwrap().enabled);
    assert_eq!(
        shell.get(&sid("shell")).unwrap().state["print"],
        Value::from("no destination")
    );
}

#[test]
fn a_menu_click_runs_the_page_capability_as_the_button_does() {
    let mut view = view("Todo");
    let filter = Clicked {
        sem: sid("todo.filter[active]"),
        action: "select".parse().unwrap(),
        param: None,
    };
    assert_eq!(
        selected(&view, "todo.filter:selected"),
        Value::Text("all".into())
    );
    assert!(view.menu_click(&filter), "the projection moved");
    assert_eq!(
        selected(&view, "todo.filter:selected"),
        Value::Text("active".into())
    );
    let first = receipt(&view, 1);
    assert_eq!(first.request.actor, Actor::Human);
    assert_eq!(first.request.target, sid("todo.filter[active]"));
    assert!(matches!(first.stage(), Some(Stage::Completed(_))));

    // a disabled capability is refused, as its button never fires
    let submit = Clicked {
        sem: sid("todo.input"),
        action: "submit".parse().unwrap(),
        param: None,
    };
    assert!(!view.menu_click(&submit));
    assert!(matches!(
        receipt(&view, 2).stage(),
        Some(Stage::Rejected(Rejection::Disabled { .. }))
    ));

    // the shell's items go the same way
    let dark = Clicked {
        sem: sid("shell.theme"),
        action: "select".parse().unwrap(),
        param: text("dark"),
    };
    assert!(view.menu_click(&dark));
    assert_eq!(view.theme_name, "dark");

    // an item of a capability that is not there asks nothing
    let gone = Clicked {
        sem: sid("todo.item[99]"),
        action: "toggle".parse().unwrap(),
        param: None,
    };
    assert!(!view.menu_click(&gone));
    assert!(
        view.panes
            .todo
            .receipt(RequestId::parse("4").unwrap())
            .is_none()
    );
}
