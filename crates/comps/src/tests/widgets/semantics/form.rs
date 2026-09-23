//! switch, slider and select.

use super::*;
use sem::Constraints;

// Switch

#[test]
fn switch_projects_a_toggle_with_on_and_a_toggle_that_reads_it() {
    let wifi = id("settings.wifi");
    let on = Switch::new(true);
    let off = Switch::new(false).disabled(true);
    let s = snap(|b| {
        on.project(b, &wifi, "wifi", None);
        off.project(b, &id("settings.bt"), "bluetooth", None);
    });
    let node = s.get(&wifi).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Toggle, "wifi"));
    assert!(node.enabled);
    assert_eq!(node.state.get("on"), Some(&Value::Bool(true)));
    assert_eq!(actions(&s, &wifi), ["toggle"]);
    assert_eq!(reads(&s, &wifi, "toggle"), ["settings.wifi:on"]);
    assert!(!s.get(&id("settings.bt")).unwrap().enabled);
}

#[test]
fn switch_binds_the_bounds_it_hit_tests_with() {
    let theme = hoff();
    let wifi = id("wifi");
    let mut sw = Switch::new(false).sem(wifi.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    sw.render(&mut c, B, &theme);
    assert_bound(&c, &wifi, B, LayerId::DEFAULT);
    assert!(click(|e| sw.handle_event(e, B)).clicked);
    assert!(sw.on);
}

#[test]
fn switch_toggle_goes_stale_when_a_click_flips_it() {
    let wifi = id("wifi");
    let mut sw = Switch::new(false);
    let seen = snap(|b| {
        sw.project(b, &wifi, "wifi", None);
    });
    let req = Request::observe(&seen, test_actor(), &wifi, "toggle", None).unwrap();
    click(|e| sw.handle_event(e, B));
    let now = snap(|b| {
        sw.project(b, &wifi, "wifi", None);
    });
    assert_stale(&req, &now, Value::Bool(false), Value::Bool(true));
}

#[test]
fn switch_hover_press_focus_and_the_knob_animation_are_visual_only() {
    let wifi = id("wifi");
    let mut sw = Switch::new(true);
    let seen = snap(|b| {
        sw.project(b, &wifi, "wifi", None);
    });
    let req = Request::observe(&seen, test_actor(), &wifi, "toggle", None).unwrap();
    sw.handle_event(&move_to(20.0, 20.0), B);
    sw.handle_event(&down(20.0, 20.0), B);
    sw.set_focused(true);
    sw.tick(1.0 / 60.0);
    assert!(sw.is_hovered() && sw.is_focused());
    let now = snap(|b| {
        sw.project(b, &wifi, "wifi", None);
    });
    assert_visual_only(&req, &seen, &now);
}

// Slider

#[test]
fn slider_projects_its_value_range_step_and_the_actions_that_move_it() {
    let volume = id("audio.volume");
    let stepped = Slider::new(0.0, 100.0, 40.0).step(10.0);
    let s = snap(|b| {
        stepped.project(b, &volume, "volume", None);
    });
    let node = s.get(&volume).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Slider, "volume"));
    assert_eq!(node.state.get("value"), Some(&Value::Num(40.0)));
    assert_eq!(
        node.constraints,
        Constraints {
            editable: true,
            range: Some((0.0, 100.0)),
            step: Some(10.0),
            ..Constraints::default()
        }
    );
    assert_eq!(
        actions(&s, &volume),
        ["set_value", "increment", "decrement"]
    );
    for action in ["set_value", "increment", "decrement"] {
        assert_eq!(reads(&s, &volume, action), ["audio.volume:value"]);
    }
    assert_eq!(
        node.actions[0].param,
        Some(ParamSpec {
            kind: ValueKind::Num,
            constrained: true
        })
    );

    let set = |v: f64| Request::observe(&s, test_actor(), &volume, "set_value", Some(v.into()));
    assert_eq!(check(&set(70.0).unwrap(), &s), Verdict::Accepted);
    assert!(matches!(
        check(&set(140.0).unwrap(), &s),
        Verdict::Rejected(Rejection::InvalidParam { .. })
    ));

    // A continuous slider has no step to move by.
    let smooth = Slider::new(0.0, 1.0, 0.5);
    let s = snap(|b| {
        smooth.project(b, &volume, "volume", None);
    });
    assert_eq!(actions(&s, &volume), ["set_value"]);
    assert_eq!(s.get(&volume).unwrap().constraints.step, None);
}

