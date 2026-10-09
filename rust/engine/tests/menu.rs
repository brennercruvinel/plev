//! Native menus as another view of an app's capabilities (task-76 item
//! 4), proven without a platform: the model built from two real
//! projections of the showcase (the goldens of the todo and the silences
//! pages), a click on the fake menu becoming the request the button
//! makes and the same receipt, the menu following a moved projection (an
//! item disabled, an item gone), and the target without a menu.

use accord::{
    Actor, Dispatcher, Execution, Operable, Outcome, Request, RequestId, Role, SemId, Semantics,
    Snapshot, SnapshotBuilder, Stage, Value,
};
use engine::runtime::platform::menu::{
    Clicked, FakeMenu, MenuError, MenuItem, MenuModel, NativeMenu, NoMenu, Predefined, Shortcut,
    Submenu,
};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

fn golden(json: &str) -> Snapshot {
    serde_json::from_str(json).expect("the golden parses")
}

fn todo() -> Snapshot {
    golden(include_str!(
        "../../plevir/tests/fixtures/uigallery-todo.snapshot.json"
    ))
}

fn silences() -> Snapshot {
    golden(include_str!(
        "../../plevir/tests/fixtures/uigallery-silences.snapshot.json"
    ))
}

fn keys(items: &[MenuItem]) -> Vec<String> {
    items.iter().map(MenuItem::key).collect()
}

/// The page menu the showcase would build: every action a person may ask
/// without a value, the rows left out (their actions belong to the row on
/// screen).
fn page_menu(snapshot: &Snapshot, title: &str) -> Submenu {
    let items = MenuItem::plain(snapshot).into_iter().filter(|item| {
        !matches!(
            snapshot.get(&item.sem).map(|n| n.role),
            Some(Role::ListItem | Role::Row | Role::Cell | Role::TreeItem | Role::Option)
        )
    });
    Submenu::new(title).items(items)
}

#[test]
fn the_todo_golden_gives_the_filters_and_a_submit_that_waits_for_text() {
    let snap = todo();
    let all = MenuItem::plain(&snap);
    assert_eq!(
        keys(&all),
        [
            "todo.input#submit",
            "todo.filter[all]#select",
            "todo.filter[active]#select",
            "todo.filter[completed]#select",
            "todo.item[1]#toggle",
            "todo.item[1]#remove",
            "todo.item[2]#toggle",
            "todo.item[2]#remove",
            "todo.item[3]#toggle",
            "todo.item[3]#remove",
        ],
        "set_value takes a text, so it is not a plain item"
    );
    let page = page_menu(&snap, "Todo");
    let items: Vec<&MenuItem> = page.items_deep().collect();
    assert_eq!(
        items.iter().map(|i| i.label.as_str()).collect::<Vec<_>>(),
        ["new todo", "All", "Active", "Completed"]
    );
    assert!(
        !items[0].enabled,
        "submit is disabled while the field is blank"
    );
    assert!(items[1..].iter().all(|i| i.enabled));
    assert!(items.iter().all(|i| i.checked.is_none()));
    assert!(items.iter().all(|i| i.param.is_none()));
}

#[test]
fn the_silences_golden_gives_the_page_actions_and_leaves_the_cuts_and_the_values_out() {
    let snap = silences();
    let page = page_menu(&snap, "Silences");
    let items: Vec<&MenuItem> = page.items_deep().collect();
    let labels: Vec<(&str, &str)> = items
        .iter()
        .map(|i| (i.label.as_str(), i.action.as_str()))
        .collect();
    assert_eq!(
        labels,
        [
            ("threshold (dB)", "increment"),
            ("threshold (dB)", "decrement"),
            ("minimum silence (s)", "increment"),
            ("minimum silence (s)", "decrement"),
            ("detect silences", "detect_silence"),
            ("apply all", "apply_all"),
        ]
    );
    assert!(items.iter().all(|i| i.enabled));
    // the cuts' actions exist for a person, on the rows
    let all = MenuItem::plain(&snap);
    assert!(all.iter().any(|i| i.key() == "silences.cut[1]#apply_cut"));
    assert!(
        all.iter().all(|i| i.action.as_str() != "set_value"),
        "a slider's set_value takes a number"
    );
}

/// A counter with one button: the app under test. `increment` runs when
/// the button is enabled; the count is projected, and the button reads
/// it, so a menu click that saw an older count is stale, as a click on
/// the button would be.
struct Counter {
    count: i64,
    enabled: bool,
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
    }
}

impl Operable for Counter {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        if req.action.as_str() != "increment" {
            return Execution::Failed(format!("no {}", req.action));
        }
        self.count += 1;
        let mut outcome = Outcome::default();
        outcome
            .result
            .insert("count".to_owned(), Value::Int(self.count));
        Execution::Done(outcome)
    }
}

fn project(app: &Counter) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    app.project(&mut b);
    b.finish().expect("the counter projects")
}

fn model(snap: &Snapshot) -> MenuModel {
    MenuModel::new()
        .menu(
            Submenu::new("plev")
                .predefined(Predefined::About)
                .separator()
                .predefined(Predefined::Quit),
        )
        .menu(page_menu(snap, "Counter"))
}

