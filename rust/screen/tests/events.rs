//! An event on a widget of the surface is a request for the capability
//! the IR bound, by the actor who acted, observed on the state the
//! surface showed, and the receipt comes from the app's own dispatcher;
//! an act on a node of the surface, by an agent, is routed the same way.

// shared with the other surface tests: this binary uses the fixtures, the
// page and the app.
#[allow(dead_code)]
use crate::support;

use accord::{Actor, Change, Rejection, Request, SemId, Stage, StateRef, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::{Rect, WidgetEvent};
use screen::{Dispatch, Surface};
use support::{App, IDE, REVIEW, id, page, snapshot, verified};

const AREA: Rect = Rect {
    x: 40.0,
    y: 30.0,
    w: 720.0,
    h: 600.0,
};

fn frame(surface: &mut Surface, theme: &Theme) -> Compositor {
    let mut c = Compositor::new();
    surface.render(&mut c, AREA, theme);
    c
}

/// The center of the rect `at` is bound to in the frame.
fn center(c: &Compositor, at: &str) -> (f32, f32) {
    let b = c
        .binding(&id(at))
        .unwrap_or_else(|| panic!("{at} is bound"));
    let [x, y, w, h] = b.rect;
    assert!(w > 0.0 && h > 0.0, "{at} is drawn: {:?}", b.rect);
    (x + w / 2.0, y + h / 2.0)
}

/// A person's click: the pointer moves there, presses and releases.
fn click(surface: &mut Surface, app: &mut App, (x, y): (f32, f32), theme: &Theme) {
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
        WidgetEvent::MouseUp { x, y },
    ] {
        surface.handle_event(&event, theme, app);
    }
}

fn names(stages: &[(Stage, u64)]) -> Vec<&'static str> {
    stages.iter().map(|(s, _)| s.name()).collect()
}

fn setup() -> (App, Surface, Theme) {
    let app = App::over(&snapshot(IDE));
    let now = app.snapshot();
    let surface = Surface::materialize(verified(REVIEW, &now), &now);
    (app, surface, Theme::hoff_core())
}

#[test]
fn a_click_is_the_bound_capability_by_the_person_through_the_apps_dispatcher() {
    let (mut app, mut surface, theme) = setup();
    let c = frame(&mut surface, &theme);
    let at = center(&c, "review.file[notes.md].stage");
    click(&mut surface, &mut app, at, &theme);

    // the app ran it, once, through its dispatcher
    assert_eq!(app.executed.len(), 1);
    let receipt = surface.last_receipt().expect("a receipt").clone();
    assert_eq!(
        app.receipt(receipt.id).map(|r| &r.request),
        Some(&receipt.request)
    );
    let req = &receipt.request;
    assert_eq!(req.target, id("changes.file[notes.md]"));
    assert_eq!(req.action.as_str(), "stage");
    assert_eq!(req.actor, Actor::Human);
    assert_eq!(
        names(&receipt.stages),
        ["requested", "accepted", "executing", "completed"]
    );
    // observed on what the surface showed: the reads of `stage`, with
    // the values of the frame
    let staged = StateRef::parse("changes.file[notes.md]:staged").unwrap();
    assert!(
        req.observed
            .values
            .contains(&(staged.clone(), Some(Value::Bool(false))))
    );
    assert_eq!(req.observed.revision, snapshot(IDE).revision());
    // and the surface shows what came out
    assert_eq!(surface.shown().value(&staged), Some(&Value::Bool(true)));
}

#[test]
fn typing_in_a_field_of_the_surface_sets_the_apps_text_and_enter_submits() {
    let (mut app, mut surface, theme) = setup();
    let c = frame(&mut surface, &theme);
    let at = center(&c, "review.message");
    click(&mut surface, &mut app, at, &theme);
    assert_eq!(surface.focused(), Some(&id("review.message")));
    assert!(app.executed.is_empty(), "a click into a field is editing");

    assert!(surface.handle_text("!", &mut app));
    let text = StateRef::parse("commit.message:text").unwrap();
    let now = app.snapshot();
    assert_eq!(now.value(&text), Some(&Value::Text("fix parser!".into())));
    let set = app.executed.last().expect("set_value ran");
    assert_eq!(set.target, id("commit.message"));
    assert_eq!(set.action.as_str(), "set_value");
    assert_eq!(set.param, Some(Value::Text("fix parser!".into())));
    assert_eq!(set.actor, Actor::Human);
    // the next key observes the text the first one made: never stale
    assert!(surface.handle_text("?", &mut app));
    let last = surface.last_receipt().expect("a receipt");
    assert_eq!(last.stages.last().map(|(s, _)| s.name()), Some("completed"));
    assert_eq!(
        app.snapshot().value(&text),
        Some(&Value::Text("fix parser!?".into()))
    );
    // the text field binds no submit in the review: Enter is consumed
    // while it has the keyboard, and asks for nothing
    let ran = app.executed.len();
    assert!(surface.handle_enter(&mut app));
    assert_eq!(app.executed.len(), ran);
    assert!(surface.handle_escape());
    assert_eq!(surface.focused(), None);
    assert!(!surface.handle_text("x", &mut app));
}

