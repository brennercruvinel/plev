//! form widgets: checkbox, switch, slider, select.

use super::*;

// Checkbox

#[test]
fn checkbox_click_toggles_checked() {
    let mut cb = Checkbox::new(false);
    let r = click(|e| cb.handle_event(e, B));
    assert!(r.clicked);
    assert!(cb.checked);
    let r = click(|e| cb.handle_event(e, B));
    assert!(r.clicked);
    assert!(!cb.checked);
}

#[test]
fn checkbox_disabled_ignores_click() {
    let mut cb = Checkbox::new(false).disabled(true);
    let r = click(|e| cb.handle_event(e, B));
    assert!(!r.clicked);
    assert!(!cb.checked);
}

#[test]
fn checkbox_release_outside_does_not_toggle() {
    let mut cb = Checkbox::new(false);
    cb.handle_event(&down(20.0, 20.0), B);
    cb.handle_event(&up(500.0, 500.0), B);
    assert!(!cb.checked);
}

// Switch

#[test]
fn switch_click_toggles_and_animates_knob() {
    let theme = Theme::dark();
    let mut sw = Switch::new(false).with_motion(&theme.motion);
    assert_eq!(sw.knob_progress(), 0.0);

    let r = click(|e| sw.handle_event(e, B));
    assert!(r.clicked);
    assert!(sw.on);
    assert!(sw.is_animating());

    sw.tick(1.0 / 60.0);
    let mid = sw.knob_progress();
    assert!(mid > 0.0 && mid < 1.0, "knob mid-flight, got {mid}");

    for _ in 0..300 {
        sw.tick(1.0 / 60.0);
    }
    assert!(!sw.is_animating());
    assert!((sw.knob_progress() - 1.0).abs() < 1e-3);
}

#[test]
fn switch_disabled_ignores_click() {
    let mut sw = Switch::new(false).disabled(true);
    let r = click(|e| sw.handle_event(e, B));
    assert!(!r.clicked);
    assert!(!sw.on);
    assert!(!sw.is_animating());
}

// Slider

#[test]
fn slider_drag_updates_value() {
    let mut s = Slider::new(0.0, 100.0, 0.0);
    let bounds = Rect::new(0.0, 0.0, 200.0, 20.0);
    s.handle_event(&down(100.0, 10.0), bounds, &hoff());
    assert!(s.is_dragging());
    assert!(s.value() > 25.0 && s.value() < 75.0, "value={}", s.value());

    s.handle_event(&move_to(1000.0, 10.0), bounds, &hoff());
    assert_eq!(s.value(), 100.0, "drag clamps at max");
    s.handle_event(&up(1000.0, 10.0), bounds, &hoff());
    assert!(!s.is_dragging());
}

#[test]
fn slider_step_snaps_value() {
    let mut s = Slider::new(0.0, 10.0, 0.0).step(2.0);
    s.set_value(3.2);
    assert_eq!(s.value(), 4.0);
}

#[test]
fn slider_disabled_ignores_drag() {
    let mut s = Slider::new(0.0, 100.0, 50.0).disabled(true);
    let bounds = Rect::new(0.0, 0.0, 200.0, 20.0);
    s.handle_event(&down(180.0, 10.0), bounds, &hoff());
    assert_eq!(s.value(), 50.0);
}

// Select

#[test]
fn select_opens_and_picks_option() {
    let mut s = Select::new(["dark", "light", "dracula"], 0);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);

    s.handle_event(&down(50.0, 20.0), bounds, &hoff());
    assert!(s.is_open());

    // Option 1 ("light"): dropdown starts at bounds bottom + gap;
    // PAD_Y(8) + OPTION_H(44) + 22 centers the second option.
    let dd = s.dropdown_rect(bounds, &hoff());
    let y = dd.y + 8.0 + 44.0 + 22.0;
    let r = s.handle_event(&down(50.0, y), bounds, &hoff());
    assert!(r.clicked);
    assert!(!s.is_open());
    assert_eq!(s.selected, 1);
    assert_eq!(s.selected_label(), Some("light"));
}

