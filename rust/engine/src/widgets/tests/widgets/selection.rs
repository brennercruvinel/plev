//! selection widgets: the toggle, the toggle group, the button group,
//! the radio group, the calendar.

use super::*;
use crate::compose::animation::set_reduced_motion;
use crate::drawing::text::TextMeasurer;
use crate::widgets::selection::toggle::Toggle;
use crate::widgets::selection::{
    ButtonGroup, Calendar, Date, DatePicker, GroupItem, RadioGroup, RadioLayout, SegmentSize,
    ToggleGroup,
};
use accord::{Role, SemId, Snapshot, SnapshotBuilder, Value};

fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn snap(project: impl FnOnce(&mut SnapshotBuilder)) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    project(&mut b);
    b.finish().unwrap()
}

fn nodes(t: &Toggle, bounds: Rect, theme: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    t.render(&mut c, bounds, theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

fn texts(nodes: &[SceneNode]) -> Vec<(String, f32)> {
    nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, x, .. } => Some((key.text.clone(), *x)),
            _ => None,
        })
        .collect()
}

#[test]
fn toggle_click_flips_on_and_back() {
    let mut t = Toggle::new("Bold");
    assert!(click(|e| t.handle_event(e, B)).clicked);
    assert!(t.on);
    assert!(click(|e| t.handle_event(e, B)).clicked);
    assert!(!t.on);
}

#[test]
fn toggle_release_outside_cancels_without_flipping() {
    let mut t = Toggle::new("Bold");
    t.handle_event(&down(20.0, 20.0), B);
    assert!(t.is_pressed());
    let r = t.handle_event(&up(500.0, 500.0), B);
    assert!(!r.clicked);
    assert!(r.changed, "the pressed look clears");
    assert!(!t.on);
}

#[test]
fn toggle_disabled_ignores_the_pointer_the_request_and_the_focus() {
    for on in [false, true] {
        let mut t = Toggle::new("Bold").on(on).disabled(true);
        let r = click(|e| t.handle_event(e, B));
        assert!(!r.clicked && !t.is_pressed());
        assert!(!t.toggle(), "a request does not flip it either");
        assert_eq!(t.on, on);
        t.set_focused(true);
        assert!(!t.is_focused());
    }
}

#[test]
fn toggle_hover_tracks_the_pointer() {
    let mut t = Toggle::icon_only("eye", "Bold");
    assert!(t.handle_event(&move_to(20.0, 20.0), B).changed);
    assert!(t.is_hovered());
    assert!(t.handle_event(&move_to(500.0, 20.0), B).changed);
    assert!(!t.is_hovered());
    assert_eq!(
        t.handle_event(&move_to(500.0, 20.0), B),
        EventResult::IGNORED
    );
}

/// Off reads `toolbar`, on reads `toolbar_active`: in the core kit the
/// rest of `toolbar` paints nothing but the content, and the selected
/// surface adds its fill, its two insets and its ring.
#[test]
fn toggle_on_and_off_paint_the_two_toolbar_recipes() {
    let theme = Theme::hoff_core();
    let off = nodes(&Toggle::icon_only("eye", "Bold"), B, &theme);
    let on = nodes(&Toggle::icon_only("eye", "Bold").on(true), B, &theme);
    assert!(on.len() > off.len(), "{} vs {}", on.len(), off.len());
    let rest = &theme.button.toolbar.rest;
    assert!(
        rest.shadow.is_empty() && rest.fill[0].0[3] == 0.0,
        "the kit's toolbar rest is bare"
    );
}

#[test]
fn toggle_sizes_follow_the_two_button_ladders() {
    let theme = Theme::hoff_core();
    let icon = Toggle::icon_only("eye", "Bold");
    let edge = IconButtonSize::Lg.metrics(&theme).size;
    assert_eq!(
        icon.preferred_size(&theme),
        (edge, edge),
        "the kit's size-10"
    );
    assert_eq!(icon.min_width(&theme), edge);

    let short = Toggle::new("B").preferred_size(&theme);
    let long = Toggle::new("Bold and underlined").preferred_size(&theme);
    assert_eq!(short.1, ButtonSize::Md.height(&theme));
    assert!(long.0 > short.0);
    let with_icon = Toggle::new("B").icon("eye").preferred_size(&theme);
    assert!(with_icon.0 > short.0);
    assert!(Toggle::new("Bold").min_width(&theme) < Toggle::new("Bold").preferred_size(&theme).0);
}