#[test]
fn a_click_on_the_fake_menu_is_the_request_the_button_makes_and_gets_its_receipt() {
    let mut app = Counter {
        count: 0,
        enabled: true,
    };
    let mut dispatcher = Dispatcher::new();
    let up = sid("counter.up");

    // the button: a person clicks it, seeing the page now
    let seen = project(&app);
    let by_button = Request::observe(&seen, Actor::Human, &up, "increment", None).unwrap();
    let button_id = dispatcher.dispatch(&mut app, by_button.clone(), 10);
    let button_receipt = dispatcher.receipt(button_id).unwrap().clone();
    assert_eq!(app.count, 1);

    // the menu: shown from the page, clicked, polled on the loop, the
    // click observed on the page as it is now
    let mut menu = FakeMenu::new();
    let seen = project(&app);
    menu.install(&model(&seen)).unwrap();
    assert!(menu.click(&up, "increment", None));
    let clicks = menu.poll();
    assert_eq!(clicks.len(), 1);
    let by_menu = clicks[0].request(&seen).unwrap();
    assert_eq!(by_menu.actor, Actor::Human);
    assert_eq!(by_menu.target, up);
    assert_eq!(by_menu.action.as_str(), "increment");
    assert_eq!(
        by_menu.observed.values,
        vec![("counter:count".parse().unwrap(), Some(Value::Int(1)))]
    );
    let menu_id = dispatcher.dispatch(&mut app, by_menu, 20);
    let menu_receipt = dispatcher.receipt(menu_id).unwrap();
    assert_eq!(app.count, 2);

    // the same stages, in the same order, with the outcome the button got
    let stages = |r: &accord::Receipt| r.stages.iter().map(|(s, _)| s.name()).collect::<Vec<_>>();
    assert_eq!(stages(menu_receipt), stages(&button_receipt));
    assert_eq!(
        stages(menu_receipt),
        ["requested", "accepted", "executing", "completed"]
    );
    let Some(Stage::Completed(outcome)) = menu_receipt.stage() else {
        panic!("completed: {:?}", menu_receipt.stage());
    };
    assert_eq!(outcome.result["count"], Value::Int(2));

    // a click that saw an older page is stale, as the button's would be
    let stale = clicks[0].request(&seen).unwrap();
    let stale_id = dispatcher.dispatch(&mut app, stale, 30);
    assert!(matches!(
        dispatcher.receipt(stale_id).unwrap().stage(),
        Some(Stage::Rejected(accord::Rejection::Stale { .. }))
    ));
    assert_eq!(app.count, 2, "nothing ran");
}

#[test]
fn the_menu_follows_the_projection_disabled_and_gone() {
    let mut app = Counter {
        count: 0,
        enabled: true,
    };
    let mut menu = FakeMenu::new();
    let first = model(&project(&app));
    menu.install(&first).unwrap();
    assert!(
        menu.installed()
            .unwrap()
            .find(&sid("counter.up"), "increment", None)
            .unwrap()
            .enabled
    );

    // the button is disabled now: the item follows, the shape holds
    app.enabled = false;
    let second = model(&project(&app));
    assert!(first.same_shape(&second));
    menu.update(&second).unwrap();
    assert_eq!(menu.updates(), 1);
    let item = menu
        .installed()
        .unwrap()
        .find(&sid("counter.up"), "increment", None)
        .unwrap();
    assert!(!item.enabled);
    assert!(
        !menu.click(&sid("counter.up"), "increment", None),
        "a disabled item takes no click"
    );

    // the capability is gone: the item leaves, the shape changes
    let mut b = SnapshotBuilder::new();
    b.node("counter", Role::Region, "counter")
        .state("count", 0i64);
    let without = b.finish().unwrap();
    let third = model(&without);
    assert!(!second.same_shape(&third));
    assert!(third.find(&sid("counter.up"), "increment", None).is_none());
    menu.update(&third).unwrap();
    assert!(
        menu.installed().unwrap().menus[1].is_empty(),
        "the page menu shows nothing to ask"
    );
    assert_eq!(menu.updates(), 2);

    // the same model again: no update
    menu.update(&third).unwrap();
    assert_eq!(menu.updates(), 2);
}

#[test]
fn a_shortcut_shows_on_an_item_and_a_bare_key_is_refused() {
    let snap = project(&Counter {
        count: 0,
        enabled: true,
    });
    let item = MenuItem::of(&snap, &sid("counter.up"), "increment")
        .unwrap()
        .shortcut(Shortcut::parse("cmd-shift-s").unwrap())
        .label("count up (menu)");
    assert_eq!(item.label, "count up (menu)");
    assert_eq!(item.shortcut.unwrap().key, 's');
    assert_eq!(
        Shortcut::parse("d"),
        Err(MenuError::BadShortcut("d".to_owned())),
        "a bare letter would take typing away"
    );
}

#[test]
fn without_a_platform_nothing_is_shown_and_nothing_breaks() {
    let snap = todo();
    let mut none: Box<dyn NativeMenu> = Box::new(NoMenu);
    let bar = model(&snap);
    assert_eq!(none.install(&bar), Err(MenuError::Unsupported));
    assert_eq!(none.update(&bar), Err(MenuError::Unsupported));
    assert_eq!(none.poll(), Vec::<Clicked>::new());
}
