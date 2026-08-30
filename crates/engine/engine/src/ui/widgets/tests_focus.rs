//! Focus-state tests for the form widgets: every interactive control
//! draws the shared accent focus ring when focused, skips it when not,
//! and disabled controls refuse focus entirely.

use super::*;
use crate::compositor::{Compositor, LayerId, SceneNode};
use crate::theme::Theme;

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
    let accent = Theme::hoff().colors.accent.0;
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
        } if color[3] == 0.0 && border_width == FOCUS_RING_WIDTH && border_color == accent => {
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
            assert_eq!(border_width, FOCUS_RING_WIDTH);
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
                if border_color == accent && border_width == FOCUS_RING_WIDTH && color[3] == 0.0)
        });
        assert!(found, "ring must resolve from the theme accent");
    }
}