/// A label longer than its bounds truncates inside them; one that fits
/// draws whole.
#[test]
fn toggle_label_truncates_to_its_bounds() {
    let theme = Theme::hoff_core();
    let t = Toggle::new("A label far longer than any toolbar would hold").icon("eye");
    let m = ButtonSize::Md.metrics(&theme);
    let narrow = Rect::new(0.0, 0.0, 160.0, m.height);
    let shown = t.shown_label(narrow, &theme);
    assert!(shown.ends_with('…'), "{shown}");
    let (w, _) = TextMeasurer::measure_styled(&shown, &t.text_style(&theme), None);
    let drawn = texts(&nodes(&t, narrow, &theme));
    assert_eq!(drawn.len(), 1);
    let (text, x) = &drawn[0];
    assert_eq!(text, &shown);
    assert!(*x >= narrow.x && x + w <= narrow.right() + 0.5, "{x} + {w}");

    let wide = Rect::new(0.0, 0.0, 2000.0, m.height);
    assert_eq!(t.shown_label(wide, &theme), t.label);
}

#[test]
fn toggle_projects_a_toggle_with_its_on_state() {
    let (form, bold) = (id("form"), id("form.bold"));
    for (on, disabled) in [(false, false), (true, false), (true, true)] {
        let t = Toggle::icon_only("eye", "Bold").on(on).disabled(disabled);
        let s = snap(|b| {
            b.node(&form, Role::Group, "form");
            t.project(b, &bold, Some(&form));
        });
        let node = s.get(&bold).unwrap();
        assert_eq!((node.role, node.label.as_str()), (Role::Toggle, "Bold"));
        assert_eq!(node.parent.as_ref(), Some(&form));
        assert_eq!(node.state.get("on"), Some(&Value::Bool(on)));
        assert_eq!(node.enabled, !disabled);
        let toggle = &node.actions[0];
        assert_eq!(toggle.name.as_str(), "toggle");
        assert_eq!(toggle.reads.len(), 1);
        assert_eq!(toggle.reads[0].to_string(), "form.bold:on");
    }
}

/// Under every built-in theme, at the two widths the POC captures, each
/// form binds the rect it draws, answers the hit at its center and
/// pushes only finite nodes.
#[test]
fn toggle_renders_and_binds_under_every_theme_at_360_and_1280() {
    let mut themes = vec![
        Theme::hoff_core(),
        Theme::hoff(),
        Theme::dark(),
        Theme::light(),
    ];
    for name in ["plev", "catppuccin", "nord", "gruvbox"] {
        themes.push(Theme::named(name).unwrap());
    }
    let who = id("m.toggle");
    for theme in &themes {
        for vw in [360.0, 1280.0] {
            for t in [
                Toggle::icon_only("eye", "Bold"),
                Toggle::new("Bold").icon("eye").on(true),
                Toggle::new("A label far longer than a phone is wide, in any theme"),
            ] {
                let mut t = t.sem(who.clone());
                t.set_focused(true);
                let (w, h) = t.preferred_size(theme);
                let r = Rect::new(16.0, 16.0, w.min(vw - 32.0).max(t.min_width(theme)), h);
                let mut c = Compositor::new();
                c.begin_frame();
                t.render(&mut c, r, theme);
                assert_eq!(c.binding(&who).unwrap().rect, [r.x, r.y, r.w, r.h]);
                let (x, y) = r.center();
                assert_eq!(c.hit(x, y), Some(&who));
                for n in c.layer(LayerId::DEFAULT).unwrap().nodes() {
                    if let SceneNode::Text { x, y, .. } = n {
                        assert!(x.is_finite() && y.is_finite(), "{vw}: {n:?}");
                    }
                }
            }
        }
    }
}

// Toggle group

/// A group the width of three segments of the kit, laid out at the
/// origin.
fn group() -> (ToggleGroup, Rect, Theme) {
    let theme = Theme::hoff_core();
    let g = ToggleGroup::new(["All", "Active", "Completed"]);
    let (w, h) = g.preferred_size(&theme);
    (g, Rect::new(0.0, 0.0, w, h), theme)
}

fn click_at(g: &mut ToggleGroup, r: Rect, bounds: Rect, theme: &Theme) -> EventResult {
    let (x, y) = r.center();
    let a = g.handle_event(&down(x, y), bounds, theme);
    a.merge(g.handle_event(&up(x, y), bounds, theme))
}

