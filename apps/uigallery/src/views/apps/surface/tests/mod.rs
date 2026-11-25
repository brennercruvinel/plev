//! The Surface page: the todo's IR fixture materialized beside the fixed
//! todo, both over the todo's one dispatcher. A person adds a todo and
//! changes the filter on the generated surface and the fixed todo shows
//! it; the fixed todo moves the surface; the page projects both as one
//! snapshot; the control switches the surface between its two forms and
//! what was typed and the filter go on; and an agent operates the surface
//! through the real port.

use std::collections::BTreeSet;

use accord::{Actor, SemId, StateRef, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Rect, WidgetEvent};
use plevir::{Capabilities, GenerationPolicy, verify_json};

use super::{FORMS, SurfacePage};
use crate::views::EditKey;
use crate::views::apps::todo::TodoPage;

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

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
    assert_eq!(add.role, accord::Role::Button);
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

/// Both forms verify against what the todo projects, and they are two
/// forms of one surface: the same id, the same controls.
#[test]
fn both_forms_verify_against_the_todo_as_one_surface() {
    let todo = TodoPage::new();
    let app = todo.snapshot().expect("the todo projects");
    let caps = Capabilities::from_snapshot(&app);
    let forms: Vec<_> = FORMS
        .iter()
        .map(|f| {
            verify_json(f.ir, &caps, &GenerationPolicy::default())
                .unwrap_or_else(|e| panic!("{}: {e:#?}", f.label))
        })
        .collect();
    let controls =
        |i: usize| -> BTreeSet<SemId> { forms[i].focus_order().iter().cloned().collect() };
    assert_eq!(forms[0].ir().surface.id, forms[1].ir().surface.id);
    assert_eq!(controls(0), controls(1));
    assert!(controls(0).contains(&sid("todo_panel.input")));
}

/// The segment `i` of the form control, where the person clicks it.
fn segment(
    page: &SurfacePage,
    todo: &TodoPage,
    content: Rect,
    theme: &Theme,
    i: usize,
) -> (f32, f32) {
    let h = page.halves(content, theme, todo);
    page.forms.item_rects(h.forms, theme)[i].center()
}

/// Frames a turn apart for as long as the page asks for one (a caret
/// flip draws one too): the last one, and how many.
fn settle(
    page: &mut SurfacePage,
    todo: &mut TodoPage,
    content: Rect,
    theme: &Theme,
) -> (Compositor, usize) {
    let mut c = frame(page, todo, content, theme);
    let mut frames = 0;
    loop {
        let wake = page.wake(TURN, todo);
        if wake.now {
            c = frame(page, todo, content, theme);
            frames += 1;
        }
        if !wake.is_continuous() {
            return (c, frames);
        }
        assert!(frames < 600, "the page never settles");
    }
}

/// The acceptance of phase 2 on the page: a person types in the
/// surface's field and filters on it, then switches the surface to two
/// columns with the control. The surface transforms in place: the new
/// form is what the page projects, the typed text and the filter go on
/// (the fixed todo still filtered), the field is the same widget (its
/// caret where it was), and the nodes slide to their places. And back.
#[test]
fn the_control_switches_the_form_and_the_typed_text_and_the_filter_go_on() {
    let (mut todo, mut page, theme) = setup();
    let content = wide();
    let field = sid("todo_panel.input");
    let c = frame(&mut page, &mut todo, content, &theme);
    let compact = c.binding(&field).expect("the field").rect;
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "todo_panel.input"),
    );
    for ch in "buy".chars() {
        assert!(page.handle_text(&ch.to_string(), &mut todo));
    }
    assert!(page.handle_edit_key(EditKey::Left, &mut todo));
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

    let to = segment(&page, &todo, content, &theme, 1);
    click(&mut page, &mut todo, content, &theme, to);
    assert_eq!(page.form, 1);
    let snap = page.snapshot(&todo).expect("the page projects");
    assert!(snap.get(&sid("todo_panel.side")).is_some());
    assert!(snap.get(&sid("todo_panel.counts")).is_none());
    // what was typed and the filter are the todo's: they go on
    assert_eq!(value(&page, &todo, "todo_panel.input:text"), text("buy"));
    assert_eq!(
        value(&page, &todo, "todo.filter:selected"),
        text("completed")
    );
    // the field is the same widget: the click on the control blurred it,
    // and focused again its caret is where it was, one place back
    assert!(page.focus(&field, &mut todo, false));
    assert!(page.handle_text("X", &mut todo));
    assert_eq!(value(&page, &todo, "todo.input:text"), text("buXy"));

    let (c, frames) = settle(&mut page, &mut todo, content, &theme);
    assert!(frames > 1, "the change of form took {frames} frames");
    let columns = c.binding(&field).expect("the field").rect;
    assert!(columns[0] > compact[0], "{compact:?} then {columns:?}");
    assert!(c.binding(&sid("todo.item[3]")).is_some(), "still filtered");
    assert!(c.binding(&sid("todo.item[1]")).is_none());

    let back = segment(&page, &todo, content, &theme, 0);
    click(&mut page, &mut todo, content, &theme, back);
    assert_eq!(page.form, 0);
    let (c, _) = settle(&mut page, &mut todo, content, &theme);
    assert_eq!(c.binding(&field).expect("the field").rect, compact);
    assert_eq!(value(&page, &todo, "todo_panel.input:text"), text("buXy"));
}

