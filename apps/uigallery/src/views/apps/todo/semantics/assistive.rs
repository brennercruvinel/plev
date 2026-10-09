//! The todo page as a screen reader meets it: the tree the showcase shows
//! (projected from the page, placed by the bindings of the frame), and
//! AccessKit action requests that reach the same state a person's click
//! reaches, through the page's one dispatcher, as `Actor::Assistive`.

use accord::{Actor, Rejection, Request, SemId, Stage};
use engine::control::accessibility::accesskit::{
    Action, ActionData, ActionRequest, Node, Role as AkRole, Toggled, TreeId, TreeUpdate,
};
use engine::control::accessibility::{self, Assist, Chrome, Direction, Shown, focus};
use engine::drawing::compositor::Compositor;
use engine::widgets::prelude::WidgetEvent;

use crate::views::ShowcaseView;

fn sid(text: &str) -> SemId {
    match SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}

/// The gallery on the todo page, one frame drawn.
fn todo_view() -> (ShowcaseView, Compositor) {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("todo");
    let mut c = Compositor::new();
    view.render(&mut c);
    (view, c)
}

/// A retina window: logical bindings, physical bounds.
const CHROME: Chrome<'static> = Chrome {
    title: "plev, design system showcase",
    scale: 2.0,
    size: (2400, 1600),
};

/// What a screen reader is shown of the view this frame, as the shell's
/// bridge builds it.
fn show(view: &ShowcaseView, c: &Compositor) -> (TreeUpdate, Shown) {
    let Some(snapshot) = view.semantics() else {
        panic!("the todo page projects its meaning");
    };
    let (update, ids) = accessibility::build(&snapshot, c, &CHROME);
    (update, Shown { snapshot, ids })
}

fn node<'a>(u: &'a TreeUpdate, shown: &Shown, id: &str) -> &'a Node {
    let want = shown.ids.node(&sid(id));
    match u.nodes.iter().find(|(n, _)| Some(*n) == want) {
        Some((_, n)) => n,
        None => panic!("{id} is not in the tree"),
    }
}

fn ask(shown: &Shown, id: &str, action: Action, data: Option<ActionData>) -> Assist {
    let Some(target) = shown.ids.node(&sid(id)) else {
        panic!("{id} has no node");
    };
    let req = ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: target,
        data,
    };
    match accessibility::translate(&req, shown) {
        Ok(assist) => assist,
        Err(e) => panic!("{id} {action:?}: {e}"),
    }
}

fn request(assist: Assist) -> Request {
    match assist {
        Assist::Request(req) => req,
        other => panic!("expected a request, got {other:?}"),
    }
}

#[test]
fn the_tree_is_the_todo_page_under_the_window() {
    let (view, c) = todo_view();
    let (u, shown) = show(&view, &c);
    let n = |id: &str| node(&u, &shown, id);
    let ids = |texts: &[&str]| -> Vec<_> {
        texts
            .iter()
            .filter_map(|t| shown.ids.node(&sid(t)))
            .collect()
    };

    let root = &u.nodes[0].1;
    assert_eq!(root.label(), Some("plev, design system showcase"));
    assert_eq!(root.children(), ids(&["todo"]));
    assert_eq!(n("todo").role(), AkRole::Region);
    assert_eq!(
        n("todo").children(),
        ids(&[
            "todo.input",
            "todo.filter",
            "todo.item[1]",
            "todo.item[2]",
            "todo.item[3]",
            "todo.counter",
        ])
    );

    let input = n("todo.input");
    assert_eq!(
        (input.role(), input.label()),
        (AkRole::TextInput, Some("new todo"))
    );
    assert_eq!(input.value(), Some(""));
    for a in [Action::Focus, Action::SetValue] {
        assert!(input.supports_action(a), "{a:?}");
    }
    // empty, its `submit` is off: no press the page would refuse
    // (task-75 phase 2a); typing turns it on
    assert!(!input.supports_action(Action::Click));
    assert!(!input.is_disabled(), "the field still edits");
    assert_eq!(n("todo.filter").role(), AkRole::TabList);
    assert_eq!(
        n("todo.filter").children(),
        ids(&[
            "todo.filter[all]",
            "todo.filter[active]",
            "todo.filter[completed]"
        ])
    );
    assert_eq!(n("todo.filter[all]").is_selected(), Some(true));
    assert_eq!(n("todo.filter[active]").is_selected(), Some(false));
    assert_eq!(n("todo.filter[active]").label(), Some("Active"));

    // a row holds its toggle as a checkbox and its remove as a button
    // (task-75 phase 2a: no adapter has custom actions)
    let item = n("todo.item[3]");
    assert_eq!(item.role(), AkRole::ListItem);
    assert_eq!(item.label(), Some("Port the todo domain (tested first)"));
    assert_eq!(item.toggled(), None);
    assert!(!item.supports_action(Action::Click));
    assert_eq!(
        item.children(),
        ids(&["todo.item[3][toggle]", "todo.item[3][remove]"])
    );
    let done = n("todo.item[3][toggle]");
    assert_eq!(
        (done.role(), done.label(), done.toggled()),
        (AkRole::CheckBox, Some("completed"), Some(Toggled::True))
    );
    assert!(done.supports_action(Action::Click));
    assert_eq!(n("todo.item[1][toggle]").toggled(), Some(Toggled::False));
    let remove = n("todo.item[3][remove]");
    assert_eq!(
        (remove.role(), remove.label()),
        (AkRole::Button, Some("remove"))
    );
    assert!(remove.supports_action(Action::Click));
    assert_eq!(n("todo.counter").role(), AkRole::Status);
    assert_eq!(n("todo.counter").label(), Some("2 active · 1 done"));

    // The row sits where the page drew it, in physical pixels.
    let Some(row) = c.binding(&sid("todo.item[1]")).map(|b| b.rect) else {
        panic!("the row is bound");
    };
    let Some(bounds) = n("todo.item[1]").bounds() else {
        panic!("a bound row has bounds");
    };
    assert_eq!(bounds.x0, f64::from(row[0]) * 2.0);
    assert_eq!(bounds.y1, f64::from(row[1] + row[3]) * 2.0);
    assert_eq!(
        u.focus,
        engine::control::accessibility::ROOT,
        "nothing focused yet"
    );
}