#[test]
fn slider_increment_and_decrement_move_one_step_and_stop_at_the_ends() {
    let mut s = Slider::new(0.0, 100.0, 90.0).step(10.0);
    assert!(s.increment());
    assert_eq!(s.value(), 100.0);
    assert!(!s.increment(), "clamped at max");
    assert!(s.decrement());
    assert_eq!(s.value(), 90.0);
    let mut smooth = Slider::new(0.0, 1.0, 0.5);
    assert!(!smooth.increment() && !smooth.decrement());
    assert_eq!(smooth.value(), 0.5);
}

#[test]
fn slider_binds_the_bounds_it_hit_tests_with() {
    let theme = hoff();
    let volume = id("volume");
    let mut slider = Slider::new(0.0, 100.0, 0.0).sem(volume.clone());
    let mut c = Compositor::new();
    c.begin_frame();
    slider.render(&mut c, B, &theme);
    assert_bound(&c, &volume, B, LayerId::DEFAULT);
    let (x, y) = B.center();
    assert!(slider.handle_event(&down(x, y), B, &theme).changed);
    assert!(slider.is_dragging(), "a press at the bound center grabs it");
}

#[test]
fn slider_request_goes_stale_when_a_drag_moves_the_value() {
    let theme = hoff();
    let volume = id("volume");
    let mut slider = Slider::new(0.0, 100.0, 0.0).step(1.0);
    let seen = snap(|b| {
        slider.project(b, &volume, "volume", None);
    });
    let req = Request::observe(&seen, test_actor(), &volume, "increment", None).unwrap();
    slider.handle_event(&down(B.x + B.w, B.y + 5.0), B, &theme);
    slider.handle_event(&up(B.x + B.w, B.y + 5.0), B, &theme);
    assert_eq!(slider.value(), 100.0);
    let now = snap(|b| {
        slider.project(b, &volume, "volume", None);
    });
    assert_stale(&req, &now, Value::Num(0.0), Value::Num(100.0));
}

#[test]
fn slider_hover_and_focus_are_visual_only() {
    let theme = hoff();
    let volume = id("volume");
    let mut slider = Slider::new(0.0, 100.0, 40.0).step(10.0);
    let seen = snap(|b| {
        slider.project(b, &volume, "volume", None);
    });
    let req =
        Request::observe(&seen, test_actor(), &volume, "set_value", Some(50.0.into())).unwrap();
    slider.handle_event(&move_to(20.0, 20.0), B, &theme);
    slider.set_focused(true);
    assert!(slider.is_hovered() && slider.is_focused());
    let now = snap(|b| {
        slider.project(b, &volume, "volume", None);
    });
    assert_visual_only(&req, &seen, &now);
}

// Select

#[test]
fn select_projects_the_selected_label_the_options_and_its_actions() {
    let size = id("form.size");
    let select = Select::new(["small", "medium", "large", "medium"], 1);
    let s = snap(|b| {
        select.project(b, &size, "size", None);
    });
    let node = s.get(&size).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Select, "size"));
    assert_eq!(node.state.get("selected"), Some(&Value::from("medium")));
    assert_eq!(node.state.get("expanded"), Some(&Value::Bool(false)));
    assert!(node.constraints.editable);
    // The accepted choices, each once.
    assert_eq!(
        node.constraints.options.as_deref(),
        Some(&["small".to_owned(), "medium".to_owned(), "large".to_owned()][..])
    );
    assert_eq!(actions(&s, &size), ["select", "open", "close"]);
    assert_eq!(reads(&s, &size, "select"), ["form.size:selected"]);
    assert_eq!(reads(&s, &size, "open"), ["form.size:expanded"]);
    assert_eq!(s.children(&size).count(), 0, "closed: no option rows");

    let pick = |v: &str| Request::observe(&s, test_actor(), &size, "select", Some(v.into()));
    assert_eq!(check(&pick("large").unwrap(), &s), Verdict::Accepted);
    assert!(matches!(
        check(&pick("huge").unwrap(), &s),
        Verdict::Rejected(Rejection::InvalidParam { .. })
    ));
}

