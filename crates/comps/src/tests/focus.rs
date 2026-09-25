//! Focus-state tests for the form widgets: every interactive control
//! draws the shared accent focus ring when focused, skips it when not,
//! and disabled controls refuse focus entirely.

use crate::core::*;
use crate::prelude::*;
use crate::recipe::*;
use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::theme::Theme;

const B: Rect = Rect {
    x: 40.0,
    y: 40.0,
    w: 160.0,
    h: 44.0,
};

fn nodes(render: impl FnOnce(&mut Compositor, &Theme)) -> Vec<SceneNode> {
    let theme = Theme::hoff();
    let mut c = Compositor::new();
    c.begin_frame();
    render(&mut c, &theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

/// The ring is a border-only rounded rect: transparent fill, 2px border
/// in the theme accent (`focus_ring`).
fn ring_of(nodes: &[SceneNode]) -> Option<(f32, f32, f32, f32, f32)> {
    let theme = Theme::hoff();
    let accent = theme.colors.accent.0;
    let ring_w = theme.control.focus_ring_width;
    nodes.iter().find_map(|n| match *n {
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h,
            color,
            corner_radius,
            border_width,
            border_color,
        } if color[3] == 0.0 && border_width == ring_w && border_color == accent => {
            Some((x, y, w, h, corner_radius))
        }
        _ => None,
    })
}

#[test]
fn focus_ring_helper_offsets_2px_outside_the_rect() {
    let theme = Theme::hoff();
    let node = focus_ring(B, 12.0, &theme);
    match node {
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h,
            corner_radius,
            border_width,
            border_color,
            color,
        } => {
            // 2px offset + 2px stroke = 4px inflation per side.
            assert_eq!((x, y), (B.x - 4.0, B.y - 4.0));
            assert_eq!((w, h), (B.w + 8.0, B.h + 8.0));
            assert_eq!(corner_radius, 16.0, "radius follows the shape");
            assert_eq!(border_width, theme.control.focus_ring_width);
            assert_eq!(border_color, theme.colors.accent.0);
            assert_eq!(color[3], 0.0, "ring has no fill");
        }
        other => panic!("expected RoundedRect ring, got {other:?}"),
    }
}

#[test]
fn button_draws_focus_ring_only_when_focused() {
    let mut b = Button::new("Save");
    assert!(ring_of(&nodes(|c, t| b.render(c, B, t))).is_none());
    b.set_focused(true);
    assert!(b.is_focused());
    let ring = ring_of(&nodes(|c, t| b.render(c, B, t))).expect("focused button must ring");
    assert_eq!((ring.0, ring.1), (B.x - 4.0, B.y - 4.0));
    b.set_focused(false);
    assert!(ring_of(&nodes(|c, t| b.render(c, B, t))).is_none());
}

#[test]
fn checkbox_rings_the_box_not_the_label_row() {
    let mut cb = Checkbox::new(false).label("Autosave");
    cb.set_focused(true);
    let ring = ring_of(&nodes(|c, t| cb.render(c, B, t))).expect("focused checkbox must ring");
    // The 18px box sits at the left edge, vertically centered.
    let by = B.y + (B.h - 18.0) / 2.0;
    assert_eq!((ring.0, ring.1), (B.x - 4.0, by - 4.0));
    assert_eq!((ring.2, ring.3), (18.0 + 8.0, 18.0 + 8.0));
}

#[test]
fn switch_rings_the_track_not_the_bounds() {
    let mut sw = Switch::new(true);
    sw.set_focused(true);
    let wide = Rect::new(0.0, 0.0, 120.0, 44.0);
    let ring = ring_of(&nodes(|c, t| sw.render(c, wide, t))).expect("focused switch must ring");
    // Track is 44x24 centered in the bounds.
    assert_eq!((ring.0, ring.1), ((120.0 - 44.0) / 2.0 - 4.0, 10.0 - 4.0));
    assert_eq!((ring.2, ring.3), (44.0 + 8.0, 24.0 + 8.0));
}

