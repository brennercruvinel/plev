//! The todo's items on the generated surface (task-95): both forms repeat
//! over `todo:items` by its live value, one instance per item named by
//! its id; checking an item on the surface is the todo's `toggle` through
//! the todo's dispatcher, by whoever acted, and the fixed todo shows it,
//! and back; a change of form keeps each item's instance.

use comps::prelude::WidgetEvent;
use sem::{Actor, Request, SemId, Stage, Value};

use super::super::{FORMS, SurfacePage};
use super::{center, click, frame, settle, setup, sid, text, value, wide};
use crate::view::apps::app::TodoPage;

/// The instances of the surface's items the page projects, in order.
fn instances(page: &SurfacePage, todo: &TodoPage) -> Vec<String> {
    let snap = page.snapshot(todo).expect("the page projects");
    snap.nodes()
        .keys()
        .map(SemId::as_str)
        .filter(|id| id.starts_with("todo_panel.item[") && id.ends_with(']'))
        .map(str::to_owned)
        .collect()
}

/// `action` on `target` by a test, through the todo's own path.
fn ask(todo: &mut TodoPage, target: &str, action: &str, param: Option<Value>) {
    let seen = todo.snapshot().expect("the todo projects");
    let by = Actor::Test {
        name: "items".into(),
    };
    let req = Request::observe(&seen, by, &sid(target), action, param).expect("observed");
    let id = todo.dispatch(req);
    let stage = todo.receipt(id).and_then(|r| r.stages.last());
    assert!(
        matches!(stage, Some((Stage::Completed(_), _))),
        "{target} {action}: {stage:?}"
    );
}

fn checked(page: &SurfacePage, todo: &TodoPage, item: u64) -> Option<Value> {
    value(page, todo, &format!("todo_panel.item[{item}].done:checked"))
}

fn completed(page: &SurfacePage, todo: &TodoPage, item: u64) -> Option<Value> {
    value(page, todo, &format!("todo.item[{item}]:completed"))
}

/// In each form the surface shows every item the todo shows, in its
/// order, by the live value of the list: its text, and whether it is
/// done; a todo added on the fixed side is an instance at the next frame,
/// and the filter takes the done one out of both halves.
#[test]
fn both_forms_show_the_items_by_the_live_list() {
    for (form, f) in FORMS.iter().enumerate() {
        let (mut todo, mut page, theme) = setup();
        let content = wide();
        if form != 0 {
            assert!(page.switch(form, &todo), "{}", f.label);
        }
        let (c, _) = settle(&mut page, &mut todo, content, &theme);
        let all = [
            "todo_panel.item[1]",
            "todo_panel.item[2]",
            "todo_panel.item[3]",
        ];
        assert_eq!(instances(&page, &todo), all, "{}", f.label);
        let snap = page.snapshot(&todo).expect("the page projects");
        for (item, said, done) in [
            (1, "Absorb the plev demos into the showcase", false),
            (2, "Measure text, never estimate it", false),
            (3, "Port the todo domain (tested first)", true),
        ] {
            let root = snap.get(&sid(&format!("todo_panel.item[{item}]")));
            assert_eq!(
                root.map(|n| n.label.as_str()),
                Some(item.to_string().as_str())
            );
            let at = format!("todo_panel.item[{item}].text:text");
            assert_eq!(value(&page, &todo, &at), text(said), "{}", f.label);
            assert_eq!(checked(&page, &todo, item), Some(Value::Bool(done)));
            let done = sid(&format!("todo_panel.item[{item}].done"));
            assert!(c.binding(&done).is_some(), "{}: {done} drawn", f.label);
        }

        ask(&mut todo, "todo.input", "set_value", text("buy milk"));
        ask(&mut todo, "todo.input", "submit", None);
        frame(&mut page, &mut todo, content, &theme);
        let at = "todo_panel.item[4].text:text";
        assert_eq!(value(&page, &todo, at), text("buy milk"), "{}", f.label);

        ask(&mut todo, "todo.filter[active]", "select", None);
        let (c, _) = settle(&mut page, &mut todo, content, &theme);
        let active = [
            "todo_panel.item[1]",
            "todo_panel.item[2]",
            "todo_panel.item[4]",
        ];
        assert_eq!(instances(&page, &todo), active, "{}", f.label);
        assert!(c.binding(&sid("todo_panel.item[3].done")).is_none());
        assert!(c.binding(&sid("todo.item[3]")).is_none(), "nor the fixed");
    }
}

