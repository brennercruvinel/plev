//! task-104 front G, navigation and data: the slot's two faces (the kit
//! in core, the look of before in every other theme) and each widget's
//! states as it draws them.

use super::*;
use crate::styling::theme::IconSize;
use crate::widgets::nav::NavigationTokens;

fn core() -> Theme {
    Theme::hoff_core()
}

fn nodes(render: impl FnOnce(&mut Compositor)) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    render(&mut c);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

/// The text runs drawn, with their color.
fn texts(nodes: &[SceneNode]) -> Vec<(String, [f32; 4])> {
    nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::Text { key, color, .. } => Some((key.text.clone(), *color)),
            _ => None,
        })
        .collect()
}

fn rounded_fills(nodes: &[SceneNode]) -> Vec<[f32; 4]> {
    nodes
        .iter()
        .filter_map(|n| match n {
            SceneNode::RoundedRect { color, .. } => Some(*color),
            _ => None,
        })
        .collect()
}

/// Every theme but core is derived, and derived is what these widgets
/// drew with before the slot: the core is the only one that moves.
#[test]
fn the_slot_is_the_kit_in_core_and_derived_elsewhere() {
    let themes = [Theme::hoff(), Theme::dark(), Theme::light()];
    for theme in themes.iter().chain(Theme::named("nord").as_ref()) {
        assert_eq!(theme.navigation, NavigationTokens::derive(theme));
    }
    let core = core();
    assert_eq!(core.navigation, NavigationTokens::hoff_core());
    assert_ne!(core.navigation, NavigationTokens::derive(&core));
}

// Breadcrumb

/// The kit's derivation: the sidebar link's text and box, the dropdown's
/// chevron; derived, the trail of before (base-2r, no box, a small icon).
#[test]
fn breadcrumb_recipe_is_the_kit_link_in_core_and_the_old_trail_elsewhere() {
    let kit = core().navigation.breadcrumb;
    assert_eq!(
        (kit.text.size, kit.text.line_height, kit.text.weight),
        (12.0, 16.0, 600)
    );
    assert_eq!(
        (kit.pad_x, kit.pad_y, kit.radius, kit.chevron),
        (8.0, 4.0, 8.0, 16.0)
    );
    assert_eq!(kit.hover_fill, crate::styling::theme::hoff_core::SHADES[2]);

    let t = hoff();
    let old = t.navigation.breadcrumb;
    let base = t.typography.base_2r();
    assert_eq!(old.text.style(&t.typography), base);
    assert_eq!((old.pad_x, old.pad_y), (0.0, 0.0));
    assert_eq!(old.hover_fill.0[3], 0.0);
    assert_eq!(old.chevron, t.control.icon(IconSize::Sm));
    assert_eq!(old.gap, t.spacing.xs);
    assert_eq!(Breadcrumb::height(&t), base.line_height);
}

/// Hovered, a crumb turns ink in its wash; pressed, its box takes the
/// rim tone; the current place keeps no box. In a derived theme no box
/// shows, as before.
#[test]
fn breadcrumb_hover_and_press_paint_the_crumb_box() {
    let t = core();
    let r = t.navigation.breadcrumb;
    let mut bc = Breadcrumb::new(["home", "docs", "adr"]);
    let bounds = Rect::new(0.0, 0.0, 400.0, Breadcrumb::height(&t));
    let rects = bc.crumb_rects(bounds, &t);
    let (x, y) = rects[0].1.center();
    let rest = nodes(|c| bc.render(c, bounds, &t));
    assert!(rounded_fills(&rest).is_empty());
    assert_eq!(texts(&rest)[0], ("home".into(), r.rest.0));
    assert_eq!(texts(&rest)[2], ("adr".into(), r.current.0));

    bc.handle_event(&move_to(x, y), bounds, &t);
    let hover = nodes(|c| bc.render(c, bounds, &t));
    assert_eq!(rounded_fills(&hover), vec![r.hover_fill.0]);
    assert_eq!(texts(&hover)[0].1, r.hover.0);

    bc.handle_event(&down(x, y), bounds, &t);
    let press = nodes(|c| bc.render(c, bounds, &t));
    assert_eq!(rounded_fills(&press), vec![r.pressed_fill.0]);

    let h = hoff();
    let mut old = Breadcrumb::new(["home", "docs", "adr"]);
    let ob = Rect::new(0.0, 0.0, 400.0, Breadcrumb::height(&h));
    let (ox, oy) = old.crumb_rects(ob, &h)[0].1.center();
    old.handle_event(&move_to(ox, oy), ob, &h);
    let drawn = nodes(|c| old.render(c, ob, &h));
    assert!(
        rounded_fills(&drawn).is_empty(),
        "no box in a derived theme"
    );
    assert_eq!(texts(&drawn)[0].1, h.glass.text_active.0);
}

