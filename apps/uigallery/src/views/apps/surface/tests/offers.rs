//! The intent field and the propose button of the Surface page (task-69
//! phase 3a): a person writes an intent and proposes, and the surface
//! takes the form the rules proposed in place, what was typed in its
//! field going on in the same widget; an intent the rules do not
//! understand leaves the surface as it was and the page says why; and the
//! page's `Operable`, lent the todo's dispatcher, leaves the todo's
//! projection and receipts exactly as the todo's own path does.

use accord::{Actor, Receipt, Request, RequestId, SemId, Stage, StateRef, Value};

use super::super::SurfacePage;
use super::super::offering::Said;
use super::{center, click, frame, narrow, setup, sid, text, value, wide};
use crate::views::EditKey;
use crate::views::apps::todo::TodoPage;

const INTENT: &str = "only what is active and the add field";

/// Types `s` where the keyboard is.
fn type_in(page: &mut SurfacePage, todo: &mut TodoPage, s: &str) {
    for ch in s.chars() {
        assert!(page.handle_text(&ch.to_string(), todo), "{ch} typed");
    }
}

fn has(page: &SurfacePage, todo: &TodoPage, id: &str) -> bool {
    let snap = page.snapshot(todo).expect("the page projects");
    snap.get(&sid(id)).is_some()
}

fn last_stage(r: &Receipt) -> &Stage {
    &r.stages.last().expect("a stage").0
}

/// The acceptance on the page, by a person: text typed in the surface's
/// field, the caret moved back; the intent written and proposed with the
/// button; the surface keeps its id and takes the proposed form (only the
/// active filter and count, the entry), the typed text goes on, and the
/// field is the same widget (its caret where it was). The proposal is a
/// request of the person through the todo's dispatcher.
#[test]
fn a_person_proposes_and_the_surface_takes_the_form_in_place() {
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
    type_in(&mut page, &mut todo, "buy");
    assert!(page.handle_edit_key(EditKey::Left, &mut todo));

    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "surface_page.intent"),
    );
    type_in(&mut page, &mut todo, INTENT);
    assert_eq!(
        value(&page, &todo, "surface_page.intent:text"),
        text(INTENT)
    );
    assert_eq!(value(&page, &todo, "todo.input:text"), text("buy"));
    let c = frame(&mut page, &mut todo, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "surface_page.propose"),
    );

    assert_eq!(page.proposing.said, Some(Said::Proposed(INTENT.to_owned())));
    assert!(page.proposing.shown);
    assert_eq!(page.id, Some(sid("todo_panel")));
    for (id, kept) in [
        ("todo_panel.input", true),
        ("todo_panel.add", true),
        ("todo_panel.filter.active", true),
        ("todo_panel.active", true),
        ("todo_panel.filter.all", false),
        ("todo_panel.filter.completed", false),
        ("todo_panel.done", false),
    ] {
        assert_eq!(has(&page, &todo, id), kept, "{id}");
    }
    assert_eq!(value(&page, &todo, "todo_panel.input:text"), text("buy"));

    // the receipt: the person's propose, in the todo's dispatcher
    let receipt = todo.receipt(RequestId::parse("4").unwrap()).expect("kept");
    assert_eq!(receipt.request.target, sid("surface_page.propose"));
    assert_eq!(receipt.request.action.as_str(), "propose");
    assert_eq!(receipt.request.actor, Actor::Human);
    let Stage::Completed(outcome) = last_stage(receipt) else {
        panic!("completed: {:?}", receipt.stages);
    };
    assert_eq!(
        outcome.result.get("surface"),
        Some(&Value::Text("todo_panel".to_owned()))
    );

    // the field is the widget it was: focused again, its caret is one
    // place back, where the person left it
    assert!(page.focus(&sid("todo_panel.input"), &mut todo, false));
    assert!(page.handle_text("X", &mut todo));
    assert_eq!(value(&page, &todo, "todo.input:text"), text("buXy"));

    // and the page draws the new form
    let c = frame(&mut page, &mut todo, content, &theme);
    assert!(c.binding(&sid("todo_panel.filter.all")).is_none());
    assert!(c.binding(&sid("todo_panel.filter.active")).is_some());
}

