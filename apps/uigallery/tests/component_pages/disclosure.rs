//! Front D, disclosure and display (task-104, #157): accordion,
//! collapsible, item, kbd, aspect ratio, carousel, pagination and table,
//! the same request from the person, the reader and an agent.

use accord::Value;
use engine::control::accessibility::focus;
use engine::widgets::prelude::WidgetEvent;
use uigallery::views::{EditKey, NavKey};

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
    // The disabled card is below the fold: the person scrolls to it.
    let delta = 400.0;
    person.view.handle_event(&WidgetEvent::Scroll {
        x: 640.0,
        y: 400.0,
        delta,
    });
    person.frame();
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

/// The arrows walk an accordion's triggers (WAI-ARIA's accordion): down
/// and up to the next and the previous, Home and End to the ends, no
/// wrap, the disabled one skipped; the rows of another accordion are not
/// reached.
#[test]
fn the_arrows_walk_the_triggers_of_one_accordion() {
    let mut g = gallery("Accordion");
    let disabled_card = "accordion_page.group[1].card[0]";
    let first = format!("{disabled_card}.item[0]");
    let last = format!("{disabled_card}.item[2]");
    while focused(&g).as_deref() != Some(first.as_str()) {
        assert!(g.view.handle_edit_key(EditKey::Tab), "Tab is taken");
        g.frame();
    }
    assert!(g.view.handle_nav(NavKey::Down));
    g.frame();
    assert_eq!(
        focused(&g).as_deref(),
        Some(last.as_str()),
        "past the disabled one"
    );
    assert!(
        !g.view.handle_nav(NavKey::Down),
        "an accordion does not wrap"
    );
    assert!(g.view.handle_edit_key(EditKey::Home));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(first.as_str()));
    assert!(g.view.handle_edit_key(EditKey::End));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(last.as_str()));
    assert!(g.view.handle_nav(NavKey::Up));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(first.as_str()));
    assert!(!g.view.handle_nav(NavKey::Up), "nor wraps up");
}

/// The Collapsible page's closed card: its trigger opens it the same from
/// the three, and its links show under its region.
#[test]
fn a_collapsible_opens_the_same_from_the_three() {
    let id = "collapsible_page.group[0].card[0]";
    let mut person = gallery("Collapsible");
    person.click(id);
    let mut reader = gallery("Collapsible");
    reader.press(id);
    let agent = by_agent("Collapsible", "open", vec![ask(id, "toggle_expanded")]);
    same("Collapsible", &person, &reader, &agent);
    assert_eq!(expanded(&person, id), Some(Value::Bool(true)));
    let link = sid(&format!("{id}.links[1]"));
    assert!(
        person.state().iter().any(|n| n.id == link),
        "its links show"
    );
}

/// The open card's second link becomes the current one the same from
/// the three.
#[test]
fn a_collapsible_s_link_is_activated_the_same_from_the_three() {
    let id = "collapsible_page.group[0].card[1].links[1]";
    let mut person = gallery("Collapsible");
    person.click(id);
    let mut reader = gallery("Collapsible");
    reader.press(id);
    let agent = by_agent("Collapsible", "link", vec![ask(id, "activate")]);
    same("Collapsible", &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{id}:active")),
        Some(Value::Bool(true))
    );
}

/// The arrows walk the options of an Item card, without wrapping, and
/// Enter chooses the focused one.
#[test]
fn the_arrows_walk_an_item_card_and_enter_chooses() {
    let mut g = gallery("Item");
    let first = "item_page.group[1].card[0].item[0]";
    let last = "item_page.group[1].card[0].item[2]";
    while focused(&g).as_deref() != Some(first) {
        assert!(g.view.handle_edit_key(EditKey::Tab), "Tab is taken");
        g.frame();
    }
    assert!(g.view.handle_edit_key(EditKey::End));
    g.frame();
    assert_eq!(focused(&g).as_deref(), Some(last));
    assert!(!g.view.handle_nav(NavKey::Down), "a card does not wrap");
    assert!(g.view.handle_enter());
    g.frame();
    assert_eq!(
        g.value(&format!("{last}:selected")),
        Some(Value::Bool(true))
    );
}

/// The carousel's next arrow pages, and a slide is chosen, the same from
/// the three.
#[test]
fn a_carousel_pages_and_chooses_the_same_from_the_three() {
    let next = "carousel_page.group[0].card[0].next";
    let mut person = gallery("Carousel");
    person.click(next);
    let mut reader = gallery("Carousel");
    reader.press(next);
    let agent = by_agent("Carousel", "page", vec![ask(next, "activate")]);
    same("Carousel", &person, &reader, &agent);

    let slide = "carousel_page.group[0].card[1].slide[0]";
    let mut person = gallery("Carousel");
    person.click(slide);
    let mut reader = gallery("Carousel");
    reader.press(slide);
    let agent = by_agent("Carousel", "slide", vec![ask(slide, "select")]);
    same("Carousel", &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{slide}:selected")),
        Some(Value::Bool(true))
    );
}

/// A pager's page goes current the same from the three.
#[test]
fn a_pager_goes_to_a_page_the_same_from_the_three() {
    let page = "pagination_page.group[0].card[0].page[2]";
    let mut person = gallery("Pagination");
    person.click(page);
    let mut reader = gallery("Pagination");
    reader.press(page);
    let agent = by_agent("Pagination", "page", vec![ask(page, "activate")]);
    same("Pagination", &person, &reader, &agent);
    assert_eq!(
        person.value(&format!("{page}:current")),
        Some(Value::Bool(true))
    );
}
