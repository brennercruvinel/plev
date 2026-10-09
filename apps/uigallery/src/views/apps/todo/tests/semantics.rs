//! The todo page as a capability (task-59): what it projects, clicks and
//! test requests reaching the same state through the one dispatcher, the
//! stale case, the bindings beside the scene, and the inert background.

use super::*;
use crate::views::apps::todo::semantics::bounds;
use accord::{
    Actor, Change, Rejection, Request, Role, SemId, Snapshot, Stage, StateRef, Value, Verdict,
};

fn tester() -> Actor {
    Actor::Test {
        name: "todo".to_owned(),
    }
}

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

/// An `Actor::Test` request over the page as it is now, through the
/// page's dispatcher; the stage its receipt ended at.
fn ask_as_test(s: &mut TodoPage, target: &str, action: &str, param: Option<Value>) -> Stage {
    let seen = s.snapshot().unwrap();
    let req = Request::observe(&seen, tester(), target, action, param).unwrap();
    let id = s.dispatch(req);
    s.receipt(id).unwrap().stage().unwrap().clone()
}

fn items(s: &TodoPage) -> Vec<(String, bool)> {
    s.model
        .visible_items()
        .iter()
        .map(|i| (i.text().to_owned(), i.completed()))
        .collect()
}

#[test]
fn the_page_projects_the_todo_model() {
    assert!(Ids::new().is_ok(), "the constant ids parse");
    let s = TodoPage::new();
    let snap = s.snapshot().unwrap();
    let node = |text: &str| snap.get(&sid(text)).unwrap();

    assert_eq!(node("todo").role, Role::Region);
    assert!(!node("todo").inert);
    let input = node("todo.input");
    assert_eq!(input.role, Role::TextInput);
    assert!(input.constraints.editable);
    let actions: Vec<&str> = input.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(actions, ["set_value", "submit"]);
    for action in &input.actions {
        let reads: Vec<String> = action.reads.iter().map(ToString::to_string).collect();
        assert_eq!(reads, ["todo.input:text"], "{}", action.name);
    }
    assert!(!input.actions[1].enabled, "a blank field cannot submit");

    assert_eq!(node("todo.filter").role, Role::TabList);
    assert_eq!(
        node("todo.filter").state.get("selected"),
        Some(&Value::from("all"))
    );
    for tab in ["all", "active", "completed"] {
        let t = node(&format!("todo.filter[{tab}]"));
        assert_eq!(t.role, Role::Tab);
        assert_eq!(t.parent, Some(sid("todo.filter")));
        assert_eq!(t.actions[0].name.as_str(), "select");
        assert_eq!(t.actions[0].reads[0].to_string(), "todo.filter:selected");
    }

    // The seed: three items, the third completed.
    let item = node("todo.item[3]");
    assert_eq!(item.role, Role::ListItem);
    assert_eq!(item.label, "Port the todo domain (tested first)");
    assert_eq!(item.state.get("completed"), Some(&Value::Bool(true)));
    assert_eq!(item.actions[0].name.as_str(), "toggle");
    assert_eq!(
        item.actions[0].reads[0].to_string(),
        "todo.item[3]:completed"
    );
    assert_eq!(item.actions[1].name.as_str(), "remove");
    assert!(item.actions[1].policy.destructive);

    let counter = node("todo.counter");
    assert_eq!(counter.role, Role::Status);
    assert_eq!(counter.state.get("active"), Some(&Value::Int(2)));
    assert_eq!(counter.state.get("done"), Some(&Value::Int(1)));
    assert_eq!(snap.focused(), None);
}

