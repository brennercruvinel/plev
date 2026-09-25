//! the widgets that are one stop of Tab (task-75 phase 2d): a tab list, a
//! tree, a list and a table make one item focusable (the focused one,
//! else the chosen one, else the first), so the keyboard's Tab stops once
//! in each and the arrows move the focus along the rest.

use super::*;

/// The ids of the nodes of `s` that take the focus.
fn stops(s: &Snapshot) -> Vec<String> {
    s.nodes()
        .values()
        .filter(|n| n.focusable)
        .map(|n| n.id.to_string())
        .collect()
}

#[test]
fn a_tab_list_is_one_stop_on_its_focused_tab_else_its_active_one() {
    let view = id("view");
    let mut tabs = Tabs::new(["files", "branches", "history"]);
    let project = |tabs: &Tabs| {
        snap(|b| {
            tabs.project(b, &view, "view", None);
        })
    };
    assert_eq!(stops(&project(&tabs)), ["view[0]"]);
    tabs.active = 2;
    assert_eq!(stops(&project(&tabs)), ["view[2]"], "the active tab");
    tabs.set_focused_tab(Some(1));
    assert_eq!(stops(&project(&tabs)), ["view[1]"], "the focused tab");
    tabs.set_focused_tab(None);
    tabs.active = 7;
    assert_eq!(
        stops(&project(&tabs)),
        ["view[0]"],
        "no such tab: the first"
    );
}

#[test]
fn a_tree_is_one_stop_on_its_focused_row_else_its_selected_one_else_the_first() {
    let fs = id("fs");
    let mut tree = Tree::new(vec![
        TreeNode::branch(1, "src", vec![TreeNode::leaf(2, "main.rs")]),
        TreeNode::leaf(3, "README.md"),
    ]);
    let project = |tree: &Tree| {
        snap(|b| {
            tree.project(b, &fs, "fs", None);
        })
    };
    assert_eq!(stops(&project(&tree)), ["fs[1]"]);
    tree.selected = Some(3);
    assert_eq!(stops(&project(&tree)), ["fs[3]"], "the selected row");
    tree.set_focused(Some(1));
    assert_eq!(stops(&project(&tree)), ["fs[1]"], "the focused row");
    tree.set_focused(Some(2));
    assert_eq!(
        stops(&project(&tree)),
        ["fs[3]"],
        "a row under a collapsed branch is no stop"
    );
}

const LIST: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 300.0,
    h: 240.0,
};

/// A list of `count` rows 24 tall in a viewport of ten.
fn list(count: usize) -> VirtualList {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(count);
    list.set_viewport(LIST);
    list
}

#[test]
fn a_list_is_one_stop_on_its_focused_row_else_the_selected_one_in_view_else_the_first() {
    let files = id("files");
    let label = |i: usize| format!("row {i}");
    let mut rows = list(1_000);
    let project = |rows: &VirtualList| {
        snap(|b| {
            rows.project(b, &files, "files", None, label);
        })
    };
    assert_eq!(rows.shown(), 0..10);
    assert_eq!(stops(&project(&rows)), ["files[0]"]);
    rows.selected = Some(4);
    assert_eq!(stops(&project(&rows)), ["files[4]"], "the selected row");
    rows.scroll.scroll_to(24.0 * 50.0);
    assert_eq!(rows.shown(), 50..60);
    assert_eq!(
        stops(&project(&rows)),
        ["files[50]"],
        "the selected row out of view: the first in view"
    );
    rows.set_focused(Some(57));
    assert_eq!(stops(&project(&rows)), ["files[57]"], "the focused row");
    rows.set_focused(Some(1_000));
    assert_eq!(rows.focused(), None, "no such row");
}

#[test]
fn a_focused_row_scrolls_into_view_and_the_focus_follows_the_wheel() {
    let theme = hoff();
    let mut rows = list(1_000);
    rows.set_focused(Some(30));
    assert_eq!(rows.shown(), 21..31, "the least scroll that shows it");
    rows.set_focused(Some(5));
    assert_eq!(rows.shown(), 5..15);
    rows.set_focused(Some(999));
    assert_eq!(rows.shown(), 990..1_000, "the last row, at the bottom");
    let wheel = WidgetEvent::Scroll {
        x: 10.0,
        y: 10.0,
        delta: -24.0 * 100.0,
    };
    rows.handle_event(&wheel, LIST, &theme);
    assert_eq!(rows.shown(), 890..900);
    assert_eq!(rows.focused(), Some(899), "the nearest row in view");
    rows.handle_event(&wheel, LIST, &theme);
    assert_eq!(rows.focused(), Some(799));
    rows.scroll.scroll_to(24.0 * 890.0);
    assert_eq!(
        rows.focused(),
        Some(890),
        "it stayed where the wheel left it, not where it began"
    );
}

#[test]
fn a_table_is_one_stop_on_a_row() {
    let theme = hoff();
    let grid = id("grid");
    let mut t = Table::new(vec![Column::new("name", &theme)], &theme);
    t.set_row_count(40);
    t.rows.set_viewport(Table::rows_rect(LIST, &theme));
    let project = |t: &Table| {
        snap(|b| {
            t.project(b, &grid, "grid", None, |i| i.to_string());
        })
    };
    assert_eq!(stops(&project(&t)), ["grid[0]"]);
    t.rows.selected = Some(3);
    assert_eq!(stops(&project(&t)), ["grid[3]"]);
    t.rows.set_focused(Some(1));
    assert_eq!(stops(&project(&t)), ["grid[1]"]);
}
