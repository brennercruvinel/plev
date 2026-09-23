//! chip, icon button and submit button.

use super::*;

// Chip

#[test]
fn chip_projects_a_toggle_a_button_or_a_label_by_how_it_is_used() {
    let (filter, tag, go) = (id("f.rust"), id("f.tag"), id("f.go"));
    let selectable = Chip::new("rust").selectable(true).selected(true);
    let interactive = Chip::new("go").interactive(true);
    let fixed = Chip::new("beta");
    let s = snap(|b| {
        b.node(id("f"), Role::Group, "filters");
        selectable.project(b, &filter, Some(&id("f")));
        interactive.project(b, &go, None);
        fixed.project(b, &tag, None);
    });

    let node = s.get(&filter).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Toggle, "rust"));
    assert_eq!(node.parent.as_ref(), Some(&id("f")));
    assert_eq!(node.state.get("selected"), Some(&Value::Bool(true)));
    assert_eq!(actions(&s, &filter), ["toggle"]);
    assert_eq!(reads(&s, &filter, "toggle"), ["f.rust:selected"]);

    let node = s.get(&go).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Button, "go"));
    assert!(node.state.is_empty());
    assert_eq!(actions(&s, &go), ["activate"]);

    // A static tag runs nothing when clicked, so it declares no action.
    let node = s.get(&tag).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Label, "beta"));
    assert!(node.actions.is_empty());
}

#[test]
fn chip_binds_the_bounds_it_hit_tests_with() {
    let theme = hoff();
    let tag = id("tag");
    let mut chip = Chip::new("rust").selectable(true).sem(tag.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    chip.render(&mut c, B, &theme);
    assert_bound(&c, &tag, B, LayerId::DEFAULT);
    // The widget's own hit test agrees at that point.
    let (x, y) = B.center();
    chip.handle_event(&down(x, y), B);
    assert!(chip.handle_event(&up(x, y), B).clicked);
    assert_eq!(c.hit(B.x + B.w + 1.0, y), None, "outside the bounds");
}

#[test]
fn selectable_chip_toggle_goes_stale_when_a_click_flips_it() {
    let tag = id("tag");
    let mut chip = Chip::new("rust").selectable(true);
    let seen = snap(|b| {
        chip.project(b, &tag, None);
    });
    let req = Request::observe(&seen, test_actor(), &tag, "toggle", None).unwrap();

    // A click on a filter chip toggles it, like a checkbox.
    assert!(click(|e| chip.handle_event(e, B)).clicked);
    assert!(chip.selected);
    let now = snap(|b| {
        chip.project(b, &tag, None);
    });
    assert_stale(&req, &now, Value::Bool(false), Value::Bool(true));
}

#[test]
fn chip_hover_and_press_are_visual_only() {
    let tag = id("tag");
    let mut chip = Chip::new("rust").selectable(true);
    let seen = snap(|b| {
        chip.project(b, &tag, None);
    });
    let req = Request::observe(&seen, test_actor(), &tag, "toggle", None).unwrap();
    assert!(chip.handle_event(&move_to(20.0, 20.0), B).changed);
    assert!(chip.is_hovered());
    chip.handle_event(&down(20.0, 20.0), B);
    let now = snap(|b| {
        chip.project(b, &tag, None);
    });
    assert_visual_only(&req, &seen, &now);
}

#[test]
fn an_interactive_chip_that_is_not_selectable_keeps_its_selected_look() {
    let mut chip = Chip::new("go").interactive(true).selected(true);
    assert!(click(|e| chip.handle_event(e, B)).clicked);
    assert!(chip.selected, "only a selectable chip toggles itself");
    let mut fixed = Chip::new("beta");
    assert!(!click(|e| fixed.handle_event(e, B)).clicked);
}

// IconButton

#[test]
fn icon_button_projects_the_label_the_app_gives_and_activate() {
    let (copy, undo, pen) = (id("bar.copy"), id("bar.undo"), id("bar.pen"));
    let plain = IconButton::new("copy");
    let off = IconButton::new("undo").disabled(true);
    let tool = IconButton::new("pen")
        .variant(IconButtonVariant::Toolbar)
        .active(true);
    let s = snap(|b| {
        plain.project(b, &copy, "copy link", None);
        off.project(b, &undo, "undo", None);
        tool.project(b, &pen, "pen", None);
    });
    let node = s.get(&copy).unwrap();
    assert_eq!(
        (node.role, node.label.as_str()),
        (Role::Button, "copy link")
    );
    assert!(node.enabled && node.state.is_empty());
    assert_eq!(actions(&s, &copy), ["activate"]);
    assert!(reads(&s, &copy, "activate").is_empty());
    assert!(!s.get(&undo).unwrap().enabled);

    // A toolbar item carries the selected tool as state.
    let node = s.get(&pen).unwrap();
    assert_eq!(node.state.get("active"), Some(&Value::Bool(true)));
    assert_eq!(reads(&s, &pen, "activate"), ["bar.pen:active"]);
}

#[test]
fn icon_button_without_a_label_fails_the_projection_instead_of_inventing_one() {
    let mut b = SnapshotBuilder::new();
    IconButton::new("copy").project(&mut b, &id("copy"), "  ", None);
    assert!(b.finish().is_err(), "a button nobody can name is refused");
}

#[test]
fn icon_button_binds_the_bounds_it_hit_tests_with_on_its_layer() {
    let theme = hoff();
    let copy = id("copy");
    let mut button = IconButton::new("copy").sem(copy.clone());
    let mut c = Compositor::new();
    let layer = c.create_layer(20);
    c.begin_frame();
    button.render_to_layer(&mut c, layer, B, &theme);
    assert_bound(&c, &copy, B, layer);
    assert!(click(|e| button.handle_event(e, B)).clicked);
}

#[test]
fn toolbar_icon_button_goes_stale_when_the_tool_changes() {
    let pen = id("pen");
    let mut tool = IconButton::new("pen").variant(IconButtonVariant::Toolbar);
    let seen = snap(|b| {
        tool.project(b, &pen, "pen", None);
    });
    let req = Request::observe(&seen, test_actor(), &pen, "activate", None).unwrap();
    tool.active = true;
    let now = snap(|b| {
        tool.project(b, &pen, "pen", None);
    });
    assert_stale(&req, &now, Value::Bool(false), Value::Bool(true));
}

#[test]
fn icon_button_hover_press_and_focus_are_visual_only() {
    let pen = id("pen");
    let mut tool = IconButton::new("pen").variant(IconButtonVariant::Toolbar);
    let seen = snap(|b| {
        tool.project(b, &pen, "pen", None);
    });
    let req = Request::observe(&seen, test_actor(), &pen, "activate", None).unwrap();
    tool.handle_event(&move_to(20.0, 20.0), B);
    tool.handle_event(&down(20.0, 20.0), B);
    tool.set_focused(true);
    assert!(tool.is_hovered() && tool.is_pressed() && tool.is_focused());
    let now = snap(|b| {
        tool.project(b, &pen, "pen", None);
    });
    assert_visual_only(&req, &seen, &now);
}

// SubmitButton

#[test]
fn submit_button_projects_busy_and_an_activate_that_reads_it() {
    let send = id("chat.send");
    let idle = SubmitButton::new();
    let s = snap(|b| {
        idle.project(b, &send, "send", None);
    });
    let node = s.get(&send).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Button, "send"));
    assert!(node.enabled);
    assert_eq!(node.state.get("busy"), Some(&Value::Bool(false)));
    assert_eq!(actions(&s, &send), ["activate"]);
    assert_eq!(reads(&s, &send, "activate"), ["chat.send:busy"]);

    let off = SubmitButton::new().disabled(true);
    let s = snap(|b| {
        off.project(b, &send, "send", None);
    });
    let req = Request::observe(&s, test_actor(), &send, "activate", None).unwrap();
    assert!(matches!(
        check(&req, &s),
        Verdict::Rejected(Rejection::Disabled { .. })
    ));
}

