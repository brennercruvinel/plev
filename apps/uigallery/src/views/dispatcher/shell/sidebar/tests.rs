//! The sidebar from the three actors (task-104 front H, #163): a person
//! (the pointer, the keyboard), a screen reader and an agent pin and
//! unpin it, show it, and go to a tab through it, and land on the same
//! state; the projection says what they all read.

use accord::{Actor, Request, Role, SemId, Snapshot, Stage, Value};
use engine::widgets::nav::SidebarDock;
use engine::widgets::prelude::{Rect, WidgetEvent};

use crate::views::navigation::place;
use crate::views::{EditKey, PAGES, ShowcaseView};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

const PIN: &str = "shell.sidebar.pin";

fn agent() -> Actor {
    Actor::Agent {
        name: "claude".into(),
    }
}

fn view(page: &str) -> ShowcaseView {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page(page);
    view
}

/// Runs the view's clock until the motions land.
fn settle(view: &mut ShowcaseView) {
    for _ in 0..120 {
        view.wake(1.0 / 60.0);
    }
}

/// `actor`'s request on what it reads: the reader its tree's snapshot,
/// an agent the gallery's, through `act`. Whether it completed.
fn ask(view: &mut ShowcaseView, actor: Actor, id: &str, action: &str) -> bool {
    let seen = match actor {
        Actor::Assistive => view.reader_semantics(),
        _ => view.gallery_semantics().ok(),
    }
    .expect("a projection");
    let req = Request::observe(&seen, actor, id, action, None)
        .unwrap_or_else(|e| panic!("{id} {action}: {e}"));
    let receipt = view.act(req).expect("a receipt");
    matches!(receipt.stage(), Some(Stage::Completed(_)))
}

/// The state the three compare: the landmark's and the pin's, the page,
/// where the page's column starts.
#[derive(Debug, PartialEq)]
struct State {
    expanded: Value,
    pinned: Value,
    mode: Value,
    on: Value,
    page: Value,
    page_x: f32,
}

fn state(view: &ShowcaseView) -> State {
    let seen = view.gallery_semantics().expect("the gallery projects");
    let get = |id: &str, key: &str| seen.get(&sid(id)).expect(id).state[key].clone();
    State {
        expanded: get("shell.sidebar", "expanded"),
        pinned: get("shell.sidebar", "pinned"),
        mode: get("shell.sidebar", "mode"),
        on: get(PIN, "on"),
        page: get("shell.page", "selected"),
        page_x: view.page_x(),
    }
}

/// The person's click on the pin, where it is drawn.
fn click_pin(view: &mut ShowcaseView) {
    view.handle_event(&WidgetEvent::MouseMove { x: 1.0, y: 400.0 });
    settle(view);
    let panel = view.panel_rect().expect("the edge shows it");
    let pin = SidebarDock::pin_rect(view.panel_head(panel), &view.theme);
    click(view, pin);
}

fn click(view: &mut ShowcaseView, r: Rect) {
    let (x, y) = r.center();
    view.handle_event(&WidgetEvent::MouseMove { x, y });
    view.handle_event(&WidgetEvent::MouseDown { x, y });
    view.handle_event(&WidgetEvent::MouseUp { x, y });
}

#[test]
fn the_sidebar_is_a_landmark_its_pin_a_toggle_and_its_tabs_links_while_it_shows() {
    let mut view = view("Theme");
    let seen = view.gallery_semantics().unwrap();
    let nav = seen.get(&sid("shell.sidebar")).unwrap();
    assert_eq!((nav.role, nav.label.as_str()), (Role::Navigation, "Pages"));
    assert_eq!(nav.parent, Some(sid("shell")));
    assert_eq!(nav.state["expanded"], Value::Bool(false));
    assert_eq!(nav.state["mode"], Value::from("peek"));
    let pin = seen.get(&sid(PIN)).unwrap();
    assert_eq!(pin.role, Role::Toggle);
    assert_eq!(pin.state["on"], Value::Bool(false));
    assert_eq!(pin.actions[0].policy, accord::Policy::default(), "anyone");
    assert!(
        seen.get(&sid("shell.sidebar.row[0]")).is_none(),
        "out, no tabs"
    );

    view.pin_sidebar(true);
    let seen = view.gallery_semantics().unwrap();
    let (row, at) = place(1);
    let tab = sid(&format!("shell.sidebar.row[{row}].links[{at}]"));
    let link = seen.get(&tab).expect("pinned, its tabs are there");
    assert_eq!(
        (link.role, link.label.as_str()),
        (Role::Link, PAGES[1].title)
    );
    let group = seen
        .get(&sid(&format!("shell.sidebar.row[{row}]")))
        .unwrap();
    assert_eq!(group.role, Role::Button);
    assert_eq!(group.state["expanded"], Value::Bool(true));
    let reader = view.reader_semantics().unwrap();
    assert!(reader.get(&tab).is_some(), "a reader meets it too");
}

