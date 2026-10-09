//! The tray as another view of an app's capabilities (task-76 item 5),
//! proven without a platform: the model built from a real projection of
//! the showcase (the todo golden) and from a small shell, a click on the
//! fake tray's menu becoming the request the button makes and the same
//! receipt, the click on the icon asking the one capability the app chose,
//! the tray following a moved projection (an item disabled, an item gone),
//! and the target without a tray.

use engine::runtime::platform::menu::{Clicked, MenuItem, Predefined, Submenu};
use engine::runtime::platform::tray::{FakeTray, NoTray, Tray, TrayError, TrayImage, TrayModel};
use accord::{
    Actor, Dispatcher, Execution, Operable, Outcome, Policy, Request, RequestId, Role, SemId,
    Semantics, Snapshot, SnapshotBuilder, Stage, Value,
};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

fn todo() -> Snapshot {
    serde_json::from_str(include_str!(
        "../../plevir/tests/fixtures/uigallery-todo.snapshot.json"
    ))
    .expect("the golden parses")
}

/// A counter with a button that counts and a button that shows the
/// window (a person's alone): the app under test. `show` records that it
/// was asked, as a shell would take the ask on its loop.
struct Counter {
    count: i64,
    enabled: bool,
    shown: usize,
}

impl Semantics for Counter {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("counter", Role::Region, "counter")
            .state("count", self.count);
        b.node("counter.up", Role::Button, "count up")
            .parent("counter")
            .enabled(self.enabled)
            .action("increment")
            .reads("counter:count");
        b.node("counter.show", Role::Button, "show the window")
            .parent("counter")
            .action("activate")
            .policy(Policy {
                actors: accord::Actors {
                    agent: false,
                    ..sem::Actors::default()
                },
                ..Policy::default()
            });
    }
}

impl Operable for Counter {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let mut outcome = Outcome::default();
        match req.action.as_str() {
            "increment" => {
                self.count += 1;
                outcome
                    .result
                    .insert("count".to_owned(), Value::Int(self.count));
            }
            "activate" => self.shown += 1,
            other => return Execution::Failed(format!("no {other}")),
        }
        Execution::Done(outcome)
    }
}

fn project(app: &Counter) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    app.project(&mut b);
    b.finish().expect("the counter projects")
}

/// The tray the app would show: the show item first, which the icon's
/// own click asks too, the counting item, and the platform's quit.
fn model(snap: &Snapshot) -> TrayModel {
    let show = MenuItem::of(snap, &sid("counter.show"), "activate").expect("show projects");
    let mut menu = Submenu::new("counter").item(show.clone()).separator();
    if let Some(up) = MenuItem::of(snap, &sid("counter.up"), "increment") {
        menu = menu.item(up);
    }
    TrayModel::new(TrayImage::mark(18), "counter")
        .menu(menu.separator().predefined(Predefined::Quit))
        .on_click(show.clicked())
}

#[test]
fn the_todo_golden_gives_a_tray_menu_of_the_same_items_the_bar_has() {
    let snap = todo();
    let items: Vec<MenuItem> = MenuItem::plain(&snap)
        .into_iter()
        .filter(|i| !matches!(snap.get(&i.sem).map(|n| n.role), Some(Role::ListItem)))
        .collect();
    let model = TrayModel::new(TrayImage::mark(18), "plev").menu(
        Submenu::new("Todo")
            .items(items.clone())
            .separator()
            .predefined(Predefined::Quit),
    );
    assert_eq!(model.items().count(), 4);
    assert_eq!(
        model.items().map(|i| i.key()).collect::<Vec<_>>(),
        [
            "todo.input#submit",
            "todo.filter[all]#select",
            "todo.filter[active]#select",
            "todo.filter[completed]#select",
        ]
    );
    assert!(
        model.on_click.is_none(),
        "nothing chosen for the icon: the menu opens"
    );
    let filter = model
        .find(&sid("todo.filter[active]"), "select", None)
        .unwrap();
    assert_eq!(filter.clicked(), items[2].clicked());
}