#[test]
fn an_open_select_projects_one_option_per_row() {
    let theme = hoff();
    let size = id("size");
    let mut select = Select::new(["small", "large"], 0);
    select.handle_event(&down(20.0, 20.0), B, &theme);
    assert!(select.is_open());
    let s = snap(|b| {
        select.project(b, &size, "size", None);
    });
    assert_eq!(
        s.get(&size).unwrap().state.get("expanded"),
        Some(&Value::Bool(true))
    );
    let rows: Vec<(&str, Role, &str)> = s
        .children(&size)
        .map(|n| (n.id.as_str(), n.role, n.label.as_str()))
        .collect();
    assert_eq!(
        rows,
        [
            ("size[0]", Role::Option, "small"),
            ("size[1]", Role::Option, "large")
        ]
    );
    assert_eq!(reads(&s, &id("size[1]"), "select"), ["size:selected"]);
}

#[test]
fn select_label_select_index_and_open_do_what_a_click_does() {
    let mut select = Select::new(["a", "b", "c"], 0);
    select.open();
    assert!(select.is_open());
    assert!(select.select_label("c"));
    assert_eq!((select.selected, select.is_open()), (2, false));
    assert!(!select.select_label("z"));
    assert!(!select.select_index(3));
    assert!(select.select_index(1));
    assert_eq!(select.selected_label(), Some("b"));
    let mut off = Select::new(["a"], 0).disabled(true);
    off.open();
    assert!(!off.is_open(), "a disabled select stays closed");
}

#[test]
fn select_binds_the_control_and_the_option_rows_a_click_picks_from() {
    let theme = hoff();
    let size = id("size");
    let mut select = Select::new(["small", "medium", "large"], 0).sem(size.clone());
    let mut c = Compositor::new();
    let overlay = c.create_layer(100);
    c.begin_frame();
    select.render(&mut c, B, &theme);
    assert_bound(&c, &size, B, LayerId::DEFAULT);
    assert_eq!(c.bindings().count(), 1, "closed: only the control");

    select.handle_event(&down(20.0, 20.0), B, &theme);
    c.begin_frame();
    select.render(&mut c, B, &theme);
    select.render_dropdown(&mut c, overlay, B, &theme);
    let panel = select.dropdown_rect(B, &theme);
    let row_h = theme.control.height(engine::theme::ControlSize::Md);
    let row = Rect::new(
        panel.x,
        panel.y + theme.control.menu_pad + row_h,
        panel.w,
        row_h,
    );
    assert_bound(&c, &size.key("1"), row, overlay);
    // A click at that center picks that option.
    let (x, y) = row.center();
    assert!(select.handle_event(&down(x, y), B, &theme).clicked);
    assert_eq!(select.selected_label(), Some("medium"));
}

#[test]
fn select_request_goes_stale_when_a_click_picks_another_option() {
    let theme = hoff();
    let size = id("size");
    let mut select = Select::new(["small", "medium", "large"], 0);
    let seen = snap(|b| {
        select.project(b, &size, "size", None);
    });
    let req = Request::observe(&seen, test_actor(), &size, "select", Some("large".into())).unwrap();

    select.handle_event(&down(20.0, 20.0), B, &theme);
    let panel = select.dropdown_rect(B, &theme);
    let row_h = theme.control.height(engine::theme::ControlSize::Md);
    let y = panel.y + theme.control.menu_pad + row_h * 1.5;
    select.handle_event(&down(panel.x + 5.0, y), B, &theme);
    assert_eq!(select.selected_label(), Some("medium"));
    let now = snap(|b| {
        select.project(b, &size, "size", None);
    });
    assert_stale(&req, &now, Value::from("small"), Value::from("medium"));
}

#[test]
fn select_hover_and_focus_are_visual_only() {
    let theme = hoff();
    let size = id("size");
    let mut select = Select::new(["small", "large"], 0);
    select.open();
    let seen = snap(|b| {
        select.project(b, &size, "size", None);
    });
    let req = Request::observe(&seen, test_actor(), &size, "close", None).unwrap();
    // Hover the control, then an option row.
    select.handle_event(&move_to(20.0, 20.0), B, &theme);
    let panel = select.dropdown_rect(B, &theme);
    select.handle_event(&move_to(panel.x + 5.0, panel.y + 20.0), B, &theme);
    select.set_focused(true);
    let now = snap(|b| {
        select.project(b, &size, "size", None);
    });
    assert_visual_only(&req, &seen, &now);
}