/// The acceptance: an AccessKit `Click` on a row's checkbox reaches the
/// same state as a person's click on it, and the receipt names assistive
/// technology.
#[test]
fn a_screen_reader_click_toggles_a_row_as_a_person_does() {
    let (mut person, c) = todo_view();
    let Some(row) = c.binding(&sid("todo.item[1]")).map(|b| b.rect) else {
        panic!("the row is bound");
    };
    // The row's toggle area starts just inside its left edge.
    let (x, y) = (row[0] + 20.0, row[1] + row[3] / 2.0);
    person.handle_event(&WidgetEvent::MouseDown { x, y });
    person.handle_event(&WidgetEvent::MouseUp { x, y });
    assert!(person.panes.todo.model.visible_items()[0].completed());

    let (mut reader, c) = todo_view();
    let (_, shown) = show(&reader, &c);
    let req = request(ask(&shown, "todo.item[1][toggle]", Action::Click, None));
    assert_eq!(req.actor, Actor::Assistive);
    assert_eq!(req.target, sid("todo.item[1]"), "the row's own toggle");
    assert_eq!(req.action.as_str(), "toggle");
    let Some(receipt) = reader.act(req) else {
        panic!("the todo page takes the request");
    };
    assert_eq!(receipt.request.actor, Actor::Assistive);
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(reader.semantics(), person.semantics(), "the same state");

    // The next frame tells the reader the row is done.
    let mut c = Compositor::new();
    reader.render(&mut c);
    let (u, shown) = show(&reader, &c);
    assert_eq!(
        node(&u, &shown, "todo.item[1][toggle]").toggled(),
        Some(Toggled::True)
    );
}

/// The row's remove, which no action of the row reached before phase 2a:
/// a reader presses the button the tree presents, and the row is gone
/// as a person's click on its delete takes it.
#[test]
fn a_screen_reader_removes_a_row_through_its_presented_button() {
    let (mut view, c) = todo_view();
    let (_, shown) = show(&view, &c);
    let req = request(ask(&shown, "todo.item[2][remove]", Action::Click, None));
    assert_eq!(
        (req.target.as_str(), req.action.as_str(), &req.actor),
        ("todo.item[2]", "remove", &Actor::Assistive)
    );
    let total = view.panes.todo.model.counts().total;
    let Some(receipt) = view.act(req) else {
        panic!("the todo page takes the request");
    };
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(view.panes.todo.model.counts().total, total - 1);
    let mut c = Compositor::new();
    view.render(&mut c);
    let (_, shown) = show(&view, &c);
    assert_eq!(shown.ids.node(&sid("todo.item[2]")), None);
    assert_eq!(shown.ids.node(&sid("todo.item[2][remove]")), None);
}

#[test]
fn with_a_modal_open_the_same_request_comes_back_inert() {
    let (mut view, c) = todo_view();
    let (_, before) = show(&view, &c);
    view.open_modal(false);
    let mut c = Compositor::new();
    view.render(&mut c);
    let (u, now) = show(&view, &c);
    assert!(node(&u, &now, "todo").is_hidden(), "the page is inert");
    assert!(node(&u, &now, "todo.item[1]").is_hidden());

    for seen in [&before, &now] {
        let req = request(ask(seen, "todo.item[1][toggle]", Action::Click, None));
        let Some(receipt) = view.act(req) else {
            panic!("the todo page takes the request");
        };
        assert_eq!(
            receipt.stage(),
            Some(&Stage::Rejected(Rejection::Inert {
                target: sid("todo.item[1]")
            }))
        );
    }
    assert!(!view.panes.todo.model.visible_items()[0].completed());
    assert!(
        !view.assist_focus(&sid("todo.input")),
        "no focus behind a modal"
    );
}