#[test]
fn toggle_group_click_chooses_a_segment_and_the_thumb_follows() {
    let (mut g, bounds, theme) = group();
    let rects = g.segment_rects(bounds, &theme);
    assert!(click_at(&mut g, rects[2], bounds, &theme).clicked);
    assert_eq!(g.selected, 2);
    assert!(g.is_animating(), "the thumb slides");
    for _ in 0..240 {
        g.tick(1.0 / 60.0);
    }
    assert!((g.thumb_position() - 2.0).abs() < 1e-3);
    // The chosen one again changes nothing.
    click_at(&mut g, rects[2], bounds, &theme);
    assert_eq!(g.selected, 2);
    assert!(!g.select(2) && !g.select(9), "same or out of range");
}

#[test]
fn toggle_group_reduced_motion_lands_the_thumb_at_once() {
    let (mut g, _, _) = group();
    set_reduced_motion(true);
    assert!(g.select(1));
    let moving = g.tick(1.0 / 60.0);
    set_reduced_motion(false);
    assert!(!moving);
    assert_eq!(g.thumb_position(), 1.0);
}

#[test]
fn toggle_group_disabled_refuses_the_pointer_the_request_and_the_focus() {
    let (g, bounds, theme) = group();
    let mut g = g.disabled(true);
    let rects = g.segment_rects(bounds, &theme);
    assert!(!click_at(&mut g, rects[1], bounds, &theme).clicked);
    assert!(!g.select(1));
    assert_eq!(g.selected, 0);
    g.set_focused(Some(1));
    assert_eq!(g.focused(), None);
}

#[test]
fn toggle_group_release_outside_its_segment_chooses_nothing() {
    let (mut g, bounds, theme) = group();
    let rects = g.segment_rects(bounds, &theme);
    let (x, y) = rects[1].center();
    g.handle_event(&down(x, y), bounds, &theme);
    let (x2, y2) = rects[2].center();
    let r = g.handle_event(&up(x2, y2), bounds, &theme);
    assert!(!r.clicked);
    assert_eq!(g.selected, 0);
}

/// The kit's geometry: a 40px group (a 32px segment in a 3px padding
/// and a 1px border) on `Md`, 36 on `Sm`; equal segments that tile the
/// inside; the widest label sets the width.
#[test]
fn toggle_group_geometry_follows_the_kit() {
    let (g, bounds, theme) = group();
    assert_eq!(bounds.h, 40.0);
    assert_eq!(g.clone().size(SegmentSize::Sm).height(&theme), 36.0);
    let rects = g.segment_rects(bounds, &theme);
    assert_eq!(rects.len(), 3);
    assert!(rects.windows(2).all(|w| (w[0].w - w[1].w).abs() < 1e-3));
    assert!((rects[0].x - 4.0).abs() < 1e-3);
    assert!((rects[2].right() - (bounds.right() - 4.0)).abs() < 1e-3);
    for (i, r) in rects.iter().enumerate() {
        assert_eq!(g.shown_label(i, *r, &theme), g.items[i], "fits at its size");
    }
    assert!(g.min_width(&theme) < bounds.w);
}

/// Narrower than its labels, each label truncates inside its segment.
#[test]
fn toggle_group_labels_truncate_in_narrow_segments() {
    let (g, bounds, theme) = group();
    let narrow = Rect::new(0.0, 0.0, g.min_width(&theme) + 30.0, bounds.h);
    let rects = g.segment_rects(narrow, &theme);
    let shown = g.shown_label(2, rects[2], &theme);
    assert!(shown.ends_with('…'), "{shown}");
    let mut c = Compositor::new();
    c.begin_frame();
    g.render(&mut c, narrow, &theme);
    for (text, x) in texts(c.layer(LayerId::DEFAULT).unwrap().nodes()) {
        let style = SegmentSize::Md.text_style(&theme);
        let (w, _) = TextMeasurer::measure_styled(&text, &style, None);
        assert!(x >= narrow.x && x + w <= narrow.right(), "{text} at {x}");
    }
}