#[test]
fn slider_select_and_tabs_ring_their_bounds() {
    let mut sl = Slider::new(0.0, 100.0, 50.0);
    sl.set_focused(true);
    assert!(ring_of(&nodes(|c, t| sl.render(c, B, t))).is_some());

    let mut se = Select::new(["A", "B"], 0);
    se.set_focused(true);
    assert!(ring_of(&nodes(|c, t| se.render(c, B, t))).is_some());

    let mut tabs = Tabs::new(["One", "Two"]);
    tabs.set_focused(true);
    assert!(ring_of(&nodes(|c, t| tabs.render(c, B, t))).is_some());
}

#[test]
fn disabled_widgets_refuse_focus() {
    let mut b = Button::new("Save").disabled(true);
    b.set_focused(true);
    assert!(!b.is_focused());
    assert!(ring_of(&nodes(|c, t| b.render(c, B, t))).is_none());

    let mut cb = Checkbox::new(false).disabled(true);
    cb.set_focused(true);
    assert!(!cb.is_focused());

    let mut sw = Switch::new(false).disabled(true);
    sw.set_focused(true);
    assert!(!sw.is_focused());

    let mut sl = Slider::new(0.0, 1.0, 0.5).disabled(true);
    sl.set_focused(true);
    assert!(!sl.is_focused());

    let mut se = Select::new(["A"], 0).disabled(true);
    se.set_focused(true);
    assert!(!se.is_focused());
}

#[test]
fn focus_ring_uses_accent_under_every_builtin_theme() {
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        let mut c = Compositor::new();
        c.begin_frame();
        let mut b = Button::new("Go");
        b.set_focused(true);
        b.render(&mut c, B, &theme);
        let accent = theme.colors.accent.0;
        let found = c.layer(LayerId::DEFAULT).unwrap().nodes().iter().any(|n| {
            matches!(*n, SceneNode::RoundedRect { border_color, border_width, color, .. }
                if border_color == accent && border_width == theme.control.focus_ring_width && color[3] == 0.0)
        });
        assert!(found, "ring must resolve from the theme accent");
    }
}

