//! Selection tests: the calendar and its dates.

use super::*;

fn d(y: i32, m: u32, day: u32) -> Date {
    Date::new(y, m, day).unwrap()
}

#[test]
fn dates_round_trip_and_know_their_weekday() {
    assert_eq!(d(1970, 1, 1).to_days(), 0);
    assert_eq!(d(1970, 1, 1).weekday(), 3, "a Thursday");
    assert_eq!(d(2026, 10, 31).weekday(), 5, "a Saturday");
    assert_eq!(d(2000, 2, 29).add_days(1), d(2000, 3, 1));
    assert_eq!(d(2026, 12, 31).add_days(1), d(2027, 1, 1));
    assert_eq!(d(1600, 3, 1).add_days(-1), d(1600, 2, 29));
    for days in [-800_000_i64, -1, 0, 1, 20_000, 2_000_000] {
        assert_eq!(Date::from_days(days).to_days(), days);
    }
    assert!(Date::new(2025, 2, 29).is_none() && Date::new(2026, 13, 1).is_none());
    assert_eq!(Date::parse("2026-10-31"), Some(d(2026, 10, 31)));
    assert_eq!(Date::parse("2026-02-30"), None);
    assert_eq!(d(2026, 1, 5).key(), "2026-01-05");
}

/// The grid starts on the Monday on or before the 1st and holds the
/// whole month in six weeks.
#[test]
fn calendar_grid_holds_the_month_from_monday() {
    let cal = Calendar::new(2026, 10);
    let grid = cal.grid();
    assert_eq!(grid.len(), 42);
    assert_eq!(grid[0], d(2026, 9, 28));
    assert_eq!(grid[0].weekday(), 0);
    assert!(grid.contains(&d(2026, 10, 1)) && grid.contains(&d(2026, 10, 31)));
    assert_eq!(cal.title(), "October 2026");
}

#[test]
fn calendar_click_chooses_a_day_and_the_arrows_turn_the_month() {
    let theme = Theme::hoff_core();
    let mut cal = Calendar::new(2026, 10).today(d(2026, 10, 31));
    let (w, h) = cal.size_within(320.0, &theme);
    let bounds = Rect::new(0.0, 0.0, w, h);
    let l = cal.layout(bounds, &theme);
    let at = cal
        .grid()
        .iter()
        .position(|x| *x == d(2026, 10, 15))
        .unwrap();
    let (x, y) = l.days[at].center();
    cal.handle_event(&down(x, y), bounds, &theme);
    assert!(cal.handle_event(&up(x, y), bounds, &theme).clicked);
    assert_eq!(cal.selected, Some(d(2026, 10, 15)));
    let (x, y) = l.next.center();
    cal.handle_event(&down(x, y), bounds, &theme);
    cal.handle_event(&up(x, y), bounds, &theme);
    assert_eq!(cal.shown, (2026, 11));
    assert!(cal.turn(-2));
    assert_eq!(cal.shown, (2026, 9));
    assert!(cal.turn(4) && cal.turn(1));
    assert_eq!(cal.shown, (2027, 2), "across the year");
}

#[test]
fn calendar_range_and_disabled_refuse_a_day() {
    let mut cal = Calendar::new(2026, 10).range(Some(d(2026, 10, 10)), Some(d(2026, 10, 20)));
    assert!(!cal.select(d(2026, 10, 9)) && !cal.select(d(2026, 10, 21)));
    assert!(cal.select(d(2026, 10, 10)));
    assert!(!cal.select(d(2026, 10, 10)), "the chosen one again");
    let mut off = Calendar::new(2026, 10).disabled(true);
    assert!(!off.select(d(2026, 10, 3)) && !off.turn(1));
    off.set_focused(Some(d(2026, 10, 3)));
    assert_eq!(off.focused(), None);
}

/// A day is a seventh of the width, clamped: the grid fits every width
/// from its minimum up and never passes the token's largest day.
#[test]
fn calendar_cells_follow_the_width_between_the_clamps() {
    let theme = Theme::hoff_core();
    let t = theme.selection.calendar.clone();
    let cal = Calendar::new(2026, 10);
    let mut last = 0.0;
    for w in (Calendar::min_width(&theme) as u32..=1600).step_by(8) {
        let (cw, ch) = cal.size_within(w as f32, &theme);
        assert!(cw <= w as f32 + 0.5, "{w}: {cw}");
        assert!(ch >= last, "{w}: grows or stays");
        last = ch;
        let bounds = Rect::new(0.0, 0.0, w as f32, ch);
        let l = cal.layout(bounds, &theme);
        for r in &l.days {
            assert!(r.w >= t.cell_min && r.w <= t.cell_max);
            assert!(r.x >= 0.0 && r.right() <= w as f32 + 0.5 && r.bottom() <= ch + 0.5);
        }
        assert!(l.prev.x >= l.title.right() && l.next.x >= l.prev.right());
    }
}

