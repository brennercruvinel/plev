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
    let mut g = gallery("Toggle");
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
        let agent = by_agent("Toggle", "toggle", vec![ask(id, "toggle")]);
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
