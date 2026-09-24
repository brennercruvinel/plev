//! The Surface page: the todo's IR fixture materialized beside the fixed
//! todo, both over the todo's one dispatcher. A person adds a todo and
//! changes the filter on the generated surface and the fixed todo shows
//! it; the fixed todo moves the surface; the page projects both as one
//! snapshot; and an agent operates the surface through the real port.

use comps::prelude::{Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;
use sem::{Actor, SemId, StateRef, Value};

use super::SurfacePage;
use crate::view::apps::app::TodoPage;

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn wide() -> Rect {
    Rect::new(288.0, 118.0, 900.0, 682.0)
}

fn narrow() -> Rect {
    Rect::new(16.0, 118.0, 358.0, 682.0)
}

fn setup() -> (TodoPage, SurfacePage, Theme) {
    let todo = TodoPage::new();
    let page = SurfacePage::new(&todo);
    (todo, page, Theme::hoff_core())
}

fn frame(page: &mut SurfacePage, todo: &mut TodoPage, content: Rect, theme: &Theme) -> Compositor {
    let mut c = Compositor::new();
    page.render(&mut c, content, theme, todo, false);
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
    page: &mut SurfacePage,
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

fn value(page: &SurfacePage, todo: &TodoPage, at: &str) -> Option<Value> {
    let snap = page.snapshot(todo).expect("the page projects");
    snap.value(&StateRef::parse(at).unwrap()).cloned()
}

fn text(s: &str) -> Option<Value> {
    Some(Value::Text(s.to_owned()))
}

#[test]
fn the_fixture_verifies_against_the_todo_and_opens() {
    let (_, page, _) = setup();
    assert_eq!(page.refused, None);
    assert_eq!(page.id, Some(sid("todo_panel")));
}

/// The acceptance: a person writes and adds a todo on the generated
/// surface. Each key is the field's `set_value`, the add is the todo's
/// `submit`, by the person, through the todo's dispatcher; the fixed todo
/// shows the new row, and the surface's counter follows.
#[test]
fn adding_on_the_surface_adds_to_the_fixed_todo_through_its_dispatcher() {
    let (mut todo, mut page, theme) = setup();
    let content = wide();
    let c = frame(&mut page, &mut todo, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "todo_panel.input"),
    );
    for ch in "buy milk".chars() {
        assert!(page.handle_text(&ch.to_string(), &mut todo));
    }
    // the fixed todo's field is the same state
    assert_eq!(value(&page, &todo, "todo.input:text"), text("buy milk"));

    let c = frame(&mut page, &mut todo, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "todo_panel.add"),
    );
    let surface = page.host.get(&sid("todo_panel")).expect("open");
    let receipt = surface.last_receipt().expect("the add asked").clone();
    assert_eq!(receipt.request.target, sid("todo.input"));
    assert_eq!(receipt.request.action.as_str(), "submit");
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(
        receipt.stages.last().map(|(s, _)| s.name()),
        Some("completed")
    );
    // the receipt came from the todo's own dispatcher
    assert_eq!(todo.receipt(receipt.id), Some(&receipt));

    let snap = page.snapshot(&todo).expect("the page projects");
    let item = snap
        .nodes()
        .values()
        .find(|n| n.label == "buy milk")
        .expect("the new todo");
    assert_eq!(item.id, sid("todo.item[4]"));
    assert_eq!(value(&page, &todo, "todo.input:text"), text(""));
    assert_eq!(
        value(&page, &todo, "todo.counter:active"),
        Some(Value::Int(3))
    );
    let c = frame(&mut page, &mut todo, content, &theme);
    assert!(
        c.binding(&sid("todo.item[4]")).is_some(),
        "the fixed todo draws the new row"
    );
    assert_eq!(
        value(&page, &todo, "todo_panel.active:value"),
        Some(Value::Int(3))
    );
}

#[test]
fn the_filter_on_the_surface_filters_the_fixed_todo() {
    let (mut todo, mut page, theme) = setup();
    let content = wide();
    let c = frame(&mut page, &mut todo, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "todo_panel.filter.completed"),
    );
    assert_eq!(
        value(&page, &todo, "todo.filter:selected"),
        text("completed")
    );
    let c = frame(&mut page, &mut todo, content, &theme);
    assert!(
        c.binding(&sid("todo.item[3]")).is_some(),
        "the done todo shows"
    );
    assert!(
        c.binding(&sid("todo.item[1]")).is_none(),
        "an active one does not"
    );
}

/// The other way: typing in the fixed todo shows on the surface's field
/// at the next frame.
#[test]
fn the_fixed_todo_moves_the_surface() {
    let (mut todo, mut page, theme) = setup();
    let content = wide();
    let c = frame(&mut page, &mut todo, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "todo.input"),
    );
    assert!(page.handle_text("x", &mut todo));
    frame(&mut page, &mut todo, content, &theme);
    assert_eq!(value(&page, &todo, "todo_panel.input:text"), text("x"));
}

#[test]
fn the_page_projects_the_todo_and_the_surface_as_one_snapshot() {
    let (todo, page, _) = setup();
    let snap = page.snapshot(&todo).expect("the page projects");
    for id in [
        "todo",
        "todo.input",
        "todo.item[1]",
        "todo_panel",
        "todo_panel.card",
        "todo_panel.input",
        "todo_panel.add",
        "todo_panel.filter.all",
        "todo_panel.active",
    ] {
        assert!(snap.get(&sid(id)).is_some(), "{id}");
    }
    let add = snap.get(&sid("todo_panel.add")).unwrap();
    assert_eq!(add.role, sem::Role::Button);
    // blank field: the add is the todo's submit, disabled as it is
    assert!(!add.enabled);
    assert_eq!(
        value(&page, &todo, "todo_panel.done:value"),
        Some(Value::Int(1))
    );
}

/// Wide, the halves sit side by side; narrow, the surface is under the
/// todo, and the page is as tall as both.
#[test]
fn the_halves_sit_side_by_side_when_wide_and_stack_when_narrow() {
    for (content, beside) in [(wide(), true), (narrow(), false)] {
        let (mut todo, mut page, theme) = setup();
        let c = frame(&mut page, &mut todo, content, &theme);
        let fixed = c.binding(&sid("todo")).expect("the todo").rect;
        let generated = c
            .binding(&sid("todo_panel.card"))
            .expect("the surface")
            .rect;
        if beside {
            assert!(
                generated[0] >= fixed[0] + fixed[2],
                "{fixed:?} {generated:?}"
            );
        } else {
            assert!(
                generated[1] >= fixed[1] + fixed[3],
                "{fixed:?} {generated:?}"
            );
        }
        for r in [fixed, generated] {
            assert!(r[0] >= content.x - 0.5 && r[0] + r[2] <= content.x + content.w + 0.5);
        }
        let height = page.content_height(content, &theme, &todo);
        assert!(content.y + height >= generated[1] + generated[3] - 0.5);
    }
}

/// An agent through the real port, no window: it observes the page and
/// operates the surface by the same contract as the todo.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod agent;
