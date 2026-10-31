//! Front B, overlays (task-104, #155): popover, hover card, dropdown menu,
//! alert dialog, menubar, sheet and drawer, the same request from the
//! person, the reader and an agent.

use accord::{Actor, Request, Value};
use uigallery::views::EditKey;

use super::harness::{Gallery, ask, by_agent, gallery, same, sid};

const PAGE: &str = "Popover";
/// The live card's trigger, its panel, and its "Zoom to 200%" preset.
const LIVE: &str = "popover_page.group[0].card[0]";
const PANEL: &str = "popover_page.group[0].card[0].panel";
const TO_200: &str = "popover_page.group[0].card[0].panel[3]";

fn expanded(g: &Gallery) -> Option<Value> {
    g.value(&format!("{LIVE}:expanded"))
}

fn label(g: &Gallery, id: &str) -> Option<String> {
    g.state()
        .into_iter()
        .find(|n| n.id == sid(id))
        .map(|n| n.label)
}

fn focused(g: &Gallery) -> Option<String> {
    g.view
        .semantics()
        .and_then(|s| s.focused().map(ToString::to_string))
}

/// The popover opens from the three: a click, a reader's press, an
/// agent's `activate`; its panel is projected, the trigger expanded.
#[test]
fn the_popover_opens_the_same_from_the_person_the_reader_and_an_agent() {
    let mut person = gallery(PAGE);
    assert_eq!(
        expanded(&person),
        Some(Value::Bool(false)),
        "it loads closed"
    );
    person.click(LIVE);
    let mut reader = gallery(PAGE);
    reader.press(LIVE);
    let agent = by_agent(PAGE, "popover-open", vec![ask(LIVE, "activate")]);
    same(PAGE, &person, &reader, &agent);
    assert_eq!(expanded(&agent), Some(Value::Bool(true)));
    assert_eq!(label(&agent, PANEL).as_deref(), Some("Zoom"));
}

/// A preset picked in the panel, from the three: the zoom the value
/// pill shows changes and the popover closes.
#[test]
fn a_preset_picked_from_the_three_sets_the_zoom_and_closes_the_panel() {
    let mut person = gallery(PAGE);
    person.click(LIVE);
    person.click(TO_200);
    let mut reader = gallery(PAGE);
    reader.press(LIVE);
    reader.press(TO_200);
    let asks = vec![ask(LIVE, "activate"), ask(TO_200, "activate")];
    let agent = by_agent(PAGE, "popover-pick", asks);
    same(PAGE, &person, &reader, &agent);
    assert_eq!(label(&agent, LIVE).as_deref(), Some("200%"));
    assert_eq!(expanded(&agent), Some(Value::Bool(false)));
    assert_eq!(label(&agent, PANEL), None, "the panel is gone");
}

/// Closed again from the three: Escape, a second press of the trigger,
/// the panel's `close`; the page is as it loaded.
#[test]
fn the_popover_closes_the_same_from_the_three() {
    let before = gallery(PAGE).state();
    let mut person = gallery(PAGE);
    person.click(LIVE);
    assert!(person.view.close_top_overlay(), "Escape closes it");
    person.frame();
    let mut reader = gallery(PAGE);
    reader.press(LIVE);
    reader.press(LIVE);
    let asks = vec![ask(LIVE, "activate"), ask(PANEL, "close")];
    let agent = by_agent(PAGE, "popover-close", asks);
    for (who, g) in [("person", &person), ("reader", &reader), ("agent", &agent)] {
        assert_eq!(g.state(), before, "{who}");
    }
}

/// The keyboard: the trigger focused, Space opens and the focus stays on
/// it (headlessui's popover); Tab walks into the panel; Escape closes it
/// and gives the focus back to the trigger; a second Escape lets it go.
#[test]
fn space_opens_tab_enters_and_escape_gives_the_focus_back() {
    let mut g = gallery(PAGE);
    assert!(
        g.view.assist_focus(&sid(LIVE)),
        "a reader moves the focus there"
    );
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(LIVE));
    assert!(g.view.handle_key(" "));
    g.frame();
    assert_eq!(expanded(&g), Some(Value::Bool(true)));
    assert_eq!(
        focused(&g).as_deref(),
        Some(LIVE),
        "opening keeps the focus"
    );
    assert!(g.view.handle_edit_key(EditKey::Tab));
    g.frame();
    let first = format!("{PANEL}[0]");
    assert_eq!(
        focused(&g).as_deref(),
        Some(first.as_str()),
        "Tab enters the panel"
    );
    assert!(g.view.close_top_overlay());
    g.frame();
    assert_eq!(expanded(&g), Some(Value::Bool(false)));
    assert_eq!(focused(&g).as_deref(), Some(LIVE), "back on the trigger");
    assert!(g.view.close_top_overlay());
    assert_eq!(focused(&g), None);
}

/// What the page refuses: a closed panel is not there to close or pick
/// from, and a forced card shows its state but does not open.
#[test]
fn a_closed_panel_and_a_forced_card_refuse_what_they_cannot_do() {
    let g = gallery(PAGE);
    let seen = g.view.semantics().expect("the page projects");
    let actor = Actor::Test {
        name: "popover".to_owned(),
    };
    for (target, action) in [(PANEL, "close"), (TO_200, "activate")] {
        let req = Request::observe(&seen, actor.clone(), &sid(target), action, None);
        assert!(req.is_err(), "{target} {action} while closed");
    }
    let forced = sid("popover_page.group[1].card[0]");
    let node = seen.get(&forced).expect("the forced trigger");
    assert_eq!(node.state.get("expanded"), Some(&Value::Bool(true)));
    let req = Request::observe(&seen, actor, &forced, "close", None);
    assert!(req.is_err(), "a forced card has nothing to close");
}