#[test]
fn calendar_projects_its_days_as_radios() {
    let cid = id("form.when");
    let cal = Calendar::new(2026, 10)
        .selected(d(2026, 10, 31))
        .range(None, Some(d(2026, 10, 30)))
        .sem(cid.clone());
    let s = snap(|b| {
        cal.project(b, &cid, "when", None);
    });
    let node = s.get(&cid).unwrap();
    assert_eq!(node.role, Role::Group);
    assert_eq!(
        node.state.get("selected"),
        Some(&Value::Text("2026-10-31".into()))
    );
    assert_eq!(
        node.state.get("month"),
        Some(&Value::Text("2026-10".into()))
    );
    let day = s.get(&cid.key("2026-10-31")).unwrap();
    assert_eq!(
        (day.role, day.label.as_str()),
        (Role::Radio, "31 October 2026")
    );
    assert_eq!(day.state.get("checked"), Some(&Value::Bool(true)));
    assert!(!day.enabled, "past the range");
    assert!(
        s.get(&cid.key("2026-09-30")).is_none(),
        "only the month shown"
    );
    assert_eq!(
        s.get(&cid.child("next").unwrap()).unwrap().role,
        Role::Button
    );
}

#[test]
fn calendar_binds_its_days_under_every_theme_at_360_and_1280() {
    let cid = id("m.when");
    for name in ["core", "hoff", "plev", "nord"] {
        let theme = Theme::named(name).unwrap();
        for vw in [360.0, 1280.0] {
            let cal = Calendar::new(2026, 10)
                .selected(d(2026, 10, 5))
                .today(d(2026, 10, 6))
                .sem(cid.clone());
            let (w, h) = cal.size_within(vw - 32.0, &theme);
            let bounds = Rect::new(16.0, 16.0, w, h);
            let mut c = Compositor::new();
            c.begin_frame();
            cal.render(&mut c, bounds, &theme);
            let l = cal.layout(bounds, &theme);
            let at = cal
                .grid()
                .iter()
                .position(|x| *x == d(2026, 10, 5))
                .unwrap();
            let (x, y) = l.days[at].center();
            assert_eq!(c.hit(x, y), Some(&cid.key("2026-10-05")), "{name} {vw}");
        }
    }
}

/// A day of another month is a blank cell: a click on it chooses
/// nothing and the month stays, as the projection offers only the month
/// shown.
#[test]
fn calendar_days_of_another_month_do_not_press() {
    let theme = Theme::hoff_core();
    let mut cal = Calendar::new(2026, 10);
    let (w, h) = cal.size_within(320.0, &theme);
    let bounds = Rect::new(0.0, 0.0, w, h);
    let (x, y) = cal.layout(bounds, &theme).days[0].center();
    assert_eq!(cal.grid()[0], d(2026, 9, 28));
    cal.handle_event(&down(x, y), bounds, &theme);
    assert!(!cal.handle_event(&up(x, y), bounds, &theme).clicked);
    assert_eq!((cal.selected, cal.shown), (None, (2026, 10)));
}

/// The month shown projects one radio a day, named as a person reads it;
/// today says so, and the days out of the range are disabled.
#[test]
fn calendar_projects_the_month_with_today_and_the_range() {
    let (f, cid) = (id("form"), id("form.due"));
    let cal = Calendar::new(2026, 10)
        .today(d(2026, 10, 31))
        .range(Some(d(2026, 10, 10)), None)
        .selected(d(2026, 10, 14));
    let s = snap(|b| {
        b.node(&f, Role::Group, "form");
        cal.project(b, &cid, "due", Some(&f));
    });
    let node = s.get(&cid).unwrap();
    assert_eq!(
        node.state.get("selected"),
        Some(&Value::Text("2026-10-14".into()))
    );
    assert_eq!(
        node.state.get("month"),
        Some(&Value::Text("2026-10".into()))
    );
    let day = |k: &str| s.get(&cid.key(k)).unwrap();
    assert_eq!(day("2026-10-31").label, "31 October 2026");
    assert_eq!(
        day("2026-10-31").state.get("today"),
        Some(&Value::Bool(true))
    );
    assert_eq!(day("2026-10-14").state.get("today"), None);
    assert!(!day("2026-10-09").enabled && day("2026-10-10").enabled);
    assert!(s.get(&cid.key("2026-09-30")).is_none(), "another month");
}