#[test]
fn a_person_a_reader_and_an_agent_pin_and_unpin_it_to_the_same_state() {
    let mut person = view("Theme");
    click_pin(&mut person);
    let mut reader = view("Theme");
    assert!(ask(&mut reader, Actor::Assistive, PIN, "toggle"));
    let mut agent_view = view("Theme");
    assert!(ask(&mut agent_view, agent(), PIN, "toggle"));
    let pinned = state(&person);
    assert_eq!(pinned.on, Value::Bool(true));
    assert_eq!(pinned.mode, Value::from("pinned"));
    assert_eq!(pinned.page_x, person.theme.navigation.dock.width);
    assert_eq!(state(&reader), pinned);
    assert_eq!(state(&agent_view), pinned);
    // the person's click left a receipt, as the others' requests did
    let receipt = person
        .panes
        .todo
        .receipt(accord::RequestId::parse("1").unwrap())
        .unwrap();
    assert_eq!(receipt.request.target, sid(PIN));
    assert_eq!(receipt.request.actor, Actor::Human);

    click_pin(&mut person);
    person.handle_event(&WidgetEvent::MouseMove { x: 900.0, y: 400.0 });
    settle(&mut person);
    assert!(ask(&mut reader, Actor::Assistive, PIN, "toggle"));
    assert!(ask(&mut agent_view, agent(), PIN, "toggle"));
    let free = state(&person);
    assert_eq!(free.on, Value::Bool(false));
    assert_eq!((free.mode.clone(), free.page_x), (Value::from("peek"), 0.0));
    assert_eq!(free.expanded, Value::Bool(false));
    assert_eq!(state(&reader), free);
    assert_eq!(state(&agent_view), free);
}

/// Tab until the focus is on `id`, at most `n` times.
fn tab_to(view: &mut ShowcaseView, id: &SemId, n: usize) {
    for _ in 0..n {
        if view
            .keyboard_semantics()
            .and_then(|s| s.focused().cloned())
            .as_ref()
            == Some(id)
        {
            return;
        }
        assert!(view.handle_edit_key(EditKey::Tab), "Tab is taken");
    }
    let at = view.keyboard_semantics().and_then(|s| s.focused().cloned());
    assert_eq!(at.as_ref(), Some(id), "Tab never reached it");
}

fn focused(view: &ShowcaseView) -> Option<SemId> {
    view.keyboard_semantics().and_then(|s| s.focused().cloned())
}

fn order(view: &ShowcaseView) -> usize {
    let seen: Snapshot = view.keyboard_semantics().unwrap_or_default();
    engine::control::accessibility::focus::order(&seen).len()
}

