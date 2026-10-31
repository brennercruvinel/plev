//! Front D, disclosure and display (task-104, #157): accordion,
//! collapsible, item, kbd, aspect ratio, carousel, pagination and table,
//! the same request from the person, the reader and an agent.

use accord::Value;
use engine::control::accessibility::focus;
use uigallery::views::EditKey;

use super::harness::{Gallery, ask, by_agent, gallery, same, sid};

/// The second section of the single-mode card, closed at first.
const MATERIALS: &str = "accordion_page.group[0].card[0].item[1]";
/// The first section of that card, open at first.
const OBJECTS: &str = "accordion_page.group[0].card[0].item[0]";
/// A disabled section, closed.
const DISABLED: &str = "accordion_page.group[1].card[0].item[1]";

fn expanded(g: &Gallery, id: &str) -> Option<Value> {
    g.value(&format!("{id}:expanded"))
}

fn focused(g: &Gallery) -> Option<String> {
    g.view
        .semantics()
        .and_then(|s| s.focused().map(ToString::to_string))
}

/// A section opens the same from the three: the person's click on its
/// trigger, the reader's press, the agent's `toggle_expanded`. In the
/// single-mode card the open section closes, and the region of the one
/// that opened is in the tree.
#[test]
fn an_accordion_section_opens_the_same_from_the_three() {
    let mut person = gallery("Accordion");
    person.click(MATERIALS);
    let mut reader = gallery("Accordion");
    reader.press(MATERIALS);
    let agent = by_agent("Accordion", "open", vec![ask(MATERIALS, "toggle_expanded")]);
    same("Accordion", &person, &reader, &agent);
    assert_eq!(expanded(&person, MATERIALS), Some(Value::Bool(true)));
    assert_eq!(expanded(&person, OBJECTS), Some(Value::Bool(false)));
    let panel = sid(&format!("{MATERIALS}.panel"));
    assert!(
        person.state().iter().any(|n| n.id == panel),
        "its region shows"
    );
}

/// A click on a disabled section changes nothing.
#[test]
fn a_disabled_section_stays_closed_for_a_click() {
    let mut person = gallery("Accordion");
    let before = person.state();
    person.click(DISABLED);
    assert_eq!(person.state(), before);
    assert_eq!(expanded(&person, DISABLED), Some(Value::Bool(false)));
}

/// Tab walks the enabled triggers in the projection's order and skips
/// the disabled one; Enter and Space flip the focused section; Escape
/// lets the focus go.
#[test]
fn tab_reaches_the_triggers_enter_and_space_flip_and_escape_blurs() {
    let mut g = gallery("Accordion");
    let order: Vec<String> = focus::order(&g.view.semantics().unwrap_or_default())
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(order.first().map(String::as_str), Some(OBJECTS));
    assert!(!order.iter().any(|id| id == DISABLED), "{order:?}");
    while focused(&g).as_deref() != Some(MATERIALS) {
        assert!(g.view.handle_edit_key(EditKey::Tab), "Tab is taken");
        g.frame();
        assert!(focused(&g).is_some_and(|f| order.contains(&f)));
    }
    assert!(g.view.handle_enter());
    g.frame();
    assert_eq!(expanded(&g, MATERIALS), Some(Value::Bool(true)));
    assert!(g.view.handle_key(" "));
    g.frame();
    assert_eq!(expanded(&g, MATERIALS), Some(Value::Bool(false)));
    assert!(g.view.close_top_overlay(), "Escape blurs");
    g.frame();
    assert_eq!(focused(&g), None);
}