/// The VoiceOver path: focus the field, type, then press it (its primary
/// action is `submit`), which adds the todo.
#[test]
fn focus_type_and_press_the_field_to_add_a_todo() {
    let (mut view, c) = todo_view();
    let (_, shown) = show(&view, &c);
    let assist = ask(&shown, "todo.input", Action::Focus, None);
    assert_eq!(assist, Assist::Focus(sid("todo.input")));
    assert!(view.assist_focus(&sid("todo.input")));
    for ch in "buy milk".chars() {
        assert!(view.handle_key(&ch.to_string()), "the field takes typing");
    }
    let mut c = Compositor::new();
    view.render(&mut c);
    let (u, shown) = show(&view, &c);
    assert_eq!(Some(u.focus), shown.ids.node(&sid("todo.input")));
    assert_eq!(node(&u, &shown, "todo.input").value(), Some("buy milk"));
    assert!(
        node(&u, &shown, "todo.input").supports_action(Action::Click),
        "with text, the press is offered"
    );

    let req = request(ask(&shown, "todo.input", Action::Click, None));
    assert_eq!(req.action.as_str(), "submit");
    let total = view.panes.todo.model.counts().total;
    let Some(receipt) = view.act(req) else {
        panic!("the todo page takes the request");
    };
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(view.panes.todo.model.counts().total, total + 1);

    // SetValue replaces the text for a reader that does not type.
    let (_, shown) = show(&view, &c);
    let text = Some(ActionData::Value("call mom".into()));
    let req = request(ask(&shown, "todo.input", Action::SetValue, text));
    let Some(receipt) = view.act(req) else {
        panic!("the todo page takes the request");
    };
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(view.panes.todo.input.text(), "call mom");
}

/// Tab follows the snapshot: the field, then the three filter tabs.
#[test]
fn the_keyboard_order_of_the_page_follows_its_snapshot() {
    let (view, _) = todo_view();
    let Some(s) = view.semantics() else {
        panic!("the todo page projects its meaning");
    };
    let order: Vec<&str> = focus::order(&s).iter().map(|id| id.as_str()).collect();
    assert_eq!(
        order,
        [
            "todo.input",
            "todo.filter[all]",
            "todo.filter[active]",
            "todo.filter[completed]"
        ]
    );
    assert_eq!(focus::next(&s, Direction::Next), Some(sid("todo.input")));
    assert_eq!(
        focus::next(&s, Direction::Previous),
        Some(sid("todo.filter[completed]"))
    );
}

#[test]
fn a_page_without_a_projection_has_no_tree_yet() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("Card");
    assert!(view.semantics().is_none());
    let Some(req) = todo_view()
        .0
        .semantics()
        .and_then(|s| Request::observe(&s, Actor::Assistive, "todo.item[1]", "toggle", None).ok())
    else {
        panic!("a request over the todo page");
    };
    assert!(view.act(req).is_none(), "no page takes it");
    assert!(!view.assist_focus(&sid("todo.input")));
}

/// The cost of the tree per frame while a reader listens: projection and
/// build, at 3 and 103 items. Run in release, by hand:
/// `cargo test -p uigallery --release tree_cost -- --ignored --nocapture`.
#[test]
#[ignore = "a measurement, not a check"]
fn tree_cost_per_frame() {
    for extra in [0usize, 100] {
        let (mut view, _) = todo_view();
        for i in 0..extra {
            view.panes.todo.model.add(&format!("item {i}"));
        }
        let mut c = Compositor::new();
        view.render(&mut c);
        let rounds = 200u32;
        let mut best = (u128::MAX, u128::MAX);
        for _ in 0..7 {
            let start = std::time::Instant::now();
            let mut nodes = 0;
            for _ in 0..rounds {
                let Some(s) = view.semantics() else {
                    panic!("projects");
                };
                let (u, _) = accessibility::build(&s, &c, &CHROME);
                nodes = u.nodes.len();
            }
            let both = start.elapsed().as_nanos() / u128::from(rounds);
            let Some(s) = view.semantics() else {
                panic!("projects");
            };
            let start = std::time::Instant::now();
            for _ in 0..rounds {
                let _ = accessibility::build(&s, &c, &CHROME);
            }
            let build = start.elapsed().as_nanos() / u128::from(rounds);
            best = (best.0.min(both), best.1.min(build));
            assert!(nodes > extra);
        }
        println!(
            "{} items: projection and tree {} ns, tree alone {} ns",
            3 + extra,
            best.0,
            best.1
        );
    }
}