/// `todo:items` lists the ids of the visible items in the order shown
/// (task-95). It follows the filter as the item nodes do, so every id in
/// it is a node of the page under `todo`, in the same order, and every
/// item node is in it; each item carries its text as state, which a
/// generated surface binds (its instances are named by the id). Adding,
/// toggling under a filter, removing and filtering move it.
#[test]
fn the_items_are_projected_as_a_list_in_the_order_shown() {
    let mut s = TodoPage::new();
    let items = StateRef::parse("todo:items").unwrap();
    let list = |s: &TodoPage| -> Vec<String> {
        let snap = s.snapshot().unwrap();
        let Some(Value::List(ids)) = snap.value(&items).cloned() else {
            panic!("todo:items is a list: {:?}", snap.value(&items));
        };
        let nodes: Vec<String> = snap
            .nodes()
            .values()
            .filter(|n| n.role == Role::ListItem)
            .map(|n| {
                assert_eq!(n.parent, Some(sid("todo")), "{}", n.id);
                assert_eq!(n.state.get("text"), Some(&Value::from(n.label.as_str())));
                n.id.to_string()
            })
            .collect();
        let named: Vec<String> = ids.iter().map(|id| format!("todo.item[{id}]")).collect();
        assert_eq!(named, nodes, "the list is the item nodes, in order");
        ids
    };
    let done = |stage: Stage| assert!(matches!(stage, Stage::Completed(_)), "{stage:?}");
    assert_eq!(list(&s), ["1", "2", "3"]);

    let milk = Some(Value::from("buy milk"));
    done(ask_as_test(&mut s, "todo.input", "set_value", milk));
    done(ask_as_test(&mut s, "todo.input", "submit", None));
    assert_eq!(list(&s), ["1", "2", "3", "4"], "a new item comes last");

    done(ask_as_test(&mut s, "todo.filter[active]", "select", None));
    assert_eq!(list(&s), ["1", "2", "4"], "the filter hides the done one");
    done(ask_as_test(&mut s, "todo.item[2]", "toggle", None));
    assert_eq!(list(&s), ["1", "4"], "done, it leaves the active list");
    done(ask_as_test(&mut s, "todo.item[1]", "remove", None));
    assert_eq!(list(&s), ["4"]);
    done(ask_as_test(
        &mut s,
        "todo.filter[completed]",
        "select",
        None,
    ));
    assert_eq!(list(&s), ["2", "3"]);
    done(ask_as_test(&mut s, "todo.filter[all]", "select", None));
    assert_eq!(list(&s), ["2", "3", "4"]);
    done(ask_as_test(&mut s, "todo.item[4]", "remove", None));
    done(ask_as_test(&mut s, "todo.item[2]", "remove", None));
    done(ask_as_test(&mut s, "todo.item[3]", "remove", None));
    assert!(list(&s).is_empty(), "no item, an empty list");
}

#[test]
fn clicks_and_test_requests_reach_the_same_state_and_snapshot() {
    let content = content_at(1500.0);

    // A person: types and presses Enter, clicks the new row's checkbox,
    // the second row's delete button and the Completed pill.
    let mut human = TodoPage::new();
    add(&mut human, content, "buy milk");
    let l = lay(&human, content);
    let cb = checkbox_rect(row_rect(l.list, 3, human.scroll.offset()));
    let r = click(&mut human, content, cb.x + 9.0, cb.y + ROW_H / 2.0);
    assert!(r.clicked && r.changed, "a toggle asks for a frame");
    let (dx, dy) = delete_rect(row_rect(lay(&human, content).list, 1, 0.0)).center();
    let r = click(&mut human, content, dx, dy);
    assert!(r.clicked && r.changed, "a remove asks for a frame");
    let (px, py) = lay(&human, content).pills[2].center();
    let r = click(&mut human, content, px, py);
    assert!(r.clicked && r.changed, "a filter change asks for a frame");
    assert!(!human.input.is_focused(), "the first click away blurred it");

    // A test: the same capabilities, by request.
    let mut test = TodoPage::new();
    let done = |stage: Stage| assert!(matches!(stage, Stage::Completed(_)), "{stage:?}");
    done(ask_as_test(
        &mut test,
        "todo.input",
        "set_value",
        Some("buy milk".into()),
    ));
    done(ask_as_test(&mut test, "todo.input", "submit", None));
    done(ask_as_test(&mut test, "todo.item[4]", "toggle", None));
    done(ask_as_test(&mut test, "todo.item[2]", "remove", None));
    done(ask_as_test(
        &mut test,
        "todo.filter[completed]",
        "select",
        None,
    ));

    assert_eq!(human.model.counts(), test.model.counts());
    assert_eq!(human.model.filter(), Filter::Completed);
    assert_eq!(items(&human), items(&test));
    assert_eq!(
        items(&test),
        [
            ("Port the todo domain (tested first)".to_owned(), true),
            ("buy milk".to_owned(), true)
        ]
    );
    let (a, b): (Snapshot, Snapshot) = (human.snapshot().unwrap(), test.snapshot().unwrap());
    assert_eq!(a, b);
    assert_eq!(a.revision(), b.revision());
}

