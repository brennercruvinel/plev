//! Selection tests: the radio group.

use super::*;

fn plans() -> RadioGroup {
    RadioGroup::new(["Free", "Pro", "Team"])
}

#[test]
fn radio_group_click_chooses_a_row() {
    let theme = Theme::hoff_core();
    let mut g = plans();
    let (w, h) = g.size_within(400.0, &theme);
    let bounds = Rect::new(0.0, 0.0, w, h);
    let rows = g.row_rects(bounds, &theme);
    let (x, y) = rows[2].center();
    g.handle_event(&down(x, y), bounds, &theme);
    assert!(g.handle_event(&up(x, y), bounds, &theme).clicked);
    assert_eq!(g.selected, 2);
    assert!(!g.select(2) && !g.select(7));
    let mut off = plans().disabled(true);
    off.handle_event(&down(x, y), bounds, &theme);
    assert!(!off.handle_event(&up(x, y), bounds, &theme).clicked);
    assert_eq!(off.selected, 0);
    off.set_focused(Some(1));
    assert_eq!(off.focused(), None);
}

/// Stacked rows are the kit's 36px lines; flowed rows wrap within the
/// width and never overlap; a label wider than its row truncates.
#[test]
fn radio_group_stacks_flows_and_truncates() {
    let theme = Theme::hoff_core();
    let g = plans();
    let (_, h) = g.size_within(400.0, &theme);
    assert_eq!(h, 36.0 * 3.0);
    let flow = plans().layout(RadioLayout::Flow);
    let (w1, h1) = flow.size_within(1000.0, &theme);
    assert_eq!(h1, 36.0, "one line when it fits");
    let narrow = flow.size_within(w1 / 2.0, &theme);
    assert!(narrow.1 > h1, "wraps");
    let rows = flow.row_rects(Rect::new(0.0, 0.0, narrow.0, narrow.1), &theme);
    for (i, a) in rows.iter().enumerate() {
        assert!(a.right() <= narrow.0 + 0.5);
        for b in &rows[i + 1..] {
            assert!(a.right() <= b.x || a.bottom() <= b.y, "{a:?} {b:?}");
        }
    }
    let long = RadioGroup::new(["A choice whose label is far longer than any row"]);
    let row = Rect::new(0.0, 0.0, long.min_width(&theme) + 40.0, 36.0);
    assert!(long.shown_label(0, row, &theme).ends_with('…'));
}

#[test]
fn radio_group_projects_a_group_of_radios() {
    let gid = id("form.plan");
    let g = plans().selected(1).disabled(true);
    let s = snap(|b| {
        g.project(b, &gid, "plan", None);
    });
    let node = s.get(&gid).unwrap();
    assert_eq!((node.role, node.enabled), (Role::Group, false));
    assert_eq!(node.state.get("selected"), Some(&Value::Text("1".into())));
    let pro = s.get(&gid.key("1")).unwrap();
    assert_eq!((pro.role, pro.label.as_str()), (Role::Radio, "Pro"));
    assert_eq!(pro.state.get("checked"), Some(&Value::Bool(true)));
    assert_eq!(pro.actions[0].reads[0].to_string(), "form.plan:selected");
}

#[test]
fn radio_group_binds_each_row_under_every_theme_at_360_and_1280() {
    let gid = id("m.plan");
    for name in ["core", "hoff", "plev", "nord"] {
        let theme = Theme::named(name).unwrap();
        for vw in [360.0, 1280.0] {
            let g = plans().layout(RadioLayout::Flow).sem(gid.clone());
            let (w, h) = g.size_within(vw - 32.0, &theme);
            let bounds = Rect::new(16.0, 16.0, w, h);
            let mut c = Compositor::new();
            c.begin_frame();
            g.render(&mut c, bounds, &theme);
            for (i, r) in g.row_rects(bounds, &theme).iter().enumerate() {
                let row = gid.key(i.to_string());
                let (x, y) = r.center();
                assert_eq!(c.hit(x, y), Some(&row), "{name} {vw}");
            }
        }
    }
}
