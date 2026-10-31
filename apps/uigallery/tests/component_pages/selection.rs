//! Front E, selection and dates (task-104, #158): toggle, toggle group,
//! button group, radio group, calendar and date picker, the same request
//! from the person, the reader and an agent.

use accord::{SemId, Value};
use engine::control::accessibility::accesskit::Action;

use super::harness::{Gallery, ask, by_agent, gallery, same, sid};

/// The Toggle page's toggles: icon only, off and on; labeled, off and
/// on; with an icon and a label, off.
const TOGGLES: [&str; 5] = [
    "toggle_page.group[0].card[0]",
    "toggle_page.group[0].card[1]",
    "toggle_page.group[1].card[0]",
    "toggle_page.group[1].card[1]",
    "toggle_page.group[2].card[0]",
];

/// A disabled toggle of the page.
const DISABLED: &str = "toggle_page.group[0].card[2]";

/// The Toggle page scrolled, as a person scrolls it, until `id` is in
/// the window (the projection has no geometry, so the three still
/// compare node for node).
fn seeing(id: &str) -> Gallery {
    seeing_on("Toggle", id)
}

/// The gallery on `page`, scrolled until `id` is in the window.
fn seeing_on(page: &str, id: &str) -> Gallery {
    let mut g = gallery(page);
    let Some(rect) = g.c.binding(&sid(id)).map(|b| b.rect) else {
        panic!("{id} is not bound");
    };
    g.view
        .scroll_page_to(g.view.page_offset() + rect[1] - rect[3] * 4.0);
    g.frame();
    g.frame();
    g
}

fn on(g: &Gallery, id: &str) -> Option<Value> {
    let seen = g.view.semantics()?;
    seen.get(&sid(id))?.state.get("on").cloned()
}

/// A toggle flips the same from the person's click, the reader's press
/// and the agent's `toggle`, off to on and on to off.
#[test]
fn a_toggle_flips_the_same_from_the_three() {
    for id in TOGGLES {
        let mut person = seeing(id);
        let before = on(&person, id);
        person.click(id);
        assert_ne!(on(&person, id), before, "{id}: the click flips it");
        let mut reader = gallery("Toggle");
        reader.press(id);
        let agent = by_agent("Toggle", "selection_toggle", vec![ask(id, "toggle")]);
        same("Toggle", &person, &reader, &agent);
    }
}

/// A disabled toggle: the click changes nothing, and the reader is not
/// offered a press.
#[test]
fn a_disabled_toggle_moves_for_no_one() {
    let before = gallery("Toggle").state();
    let mut person = gallery("Toggle");
    person.click(DISABLED);
    assert_eq!(person.state(), before);
    assert!(!gallery("Toggle").offers(DISABLED, Action::Click));
    assert!(gallery("Toggle").offers(TOGGLES[0], Action::Click));
}

/// With the focus on a toggle, Space flips it as the click does and
/// Enter flips it back; the focus stays on it.
#[test]
fn space_and_enter_flip_the_focused_toggle_as_its_click_does() {
    let id = TOGGLES[0];
    let mut clicked = gallery("Toggle");
    clicked.click(id);
    let mut g = gallery("Toggle");
    let before = on(&g, id);
    assert!(g.view.assist_focus(&SemId::parse(id).unwrap()));
    g.frame();
    assert!(g.view.handle_key(" "), "Space");
    g.frame();
    assert_ne!(on(&g, id), before, "Space toggles");
    assert_eq!(on(&g, id), on(&clicked, id), "as the click does");
    assert!(g.view.handle_enter(), "Enter");
    g.frame();
    assert_eq!(on(&g, id), before, "Enter toggles back");
    let focused = g
        .view
        .semantics()
        .and_then(|s| s.focused().map(ToString::to_string));
    assert_eq!(focused.as_deref(), Some(id));
}

/// Toggle Group segments: one in the regular group, one in the small.
const SEGMENTS: [&str; 2] = [
    "toggle_group_page.group[0].card[0][2]",
    "toggle_group_page.group[1].card[0][0]",
];

/// A segment is chosen the same from the person's click, the reader's
/// press and the agent's `select`.
#[test]
fn a_segment_is_chosen_the_same_from_the_three() {
    for id in SEGMENTS {
        let mut person = gallery("Toggle Group");
        person.click(id);
        let mut reader = gallery("Toggle Group");
        reader.press(id);
        let agent = by_agent("Toggle Group", "selection_segment", vec![ask(id, "select")]);
        same("Toggle Group", &person, &reader, &agent);
        let checked = person.view.semantics().and_then(|s| {
            s.get(&sid(id))
                .and_then(|n| n.state.get("checked").cloned())
        });
        assert_eq!(checked, Some(Value::Bool(true)), "{id}");
    }
}

/// A segment of the disabled group: the click changes nothing and the
/// reader is not offered a press.
#[test]
fn a_disabled_toggle_group_moves_for_no_one() {
    let id = "toggle_group_page.group[0].card[1][0]";
    let before = gallery("Toggle Group").state();
    let mut person = gallery("Toggle Group");
    person.click(id);
    assert_eq!(person.state(), before);
    assert!(!gallery("Toggle Group").offers(id, Action::Click));
}

/// A tool of the button group's picker: the person's click, the reader's
/// press and the agent's `toggle` turn it on and the first tool off.
#[test]
fn a_tool_is_picked_the_same_from_the_three() {
    let id = "button_group_page.group[0].card[0].item[2]";
    let first = "button_group_page.group[0].card[0].item[0]";
    let mut person = gallery("Button Group");
    person.click(id);
    let mut reader = gallery("Button Group");
    reader.press(id);
    let agent = by_agent("Button Group", "selection_tool", vec![ask(id, "toggle")]);
    same("Button Group", &person, &reader, &agent);
    assert_eq!(on(&person, id), Some(Value::Bool(true)));
    assert_eq!(on(&person, first), Some(Value::Bool(false)));
}

/// An action of the group presses from the three and changes nothing.
#[test]
fn a_group_action_presses_and_changes_nothing() {
    let id = "button_group_page.group[0].card[0].item[5]";
    let before = gallery("Button Group").state();
    let mut person = gallery("Button Group");
    person.click(id);
    let mut reader = gallery("Button Group");
    reader.press(id);
    let agent = by_agent(
        "Button Group",
        "selection_action",
        vec![ask(id, "activate")],
    );
    assert_eq!(person.state(), before);
    assert_eq!(reader.state(), before);
    assert_eq!(agent.state(), before);
}

/// A radio row is chosen the same from the person's click, the reader's
/// press and the agent's `select`.
#[test]
fn a_radio_is_chosen_the_same_from_the_three() {
    for id in [
        "radio_group_page.group[0].card[0][2]",
        "radio_group_page.group[1].card[0][0]",
    ] {
        let mut person = seeing_on("Radio Group", id);
        person.click(id);
        let mut reader = gallery("Radio Group");
        reader.press(id);
        let agent = by_agent("Radio Group", "selection_radio", vec![ask(id, "select")]);
        same("Radio Group", &person, &reader, &agent);
    }
}
