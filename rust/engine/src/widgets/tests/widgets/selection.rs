//! selection widgets: the toggle.

use super::*;
use crate::drawing::text::TextMeasurer;
use crate::widgets::selection::toggle::Toggle;
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