#[test]
fn submit_returns_the_new_item_and_a_blank_field_cannot_submit() {
    let mut s = TodoPage::new();
    ask_as_test(&mut s, "todo.input", "set_value", Some("  ".into()));
    let stage = ask_as_test(&mut s, "todo.input", "submit", None);
    assert!(
        matches!(stage, Stage::Rejected(Rejection::Disabled { .. })),
        "{stage:?}"
    );
    assert_eq!(s.model.counts().total, 3);

    ask_as_test(&mut s, "todo.input", "set_value", Some("ship it".into()));
    let Stage::Completed(outcome) = ask_as_test(&mut s, "todo.input", "submit", None) else {
        panic!("submit completes");
    };
    assert_eq!(
        outcome.result.get("item"),
        Some(&Value::from("todo.item[4]"))
    );
    assert!(s.input.is_empty(), "submit clears the field");

    // A parameter of the wrong kind never reaches execute.
    let stage = ask_as_test(&mut s, "todo.input", "set_value", Some(Value::Int(3)));
    assert!(matches!(
        stage,
        Stage::Rejected(Rejection::InvalidParam { .. })
    ));
}

#[test]
fn a_test_submit_goes_stale_when_the_text_moves_and_not_when_the_caret_or_hover_does() {
    let content = content_at(1500.0);
    let mut s = TodoPage::new();
    ask_as_test(&mut s, "todo.input", "set_value", Some("a".into()));
    let seen = s.snapshot().unwrap();
    let text = StateRef::parse("todo.input:text").unwrap();
    assert_eq!(seen.value(&text), Some(&Value::from("a")));
    let stale = Request::observe(&seen, tester(), "todo.input", "submit", None).unwrap();

    // The person types a "b" after the test looked.
    focus(&mut s, content);
    assert!(s.handle_edit_key(EditKey::End));
    assert!(s.handle_text("b"));
    assert_eq!(s.input.text(), "ab");
    let id = s.dispatch(stale);
    let Some(Stage::Rejected(Rejection::Stale { changes, .. })) = s.receipt(id).unwrap().stage()
    else {
        panic!("the submit must go stale");
    };
    assert!(matches!(
        &changes[..],
        [Change::StateChanged { at, before: Some(Value::Text(b)), after: Some(Value::Text(a)), .. }]
            if *at == text && b == "a" && a == "ab"
    ));
    assert_eq!(s.model.counts().total, 3, "nothing was added");

    // Observed again, then only the caret, the hover and the focus move:
    // none of them is state the submit reads.
    let seen = s.snapshot().unwrap();
    let fresh = Request::observe(&seen, tester(), "todo.input", "submit", None).unwrap();
    assert!(s.handle_edit_key(EditKey::Left));
    let (hx, hy) = row_rect(lay(&s, content).list, 0, 0.0).center();
    s.handle_event(
        &WidgetEvent::MouseMove { x: hx, y: hy },
        content,
        &Theme::hoff(),
    );
    assert!(s.hover_row.is_some());
    assert!(s.handle_escape());
    let id = s.dispatch(fresh);
    assert!(matches!(
        s.receipt(id).unwrap().stage(),
        Some(Stage::Completed(_))
    ));
    assert_eq!(s.model.visible_items().last().unwrap().text(), "ab");
}

