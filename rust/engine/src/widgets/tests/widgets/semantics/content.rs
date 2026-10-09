//! virtual list, table and tree.

use super::*;

fn label_of(i: usize) -> String {
    format!("row {i}")
}

/// The part of `row` inside `bounds`, when any.
fn inside(row: Rect, bounds: Rect) -> Option<Rect> {
    let top = row.y.max(bounds.y);
    let bottom = (row.y + row.h).min(bounds.y + bounds.h);
    (bottom >= top).then(|| Rect::new(row.x, top, row.w, bottom - top))
}

// VirtualList

const LIST: Rect = Rect {
    x: 10.0,
    y: 10.0,
    w: 300.0,
    h: 200.0,
};

fn list(count: usize) -> VirtualList {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(count);
    list.set_viewport(LIST);
    list
}

#[test]
fn virtual_list_projects_its_count_selection_and_only_the_rows_it_holds() {
    let files = id("files");
    let mut rows = list(10_000);
    rows.selected = Some(3);
    let s = snap(|b| {
        rows.project(b, &files, "files", None, label_of);
    });
    let node = s.get(&files).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::List, "files"));
    assert_eq!(node.state.get("count"), Some(&Value::Int(10_000)));
    assert_eq!(node.state.get("selected"), Some(&Value::from("3")));
    let held = rows.visible_range();
    assert!(held.len() < 20, "a window, not the dataset");
    let kids: Vec<(String, Role, String)> = s
        .children(&files)
        .map(|n| (n.id.to_string(), n.role, n.label.clone()))
        .collect();
    let expected: Vec<(String, Role, String)> = held
        .map(|i| (format!("files[{i}]"), Role::ListItem, label_of(i)))
        .collect();
    assert_eq!(kids, expected);
    assert_eq!(reads(&s, &files.key("2"), "select"), ["files:selected"]);

    // Nothing selected, and before any layout the list holds no row.
    let fresh = VirtualList::new(24.0);
    let s = snap(|b| {
        fresh.project(b, &files, "files", None, label_of);
    });
    assert_eq!(
        s.get(&files).unwrap().state.get("selected"),
        Some(&Value::from(""))
    );
    assert_eq!(s.children(&files).count(), 0);
}

#[test]
fn virtual_list_binds_the_part_of_each_row_the_hit_test_reaches() {
    let theme = hoff();
    let files = id("files");
    let mut rows = list(100).sem(files.clone());
    rows.scroll.scroll_by(30.0);
    let mut c = Compositor::new();
    let layer = c.create_layer(3);
    c.begin_frame();
    rows.render_with_to_layer(&mut c, layer, LIST, &theme, |_, _, _, _, _| {});
    assert_eq!(c.binding(&files).unwrap().rect, rect_of(LIST));
    let mut bound = 0;
    for i in rows.visible_range() {
        match inside(rows.item_rect(i), LIST) {
            Some(part) if part.h > 0.0 => {
                assert_bound(&c, &files.key(i.to_string()), part, layer);
                bound += 1;
            }
            Some(_) => {}
            None => assert!(c.binding(&files.key(i.to_string())).is_none(), "{i}"),
        }
    }
    assert!(bound > 5);
    // The overscan row above the viewport is held and not reachable.
    assert!(rows.visible_range().start == 0 && c.binding(&files.key("0")).is_none());
    let (x, y) = inside(rows.item_rect(4), LIST).unwrap().center();
    assert_eq!(c.hit(x, y), Some(&files.key("4")));
    assert!(rows.handle_event(&down(x, y), LIST, &theme).clicked);
    assert_eq!(rows.selected, Some(4));
}

#[test]
fn list_row_select_goes_stale_when_a_click_selects_another_row() {
    let theme = hoff();
    let files = id("files");
    let mut rows = list(50);
    let seen = snap(|b| {
        rows.project(b, &files, "files", None, label_of);
    });
    let req = Request::observe(&seen, test_actor(), files.key("5"), "select", None).unwrap();
    let (x, y) = rows.item_rect(2).center();
    rows.handle_event(&down(x, y), LIST, &theme);
    let now = snap(|b| {
        rows.project(b, &files, "files", None, label_of);
    });
    assert_stale(&req, &now, Value::from(""), Value::from("2"));
}