/// The proposer sits beside the intent field where the field keeps a
/// card's minimum width, and under it where it does not (the page's
/// halves at 1200 wide, a phone); the cancel ends the message line
/// either way, and nothing leaves the width.
#[test]
fn the_proposer_sits_beside_the_field_when_wide_and_under_it_when_not() {
    let (mut todo, mut page, theme) = setup();
    for (content, beside) in [
        (Rect::new(40.0, 118.0, 1500.0, 682.0), true),
        (wide(), false),
        (narrow(), false),
    ] {
        let c = frame(&mut page, &mut todo, content, &theme);
        let rect = |id: &str| c.binding(&sid(id)).unwrap_or_else(|| panic!("{id}")).rect;
        let field = rect("surface_page.intent");
        let proposer = rect("surface_page.proposer");
        let propose = rect("surface_page.propose");
        let cancel = rect("surface_page.cancel");
        let status = rect("surface_page.status");
        if beside {
            assert_eq!(proposer[1], field[1], "{content:?}: one line");
            assert!(proposer[0] >= field[0] + field[2], "{content:?}: beside");
            assert!(
                field[2] >= theme.size.card_min_w,
                "{content:?}: a wide field"
            );
        } else {
            assert!(proposer[1] >= field[1] + field[3], "{content:?}: under");
            assert_eq!(proposer[0], field[0], "{content:?}: at the left");
        }
        assert!(
            propose[0] >= proposer[0] + proposer[2],
            "{content:?}: the button after"
        );
        // cancel ends the button's line when it has the room, else it
        // goes under it
        let after = cancel[0] >= propose[0] + propose[2] && cancel[1] < propose[1] + propose[3];
        assert!(
            after || cancel[1] >= propose[1] + propose[3] - 0.5,
            "{content:?}: cancel after the button or below it"
        );
        assert!(
            cancel[0] + cancel[2] <= content.x + content.w + 0.5,
            "{content:?}"
        );
        assert!(
            status[1] >= propose[1],
            "{content:?}: the message below the button"
        );
        for (id, r) in [
            ("proposer", proposer),
            ("propose", propose),
            ("cancel", cancel),
        ] {
            assert!(
                r[0] >= field[0] - 0.5 && r[0] + r[2] <= field[0] + content.w + 0.5,
                "{content:?}: {id} inside"
            );
        }
    }
}

/// An agent through the real port, no window: it observes the page and
/// operates the surface by the same contract as the todo.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod agent;

/// The page in both forms, as the gallery draws it without a window: its
/// scene, its projection, and with a GPU its pixels.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod forms;

/// What an intent costs until the surface is verified and until its
/// first frame (ignored: run by hand).
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod measure;

/// The todo's items on the surface (task-95): both forms show them by the
/// live list, a check on either half reaches the other through the todo's
/// dispatcher, and a change of form keeps each item's instance.
mod items;

/// The Claude asked from the page, against the fake claude: the request
/// starts and the page goes on, the answer arrives through the wake, a
/// cancel kills the child, the rules alone without a program, and the
/// real port; the real claude in one ignored test.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod claude;
/// The intent field and the propose button: a person proposes, the
/// surface takes the form in place; a refusal says why; the page's
/// `Operable` leaves the todo as the todo's own path does.
mod offers;

/// Two columns where the width allows, one where it does not: at a
/// phone's width the second form wraps and nothing of the surface
/// leaves the page's width.
#[test]
fn the_two_column_form_fits_a_phone_width() {
    let (mut todo, mut page, theme) = setup();
    let content = narrow();
    frame(&mut page, &mut todo, content, &theme);
    assert!(page.switch(1, &todo));
    let (c, _) = settle(&mut page, &mut todo, content, &theme);
    let side = c.binding(&sid("todo_panel.side")).expect("drawn").rect;
    let entry = c.binding(&sid("todo_panel.entry")).expect("drawn").rect;
    assert!(
        entry[1] >= side[1] + side[3],
        "one column: {side:?} {entry:?}"
    );
    let bound: Vec<_> = c
        .bindings()
        .filter(|(id, _)| id.as_str().starts_with("todo_panel"))
        .map(|(id, b)| (id.clone(), b.rect))
        .collect();
    assert!(bound.len() > 10);
    for (id, [x, _, w, _]) in bound {
        assert!(
            x >= content.x - 0.5 && x + w <= content.x + content.w + 0.5,
            "{id} leaves the page: {x} + {w}"
        );
    }
}

/// A form the surface cannot take leaves it as it was: the form it
/// shows, the control on it, the nodes it projects.
#[test]
fn a_form_that_does_not_transform_leaves_the_surface_as_it_was() {
    let (todo, mut page, _) = setup();
    assert!(!page.switch(FORMS.len(), &todo));
    assert_eq!((page.form, page.forms.active), (0, 0));
    let snap = page.snapshot(&todo).expect("the page projects");
    assert!(snap.get(&sid("todo_panel.counts")).is_some());
    assert!(snap.get(&sid("todo_panel.side")).is_none());
}