/// Regression: with more options than `dropdown_max_h` fits, the first
/// option the panel does not draw kept a hit band at the foot of the
/// panel (4px in hoff), so a click there picked an option nobody could
/// see. A click (the widget's `option_at`) and `hit` now agree at every
/// drawn row and in that band, where both find nothing.
#[test]
fn a_click_at_the_foot_of_a_full_select_panel_picks_no_hidden_option() {
    let theme = hoff();
    let size = id("size");
    // Nine options, the last selected. Opening scrolls it into view, so
    // the panel is scrolled back to the top: the last option is not drawn
    // there, and any option a click picks changes the selection and shows
    // up.
    let options: Vec<String> = (0..9).map(|i| format!("option {i}")).collect();
    let mut open = Select::new(options, 8).sem(size.clone());
    open.open();
    let (x, y) = open.dropdown_rect(B, &theme).center();
    assert!(open.handle_event(&wheel(x, y, -1000.0), B, &theme).changed);
    let mut c = Compositor::new();
    let overlay = c.create_layer(100);
    c.begin_frame();
    open.render(&mut c, B, &theme);
    open.render_dropdown(&mut c, overlay, B, &theme);

    let panel = open.dropdown_rect(B, &theme);
    let pad = theme.control.menu_pad;
    let row_h = theme.control.height(engine::theme::ControlSize::Md);
    let drawn = ((panel.h - pad) / row_h).floor() as usize;
    assert_eq!(drawn, 7, "hoff draws seven rows under dropdown_max_h");
    let foot = panel.y + pad + drawn as f32 * row_h;
    let bottom = panel.y + panel.h;
    assert!(bottom - foot > 0.0, "a band is left at the foot");

    // The option a click at (x, y) picks, if any.
    let pick = |x: f32, y: f32| {
        let mut s = open.clone();
        let r = s.handle_event(&down(x, y), B, &theme);
        r.clicked.then_some(s.selected)
    };
    let x = panel.x + panel.w / 2.0;
    for i in 0..drawn {
        let y = panel.y + pad + (i as f32 + 0.5) * row_h;
        assert_eq!(c.hit(x, y), Some(&size.key(i.to_string())), "row {i}");
        assert_eq!(pick(x, y), Some(i), "row {i}");
    }
    let band = (foot + bottom) / 2.0;
    assert_eq!(c.hit(x, band), None, "nothing is drawn in the band");
    assert_eq!(pick(x, band), None, "and a click there picks nothing");
    assert_eq!(c.binding(&size.key(drawn.to_string())), None);
}

// Select: the scrolling panel (task-81)

/// Twenty options, far more than `dropdown_max_h` fits in any theme.
fn long_select(selected: usize) -> Select {
    Select::new((0..20).map(|i| format!("option {i}")), selected)
}

/// One frame of `select`: the control on the default layer, the open
/// panel on an overlay.
fn panel_frame(select: &Select, theme: &Theme) -> (Compositor, LayerId) {
    let mut c = Compositor::new();
    let overlay = c.create_layer(100);
    c.begin_frame();
    select.render(&mut c, B, theme);
    select.render_dropdown(&mut c, overlay, B, theme);
    (c, overlay)
}

/// The strip of the panel the rows scroll in: inside the menu padding.
fn strip(select: &Select, theme: &Theme) -> Rect {
    let panel = select.dropdown_rect(B, theme);
    let pad = theme.control.menu_pad;
    Rect::new(panel.x, panel.y + pad, panel.w, panel.h - pad * 2.0)
}

fn row_h(theme: &Theme) -> f32 {
    theme.control.height(engine::theme::ControlSize::Md)
}

/// `who` is drawn whole inside `strip`, and `hit` at its center answers
/// it. Returns the center.
#[track_caller]
fn assert_shows_whole(c: &Compositor, who: &SemId, strip: Rect) -> (f32, f32) {
    let [x, y, w, h] = c
        .binding(who)
        .unwrap_or_else(|| panic!("{who} is not bound"))
        .rect;
    assert!(
        y >= strip.y && y + h <= strip.y + strip.h,
        "{who} at {:?} is cut by the strip {strip:?}",
        [x, y, w, h]
    );
    let center = (x + w / 2.0, y + h / 2.0);
    assert_eq!(c.hit(center.0, center.1), Some(who));
    center
}