#[test]
fn submit_button_binds_the_bounds_it_hit_tests_with() {
    let theme = hoff();
    let send = id("send");
    let mut button = SubmitButton::new().sem(send.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    button.render(&mut c, B, &theme);
    assert_bound(&c, &send, B, LayerId::DEFAULT);
    assert!(click(|e| button.handle_event(e, B)).clicked);
}

#[test]
fn submit_decided_on_an_idle_button_goes_stale_once_one_is_in_flight() {
    let send = id("send");
    let mut button = SubmitButton::new();
    let seen = snap(|b| {
        button.project(b, &send, "send", None);
    });
    let req = Request::observe(&seen, test_actor(), &send, "activate", None).unwrap();
    button.active = true;
    let now = snap(|b| {
        button.project(b, &send, "send", None);
    });
    assert_stale(&req, &now, Value::Bool(false), Value::Bool(true));
}

#[test]
fn submit_button_hover_and_press_are_visual_only() {
    let send = id("send");
    let mut button = SubmitButton::new();
    let seen = snap(|b| {
        button.project(b, &send, "send", None);
    });
    let req = Request::observe(&seen, test_actor(), &send, "activate", None).unwrap();
    button.handle_event(&move_to(20.0, 20.0), B);
    button.handle_event(&down(20.0, 20.0), B);
    assert!(button.is_hovered());
    let now = snap(|b| {
        button.project(b, &send, "send", None);
    });
    assert_visual_only(&req, &seen, &now);
}
