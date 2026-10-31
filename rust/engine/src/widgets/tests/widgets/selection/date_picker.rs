//! Selection tests: the date picker.

use super::*;

fn picker() -> DatePicker {
    let day = Date::new(2026, 10, 14).unwrap();
    DatePicker::new(
        "Due date",
        "Pick a day",
        Calendar::new(2026, 10).selected(day),
    )
    .sem(id("page.picker"))
}

/// A press on the box opens the panel below it; a day pressed in the
/// panel fills the box and closes it; a press outside closes it; a
/// disabled picker never opens.
#[test]
fn date_picker_opens_on_the_box_and_a_day_fills_it() {
    let theme = Theme::hoff_core();
    let mut p = picker();
    assert_eq!(p.field.input.text(), "14 October 2026");
    let container = Rect::new(0.0, 0.0, 360.0, 600.0);
    let bounds = Rect::new(0.0, 0.0, 360.0, p.field_height(360.0, &theme));
    let press = |p: &mut DatePicker, (x, y): (f32, f32)| {
        let a = p.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, container, &theme);
        let b = p.handle_event(&WidgetEvent::MouseUp { x, y }, bounds, container, &theme);
        a.merge(b)
    };
    let l = p.layout(bounds, container, &theme);
    assert!(press(&mut p, l.field.input.center()).clicked);
    assert!(p.is_open());
    let l = p.layout(bounds, container, &theme);
    assert!(l.panel.y >= l.field.input.bottom(), "below the box");
    assert!(l.panel.right() <= container.right() + 0.5);
    let grid = p.calendar.grid();
    let at = grid
        .iter()
        .position(|d| *d == Date::new(2026, 10, 20).unwrap())
        .unwrap();
    let cell = p.calendar.layout(l.calendar, &theme).days[at];
    assert!(press(&mut p, cell.center()).clicked);
    assert_eq!(p.chosen(), Date::new(2026, 10, 20));
    assert_eq!(p.field.input.text(), "20 October 2026");
    assert!(!p.is_open(), "a day closes it");

    press(&mut p, l.field.input.center());
    assert!(p.is_open());
    press(&mut p, (container.right() - 1.0, container.bottom() - 1.0));
    assert!(!p.is_open(), "a press outside closes it");

    let mut off = picker().disabled(true);
    press(&mut off, l.field.input.center());
    assert!(!off.is_open() && !off.toggle() && !off.open());
}

/// The box projects as a button named by the label, described by the
/// day, with its `expanded`; open, the panel holds the calendar and its
/// days.
#[test]
fn date_picker_projects_its_box_and_while_open_its_calendar() {
    let mut p = picker();
    let root = id("page");
    let project = |p: &DatePicker| {
        snap(|b| {
            b.node(&root, Role::Group, "page");
            p.project(b, &id("page.picker"), Some(&root));
        })
    };
    let s = project(&p);
    let node = s.get(&id("page.picker")).unwrap();
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.state.get("expanded"), Some(&Value::Bool(false)));
    assert_eq!(
        node.state.get("value"),
        Some(&Value::Text("2026-10-14".into()))
    );
    assert!(s.get(&id("page.picker.panel")).is_none());
    assert!(p.open());
    let s = project(&p);
    assert!(s.get(&id("page.picker.panel")).is_some());
    let day = s.get(&id("page.picker.calendar[2026-10-14]")).unwrap();
    assert_eq!(day.state.get("checked"), Some(&Value::Bool(true)));
}

/// From 280 to 1600 every 8 px under the core and the hoff themes, the
/// open panel stays inside its container, below the box, and its
/// calendar inside it.
#[test]
fn date_picker_panel_fits_its_container_from_narrow_to_wide() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let mut p = picker();
        p.open();
        for w in (280..=1600).step_by(8) {
            let w = w as f32;
            let field_h = p.field_height(w, &theme);
            let (_, panel_h) = p.panel_size(w, &theme);
            let container = Rect::new(0.0, 0.0, w, field_h * 2.0 + panel_h);
            let bounds = Rect::new(0.0, 0.0, w, field_h);
            let l = p.layout(bounds, container, &theme);
            assert!(l.panel.x >= -0.5 && l.panel.right() <= w + 0.5, "{w}");
            assert!(l.panel.y >= l.field.input.bottom(), "{w}: below");
            assert!(l.panel.bottom() <= container.bottom() + 0.5, "{w}");
            assert!(l.calendar.x >= l.panel.x && l.calendar.right() <= l.panel.right() + 0.5);
        }
    }
}
