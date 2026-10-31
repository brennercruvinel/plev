//! selection widgets: the toggle and the toggle group.

use super::*;
use crate::compose::animation::set_reduced_motion;
use crate::drawing::text::TextMeasurer;
use crate::widgets::selection::toggle::Toggle;
use crate::widgets::selection::{SegmentSize, ToggleGroup};
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