#[test]
fn select_click_outside_closes_without_change() {
    let mut s = Select::new(["a", "b"], 0);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);
    s.handle_event(&down(50.0, 20.0), bounds, &hoff());
    assert!(s.is_open());
    let r = s.handle_event(&down(700.0, 500.0), bounds, &hoff());
    assert!(!s.is_open());
    assert!(!r.clicked);
    assert_eq!(s.selected, 0);
}

#[test]
fn select_disabled_never_opens() {
    let mut s = Select::new(["a"], 0).disabled(true);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);
    s.handle_event(&down(50.0, 20.0), bounds, &hoff());
    assert!(!s.is_open());
}

fn twenty() -> Select {
    Select::new((0..20).map(|i| format!("option {i}")), 0)
}

#[test]
fn select_wheel_belongs_to_the_open_panel_only_where_the_panel_is() {
    let theme = hoff();
    let mut s = twenty();
    let (x, y) = B.center();
    assert_eq!(
        s.handle_event(&wheel(x, y, 40.0), B, &theme),
        EventResult::IGNORED,
        "a closed select does not scroll"
    );
    s.open();
    assert_eq!(
        s.handle_event(&wheel(700.0, 500.0, 40.0), B, &theme),
        EventResult::IGNORED,
        "outside the panel the wheel is the page's"
    );
    let (px, py) = s.dropdown_rect(B, &theme).center();
    assert!(s.handle_event(&wheel(px, py, 40.0), B, &theme).changed);
    // Past the end nothing moves, and the wheel still stops at the panel.
    assert!(s.handle_event(&wheel(px, py, 10_000.0), B, &theme).changed);
    let end = s.handle_event(&wheel(px, py, 40.0), B, &theme);
    assert!(end.handled && !end.changed);

    // A panel whose options all fit does not scroll (the pointer rests
    // there first, so the highlight under it is already set).
    let mut short = Select::new(["a", "b", "c"], 0);
    short.open();
    let (px, py) = short.dropdown_rect(B, &theme).center();
    short.handle_event(&move_to(px, py), B, &theme);
    let r = short.handle_event(&wheel(px, py, 40.0), B, &theme);
    assert!(r.handled && !r.changed);
}

#[test]
fn select_highlight_follows_the_row_under_a_still_pointer_as_the_wheel_scrolls() {
    let theme = hoff();
    let mut s = twenty();
    s.open();
    let panel = s.dropdown_rect(B, &theme);
    let row_h = theme.control.height(engine::theme::ControlSize::Md);
    let (x, y) = (
        panel.x + panel.w / 2.0,
        panel.y + theme.control.menu_pad + row_h / 2.0,
    );
    assert!(s.handle_event(&move_to(x, y), B, &theme).changed);
    assert_eq!(s.highlighted(), Some(0));
    assert!(s.handle_event(&wheel(x, y, row_h), B, &theme).changed);
    assert_eq!(s.highlighted(), Some(1), "the rows moved under the pointer");
}

#[test]
fn select_with_no_options_opens_an_empty_panel_the_keys_leave_alone() {
    let theme = hoff();
    let mut s = Select::new(Vec::<String>::new(), 0);
    s.open();
    assert_eq!(s.highlight_next(&theme), EventResult::IGNORED);
    assert_eq!(s.highlight_prev(&theme), EventResult::IGNORED);
    let mut c = Compositor::new();
    c.begin_frame();
    s.render_dropdown(&mut c, LayerId::DEFAULT, B, &theme);
    let texts = c
        .layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter(|n| matches!(n, SceneNode::Text { .. }))
        .count();
    assert_eq!(texts, 0, "no option row is drawn");
    let (px, py) = s.dropdown_rect(B, &theme).center();
    assert!(!s.handle_event(&down(px, py), B, &theme).clicked);
    assert!(!s.is_open(), "a click in the empty panel only closes it");
}
