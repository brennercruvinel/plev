//! tabs, nav link, breadcrumb and split pane.

use super::*;
use sem::Constraints;

// Tabs

#[test]
fn tabs_project_a_tab_list_with_one_tab_per_label() {
    let view = id("view");
    let mut tabs = Tabs::new(["list", "board", "calendar"]);
    tabs.active = 1;
    let s = snap(|b| {
        tabs.project(b, &view, "view", None);
    });
    let node = s.get(&view).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::TabList, "view"));
    assert_eq!(node.state.get("selected"), Some(&Value::from("1")));
    assert!(node.actions.is_empty());
    let kids: Vec<(&str, Role, &str)> = s
        .children(&view)
        .map(|n| (n.id.as_str(), n.role, n.label.as_str()))
        .collect();
    assert_eq!(
        kids,
        [
            ("view[0]", Role::Tab, "list"),
            ("view[1]", Role::Tab, "board"),
            ("view[2]", Role::Tab, "calendar"),
        ]
    );
    assert_eq!(actions(&s, &view.key("2")), ["select"]);
    assert_eq!(reads(&s, &view.key("2"), "select"), ["view:selected"]);

    let empty = Tabs::new(Vec::<String>::new());
    let s = snap(|b| {
        empty.project(b, &view, "view", None);
    });
    assert_eq!(
        s.get(&view).unwrap().state.get("selected"),
        Some(&Value::from(""))
    );
}

#[test]
fn tabs_bind_the_strip_and_each_segment_they_hit_test_with() {
    let theme = hoff();
    let view = id("view");
    let mut tabs = Tabs::new(["list", "board", "calendar"]).sem(view.clone());
    let bounds = Rect::new(10.0, 10.0, 300.0, Tabs::height(&theme));
    let mut c = Compositor::new();
    c.begin_frame();
    tabs.render(&mut c, bounds, &theme);
    let segments = tabs.item_rects(bounds, &theme);
    for (i, r) in segments.iter().enumerate() {
        assert_bound(&c, &view.key(i.to_string()), *r, LayerId::DEFAULT);
    }
    // The strip's padding belongs to the list, not to a tab.
    assert_eq!(c.binding(&view).unwrap().rect, rect_of(bounds));
    assert_eq!(c.hit(bounds.x + 1.0, bounds.y + 1.0), Some(&view));
    let (x, y) = segments[2].center();
    assert!(tabs.handle_event(&down(x, y), bounds, &theme).clicked);
    assert_eq!(tabs.active, 2);
}

#[test]
fn tab_select_goes_stale_when_a_click_switches_tabs() {
    let theme = hoff();
    let view = id("view");
    let mut tabs = Tabs::new(["list", "board", "calendar"]);
    let seen = snap(|b| {
        tabs.project(b, &view, "view", None);
    });
    let req = Request::observe(&seen, test_actor(), view.key("2"), "select", None).unwrap();
    let bounds = Rect::new(10.0, 10.0, 300.0, Tabs::height(&theme));
    let (x, y) = tabs.item_rects(bounds, &theme)[1].center();
    tabs.handle_event(&down(x, y), bounds, &theme);
    let now = snap(|b| {
        tabs.project(b, &view, "view", None);
    });
    assert_stale(&req, &now, Value::from("0"), Value::from("1"));
}

#[test]
fn tabs_hover_and_focus_are_visual_only() {
    let theme = hoff();
    let view = id("view");
    let mut tabs = Tabs::new(["list", "board"]);
    let seen = snap(|b| {
        tabs.project(b, &view, "view", None);
    });
    let req = Request::observe(&seen, test_actor(), view.key("1"), "select", None).unwrap();
    let bounds = Rect::new(10.0, 10.0, 300.0, Tabs::height(&theme));
    let (x, y) = tabs.item_rects(bounds, &theme)[1].center();
    tabs.handle_event(&move_to(x, y), bounds, &theme);
    tabs.set_focused(true);
    assert_eq!(tabs.hovered(), Some(1));
    let now = snap(|b| {
        tabs.project(b, &view, "view", None);
    });
    assert_visual_only(&req, &seen, &now);
}

// NavLink

#[test]
fn nav_link_projects_a_link_with_active_and_activate() {
    let cards = id("nav.cards");
    let link = NavLink::new("Cards")
        .icon("clipboard")
        .badge(Badge::count("3"))
        .active(true);
    let s = snap(|b| {
        link.project(b, &cards, None);
    });
    let node = s.get(&cards).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Link, "Cards"));
    assert_eq!(node.state.get("active"), Some(&Value::Bool(true)));
    assert_eq!(actions(&s, &cards), ["activate"]);
    assert_eq!(reads(&s, &cards, "activate"), ["nav.cards:active"]);
}

