//! Choices in a collection: the same tab, row, item, link, crumb, tree
//! row and split from the three.

use accord::Value;
use engine::control::accessibility::accesskit::{Action, Role as AkRole};

use super::harness::{ask, ask_with, by_agent, gallery, same, sid};

/// reader's press and the agent's action select or activate the same.
#[test]
fn a_tab_a_row_an_item_and_a_link_are_chosen_the_same_from_the_three() {
    for (page, id, action) in [
        ("Tabs", "tabs_page.tabs[2]", "select"),
        ("Data Table", "table_page.table[3]", "select"),
        ("Table", "table_page.table[3]", "select"),
        ("Scroll Area", "scroll_area_page.list[4]", "select"),
        ("Item", "item_page.group[0].card[0].item[1]", "select"),
        (
            "Navigation Menu",
            "navigation_menu_page.link[2]",
            "activate",
        ),
        ("Kbd", "kbd_page.group[1].card[0].link[2]", "activate"),
        ("Breadcrumb", "breadcrumb_page.trail[1]", "activate"),
    ] {
        let mut person = gallery(page);
        person.click(id);
        let mut reader = gallery(page);
        reader.press(id);
        let agent = by_agent(page, "choose", vec![ask(id, action)]);
        same(page, &person, &reader, &agent);
    }
    // the row a reader chose is the selected one in the tree (the table
    // says it in `selected_row`), the others not
    let mut g = gallery("Data Table");
    g.press("table_page.table[3]");
    let (u, shown) = g.shown();
    let selected = |id: &str| {
        let at = shown.ids.node(&sid(id));
        u.nodes
            .iter()
            .find(|(n, _)| Some(*n) == at)
            .and_then(|(_, n)| n.is_selected())
    };
    assert_eq!(selected("table_page.table[3]"), Some(true));
    assert_eq!(selected("table_page.table[2]"), Some(false));

    let mut g = gallery("Breadcrumb");
    g.press("breadcrumb_page.trail[1]");
    let (u, shown) = g.shown();
    let echo = shown.ids.node(&sid("breadcrumb_page.clicked"));
    let echo = u
        .nodes
        .iter()
        .find(|(n, _)| Some(*n) == echo)
        .map(|(_, n)| n);
    assert_eq!(echo.map(|n| n.role()), Some(AkRole::Status));
    assert_eq!(echo.and_then(|n| n.value()), Some("clicked crumb 1"));
}

/// (`toggle_expanded`, then `select`).
#[test]
fn a_tree_opens_and_selects_the_same_from_the_three() {
    let leaf = "tree_page.tree[5]";
    let mut person = gallery("Tree");
    person.click(leaf);
    let mut reader = gallery("Tree");
    reader.press(leaf);
    let agent = by_agent("Tree", "leaf", vec![ask(leaf, "select")]);
    same("Tree", &person, &reader, &agent);

    let branch = "tree_page.tree[7]";
    let mut person = gallery("Tree");
    person.click(branch);
    let mut reader = gallery("Tree");
    reader.press("tree_page.tree[7][toggle_expanded]");
    reader.press(branch);
    let agent = by_agent(
        "Tree",
        "branch",
        vec![ask(branch, "toggle_expanded"), ask(branch, "select")],
    );
    same("Tree", &person, &reader, &agent);
    assert_eq!(
        person.value("tree_page.tree[7]:expanded"),
        Some(Value::Bool(true))
    );
    assert!(
        person
            .state()
            .iter()
            .any(|n| n.id == sid("tree_page.tree[8]")),
        "its children show"
    );
}

/// role): the split offers no value to set, and hears its share.
#[test]
fn a_split_moves_the_same_from_the_person_and_an_agent() {
    let id = "resizable_page.split";
    let mut person = gallery("Resizable");
    person.drag(id, 100.0);
    let Some(Value::Num(r)) = person.value("resizable_page.split:ratio") else {
        panic!("the split has a ratio");
    };
    assert!(r > 0.35, "the drag moved it: {r}");
    let agent = by_agent(
        "Resizable",
        "split",
        vec![ask_with(id, "set_ratio", Value::Num(r))],
    );
    assert_eq!(agent.state(), person.state());
    assert!(!person.offers(id, Action::SetValue));
    let (u, shown) = person.shown();
    let split = shown.ids.node(&sid(id));
    let said = u
        .nodes
        .iter()
        .find(|(n, _)| Some(*n) == split)
        .and_then(|(_, n)| n.description());
    assert!(said.is_some_and(|d| d.starts_with("the first pane takes")));
}
