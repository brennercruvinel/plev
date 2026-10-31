//! Front G, navigation and data (task-104): breadcrumb, navigation menu,
//! stat, code block, chart and graph, the same request from the person,
//! the reader and an agent, and what a readout says to each.

use accord::Value;

use super::harness::{ask, by_agent, gallery, same, sid};

/// A crumb of the folded trail: the person's click, the reader's press
/// and the agent's `activate` echo it the same.
#[test]
fn a_folded_trail_s_crumb_is_chosen_the_same_from_the_three() {
    let crumb = "breadcrumb_page.folded[0]";
    let mut person = gallery("Breadcrumb");
    person.click(crumb);
    let mut reader = gallery("Breadcrumb");
    reader.press(crumb);
    let agent = by_agent("Breadcrumb", "folded", vec![ask(crumb, "activate")]);
    same("Breadcrumb", &person, &reader, &agent);
}

/// A link of another card than the menu moves the one active mark the
/// same from the three, and the menu's first link lets it go.
#[test]
fn a_trailing_link_takes_the_active_mark_the_same_from_the_three() {
    let link = "navigation_menu_page.link[5]";
    let mut person = gallery("Navigation Menu");
    person.click(link);
    let mut reader = gallery("Navigation Menu");
    reader.press(link);
    let agent = by_agent("Navigation Menu", "badge", vec![ask(link, "activate")]);
    same("Navigation Menu", &person, &reader, &agent);
    let active = |id: &str| person.value(&format!("{id}:active"));
    assert_eq!(active(link), Some(Value::Bool(true)));
    assert_eq!(
        active("navigation_menu_page.link[0]"),
        Some(Value::Bool(false))
    );
}

/// A readout is read, not operated: what the agent observes (the
/// view's projection, which the port serves), what the reader's tree has
/// and what the person sees bound are the one readout.
#[test]
fn a_stat_says_its_value_to_the_reader_and_the_agent() {
    let id = "stat_page.group[0].card[1]";
    let g = gallery("Stat");
    let agent = g.view.semantics().expect("the page projects");
    let read = agent.get(&sid(id)).expect("the agent observes it");
    assert_eq!(read.state.get("value"), Some(&Value::Text("1.3 GB".into())));
    assert!(read.actions.is_empty(), "nothing to operate");
    let (_, shown) = g.shown();
    assert_eq!(
        shown.snapshot.get(&sid(id)),
        Some(read),
        "the reader hears the same"
    );
    assert!(shown.ids.node(&sid(id)).is_some(), "in the reader's tree");
    let _ = g.middle(id);
}