#[test]
fn nav_link_binds_its_bounds_in_the_row_and_in_the_rail() {
    let theme = hoff();
    let cards = id("cards");
    let mut link = NavLink::new("Cards").icon("clipboard").sem(cards.clone());
    let row = Rect::new(10.0, 10.0, 220.0, NavLink::height(&theme));
    let rail = Rect::new(
        10.0,
        10.0,
        NavLink::icon_slot(&theme),
        NavLink::height(&theme),
    );
    for bounds in [row, rail] {
        let mut c = Compositor::new();
        let layer = c.create_layer(5);
        c.begin_frame();
        link.render_to_layer(&mut c, layer, bounds, &theme);
        assert_bound(&c, &cards, bounds, layer);
        let (x, y) = bounds.center();
        link.handle_event(&down(x, y), bounds);
        assert!(link.handle_event(&up(x, y), bounds).clicked);
    }
}

#[test]
fn nav_link_activate_goes_stale_when_the_link_becomes_active() {
    let cards = id("cards");
    let mut link = NavLink::new("Cards");
    let seen = snap(|b| {
        link.project(b, &cards, None);
    });
    let req = Request::observe(&seen, test_actor(), &cards, "activate", None).unwrap();
    link.active = true;
    let now = snap(|b| {
        link.project(b, &cards, None);
    });
    assert_stale(&req, &now, Value::Bool(false), Value::Bool(true));
}

#[test]
fn nav_link_hover_and_press_are_visual_only() {
    let cards = id("cards");
    let mut link = NavLink::new("Cards");
    let seen = snap(|b| {
        link.project(b, &cards, None);
    });
    let req = Request::observe(&seen, test_actor(), &cards, "activate", None).unwrap();
    link.handle_event(&move_to(20.0, 20.0), B);
    link.handle_event(&down(20.0, 20.0), B);
    assert!(link.is_hovered());
    let now = snap(|b| {
        link.project(b, &cards, None);
    });
    assert_visual_only(&req, &seen, &now);
}

// Breadcrumb

const TRAIL: [&str; 5] = ["home", "projects", "plev", "crates", "comps"];

#[test]
fn breadcrumb_projects_links_to_every_crumb_but_the_current_one() {
    let path = id("path");
    let trail = Breadcrumb::new(TRAIL);
    let s = snap(|b| {
        trail.project(b, &path, "location", None);
    });
    let node = s.get(&path).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Group, "location"));
    let kids: Vec<(&str, Role, &str, usize)> = s
        .children(&path)
        .map(|n| (n.id.as_str(), n.role, n.label.as_str(), n.actions.len()))
        .collect();
    assert_eq!(
        kids,
        [
            ("path[0]", Role::Link, "home", 1),
            ("path[1]", Role::Link, "projects", 1),
            ("path[2]", Role::Link, "plev", 1),
            ("path[3]", Role::Link, "crates", 1),
            ("path[4]", Role::Label, "comps", 0),
        ]
    );
    assert_eq!(actions(&s, &path.key("1")), ["activate"]);
}

#[test]
fn breadcrumb_binds_the_crumbs_it_draws_and_not_the_collapsed_ones() {
    let theme = hoff();
    let path = id("path");
    let mut trail = Breadcrumb::new(TRAIL).sem(path.clone());
    // Narrow enough to collapse the middle of the trail.
    let bounds = Rect::new(10.0, 10.0, 170.0, 24.0);
    let mut c = Compositor::new();
    c.begin_frame();
    trail.render(&mut c, bounds, &theme);
    let rects = trail.crumb_rects(bounds, &theme);
    assert!(
        rects.iter().any(|(i, _)| i.is_none()),
        "the trail collapsed"
    );
    assert_eq!(c.binding(&path).unwrap().rect, rect_of(bounds));
    for (i, r) in &rects {
        if let Some(i) = i {
            assert_bound(&c, &path.key(i.to_string()), *r, LayerId::DEFAULT);
        }
    }
    let drawn: Vec<usize> = rects.iter().filter_map(|(i, _)| *i).collect();
    for i in (0..TRAIL.len()).filter(|i| !drawn.contains(i)) {
        assert!(
            c.binding(&path.key(i.to_string())).is_none(),
            "{i} is collapsed"
        );
    }
    // A click at the first crumb's center activates it.
    let (x, y) = rects[0].1.center();
    trail.handle_event(&down(x, y), bounds, &theme);
    assert_eq!(trail.handle_event(&up(x, y), bounds, &theme).1, Some(0));
}