#[test]
fn list_hover_is_visual_only_and_scrolling_never_stales_a_row() {
    let theme = hoff();
    let files = id("files");
    let mut rows = list(50);
    let seen = snap(|b| {
        rows.project(b, &files, "files", None, label_of);
    });
    let req = Request::observe(&seen, test_actor(), files.key("5"), "select", None).unwrap();
    let (x, y) = rows.item_rect(1).center();
    rows.handle_event(&move_to(x, y), LIST, &theme);
    assert_eq!(rows.hovered(), Some(1));
    let now = snap(|b| {
        rows.project(b, &files, "files", None, label_of);
    });
    assert_visual_only(&req, &seen, &now);

    // Scrolling changes which rows the list holds, never what a row's
    // select reads.
    rows.handle_event(&WidgetEvent::Scroll { x, y, delta: 48.0 }, LIST, &theme);
    let now = snap(|b| {
        rows.project(b, &files, "files", None, label_of);
    });
    assert_ne!(now.revision(), seen.revision(), "the held rows moved");
    assert_eq!(check(&req, &now), Verdict::Accepted);
}

// Table

fn table() -> Table {
    let theme = hoff();
    let mut t = Table::new(
        vec![
            Column::new("name", &theme),
            Column::new("size", &theme).priority(0),
        ],
        &theme,
    );
    t.set_row_count(40);
    t
}

const GRID: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 400.0,
    h: 300.0,
};

#[test]
fn table_projects_its_count_selected_row_and_the_rows_it_holds() {
    let theme = hoff();
    let grid = id("grid");
    let mut t = table();
    t.rows.set_viewport(Table::rows_rect(GRID, &theme));
    t.rows.selected = Some(2);
    let s = snap(|b| {
        t.project(b, &grid, "files", None, label_of);
    });
    let node = s.get(&grid).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Table, "files"));
    assert_eq!(node.state.get("count"), Some(&Value::Int(40)));
    assert_eq!(node.state.get("selected_row"), Some(&Value::from("2")));
    assert!(node.actions.is_empty(), "no sort in this table");
    let kids: Vec<(String, Role)> = s
        .children(&grid)
        .map(|n| (n.id.to_string(), n.role))
        .collect();
    let expected: Vec<(String, Role)> = t
        .rows
        .visible_range()
        .map(|i| (format!("grid[{i}]"), Role::Row))
        .collect();
    assert_eq!(kids, expected);
    assert_eq!(s.get(&grid.key("0")).unwrap().label, "row 0");
    assert_eq!(reads(&s, &grid.key("1"), "select"), ["grid:selected_row"]);
}

#[test]
fn table_binds_itself_and_the_rows_under_its_header() {
    let theme = hoff();
    let grid = id("grid");
    let mut t = table().sem(grid.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    t.render_with(&mut c, GRID, &theme, |_, _, _, _| {});
    assert_eq!(c.binding(&grid).unwrap().rect, rect_of(GRID));
    // The header is the table's, not a row's.
    assert_eq!(c.hit(GRID.x + 5.0, GRID.y + 2.0), Some(&grid));
    let rows = Table::rows_rect(GRID, &theme);
    let row = inside(t.rows.item_rect(3), rows).unwrap();
    assert_bound(&c, &grid.key("3"), row, LayerId::DEFAULT);
    let (x, y) = row.center();
    assert!(t.handle_event(&down(x, y), GRID, &theme).clicked);
    assert_eq!(t.selected(), Some(3));
}

#[test]
fn table_row_select_goes_stale_when_a_click_selects_another_row() {
    let theme = hoff();
    let grid = id("grid");
    let mut t = table();
    t.rows.set_viewport(Table::rows_rect(GRID, &theme));
    let seen = snap(|b| {
        t.project(b, &grid, "files", None, label_of);
    });
    let req = Request::observe(&seen, test_actor(), grid.key("3"), "select", None).unwrap();
    let (x, y) = t.rows.item_rect(1).center();
    t.handle_event(&down(x, y), GRID, &theme);
    let now = snap(|b| {
        t.project(b, &grid, "files", None, label_of);
    });
    assert_stale(&req, &now, Value::from(""), Value::from("1"));
}

#[test]
fn table_hover_is_visual_only() {
    let theme = hoff();
    let grid = id("grid");
    let mut t = table();
    t.rows.set_viewport(Table::rows_rect(GRID, &theme));
    let seen = snap(|b| {
        t.project(b, &grid, "files", None, label_of);
    });
    let req = Request::observe(&seen, test_actor(), grid.key("3"), "select", None).unwrap();
    let (x, y) = t.rows.item_rect(2).center();
    t.handle_event(&move_to(x, y), GRID, &theme);
    assert_eq!(t.rows.hovered(), Some(2));
    let now = snap(|b| {
        t.project(b, &grid, "files", None, label_of);
    });
    assert_visual_only(&req, &seen, &now);
}

// Tree

fn files() -> Tree {
    Tree::new(vec![
        TreeNode::branch(
            1,
            "src",
            vec![
                TreeNode::leaf(2, "lib.rs"),
                TreeNode::branch(3, "tests", vec![TreeNode::leaf(4, "a.rs")]),
            ],
        )
        .expanded(true),
        TreeNode::leaf(5, "Cargo.toml"),
    ])
}

const TREE: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 240.0,
    h: 400.0,
};