/// The acceptance: a person checks an item on the surface. It is the
/// todo's `toggle`, by the person, through the todo's dispatcher, and the
/// fixed todo shows it done. The other way: the person checks another
/// item on the fixed todo, and removes it there (the surface does not
/// remove: a destructive action binds only to a dialog's confirm), and
/// the surface follows at its next frame.
#[test]
fn checking_an_item_on_the_surface_checks_it_on_the_fixed_todo_and_back() {
    let (mut todo, mut page, theme) = setup();
    let content = wide();
    let c = frame(&mut page, &mut todo, content, &theme);
    let at = center(&c, "todo_panel.item[1].done");
    click(&mut page, &mut todo, content, &theme, at);

    let surface = page.host.get(&sid("todo_panel")).expect("open");
    let receipt = surface.last_receipt().expect("the check asked").clone();
    assert_eq!(receipt.request.target, sid("todo.item[1]"));
    assert_eq!(receipt.request.action.as_str(), "toggle");
    assert_eq!(receipt.request.actor, Actor::Human);
    assert!(matches!(
        receipt.stages.last(),
        Some((Stage::Completed(_), _))
    ));
    assert_eq!(todo.receipt(receipt.id), Some(&receipt), "the todo's");
    assert_eq!(completed(&page, &todo, 1), Some(Value::Bool(true)));
    assert_eq!(
        value(&page, &todo, "todo.counter:done"),
        Some(Value::Int(2))
    );
    frame(&mut page, &mut todo, content, &theme);
    assert_eq!(checked(&page, &todo, 1), Some(Value::Bool(true)));
    assert_eq!(
        value(&page, &todo, "todo_panel.done:value"),
        Some(Value::Int(2))
    );

    // the fixed todo: its row's checkbox, then its delete button
    let c = frame(&mut page, &mut todo, content, &theme);
    let row = c.binding(&sid("todo.item[2]")).expect("the row").rect;
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "todo.item[2]"),
    );
    assert_eq!(completed(&page, &todo, 2), Some(Value::Bool(true)));
    frame(&mut page, &mut todo, content, &theme);
    assert_eq!(checked(&page, &todo, 2), Some(Value::Bool(true)));

    let delete = (row[0] + row[2] - 20.0, row[1] + row[3] / 2.0);
    click(&mut page, &mut todo, content, &theme, delete);
    assert!(completed(&page, &todo, 2).is_none(), "removed");
    let (c, _) = settle(&mut page, &mut todo, content, &theme);
    let left = ["todo_panel.item[1]", "todo_panel.item[3]"];
    assert_eq!(instances(&page, &todo), left);
    assert!(c.binding(&sid("todo_panel.item[2].done")).is_none());
}

/// An agent checks an item on the surface: the act on the surface's node
/// is the todo's `toggle` on the item, by the agent, observed on what the
/// surface shows, and kept by the todo's dispatcher.
#[test]
fn an_agent_checks_an_item_on_the_surface_as_itself() {
    let (mut todo, mut page, theme) = setup();
    frame(&mut page, &mut todo, wide(), &theme);
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let seen = page.snapshot(&todo).expect("the page projects");
    let target = sid("todo_panel.item[3].done");
    let reads: Vec<String> = seen
        .get(&target)
        .and_then(|n| n.actions.first())
        .map(|a| a.reads.iter().map(ToString::to_string).collect())
        .unwrap_or_default();
    assert_eq!(reads, ["todo_panel.item[3].done:checked"], "as it shows it");
    let req = Request::observe(&seen, agent.clone(), &target, "toggle", None).expect("observed");
    let receipt = page.act(req, &mut todo, false).expect("a receipt");
    assert_eq!(receipt.request.target, sid("todo.item[3]"));
    assert_eq!(receipt.request.actor, agent);
    assert!(matches!(
        receipt.stages.last(),
        Some((Stage::Completed(_), _))
    ));
    assert_eq!(completed(&page, &todo, 3), Some(Value::Bool(false)));
    assert_eq!(checked(&page, &todo, 3), Some(Value::Bool(false)));
}

/// A change of form keeps each item's instance: the person presses an
/// item's checkbox, the surface takes the other form, and the release
/// completes the click the press began (a new widget would hold no
/// press). The instances are the same ids before and after, and the
/// checkbox then sits elsewhere.
#[test]
fn a_change_of_form_keeps_each_items_instance() {
    let (mut todo, mut page, theme) = setup();
    let content = wide();
    let c = frame(&mut page, &mut todo, content, &theme);
    let done = sid("todo_panel.item[1].done");
    let before = c.binding(&done).expect("drawn").rect;
    let (x, y) = center(&c, "todo_panel.item[1].done");
    let ids = instances(&page, &todo);
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
    ] {
        page.handle_event(&event, content, &theme, &mut todo, false);
    }
    assert_eq!(completed(&page, &todo, 1), Some(Value::Bool(false)));

    assert!(page.switch(1, &todo), "two columns");
    assert_eq!(instances(&page, &todo), ids, "the same instances");
    page.handle_event(
        &WidgetEvent::MouseUp { x, y },
        content,
        &theme,
        &mut todo,
        false,
    );
    assert_eq!(
        completed(&page, &todo, 1),
        Some(Value::Bool(true)),
        "the press survived the change of form"
    );
    let (c, _) = settle(&mut page, &mut todo, content, &theme);
    let after = c.binding(&done).expect("drawn").rect;
    assert_ne!(before, after, "the same checkbox, elsewhere");
    assert_eq!(checked(&page, &todo, 1), Some(Value::Bool(true)));
}