#[test]
fn a_click_on_the_fake_tray_is_the_request_the_button_makes_and_gets_its_receipt() {
    let mut app = Counter {
        count: 0,
        enabled: true,
        shown: 0,
    };
    let mut dispatcher = Dispatcher::new();
    let up = sid("counter.up");

    // the button: a person clicks it, seeing the page now
    let seen = project(&app);
    let by_button = Request::observe(&seen, Actor::Human, &up, "increment", None).unwrap();
    let button_id = dispatcher.dispatch(&mut app, by_button, 10);
    let button_receipt = dispatcher.receipt(button_id).unwrap().clone();
    assert_eq!(app.count, 1);

    // the tray: shown from the page, its menu clicked, polled on the loop,
    // the click observed on the page as it is now
    let mut tray = FakeTray::new();
    let seen = project(&app);
    tray.install(&model(&seen)).unwrap();
    assert!(tray.click(&up, "increment", None));
    let clicks = tray.poll();
    assert_eq!(clicks.len(), 1);
    let by_tray = clicks[0].request(&seen).unwrap();
    assert_eq!(by_tray.actor, Actor::Human);
    assert_eq!(by_tray.target, up);
    assert_eq!(
        by_tray.observed.values,
        vec![("counter:count".parse().unwrap(), Some(Value::Int(1)))]
    );
    let tray_id = dispatcher.dispatch(&mut app, by_tray, 20);
    let tray_receipt = dispatcher.receipt(tray_id).unwrap();
    assert_eq!(app.count, 2);
    let stages = |r: &accord::Receipt| r.stages.iter().map(|(s, _)| s.name()).collect::<Vec<_>>();
    assert_eq!(stages(tray_receipt), stages(&button_receipt));
    assert_eq!(
        stages(tray_receipt),
        ["requested", "accepted", "executing", "completed"]
    );

    // the icon itself: the capability the app chose, the same way
    assert!(tray.click_icon());
    let clicks = tray.poll();
    assert_eq!(clicks[0].sem, sid("counter.show"));
    let seen = project(&app);
    let show = clicks[0].request(&seen).unwrap();
    assert_eq!(show.actor, Actor::Human);
    let show_id = dispatcher.dispatch(&mut app, show, 30);
    assert!(matches!(
        dispatcher.receipt(show_id).unwrap().stage(),
        Some(Stage::Completed(_))
    ));
    assert_eq!(app.shown, 1);

    // a click that saw an older page is stale, as the button's would be
    let stale = Clicked {
        sem: up.clone(),
        action: "increment".parse().unwrap(),
        param: None,
    }
    .request(&project(&Counter {
        count: 0,
        enabled: true,
        shown: 0,
    }))
    .unwrap();
    let stale_id = dispatcher.dispatch(&mut app, stale, 40);
    assert!(matches!(
        dispatcher.receipt(stale_id).unwrap().stage(),
        Some(Stage::Rejected(accord::Rejection::Stale { .. }))
    ));
    assert_eq!(app.count, 2, "nothing ran");

    // an agent asking the icon's capability by hand is refused by the
    // policy, as it would be at the button
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let seen = project(&app);
    let req = Request::observe(&seen, agent.clone(), "counter.show", "activate", None).unwrap();
    let id = dispatcher.dispatch(&mut app, req, 50);
    assert!(matches!(
        dispatcher.receipt(id).unwrap().stage(),
        Some(Stage::Rejected(accord::Rejection::NotAuthorized { actor })) if *actor == agent
    ));
    assert_eq!(app.shown, 1);
}

#[test]
fn the_tray_follows_the_projection_disabled_and_gone() {
    let mut app = Counter {
        count: 0,
        enabled: true,
        shown: 0,
    };
    let mut tray = FakeTray::new();
    let first = model(&project(&app));
    tray.install(&first).unwrap();
    assert!(
        tray.installed()
            .unwrap()
            .find(&sid("counter.up"), "increment", None)
            .unwrap()
            .enabled
    );

    // the button is disabled now: the item follows, the shape holds
    app.enabled = false;
    let second = model(&project(&app));
    assert!(first.same_shape(&second));
    tray.update(&second).unwrap();
    assert_eq!(tray.updates(), 1);
    assert!(
        !tray.click(&sid("counter.up"), "increment", None),
        "a disabled item takes no click"
    );
    assert!(tray.click_icon(), "the icon still asks to show the window");

    // the capability is gone: the item leaves, the shape changes
    let mut b = SnapshotBuilder::new();
    b.node("counter", Role::Region, "counter")
        .state("count", 0i64);
    b.node("counter.show", Role::Button, "show the window")
        .parent("counter")
        .action("activate");
    let without = b.finish().unwrap();
    let third = model(&without);
    assert!(!second.same_shape(&third));
    assert!(third.find(&sid("counter.up"), "increment", None).is_none());
    tray.update(&third).unwrap();
    assert_eq!(tray.updates(), 2);
    assert!(!tray.click(&sid("counter.up"), "increment", None));

    // the same model again: no update
    tray.update(&third).unwrap();
    assert_eq!(tray.updates(), 2);
    assert_eq!(tray.installs(), 1);
}

#[test]
fn without_a_platform_nothing_is_shown_and_nothing_breaks() {
    let mut none: Box<dyn Tray> = Box::new(NoTray);
    let model = model(&project(&Counter {
        count: 0,
        enabled: true,
        shown: 0,
    }));
    assert_eq!(none.install(&model), Err(TrayError::Unsupported));
    assert_eq!(none.update(&model), Err(TrayError::Unsupported));
    assert_eq!(none.poll(), Vec::<Clicked>::new());
}
