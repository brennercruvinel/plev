//! Selection tests: the toggle group.

use super::*;

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