#[test]
fn a_long_select_scrolls_to_its_last_option_and_a_click_there_picks_it() {
    let theme = hoff();
    let size = id("size");
    let mut select = long_select(0).sem(size.clone());
    let (x, y) = B.center();
    assert!(select.handle_event(&down(x, y), B, &theme).changed);
    let (c, _) = panel_frame(&select, &theme);
    assert_eq!(
        c.binding(&size.key("19")),
        None,
        "out of the window at first"
    );

    // The wheel over the panel, far past the end: the rows stop at the last.
    let (px, py) = select.dropdown_rect(B, &theme).center();
    assert!(
        select
            .handle_event(&wheel(px, py, 10_000.0), B, &theme)
            .changed
    );
    let (c, overlay) = panel_frame(&select, &theme);
    let s = strip(&select, &theme);
    let last = Rect::new(s.x, s.y + s.h - row_h(&theme), s.w, row_h(&theme));
    assert_bound(&c, &size.key("19"), last, overlay);
    assert_eq!(
        c.binding(&size.key("0")),
        None,
        "the first row scrolled out"
    );

    let (x, y) = last.center();
    assert!(select.handle_event(&down(x, y), B, &theme).clicked);
    assert_eq!(select.selected_label(), Some("option 19"));
    assert!(!select.is_open());
}

/// After a scroll that cuts a row at the top of the strip and another at
/// its foot, `hit` and a click agree at every pixel line of the panel, on
/// its left edge, its middle and its right edge: the same row, or nothing
/// in the padding above and in the band at the foot.
#[test]
fn hit_and_a_click_agree_at_every_point_of_a_scrolled_select_panel() {
    let theme = hoff();
    let size = id("size");
    let mut select = long_select(0).sem(size.clone());
    select.open();
    let panel = select.dropdown_rect(B, &theme);
    let (px, py) = panel.center();
    let scroll = wheel(px, py, row_h(&theme) * 3.5);
    assert!(select.handle_event(&scroll, B, &theme).changed);
    let (c, _) = panel_frame(&select, &theme);

    // Rows 3 (cut at the top) to 10 (cut at the foot) are drawn and bound.
    let bound: Vec<&str> = c
        .bindings()
        .map(|(who, _)| who.as_str())
        .filter(|who| *who != "size")
        .collect();
    let window: Vec<String> = (3..11).map(|i| format!("size[{i}]")).collect();
    assert_eq!(bound, window);

    // What a click at (x, y) picks. Option 0 is selected and out of the
    // window, so every pick changes the selection.
    let pick = |x: f32, y: f32| {
        let mut s = select.clone();
        let r = s.handle_event(&down(x, y), B, &theme);
        r.clicked.then(|| size.key(s.selected.to_string()))
    };
    let mut reached = std::collections::BTreeSet::new();
    for x in [panel.x, panel.x + panel.w / 2.0, panel.x + panel.w] {
        let mut y = panel.y;
        while y <= panel.y + panel.h {
            let hit = c.hit(x, y).cloned();
            assert_eq!(hit, pick(x, y), "at ({x}, {y})");
            reached.extend(hit);
            y += 1.0;
        }
    }
    assert_eq!(reached.len(), window.len(), "every drawn row is reached");
    let pad = theme.control.menu_pad;
    let x = panel.x + panel.w / 2.0;
    for y in [panel.y + pad / 2.0, panel.y + panel.h - pad / 2.0] {
        assert_eq!((c.hit(x, y), pick(x, y)), (None, None), "the padding");
    }
}