#[test]
fn tree_projects_the_visible_rows_nested_as_the_tree_nests_them() {
    let fs = id("fs");
    let mut tree = files();
    tree.selected = Some(2);
    let s = snap(|b| {
        tree.project(b, &fs, "files", None);
    });
    let node = s.get(&fs).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Tree, "files"));
    let parent = |who: &str| {
        s.get(&id(who))
            .and_then(|n| n.parent.as_ref())
            .map(|p| p.to_string())
    };
    assert_eq!(parent("fs[1]").as_deref(), Some("fs"));
    assert_eq!(parent("fs[2]").as_deref(), Some("fs[1]"));
    assert_eq!(parent("fs[3]").as_deref(), Some("fs[1]"));
    assert_eq!(parent("fs[5]").as_deref(), Some("fs"));
    // A collapsed branch's children are not rows.
    assert!(s.get(&id("fs[4]")).is_none());
    let roots: Vec<&str> = s.children(&fs).map(|n| n.id.as_str()).collect();
    assert_eq!(roots, ["fs[1]", "fs[5]"]);

    let src = s.get(&id("fs[1]")).unwrap();
    assert_eq!((src.role, src.label.as_str()), (Role::TreeItem, "src"));
    assert_eq!(src.state.get("expanded"), Some(&Value::Bool(true)));
    assert_eq!(src.state.get("selected"), Some(&Value::Bool(false)));
    assert_eq!(actions(&s, &id("fs[1]")), ["select", "toggle_expanded"]);
    assert_eq!(
        reads(&s, &id("fs[1]"), "toggle_expanded"),
        ["fs[1]:expanded"]
    );
    let lib = s.get(&id("fs[2]")).unwrap();
    assert_eq!(lib.state.get("selected"), Some(&Value::Bool(true)));
    assert_eq!(lib.state.get("expanded"), None, "a leaf does not expand");
    assert_eq!(actions(&s, &id("fs[2]")), ["select"]);
    assert_eq!(reads(&s, &id("fs[2]"), "select"), ["fs[2]:selected"]);
}

#[test]
fn tree_binds_itself_and_each_row_it_draws() {
    let theme = hoff();
    let fs = id("fs");
    let mut tree = files().sem(fs.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    tree.render(&mut c, TREE, &theme);
    assert_eq!(c.binding(&fs).unwrap().rect, rect_of(TREE));
    let h = tree.row_height(&theme);
    for (i, row) in tree.visible_rows().iter().enumerate() {
        let r = Rect::new(TREE.x, TREE.y + i as f32 * h, TREE.w, h);
        assert_bound(&c, &fs.key(row.id.to_string()), r, LayerId::DEFAULT);
    }
    // The last visible row is `Cargo.toml` (fs[5]); a click selects it.
    let (x, y) = Rect::new(TREE.x, TREE.y + 3.0 * h, TREE.w, h).center();
    assert_eq!(c.hit(x, y), Some(&fs.key("5")));
    assert!(tree.handle_event(&down(x, y), TREE, &theme).clicked);
    assert_eq!(tree.selected, Some(5));
}

#[test]
fn tree_toggle_expanded_goes_stale_when_a_click_expands_the_branch() {
    let theme = hoff();
    let fs = id("fs");
    let mut tree = files();
    let seen = snap(|b| {
        tree.project(b, &fs, "files", None);
    });
    let tests = id("fs[3]");
    let req = Request::observe(&seen, test_actor(), &tests, "toggle_expanded", None).unwrap();
    let h = tree.row_height(&theme);
    // `tests` is the third visible row.
    tree.handle_event(&down(20.0, 2.5 * h), TREE, &theme);
    let now = snap(|b| {
        tree.project(b, &fs, "files", None);
    });
    assert!(
        now.get(&id("fs[4]")).is_some(),
        "expanded: its child is a row"
    );
    assert_stale(&req, &now, Value::Bool(false), Value::Bool(true));
}

#[test]
fn tree_hover_is_visual_only() {
    let theme = hoff();
    let fs = id("fs");
    let mut tree = files();
    let seen = snap(|b| {
        tree.project(b, &fs, "files", None);
    });
    let req = Request::observe(&seen, test_actor(), id("fs[2]"), "select", None).unwrap();
    tree.handle_event(&move_to(20.0, 5.0), TREE, &theme);
    let now = snap(|b| {
        tree.project(b, &fs, "files", None);
    });
    assert_visual_only(&req, &seen, &now);
}