/// Enter in the intent field proposes too; an intent the rules do not
/// understand is refused with the reason: the surface keeps its form, the
/// status says why (the words that are not the app's, and the app's),
/// and the receipt failed with the same text.
#[test]
fn an_intent_the_rules_do_not_understand_leaves_the_surface_and_says_why() {
    let (mut todo, mut page, theme) = setup();
    let content = wide();
    let c = frame(&mut page, &mut todo, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "surface_page.intent"),
    );
    type_in(&mut page, &mut todo, "só o que está ativo");
    let before = page.snapshot(&todo).expect("projects");
    assert!(page.handle_enter(&mut todo, false));

    let Some(Said::Refused(why)) = page.proposing.said.clone() else {
        panic!("refused: {:?}", page.proposing.said);
    };
    assert!(
        why.starts_with("\"ativo\" is not a word of the app; its words are active"),
        "{why}"
    );
    assert!(!page.proposing.shown);
    let after = page.snapshot(&todo).expect("projects");
    for node in before
        .nodes()
        .keys()
        .filter(|id| id.as_str().starts_with("todo_panel"))
    {
        assert!(after.get(node).is_some(), "{node} stays");
    }
    let status = StateRef::parse("surface_page.status:message").unwrap();
    assert_eq!(after.value(&status), Some(&Value::Text(why.clone())));
    let receipt = todo.receipt(RequestId::parse("1").unwrap()).expect("kept");
    assert_eq!(last_stage(receipt), &Stage::Failed { error: why.clone() });
    // the message is drawn, bound to the status
    let c = frame(&mut page, &mut todo, content, &theme);
    let bound = c.binding(&sid("surface_page.status")).expect("bound").rect;
    assert!(bound[3] > 0.0);

    // a blank intent cannot be proposed: the button is disabled, and the
    // capability refuses
    let snap = page.snapshot(&todo).expect("projects");
    assert!(
        snap.get(&sid("surface_page.propose"))
            .is_some_and(|n| n.enabled)
    );
    page.proposing.intent.set_text("  ");
    let snap = page.snapshot(&todo).expect("projects");
    assert!(
        snap.get(&sid("surface_page.propose"))
            .is_some_and(|n| !n.enabled)
    );
}

/// The one method outside the page's files (`TodoPage::with_dispatcher`)
/// changes nothing of the todo: the same requests, through the todo's
/// own path on one todo and through the page's `Operable` on another,
/// get the same ids, the same stages and outcomes, and leave the same
/// projection.
#[test]
fn the_page_operable_forwards_to_the_todo_as_the_todo_runs_it() {
    let (mut alone, _, _) = setup();
    let (mut todo, mut page, _) = setup();
    let asks: [(&str, &str, Option<Value>); 6] = [
        ("todo.input", "set_value", Some(Value::Text("milk".into()))),
        ("todo.input", "submit", None),
        ("todo.filter[active]", "select", None),
        ("todo.item[1]", "toggle", None),
        ("todo.item[2]", "remove", None),
        ("todo.item[9]", "toggle", None),
    ];
    // the request as asked, but for the revision it was observed at (the
    // page's snapshot holds more than the todo's), and every stage
    let shape = |r: &Receipt| -> (Request, Vec<Stage>) {
        let mut req = r.request.clone();
        req.observed.revision = accord::Snapshot::default().revision();
        (req, r.stages.iter().map(|(s, _)| s.clone()).collect())
    };
    for (target, action, param) in asks {
        let seen = alone.snapshot().unwrap();
        let by = Actor::Test {
            name: "forward".into(),
        };
        let own = Request::observe(&seen, by.clone(), &sid(target), action, param.clone());
        let seen = page.snapshot(&todo).unwrap();
        let lent = Request::observe(&seen, by, &sid(target), action, param);
        let a = own.map(|req| alone.dispatch(req));
        let b = lent.map(|req| page.page(&mut todo).dispatch(req));
        let (Ok(a), Ok(b)) = (a, b) else {
            // an item the todo does not project cannot be observed either way
            assert_eq!(target, "todo.item[9]");
            continue;
        };
        assert_eq!(a, b, "{target} {action}: the same numbering");
        let (a, b) = (alone.receipt(a).unwrap(), todo.receipt(b).unwrap());
        assert_eq!(shape(a), shape(b), "{target} {action}");
    }
    assert_eq!(alone.snapshot().unwrap(), todo.snapshot().unwrap());
}

/// Narrow, the proposing row sits under the todo, above the surface, and
/// nothing of it leaves the page's width.
#[test]
fn the_proposing_row_fits_a_phone_width() {
    let (mut todo, mut page, theme) = setup();
    let content = narrow();
    let c = frame(&mut page, &mut todo, content, &theme);
    let todo_rect = c.binding(&sid("todo")).expect("the todo").rect;
    let card = c
        .binding(&sid("todo_panel.card"))
        .expect("the surface")
        .rect;
    for id in ["surface_page.intent", "surface_page.propose"] {
        let [x, y, w, h] = c.binding(&sid(id)).expect(id).rect;
        assert!(y >= todo_rect[1] + todo_rect[3], "{id} under the todo");
        assert!(y + h <= card[1], "{id} above the surface");
        assert!(
            x >= content.x - 0.5 && x + w <= content.x + content.w + 0.5,
            "{id}"
        );
    }
}

/// The proposing row's id stays clear of the ids a surface or the todo
/// use.
#[test]
fn the_proposing_nodes_are_the_pages_own() {
    let (todo, page, _) = setup();
    let snap = page.snapshot(&todo).expect("projects");
    let region = snap.get(&sid("surface_page")).expect("the region");
    assert_eq!(region.role, accord::Role::Region);
    let intent = snap.get(&sid("surface_page.intent")).expect("the intent");
    assert_eq!(intent.role, accord::Role::TextInput);
    let actions: Vec<&str> = snap
        .get(&sid("surface_page.propose"))
        .expect("the button")
        .actions
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    assert_eq!(actions, ["propose"]);
    assert!(
        page.proposing
            .holds(&SemId::parse("surface_page.status").unwrap())
    );
    assert!(!page.proposing.holds(&sid("todo_panel")));
}