/// One crumb never folds into an ellipsis, and a current place wider
/// than its room truncates inside the bounds instead of running past.
#[test]
fn breadcrumb_current_place_truncates_and_never_folds_alone() {
    let t = core();
    let one = Breadcrumb::new(["A place whose name is far too long for its room"]);
    let bounds = Rect::new(0.0, 0.0, 120.0, Breadcrumb::height(&t));
    assert_eq!(one.visible(bounds.w, &t), vec![Some(0)]);
    let rects = one.crumb_rects(bounds, &t);
    assert!(rects[0].1.right() <= bounds.right() + 0.01);
    let drawn = texts(&nodes(|c| one.render(c, bounds, &t)));
    assert!(drawn[0].0.ends_with('\u{2026}'), "{}", drawn[0].0);

    let deep = Breadcrumb::new(["crates", "engine", "src", "a-very-long-file-name.rs"]);
    let narrow = Rect::new(0.0, 0.0, 90.0, Breadcrumb::height(&t));
    let shown = deep.crumb_rects(narrow, &t);
    assert_eq!(
        shown.first().map(|s| s.0),
        Some(None),
        "folded to the ellipsis"
    );
    assert_eq!(shown.last().map(|s| s.0), Some(Some(3)));
    assert!(
        shown
            .iter()
            .all(|(_, r)| r.right() <= narrow.right() + 0.01)
    );
}

/// A crumb's box is the hit target: a press on its padding, beside the
/// text, still lands on it.
#[test]
fn breadcrumb_box_is_the_target() {
    let t = core();
    let mut bc = Breadcrumb::new(["home", "docs", "adr"]);
    let bounds = Rect::new(0.0, 0.0, 400.0, Breadcrumb::height(&t));
    let r = bc.crumb_rects(bounds, &t)[1].1;
    let (x, y) = (r.x + 1.0, r.y + 1.0);
    bc.handle_event(&down(x, y), bounds, &t);
    let (res, clicked) = bc.handle_event(&up(x, y), bounds, &t);
    assert!(res.clicked);
    assert_eq!(clicked, Some(1));
}

/// The keyboard focus on a folded crumb unfolds it: Tab never lands on
/// a crumb that is not drawn; the focus gone, the trail folds back.
#[test]
fn breadcrumb_focus_unfolds_the_crumb_it_lands_on() {
    let t = core();
    let mut bc = Breadcrumb::new(["one", "two", "three", "four", "five", "six"]);
    let w = bc.preferred_width(&t) * 0.7;
    let folded = bc.visible(w, &t);
    assert!(!folded.contains(&Some(1)), "{folded:?}");
    bc.set_focused(Some(1));
    let shown = bc.visible(w, &t);
    assert!(shown.contains(&Some(1)), "{shown:?}");
    assert_eq!(shown.last(), Some(&Some(5)));
    let narrow = Rect::new(0.0, 0.0, 60.0, Breadcrumb::height(&t));
    let rects = bc.crumb_rects(narrow, &t);
    assert!(rects.iter().any(|(i, _)| *i == Some(1)));
    assert!(
        rects
            .iter()
            .all(|(_, r)| r.right() <= narrow.right() + 0.01)
    );
    bc.set_focused(None);
    assert_eq!(bc.visible(w, &t), folded);
}
