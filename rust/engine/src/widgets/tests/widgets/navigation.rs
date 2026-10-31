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
            SceneNode::RoundedRect { color, .. } if color[3] > 0.0 => Some(*color),
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
    let kit = NavigationTokens::hoff_core();
    assert_eq!(core.navigation.breadcrumb, kit.breadcrumb);
    assert_eq!(core.navigation.link, kit.link);
    assert_ne!(core.navigation, NavigationTokens::derive(&core));
    // The sidebar is the chrome: the look of before in core too.
    let before = NavigationTokens::sidebar_link_of_before(&core);
    assert_eq!(core.navigation.sidebar_link, before);
    assert_eq!(
        core.navigation.sidebar_link,
        NavigationTokens::derive(&core).link
    );
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

// Nav link

fn link_faces(link: &NavLink, bounds: Rect, theme: &Theme) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
    let drawn = nodes(|c| link.render(c, bounds, theme));
    let strokes = drawn
        .iter()
        .filter_map(|n| match n {
            SceneNode::RoundedRect {
                border_width,
                border_color,
                ..
            } if *border_width > 0.0 => Some(*border_color),
            _ => None,
        })
        .collect();
    (rounded_fills(&drawn), strokes)
}

/// The kit's link: 40 tall, its box filled on hover and press, filled
/// and rimmed while active with the tile raised under the icon; the
/// label stays ink and the icon turns from gray to ink.
#[test]
fn nav_link_states_are_the_kit_link_on_its_own() {
    let t = core();
    let r = t.navigation.link;
    assert_eq!((NavLink::height(&t), NavLink::icon_slot(&t)), (40.0, 32.0));
    let b = Rect::new(0.0, 0.0, 240.0, NavLink::height(&t));
    let mut link = NavLink::new("Cards").icon("clipboard");
    let (fills, rims) = link_faces(&link, b, &t);
    assert!(
        fills.is_empty() && rims.is_empty(),
        "nothing behind a link at rest"
    );
    let label = |l: &NavLink| texts(&nodes(|c| l.render(c, b, &t)))[0].1;
    assert_eq!(label(&link), r.rest.label.0);

    link.handle_event(&move_to(20.0, 20.0), b);
    assert_eq!(link_faces(&link, b, &t).0, vec![r.hover.fill.0]);
    link.handle_event(&down(20.0, 20.0), b);
    assert_eq!(link_faces(&link, b, &t).0, vec![r.pressed.fill.0]);
    link.handle_event(&up(20.0, 20.0), b);

    let active = NavLink::new("Cards").icon("clipboard").active(true);
    let (fills, rims) = link_faces(&active, b, &t);
    let tile = r.tile.expect("the kit link has a tile");
    assert!(fills.contains(&r.active.fill.0));
    assert!(fills.contains(&tile.fill_current.0), "the tile rises");
    assert!(rims.contains(&r.active.border.0));
    assert_eq!(label(&active), r.active.label.0);
}

/// The sidebar draws its links with its own recipe: the look of before,
/// with no tile and no hover box, in core as in hoff.
#[test]
fn nav_link_in_the_sidebar_keeps_the_look_of_before() {
    for t in [core(), hoff()] {
        let r = t.navigation.sidebar_link;
        assert!(r.tile.is_none());
        assert_eq!(r.hover.fill.0[3], 0.0);
        let b = Rect::new(0.0, 0.0, 240.0, r.height);
        let mut link = NavLink::new("Cards").icon("clipboard");
        link.handle_event(&move_to(20.0, 20.0), b);
        let drawn = nodes(|c| link.render_with(c, LayerId::DEFAULT, b, &t, &r));
        assert!(rounded_fills(&drawn).is_empty());
        assert_eq!(texts(&drawn)[0].1, t.glass.text_default.0);
    }
}