#[test]
fn the_focus_shows_it_tab_and_escape_hide_it_and_the_agent_shows_it_by_asking() {
    // the keyboard: past the page's last stop, the pin
    let mut person = view("Button");
    let page_stops = order(&person) - 1;
    tab_to(&mut person, &sid(PIN), page_stops + 2);
    assert_eq!(person.sidebar_focus(), Some(&sid(PIN)));
    let shown = state(&person);
    assert_eq!(shown.expanded, Value::Bool(true), "the focus shows it");
    assert_eq!(shown.page_x, 0.0, "over the page");
    // the reader moves its focus there
    let mut reader = view("Button");
    assert!(reader.assist_focus(&sid(PIN)));
    assert_eq!(state(&reader), shown);
    // an agent has no focus: it asks the landmark
    let mut agent_view = view("Button");
    assert!(ask(
        &mut agent_view,
        agent(),
        "shell.sidebar",
        "toggle_expanded"
    ));
    assert_eq!(state(&agent_view), shown);

    // inside, the focus keeps it: the logo, the dropdowns, the tabs
    assert!(person.handle_edit_key(EditKey::Tab));
    let inside = focused(&person).unwrap();
    assert!(person.is_sidebar(&inside), "{inside}");
    assert_eq!(state(&person).expanded, Value::Bool(true));
    // Escape lets it go and hides it
    assert!(person.close_top_overlay());
    assert_eq!(person.sidebar_focus(), None);
    assert_eq!(state(&person).expanded, Value::Bool(false));
    // Tab out of it, past its last stop, to the page: hidden again
    tab_to(&mut person, &sid(PIN), page_stops + 2);
    let all = order(&person);
    for _ in 0..all {
        let at = focused(&person).unwrap();
        if !person.is_sidebar(&at) {
            break;
        }
        assert!(person.handle_edit_key(EditKey::Tab));
    }
    let at = focused(&person).unwrap();
    assert!(!person.is_sidebar(&at), "back on the page: {at}");
    assert_eq!(state(&person).expanded, Value::Bool(false));
    // the reader's focus back on the page hides it as well
    assert!(reader.assist_focus(&at));
    assert_eq!(state(&reader).expanded, Value::Bool(false));
    // pinned, the focus leaving changes nothing
    reader.pin_sidebar(true);
    assert!(reader.assist_focus(&sid(PIN)));
    assert!(reader.assist_focus(&at));
    assert_eq!(state(&reader).expanded, Value::Bool(true));
}

#[test]
fn a_person_a_reader_and_an_agent_go_to_a_tab_through_the_sidebar() {
    let target = 1;
    let (row, at) = place(target);
    let tab = format!("shell.sidebar.row[{row}].links[{at}]");
    // the person: the edge shows it, a click on the tab
    let mut person = view("Theme");
    person.handle_event(&WidgetEvent::MouseMove { x: 1.0, y: 400.0 });
    settle(&mut person);
    let panel = person.panel_rect().unwrap();
    let r = person.nav.tab_rect(target, panel, &person.theme).unwrap();
    click(&mut person, r);
    // the keyboard: Tab to the tab, Enter
    let mut keys = view("Theme");
    tab_to(&mut keys, &sid(&tab), 40);
    assert!(keys.handle_enter());
    // the reader: its press on the link, as the tree offers it
    let mut reader = view("Theme");
    assert!(reader.assist_focus(&sid(PIN)), "its focus shows the panel");
    assert!(ask(&mut reader, Actor::Assistive, &tab, "activate"));
    // the agent: the landmark shown, the link activated
    let mut agent_view = view("Theme");
    assert!(ask(
        &mut agent_view,
        agent(),
        "shell.sidebar",
        "toggle_expanded"
    ));
    assert!(ask(&mut agent_view, agent(), &tab, "activate"));
    for v in [&person, &keys, &reader, &agent_view] {
        assert_eq!(v.page_title(), PAGES[target].title);
        let seen = v.gallery_semantics().unwrap();
        let page = &seen.get(&sid("shell.page")).unwrap().state["selected"];
        assert_eq!(page, &Value::from(PAGES[target].title));
        assert_eq!(v.nav.current(), target, "the sidebar marks it");
    }
    // the reader's focus keeps the panel shown: its tab is the current
    let link = reader.reader_semantics().unwrap();
    assert_eq!(
        link.get(&sid(&tab)).unwrap().state["active"],
        Value::Bool(true)
    );
    // the agent's picked tab lets the panel it asked for go
    assert_eq!(state(&agent_view).expanded, Value::Bool(false));
}

#[test]
fn under_a_modal_the_sidebar_is_inert() {
    let mut view = view("Dialog");
    view.pin_sidebar(true);
    view.open_modal(false);
    let seen = view.keyboard_semantics().unwrap();
    assert!(seen.get(&sid("shell.sidebar")).unwrap().inert);
    assert!(!view.assist_focus(&sid(PIN)), "the modal keeps the focus");
    let gallery = view.gallery_semantics().unwrap();
    let req = Request::observe(&gallery, agent(), PIN, "toggle", None).unwrap();
    let receipt = view.act(req).unwrap();
    assert!(matches!(receipt.stage(), Some(Stage::Rejected(_))), "inert");
}