/// The rule of observation: a person acts on what the surface showed.
/// When another actor moved the state the capability reads and the
/// surface has not shown it yet, the click is stale, with the delta.
#[test]
fn a_click_on_state_the_surface_has_not_shown_yet_is_stale() {
    let (mut app, mut surface, theme) = setup();
    let c = frame(&mut surface, &theme);
    app.set("changes.file[notes.md]:staged", Value::Bool(true));
    click(
        &mut surface,
        &mut app,
        center(&c, "review.file[notes.md].stage"),
        &theme,
    );
    let receipt = surface.last_receipt().expect("a receipt");
    assert_eq!(names(&receipt.stages), ["requested", "rejected"]);
    let Some((Stage::Rejected(Rejection::Stale { changes, .. }), _)) = receipt.stages.last() else {
        panic!("stale: {:?}", receipt.stages);
    };
    let staged = StateRef::parse("changes.file[notes.md]:staged").unwrap();
    assert!(changes.iter().any(
        |c| matches!(c, Change::StateChanged { at, after, .. } if *at == staged && *after == Some(Value::Bool(true)))
    ));
    assert!(app.executed.is_empty());
    // after the refusal the surface shows the state as it is
    assert_eq!(surface.shown().value(&staged), Some(&Value::Bool(true)));
}

/// An agent observes the page (the app and the surface, one snapshot)
/// and acts on a node of the surface: the act is the capability the node
/// is bound to, with the agent as the actor, through the same dispatcher.
#[test]
fn an_agent_acts_on_a_node_of_the_surface_through_the_same_contract() {
    let (mut app, surface, _) = setup();
    let seen = page(&app.snapshot(), &surface);
    let unstage = id("review.file[src/lexer.rs].unstage");
    let node = seen.get(&unstage).expect("the surface projects it");
    assert_eq!(node.actions[0].name.as_str(), "activate");
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let req = Request::observe(&seen, agent.clone(), &unstage, "activate", None).unwrap();
    let routed = surface.route(req);
    assert_eq!(routed.target, id("changes.file[src/lexer.rs]"));
    assert_eq!(routed.action.as_str(), "unstage");
    let receipt = app.dispatch(routed).expect("a receipt");
    assert_eq!(receipt.request.actor, agent);
    assert_eq!(
        names(&receipt.stages),
        ["requested", "accepted", "executing", "completed"]
    );
    let staged = StateRef::parse("changes.file[src/lexer.rs]:staged").unwrap();
    assert_eq!(app.snapshot().value(&staged), Some(&Value::Bool(false)));
    // a node of the app's own and an unbound action go as they are: the
    // dispatcher answers them
    let direct = Request::observe(&seen, agent, "changes.file[notes.md]", "stage", None).unwrap();
    assert_eq!(surface.route(direct.clone()), direct);
}

/// A capability that is disabled in the app leaves its control disabled
/// on the surface: a click there asks for nothing.
#[test]
fn a_disabled_capability_leaves_its_control_disabled() {
    let (mut app, mut surface, theme) = setup();
    // in the fixture, src/lexer.rs is staged, so its `stage` is disabled
    let seen = page(&app.snapshot(), &surface);
    let stage = seen.get(&id("review.file[src/lexer.rs].stage")).unwrap();
    assert!(!stage.enabled);
    let c = frame(&mut surface, &theme);
    click(
        &mut surface,
        &mut app,
        center(&c, "review.file[src/lexer.rs].stage"),
        &theme,
    );
    assert!(app.executed.is_empty());
    assert!(surface.last_receipt().is_none());
}

/// While inert (an overlay holds the page) the surface takes no input
/// and projects inert.
#[test]
fn an_inert_surface_takes_no_input() {
    let (mut app, mut surface, theme) = setup();
    let c = frame(&mut surface, &theme);
    surface.set_inert(true);
    click(
        &mut surface,
        &mut app,
        center(&c, "review.file[notes.md].stage"),
        &theme,
    );
    assert!(app.executed.is_empty());
    let seen = page(&app.snapshot(), &surface);
    let region: &SemId = surface.id();
    assert!(seen.get(region).is_some_and(|n| n.inert));
    assert!(seen.get(&id("review.commit")).is_some_and(|n| n.inert));
}

/// The geometry is kept by id, not by place in the tree: after the list
/// moved and before the next frame, a click lands on the node the person
/// saw there, and where a node that left was drawn nothing is hit.
#[test]
fn before_the_next_frame_a_click_hits_what_the_person_saw() {
    let (mut app, mut surface, theme) = setup();
    let c = frame(&mut surface, &theme);
    let notes = center(&c, "review.file[notes.md].stage");
    let lexer = center(&c, "review.file[src/lexer.rs].unstage");
    // src/lexer.rs leaves the list: every instance after it moves up a
    // place in the tree
    let files = ["notes.md", "src/parser.rs"].map(str::to_owned);
    app.set("changes:files", Value::List(files.to_vec()));
    assert!(surface.sync(&app.snapshot()));

    click(&mut surface, &mut app, lexer, &theme);
    assert!(app.executed.is_empty(), "a node that left is not hit");
    click(&mut surface, &mut app, notes, &theme);
    let req = app.executed.last().expect("the stage ran");
    assert_eq!(req.target, id("changes.file[notes.md]"));
    assert_eq!(req.action.as_str(), "stage");
}