/// Narrower than its tile, its paddings and a label's least room, the
/// kit link draws its rail form: the icon alone, centered on its row.
#[test]
fn nav_link_rail_form_centers_the_tile() {
    let t = core();
    let link = NavLink::new("Cards")
        .icon("clipboard")
        .hint("1")
        .active(true);
    let rail = Rect::new(0.0, 0.0, 48.0, NavLink::height(&t));
    let drawn = nodes(|c| link.render(c, rail, &t));
    assert!(texts(&drawn).is_empty(), "no label or hint in the rail");
    let tile = t.navigation.link.tile.unwrap();
    let raised = drawn.iter().find_map(|n| match n {
        SceneNode::RoundedRect { x, w, color, .. } if *color == tile.fill_current.0 => {
            Some((*x, *w))
        }
        _ => None,
    });
    assert_eq!(raised, Some(((48.0 - tile.size) / 2.0, tile.size)));
    let full = Rect::new(0.0, 0.0, 240.0, NavLink::height(&t));
    let shown = texts(&nodes(|c| link.render(c, full, &t)));
    assert_eq!(
        shown.iter().map(|s| s.0.as_str()).collect::<Vec<_>>(),
        ["1", "Cards"]
    );
}

/// A trail given exactly its preferred width draws every crumb whole:
/// the sums of measured widths never cut the current place.
#[test]
fn breadcrumb_at_its_preferred_width_draws_every_crumb_whole() {
    for t in [core(), hoff()] {
        let bc = Breadcrumb::new(["Home", "Projects", "plev", "Components"]);
        for x in [0.0, 13.3, 288.0, 1001.7] {
            let w = bc.preferred_width(&t);
            let bounds = Rect::new(x, 0.0, w, Breadcrumb::height(&t));
            let drawn: Vec<String> = texts(&nodes(|c| bc.render(c, bounds, &t)))
                .into_iter()
                .map(|(s, _)| s)
                .collect();
            assert_eq!(drawn, ["Home", "Projects", "plev", "Components"], "{x}");
        }
    }
}

// Stat

/// The kit's derivation is the pricing card's price for the large stat;
/// derived, the ramp of before.
#[test]
fn stat_recipe_is_the_price_in_core_and_the_old_ramp_elsewhere() {
    let kit = core().navigation.stat;
    assert_eq!(
        (
            kit.lg.value.size,
            kit.lg.value.line_height,
            kit.lg.value.weight
        ),
        (40.0, 48.0, 400)
    );
    assert_eq!(kit.lg.value.tracking_em, Some(-0.03));
    let t = hoff();
    let s = Stat::new("1.3", "GB").size(StatSize::Lg);
    assert_eq!(s.value_style(&t), t.typography.headline());
    assert_eq!(s.label_style(&t), t.typography.base_2r());
    let tracked = Stat::new("1.3", "GB")
        .size(StatSize::Lg)
        .value_style(&core());
    assert_eq!(tracked.letter_spacing, -0.03 * 40.0);
}

/// Narrower than its value row and delta, the delta wraps under the
/// value and the stat grows by a row; a long label truncates to the
/// bounds.
#[test]
fn stat_delta_wraps_and_label_truncates_when_narrow() {
    let t = core();
    let s = Stat::new("1,3", "armazenamento usado no disco do servidor")
        .unit("TB")
        .size(StatSize::Lg)
        .delta("+12% desde ontem", Intent::Constructive);
    let (w, h) = s.preferred_size(&t);
    assert!(!s.delta_wraps(w, &t));
    let narrow = s.min_width(&t);
    assert!(narrow < w);
    assert!(s.delta_wraps(narrow, &t));
    assert!(s.height_at(narrow, &t) > h);
    let drawn = texts(&nodes(|c| {
        s.render(c, Rect::new(0.0, 0.0, narrow, 200.0), &t)
    }));
    let label = &drawn.last().expect("the label is drawn").0;
    assert!(label.ends_with('\u{2026}'), "{label}");
}

/// The readout projects as a label: its value with the unit as the
/// value, its delta as the description, no action.
#[test]
fn stat_projects_a_label_with_its_value() {
    use accord::{SemId, SnapshotBuilder, Value};
    let id = SemId::parse("disk").unwrap();
    let s = Stat::new("1.3", "on disk")
        .unit("GB")
        .delta("+12%", Intent::Constructive);
    let mut b = SnapshotBuilder::new();
    s.project(&mut b, &id, None);
    let snap = b.finish().unwrap();
    let node = snap.get(&id).unwrap();
    assert_eq!(node.label, "on disk");
    assert_eq!(node.state.get("value"), Some(&Value::Text("1.3 GB".into())));
    assert_eq!(node.description.as_deref(), Some("+12%"));
    assert!(node.actions.is_empty());
}