#[test]
fn a_test_tab_select_goes_stale_when_the_filter_moves() {
    let content = content_at(1500.0);
    let mut s = TodoPage::new();
    let seen = s.snapshot().unwrap();
    let stale =
        Request::observe(&seen, tester(), "todo.filter[completed]", "select", None).unwrap();

    // The person clicks the Active pill after the test looked.
    let (px, py) = lay(&s, content).pills[1].center();
    assert!(click(&mut s, content, px, py).clicked);
    assert_eq!(s.model.filter(), Filter::Active);
    let id = s.dispatch(stale);
    let Some(Stage::Rejected(Rejection::Stale { changes, .. })) = s.receipt(id).unwrap().stage()
    else {
        panic!("the switch must go stale");
    };
    let selected = StateRef::parse("todo.filter:selected").unwrap();
    assert!(matches!(
        &changes[..],
        [Change::StateChanged { at, before: Some(Value::Text(b)), after: Some(Value::Text(a)), .. }]
            if *at == selected && b == "all" && a == "active"
    ));
    assert_eq!(
        s.model.filter(),
        Filter::Active,
        "the person's choice holds"
    );

    // Observed again, the switch goes through.
    let stage = ask_as_test(&mut s, "todo.filter[completed]", "select", None);
    assert!(matches!(stage, Stage::Completed(_)), "{stage:?}");
    assert_eq!(s.model.filter(), Filter::Completed);
}

#[test]
fn narrow_and_wide_viewports_move_the_bindings_and_keep_the_ids() {
    let mut s = TodoPage::new();
    let (narrow, wide) = (content_at(600.0), content_at(1500.0));
    let a = render(&mut s, narrow);
    let b = render(&mut s, wide);
    let ids =
        |c: &Compositor| -> Vec<String> { c.bindings().map(|(id, _)| id.to_string()).collect() };
    assert_eq!(ids(&a), ids(&b), "the same capabilities are bound");
    assert_eq!(
        ids(&a),
        [
            "todo",
            "todo.input",
            "todo.item[1]",
            "todo.item[2]",
            "todo.item[3]",
            "todo.filter[all]",
            "todo.filter[active]",
            "todo.filter[completed]"
        ]
    );

    let input = sid("todo.input");
    let (ra, rb) = (a.binding(&input).unwrap(), b.binding(&input).unwrap());
    assert_ne!(ra.rect, rb.rect, "the field moved with the layout");
    for (c, content) in [(&a, narrow), (&b, wide)] {
        let l = lay(&s, content);
        // One geometry: the bound rect is the rect the page hit-tests.
        assert_eq!(c.binding(&input).unwrap().rect, bounds(l.input));
        let (x, y) = l.input.center();
        assert_eq!(c.hit(x, y), Some(&input));
        // The delete button sits in its row: the hit names the item.
        let (dx, dy) = delete_rect(row_rect(l.list, 1, 0.0)).center();
        assert_eq!(c.hit(dx, dy), Some(&sid("todo.item[2]")));
        let (px, py) = l.pills[1].center();
        assert_eq!(c.hit(px, py), Some(&sid("todo.filter[active]")));
        // Rows bind inside the list clip; the card margin is the region.
        let row = c.binding(&sid("todo.item[1]")).unwrap();
        assert_eq!(row.clip, Some(bounds(l.list)));
        assert_eq!(c.hit(l.panel.x + 2.0, l.panel.y + 4.0), Some(&sid("todo")));
    }
}

#[test]
fn rows_scrolled_out_of_the_list_are_not_bound_but_stay_projected() {
    let content = content_at(1500.0);
    let mut s = TodoPage::new();
    for i in 0..14 {
        add(&mut s, content, &format!("item {i}"));
    }
    blur(&mut s, content);
    assert!(s.scroll.offset() > 0.0, "enter revealed the newest row");
    let c = render(&mut s, content);
    let first = sid("todo.item[1]");
    assert!(c.binding(&first).is_none(), "scrolled away: no rect");
    assert!(
        s.snapshot().unwrap().get(&first).is_some(),
        "still projected"
    );
    assert!(c.binding(&sid("todo.item[17]")).is_some());
}