#[test]
fn toggle_group_projects_a_group_of_radios() {
    let (f, gid) = (id("form"), id("form.view"));
    let g = ToggleGroup::new(["Grid", "List"]).selected(1);
    let s = snap(|b| {
        b.node(&f, Role::Group, "form");
        g.project(b, &gid, "view", Some(&f));
    });
    let node = s.get(&gid).unwrap();
    assert_eq!((node.role, node.label.as_str()), (Role::Group, "view"));
    assert_eq!(node.state.get("selected"), Some(&Value::Text("1".into())));
    for (i, label) in ["Grid", "List"].into_iter().enumerate() {
        let item = s.get(&gid.key(i.to_string())).unwrap();
        assert_eq!((item.role, item.label.as_str()), (Role::Radio, label));
        assert_eq!(item.state.get("checked"), Some(&Value::Bool(i == 1)));
        assert_eq!(item.actions[0].name.as_str(), "select");
        assert_eq!(item.actions[0].reads[0].to_string(), "form.view:selected");
    }
}

#[test]
fn toggle_group_binds_each_segment_under_every_theme_at_360_and_1280() {
    let gid = id("m.group");
    for name in ["core", "hoff", "plev", "nord"] {
        let theme = Theme::named(name).unwrap();
        for vw in [360.0, 1280.0] {
            let g = ToggleGroup::new(["Day", "Week", "Month"])
                .selected(1)
                .sem(gid.clone());
            let (w, h) = g.preferred_size(&theme);
            let bounds = Rect::new(16.0, 16.0, w.min(vw - 32.0), h);
            let mut c = Compositor::new();
            c.begin_frame();
            g.render(&mut c, bounds, &theme);
            for (i, r) in g.segment_rects(bounds, &theme).iter().enumerate() {
                let seg = gid.key(i.to_string());
                assert_eq!(c.binding(&seg).unwrap().rect, [r.x, r.y, r.w, r.h]);
                let (x, y) = r.center();
                assert_eq!(c.hit(x, y), Some(&seg), "{name} {vw}");
            }
        }
    }
}

// Button group

fn tool(name: &str) -> GroupItem {
    GroupItem::Toggle(Toggle::icon_only("eye", name))
}

fn action(label: &str) -> GroupItem {
    GroupItem::Action {
        button: IconButton::new("x").variant(IconButtonVariant::Toolbar),
        label: label.into(),
    }
}

/// The kit's toolbar: three tools, a separator, two actions.
fn toolbar() -> ButtonGroup {
    ButtonGroup::new([
        tool("Select"),
        tool("Move"),
        tool("Crop"),
        GroupItem::Separator,
        action("Undo"),
        action("Redo"),
    ])
}

/// One row in the kit's measure: 40px items, 8px gap and padding, a 1px
/// separator, a 56px shell.
#[test]
fn button_group_lays_one_row_out_as_the_kit() {
    let theme = Theme::hoff_core();
    let g = toolbar();
    let (w, h) = g.preferred_size(&theme);
    assert_eq!(h, 56.0);
    assert_eq!(w, 8.0 * 2.0 + 40.0 * 5.0 + 1.0 + 8.0 * 5.0);
    let rects = g.item_rects(Rect::new(0.0, 0.0, w, h), &theme);
    let sep = rects[3].unwrap();
    assert_eq!((sep.y, sep.h), (0.0, 56.0), "the hairline spans the shell");
    assert!(rects.iter().flatten().all(|r| r.right() <= w - 8.0 + 1e-3));
}

/// Narrower than its row, the group wraps: never past its width, no
/// overlap, a separator never starts a row, and the height grows.
#[test]
fn button_group_wraps_when_the_width_runs_out() {
    let theme = Theme::hoff_core();
    let g = toolbar();
    let one_row = g.preferred_size(&theme);
    for max_w in [g.min_width(&theme), 120.0, 160.0, 200.0, one_row.0 - 1.0] {
        let (w, h) = g.size_within(max_w, &theme);
        assert!(w <= max_w.max(g.min_width(&theme)) + 0.5, "{max_w}: {w}");
        assert!(h > one_row.1, "{max_w}: wraps");
        let bounds = Rect::new(0.0, 0.0, w, h);
        let rects: Vec<Rect> = g.item_rects(bounds, &theme).into_iter().flatten().collect();
        for (i, a) in rects.iter().enumerate() {
            assert!(a.x >= 0.0 && a.right() <= w + 0.5 && a.bottom() <= h + 0.5);
            for b in &rects[i + 1..] {
                let apart =
                    a.right() <= b.x || b.right() <= a.x || a.bottom() <= b.y || b.bottom() <= a.y;
                assert!(apart, "{max_w}: {a:?} {b:?}");
            }
        }
    }
}

