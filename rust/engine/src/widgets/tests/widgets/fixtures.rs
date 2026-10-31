//! the measured fixtures: a widget rendered under a theme must emit exactly the nodes its spec says.

use super::*;

#[test]
fn core_primary_button_is_the_kit_recipe() {
    // The primary surface: the drop shadow first, then the ring (a
    // spread-1 shadow), the gradient fill, the inset key-light, the label.
    let theme = Theme::hoff_core();
    let mut c = Compositor::new();
    c.begin_frame();
    let b = Button::new("Button");
    b.render(&mut c, Rect::new(0.0, 0.0, 120.0, 40.0), &theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    let drops = nodes
        .iter()
        .filter(|n| matches!(n, SceneNode::Shadow { inset: false, .. }))
        .count();
    let insets = nodes
        .iter()
        .filter(|n| matches!(n, SceneNode::Shadow { inset: true, .. }))
        .count();
    assert_eq!(drops, 2, "0 3px 4px -1px black/15 and the D4D4D4 ring");
    assert_eq!(insets, 1, "inset 0 1px 0 white/33");
    let fill = nodes
        .iter()
        .find(|n| matches!(n, SceneNode::GradientRect { .. }))
        .expect("gradient fill");
    match fill {
        SceneNode::GradientRect {
            color,
            color2,
            corner_radius,
            ..
        } => {
            assert!((color[0] - 0xE5 as f32 / 255.0).abs() < 1e-5, "E5E5E5 top");
            assert!(
                (color2[0] - 0xE2 as f32 / 255.0).abs() < 1e-5,
                "E2E2E2 bottom"
            );
            assert_eq!(*corner_radius, 12.0, "rounded-xl");
        }
        other => panic!("expected the gradient fill, got {other:?}"),
    }
    assert!(matches!(nodes.last(), Some(SceneNode::Text { .. })));
}

#[test]
fn hoff_switch_track_and_knob_match_spec() {
    let theme = Theme::hoff();
    let mut c = Compositor::new();
    c.begin_frame();
    let sw = Switch::new(false);
    sw.render(&mut c, Rect::new(0.0, 0.0, 44.0, 24.0), &theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();

    match nodes[0] {
        SceneNode::RoundedRect {
            w,
            h,
            corner_radius,
            color,
            ..
        } => {
            assert_eq!((w, h), (44.0, 24.0));
            assert_eq!(corner_radius, 12.0);
            assert!((color[3] - 0.05).abs() < 1e-5, "off track rgba($n2,.05)");
        }
        ref other => panic!("expected track, got {other:?}"),
    }
    match nodes[1] {
        SceneNode::GradientRect { x, y, w, h, .. } => {
            assert_eq!((w, h), (16.0, 16.0), "16px knob");
            assert_eq!((x, y), (4.0, 4.0), "knob rests at (4,4)");
        }
        ref other => panic!("expected knob, got {other:?}"),
    }
}

#[test]
fn hoff_switch_on_knob_is_white_gradient() {
    let theme = Theme::hoff();
    let mut sw = Switch::new(true);
    // Settle the spring at the on position.
    for _ in 0..300 {
        sw.tick(1.0 / 60.0);
    }
    let mut c = Compositor::new();
    c.begin_frame();
    sw.render(&mut c, Rect::new(0.0, 0.0, 44.0, 24.0), &theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    match nodes[1] {
        SceneNode::GradientRect {
            x, color, color2, ..
        } => {
            assert!((x - 24.0).abs() < 0.1, "knob travelled 20px");
            assert!((color[3] - 0.90).abs() < 0.01, "top stop .90");
            assert!((color2[3] - 0.30).abs() < 0.01, "bottom stop .30");
        }
        ref other => panic!("expected gradient knob, got {other:?}"),
    }
}

#[test]
fn hoff_tooltip_is_solid_262626() {
    let theme = Theme::hoff();
    let mut tip = Tooltip::new("hint").delay(0.0);
    tip.set_hover(true, Rect::new(100.0, 100.0, 50.0, 20.0));
    tip.tick(0.1);
    let mut c = Compositor::new();
    c.begin_frame();
    let layer = c.create_layer(500);
    tip.render(&mut c, layer, &theme, 800.0, 600.0);
    let nodes = c.layer(layer).unwrap().nodes().to_vec();
    assert!(matches!(nodes[0], SceneNode::Shadow { .. }));
    assert!(
        matches!(nodes[1], SceneNode::RoundedRect { color, corner_radius, .. }
        if (color[0] - 0x26 as f32 / 255.0).abs() < 1e-5 && corner_radius == 8.0)
    );
}

/// The rim's bottom stop: whole under the core kit (a plain border), gone
/// under the HOFF glass (the top-lit edge), from the same call.
#[test]
fn a_rim_is_a_plain_border_in_core_and_top_lit_in_hoff() {
    let rect = Rect::new(0.0, 0.0, 100.0, 40.0);
    let edge = [0.5, 0.5, 0.5, 0.8];
    let bottom = |theme: &Theme| match glass_pill(rect, 12.0, edge, 1.0, [1.0; 4], theme)[0] {
        SceneNode::GradientRect { color2, .. } => color2[3],
        ref other => panic!("expected the rim underlay, got {other:?}"),
    };
    assert_eq!(bottom(&Theme::hoff_core()), 0.8);
    assert_eq!(bottom(&Theme::hoff()), 0.0);
    assert_eq!(bottom(&Theme::dark()), 0.0);
}

/// The core kit lights no surface; the HOFF glass keeps its
/// `inset 2px 4px 16px` glint.
#[test]
fn only_the_glass_themes_light_a_surface() {
    let count = |theme: &Theme| {
        let mut c = Compositor::new();
        c.begin_frame();
        inset_keylight(
            &mut c,
            LayerId::DEFAULT,
            Rect::new(0.0, 0.0, 80.0, 40.0),
            8.0,
            theme,
        );
        c.layer(LayerId::DEFAULT).map_or(0, |l| l.nodes().len())
    };
    assert_eq!(count(&Theme::hoff_core()), 0);
    assert_eq!(count(&Theme::hoff()), 1);
    let spec = Theme::hoff().glass.keylight.expect("hoff key-light");
    assert_eq!(
        (spec.offset, spec.blur, spec.inset),
        ([2.0, 4.0], 16.0, true)
    );
}

/// A css ring is never painted under its box: on a clear surface (the
/// derived themes' outline) it is a band just outside the bounds, not a
/// shadow that would fill the button.
#[test]
fn a_ring_on_a_clear_surface_is_a_band_outside_it() {
    let theme = Theme::hoff();
    let mut c = Compositor::new();
    c.begin_frame();
    let b = IconButton::new("x").variant(IconButtonVariant::Outline);
    let at = Rect::new(10.0, 10.0, 40.0, 40.0);
    b.render(&mut c, at, &theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    assert!(
        !nodes.iter().any(|n| matches!(n, SceneNode::Shadow { .. })),
        "no shadow under a clear outline"
    );
    let band = nodes.iter().find_map(|n| match n {
        SceneNode::RoundedRect {
            x,
            w,
            border_width,
            color,
            ..
        } if *border_width > 0.0 => Some((*x, *w, color[3])),
        _ => None,
    });
    let (x, w, fill) = band.expect("the ring as a band");
    assert_eq!((x, w, fill), (9.0, 42.0, 0.0));
}