#[test]
fn the_arrow_keys_move_the_highlight_past_the_window_and_scroll_it_into_view() {
    let theme = hoff();
    let size = id("size");
    let mut select = long_select(0).sem(size.clone());
    assert_eq!(select.highlight_next(&theme), EventResult::IGNORED);
    assert_eq!(select.pick_highlighted(), EventResult::IGNORED);
    select.open();
    let s = strip(&select, &theme);

    for i in 1..20 {
        assert!(select.highlight_next(&theme).changed, "down to {i}");
        assert_eq!(select.highlighted(), Some(i));
        let (c, _) = panel_frame(&select, &theme);
        assert_shows_whole(&c, &size.key(i.to_string()), s);
    }
    let (c, _) = panel_frame(&select, &theme);
    assert_eq!(c.binding(&size.key("0")), None, "the window moved down");
    let end = select.highlight_next(&theme);
    assert!(end.handled && !end.changed, "the last option stays");

    for i in (0..19).rev() {
        assert!(select.highlight_prev(&theme).changed, "up to {i}");
        assert_eq!(select.highlighted(), Some(i));
        let (c, _) = panel_frame(&select, &theme);
        assert_shows_whole(&c, &size.key(i.to_string()), s);
    }
    let (c, _) = panel_frame(&select, &theme);
    assert_eq!(c.binding(&size.key("19")), None, "the window moved up");
    assert!(
        !select.highlight_prev(&theme).changed,
        "the first option stays"
    );

    // Enter picks the highlighted option, like a click on its row.
    select.highlight_next(&theme);
    assert!(select.pick_highlighted().clicked);
    assert_eq!((select.selected, select.is_open()), (1, false));
    // Nothing highlighted: Enter just closes.
    select.open();
    let r = select.pick_highlighted();
    assert!(r.changed && !r.clicked);
    assert_eq!((select.selected, select.is_open()), (1, false));
}

#[test]
fn opening_a_long_select_scrolls_the_selected_option_into_view() {
    let theme = hoff();
    let size = id("size");
    // The 18th option, opened by a click on the control and by the action.
    let mut clicked = long_select(17).sem(size.clone());
    let (x, y) = B.center();
    clicked.handle_event(&down(x, y), B, &theme);
    let mut opened = long_select(17).sem(size.clone());
    opened.open();
    for select in [clicked, opened] {
        assert!(select.is_open());
        let (c, _) = panel_frame(&select, &theme);
        let (x, y) = assert_shows_whole(&c, &size.key("17"), strip(&select, &theme));
        // A click there keeps it, and closes the panel.
        let mut s = select.clone();
        let r = s.handle_event(&down(x, y), B, &theme);
        assert!(r.changed && !r.clicked);
        assert_eq!((s.selected, s.is_open()), (17, false));
    }

    // One in the middle opens as the first row of the window.
    let mut mid = long_select(8).sem(size.clone());
    mid.open();
    let (c, _) = panel_frame(&mid, &theme);
    let s = strip(&mid, &theme);
    assert_shows_whole(&c, &size.key("8"), s);
    assert_eq!(c.binding(&size.key("8")).unwrap().rect[1], s.y);
    assert_eq!(c.binding(&size.key("7")), None);
}

#[test]
fn an_open_long_select_projects_every_option_and_binds_only_the_rows_it_draws() {
    let theme = hoff();
    let size = id("size");
    let mut select = long_select(0).sem(size.clone());
    select.open();
    let s = snap(|b| {
        select.project(b, &size, "size", None);
    });
    assert_eq!(s.children(&size).count(), 20, "every option is a child");
    let (c, _) = panel_frame(&select, &theme);
    let bound: Vec<&str> = c
        .bindings()
        .map(|(who, _)| who.as_str())
        .filter(|who| *who != "size")
        .collect();
    let window: Vec<String> = (0..7).map(|i| format!("size[{i}]")).collect();
    assert_eq!(bound, window, "only the rows the panel draws");
}

#[test]
fn select_scroll_and_the_keyboard_highlight_are_visual_only() {
    let theme = hoff();
    let size = id("size");
    let mut select = long_select(0);
    select.open();
    let seen = snap(|b| {
        select.project(b, &size, "size", None);
    });
    let req = Request::observe(&seen, test_actor(), &size, "close", None).unwrap();
    let (x, y) = select.dropdown_rect(B, &theme).center();
    assert!(select.handle_event(&wheel(x, y, 200.0), B, &theme).changed);
    assert!(select.highlight_next(&theme).changed);
    let now = snap(|b| {
        select.project(b, &size, "size", None);
    });
    assert_visual_only(&req, &seen, &now);
}