/// An exclusive group keeps one tool on: a click on another moves it, a
/// click on the one on keeps it; a free group flips each.
#[test]
fn button_group_exclusive_keeps_one_tool_on() {
    let theme = Theme::hoff_core();
    let mut g = toolbar().exclusive(true);
    let (w, h) = g.preferred_size(&theme);
    let bounds = Rect::new(0.0, 0.0, w, h);
    let on = |g: &ButtonGroup| -> Vec<bool> {
        g.items
            .iter()
            .filter_map(|i| match i {
                GroupItem::Toggle(t) => Some(t.on),
                _ => None,
            })
            .collect()
    };
    let rects = g.item_rects(bounds, &theme);
    let tap = |g: &mut ButtonGroup, i: usize| {
        let (x, y) = rects[i].unwrap().center();
        g.handle_event(&down(x, y), bounds, &theme);
        g.handle_event(&up(x, y), bounds, &theme)
    };
    assert!(tap(&mut g, 1).clicked);
    assert_eq!(on(&g), [false, true, false]);
    tap(&mut g, 1);
    assert_eq!(on(&g), [false, true, false], "the one on stays on");
    assert!(g.press(2));
    assert_eq!(on(&g), [false, false, true]);
    assert!(!g.press(4), "an action holds no state");

    let mut free = toolbar();
    assert!(free.press(0) && free.press(1));
    assert_eq!(on(&free), [true, true, false]);
}

#[test]
fn button_group_projects_a_toolbar_of_its_items() {
    let tb = id("m.tools");
    let mut g = toolbar();
    for (i, item) in g.items.iter_mut().enumerate() {
        let at = tb.child("item").unwrap().key(i.to_string());
        match item {
            GroupItem::Toggle(t) => *t = t.clone().sem(at),
            GroupItem::Action { button, .. } => *button = button.clone().sem(at),
            GroupItem::Separator => {}
        }
    }
    let s = snap(|b| {
        g.project(b, &tb, "tools", None);
    });
    assert_eq!(s.get(&tb).unwrap().role, Role::Toolbar);
    let item = |i: usize| s.get(&tb.child("item").unwrap().key(i.to_string()));
    assert_eq!(
        item(0).map(|n| (n.role, n.label.as_str())),
        Some((Role::Toggle, "Select"))
    );
    assert_eq!(
        item(4).map(|n| (n.role, n.label.as_str())),
        Some((Role::Button, "Undo"))
    );
    assert!(item(3).is_none(), "a separator is not a control");
    assert_eq!(item(0).unwrap().parent.as_ref(), Some(&tb));
}

// Radio group

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

// Calendar

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

/// An informational chip and a constructive one stand apart on the color
/// wheel under every theme: the hoff's info is a green, as its success,
/// so its chip takes the kit's blue.
#[test]
fn an_informational_chip_is_not_a_constructive_one() {
    use crate::widgets::selection::tokens::hue_sextant;
    for name in ["core", "hoff", "hoff-light", "plev", "catppuccin"] {
        let theme = Theme::named(name).unwrap();
        let info = theme.selection.chip.info;
        assert!(hue_sextant(info).is_some(), "{name}: a hue");
        assert_ne!(
            hue_sextant(info),
            hue_sextant(theme.colors.success),
            "{name}"
        );
    }
    assert_eq!(
        Theme::hoff_core().selection.chip.info,
        Theme::hoff_core().colors.info
    );
    assert_eq!(
        hue_sextant(crate::styling::color::Color::hex(0x808080)),
        None
    );
}

/// A disabled box that is on keeps its check: the check is the token's
/// color that reads on the faded box (`theme/tests/contrast.rs`), drawn
/// whole, never the knob's paper faded with the box. Under the core kit
/// that is a gray, the paper vanishing there.
#[test]
fn a_disabled_checked_box_keeps_its_check() {
    use crate::widgets::form::Checkbox;
    let core = Theme::hoff_core();
    assert_ne!(
        core.selection.checkbox.mark_disabled,
        core.selection.checkbox.mark
    );
    for theme in [core.clone(), Theme::hoff()] {
        let t = &theme.selection.checkbox;
        let mut c = Compositor::new();
        c.begin_frame();
        Checkbox::new(true)
            .disabled(true)
            .render(&mut c, Rect::new(0.0, 0.0, 18.0, 18.0), &theme);
        let check = c
            .layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .find_map(|n| match n {
                SceneNode::Path { data } => data.vertices.first().map(|v| v.color),
                _ => None,
            });
        assert_eq!(check, Some(t.mark_disabled.0));
    }
}

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