#[test]
fn an_inert_page_refuses_every_request_and_takes_no_input() {
    let content = content_at(1500.0);
    let mut s = TodoPage::new();
    focus(&mut s, content);
    s.set_inert(true);
    let snap = s.snapshot().unwrap();
    assert!(snap.get(&sid("todo.item[1]")).unwrap().inert, "inherited");

    let stage = ask_as_test(&mut s, "todo.item[1]", "toggle", None);
    assert_eq!(
        stage,
        Stage::Rejected(Rejection::Inert {
            target: sid("todo.item[1]")
        })
    );
    assert!(!s.model.visible_items()[0].completed());
    assert!(!s.handle_text("x"), "keys fall through to the chrome");
    assert!(!s.handle_escape(), "Escape is the overlay's");

    // The request a click would build is refused the same way.
    let seen = s.snapshot().unwrap();
    let req = Request::observe(&seen, Actor::Human, "todo.item[2]", "remove", None).unwrap();
    assert!(matches!(
        accord::check(&req, &seen),
        Verdict::Rejected(Rejection::Inert { .. })
    ));

    s.set_inert(false);
    let stage = ask_as_test(&mut s, "todo.item[1]", "toggle", None);
    assert!(matches!(stage, Stage::Completed(_)));
}

#[test]
fn an_open_modal_leaves_the_todo_page_inert() {
    use crate::views::ShowcaseView;

    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("todo");
    let mut c = Compositor::new();
    view.render(&mut c);
    let (x, y) = view
        .panes
        .todo
        .input_rect(view.page_rect(), &view.theme)
        .center();
    assert!(view.handle_event(&WidgetEvent::MouseDown { x, y }));
    assert!(view.handle_key("n"));
    let total = view.panes.todo.model.counts().total;

    view.open_modal(false);
    view.render(&mut c);
    let stage = ask_as_test(&mut view.panes.todo, "todo.item[1]", "toggle", None);
    assert!(
        matches!(stage, Stage::Rejected(Rejection::Inert { .. })),
        "{stage:?}"
    );
    // Enter is the modal's while it holds the pointer (task-75 phase
    // 2c): it presses the focused cancel; the page beneath never sees it.
    assert!(view.handle_enter(), "Enter is consumed");
    assert_eq!(view.panes.todo.model.counts().total, total, "and refused");

    // The modal goes away: the page takes requests again.
    assert!(!view.overlay_holds_pointer(), "the cancel answered it");
    for _ in 0..600 {
        if !view.tick(1.0 / 60.0) {
            break;
        }
    }
    view.render(&mut c);
    assert!(view.handle_enter());
    assert_eq!(view.panes.todo.model.counts().total, total + 1);
}

/// The cost of the semantic layer on this page, per frame: one
/// projection (what accessibility will do each frame) and the bindings
/// (a render with ids against one without). Printed, not asserted: there
/// is no target to invent. For the numbers, run it in release:
/// `cargo test -p showcase --release --lib semantic_cost -- --nocapture`.
#[test]
fn semantic_cost_per_frame() {
    use std::time::Duration;

    /// The fastest of `ROUNDS` averages of `RUNS` calls: the least noisy
    /// estimate of the steady cost.
    fn best(mut f: impl FnMut()) -> Duration {
        const ROUNDS: usize = 7;
        const RUNS: u32 = 200;
        (0..ROUNDS)
            .map(|_| {
                let start = web_time::Instant::now();
                for _ in 0..RUNS {
                    f();
                }
                start.elapsed() / RUNS
            })
            .min()
            .unwrap_or_default()
    }

    let content = content_at(1500.0);
    for extra in [0usize, 100] {
        let mut s = TodoPage::new();
        for i in 0..extra {
            s.model.add(&format!("item {i}"));
        }
        let mut bare = TodoPage::new();
        bare.model = s.model.clone();
        bare.ids = None;
        bare.input = TextField::new("What needs doing? Enter adds it.");

        let project = best(|| assert!(s.snapshot().is_ok()));
        let (mut c, mut d) = (Compositor::new(), Compositor::new());
        let frame = |c: &mut Compositor, page: &mut TodoPage| {
            c.begin_frame();
            page.render(c, content, &Theme::hoff());
        };
        // Warm the text caches both renders share before timing either.
        frame(&mut c, &mut s);
        frame(&mut d, &mut bare);
        let bound = best(|| frame(&mut c, &mut s));
        let unbound = best(|| frame(&mut d, &mut bare));
        let n = c.bindings().count();
        assert!(n > 0 && d.bindings().count() == 0);
        println!(
            "todo, {} items: projection {project:?}; render {bound:?} with {n} bindings, \
             {unbound:?} without",
            s.model.counts().total
        );
    }
}