#[test]
fn breadcrumb_hover_and_press_are_visual_only() {
    let theme = hoff();
    let path = id("path");
    let mut trail = Breadcrumb::new(TRAIL);
    let seen = snap(|b| {
        trail.project(b, &path, "location", None);
    });
    let req = Request::observe(&seen, test_actor(), path.key("0"), "activate", None).unwrap();
    let bounds = Rect::new(10.0, 10.0, 600.0, 24.0);
    let (x, y) = trail.crumb_rects(bounds, &theme)[0].1.center();
    trail.handle_event(&move_to(x, y), bounds, &theme);
    trail.handle_event(&down(x, y), bounds, &theme);
    assert_eq!(trail.hovered(), Some(0));
    let now = snap(|b| {
        trail.project(b, &path, "location", None);
    });
    assert_visual_only(&req, &seen, &now);
}

// SplitPane

fn split_bounds() -> Rect {
    Rect::new(0.0, 0.0, 800.0, 400.0)
}

#[test]
fn split_pane_projects_its_ratio_and_a_constrained_set_ratio() {
    let split = id("editor.split");
    let pane = SplitPane::new(SplitDirection::Horizontal, 0.25, &hoff());
    let s = snap(|b| {
        pane.project(b, &split, "editor", None);
    });
    let node = s.get(&split).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Group, "editor"));
    assert_eq!(node.state.get("ratio"), Some(&Value::Num(0.25)));
    assert_eq!(
        node.constraints,
        Constraints {
            editable: true,
            range: Some((0.0, 1.0)),
            ..Constraints::default()
        }
    );
    assert_eq!(actions(&s, &split), ["set_ratio"]);
    assert_eq!(reads(&s, &split, "set_ratio"), ["editor.split:ratio"]);
    let set = |v: f64| Request::observe(&s, test_actor(), &split, "set_ratio", Some(v.into()));
    assert_eq!(check(&set(0.5).unwrap(), &s), Verdict::Accepted);
    assert!(matches!(
        check(&set(1.5).unwrap(), &s),
        Verdict::Rejected(Rejection::InvalidParam { .. })
    ));
}

#[test]
fn split_pane_binds_the_grab_area_it_hit_tests_with() {
    let theme = hoff();
    let split = id("split");
    let mut pane = SplitPane::new(SplitDirection::Horizontal, 0.5, &theme).sem(split.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    pane.render(&mut c, split_bounds(), &theme);
    let grab = pane.grab_rect(split_bounds());
    assert_bound(&c, &split, grab, LayerId::DEFAULT);
    // The drawn line sits inside the grab area.
    let line = pane.divider_rect(split_bounds());
    assert!(grab.x <= line.x && line.x + line.w <= grab.x + grab.w);
    // A pane's content is not the divider.
    let (x, y) = pane.first_rect(split_bounds()).center();
    assert_eq!(c.hit(x, y), None);
    let (x, y) = grab.center();
    pane.handle_event(&down(x, y), split_bounds());
    assert!(pane.is_dragging());
}

#[test]
fn set_ratio_goes_stale_when_a_drag_moves_the_divider() {
    let theme = hoff();
    let split = id("split");
    let mut pane = SplitPane::new(SplitDirection::Horizontal, 0.5, &theme);
    let seen = snap(|b| {
        pane.project(b, &split, "split", None);
    });
    let req = Request::observe(&seen, test_actor(), &split, "set_ratio", Some(0.3.into())).unwrap();
    let (x, y) = pane.grab_rect(split_bounds()).center();
    pane.handle_event(&down(x, y), split_bounds());
    let to = (split_bounds().w - pane.divider) * 0.75;
    pane.handle_event(&move_to(to, y), split_bounds());
    pane.handle_event(&up(to, y), split_bounds());
    let now = snap(|b| {
        pane.project(b, &split, "split", None);
    });
    assert_stale(&req, &now, Value::Num(0.5), Value::Num(0.75));
}

#[test]
fn split_pane_hover_is_visual_only() {
    let theme = hoff();
    let split = id("split");
    let mut pane = SplitPane::new(SplitDirection::Vertical, 0.5, &theme);
    let seen = snap(|b| {
        pane.project(b, &split, "split", None);
    });
    let req = Request::observe(&seen, test_actor(), &split, "set_ratio", Some(0.3.into())).unwrap();
    let (x, y) = pane.grab_rect(split_bounds()).center();
    pane.handle_event(&move_to(x, y), split_bounds());
    assert!(pane.is_hovered());
    let now = snap(|b| {
        pane.project(b, &split, "split", None);
    });
    assert_visual_only(&req, &seen, &now);
}