/// Every ring in `nodes`: the stroke `focus_ring` draws, at its corner.
fn rings_of(nodes: &[SceneNode]) -> Vec<(f32, f32, f32, f32)> {
    let theme = Theme::hoff();
    let accent = theme.colors.accent.0;
    let ring_w = theme.control.focus_ring_width;
    nodes
        .iter()
        .filter_map(|n| match *n {
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                color,
                border_width,
                border_color,
                ..
            } if color[3] == 0.0 && border_width == ring_w && border_color == accent => {
                Some((x, y, w, h))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_filter_chip_rings_and_a_static_one_refuses_focus() {
    let mut chip = Chip::new("Rust").selectable(true);
    assert!(ring_of(&nodes(|c, t| chip.render(c, B, t))).is_none());
    chip.set_focused(true);
    let ring = ring_of(&nodes(|c, t| chip.render(c, B, t))).expect("focused chip must ring");
    assert_eq!((ring.0, ring.1), (B.x - 4.0, B.y - 4.0));
    let mut plain = Chip::new("Draft");
    plain.set_focused(true);
    assert!(!plain.is_focused(), "a static chip is no control");
}

#[test]
fn a_crumb_and_a_nav_link_ring_only_when_focused() {
    let wide = Rect::new(0.0, 0.0, 600.0, 32.0);
    let mut trail = Breadcrumb::new(["Home", "Docs", "Guide"]);
    assert!(rings_of(&nodes(|c, t| trail.render(c, wide, t))).is_empty());
    trail.set_focused(Some(1));
    assert_eq!(trail.focused(), Some(1));
    let crumb = trail.crumb_rects(wide, &Theme::hoff())[1].1;
    let rings = rings_of(&nodes(|c, t| trail.render(c, wide, t)));
    assert_eq!(rings.len(), 1, "one ring, on the focused crumb");
    assert_eq!((rings[0].0, rings[0].1), (crumb.x - 4.0, crumb.y - 4.0));
    trail.set_focused(Some(9));
    assert_eq!(trail.focused(), None, "no such crumb");

    let mut link = NavLink::new("Inbox");
    assert!(ring_of(&nodes(|c, t| link.render(c, B, t))).is_none());
    link.set_focused(true);
    assert!(link.is_focused());
    let ring = ring_of(&nodes(|c, t| link.render(c, B, t))).expect("focused link must ring");
    assert_eq!((ring.0, ring.1), (B.x - 4.0, B.y - 4.0));
}

#[test]
fn a_focused_tab_and_a_focused_tree_row_ring_inside_themselves() {
    let theme = Theme::hoff();
    let mut tabs = Tabs::new(["One", "Two", "Three"]);
    tabs.set_focused_tab(Some(2));
    assert_eq!(tabs.focused_tab(), Some(2));
    let segment = tabs.item_rects(B, &theme)[2];
    let rings = rings_of(&nodes(|c, t| tabs.render(c, B, t)));
    assert_eq!(rings.len(), 1, "the segment rings, not the strip");
    assert_eq!(
        rings[0],
        (segment.x, segment.y, segment.w, segment.h),
        "inside the segment, its outer edge on the segment's"
    );
    tabs.set_focused_tab(Some(3));
    assert_eq!(tabs.focused_tab(), None, "no such tab");

    let mut tree = Tree::new(vec![
        TreeNode::branch(1, "src", vec![TreeNode::leaf(2, "main.rs")]),
        TreeNode::leaf(3, "README.md"),
    ]);
    let bounds = Rect::new(0.0, 0.0, 240.0, 200.0);
    tree.set_focused(Some(3));
    assert_eq!(tree.focused(), Some(3));
    let row_h = tree.row_height(&theme);
    let rings = rings_of(&nodes(|c, t| tree.render(c, bounds, t)));
    assert_eq!(rings, vec![(0.0, row_h, 240.0, row_h)], "the second row");
    tree.set_focused(Some(2));
    assert!(
        rings_of(&nodes(|c, t| tree.render(c, bounds, t))).is_empty(),
        "a row under a collapsed branch shows no ring"
    );
}

#[test]
fn the_modal_rings_the_button_it_is_told_and_reports_it() {
    let mut modal = Modal::new("Apply?", "It reloads.", "Apply", "Cancel");
    let draw = |m: &Modal| nodes(|c, t| m.render(c, LayerId::DEFAULT, t, 800.0, 600.0));
    assert!(rings_of(&draw(&modal)).is_empty(), "no focus, no ring");
    let theme = Theme::hoff();
    let dialog = modal.dialog_rect(&theme, 800.0, 600.0);
    let (confirm, cancel) = modal.button_rects(&theme, dialog);
    for (button, rect) in [
        (ModalAction::Cancel, cancel),
        (ModalAction::Confirm, confirm),
    ] {
        modal.set_focused(button);
        assert_eq!(modal.focused(), button);
        let rings = rings_of(&draw(&modal));
        assert_eq!(rings.len(), 1, "{button:?}");
        assert_eq!((rings[0].0, rings[0].1), (rect.x - 4.0, rect.y - 4.0));
    }
    modal.set_focused(ModalAction::None);
    assert_eq!(modal.focused(), ModalAction::None);
    assert!(rings_of(&draw(&modal)).is_empty());
}

#[test]
fn the_keys_move_the_one_highlight_of_a_menu_and_of_an_open_select() {
    let mut menu = ContextMenu::new(vec![
        MenuEntry::item(7, "Copy"),
        MenuEntry::Separator,
        MenuEntry::item(9, "Paste"),
    ]);
    menu.set_highlighted(Some(9));
    assert_eq!(menu.hovered(), Some(2), "the entry of item 9");
    menu.set_highlighted(Some(8));
    assert_eq!(menu.hovered(), None, "no such item");

    let theme = Theme::hoff();
    let options: Vec<String> = (0..40).map(|i| format!("option {i}")).collect();
    let id = sem::SemId::name("pick").expect("an id");
    let mut select = Select::new(options, 0).sem(id.clone());
    select.set_highlighted(Some(3));
    assert_eq!(
        select.highlighted(),
        None,
        "a closed panel highlights nothing"
    );
    select.open();
    select.set_highlighted(Some(39));
    assert_eq!(select.highlighted(), Some(39));
    let mut c = Compositor::new();
    c.begin_frame();
    select.render(&mut c, B, &theme);
    select.render_dropdown(&mut c, LayerId::DEFAULT, B, &theme);
    assert!(
        c.binding(&id.key("39")).is_some(),
        "the panel scrolled to show the highlighted option"
    );
    assert!(c.binding(&id.key("0")).is_none(), "and the first is out");
}
