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

/// The fills a switch draws, in order: the well, then the knob.
fn switch_fills(sw: &Switch, theme: &Theme) -> Vec<(Rect, f32, [f32; 4])> {
    let mut c = Compositor::new();
    c.begin_frame();
    sw.render(&mut c, Rect::new(0.0, 0.0, 40.0, 22.0), theme);
    c.layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter_map(|n| match *n {
            SceneNode::GradientRect {
                x,
                y,
                w,
                h,
                corner_radius,
                color,
                ..
            } => Some((Rect::new(x, y, w, h), corner_radius, color)),
            _ => None,
        })
        .collect()
}

/// The kit's switch (components/Switch/index.tsx:11, :17): a 40 by 22
/// pill well, `#ECECEC` off and ink at 70% on, the 18 knob at the `p-0.5`
/// pad, travelling `translate-x-4.5`.
#[test]
fn core_switch_is_the_kits_well_and_knob() {
    let theme = Theme::hoff_core();
    let off = switch_fills(&Switch::new(false), &theme);
    assert_eq!(off.len(), 2, "{off:?}");
    let (well, radius, fill) = off[0];
    assert_eq!((well, radius), (Rect::new(0.0, 0.0, 40.0, 22.0), 11.0));
    assert_eq!(fill, crate::styling::color::Color::hex(0xECECEC).0);
    assert_eq!(off[1].0, Rect::new(2.0, 2.0, 18.0, 18.0));
    let mut on = Switch::new(true);
    for _ in 0..300 {
        on.tick(1.0 / 60.0);
    }
    let on = switch_fills(&on, &theme);
    assert!((on[0].2[3] - 0.70).abs() < 1e-3, "ink at 70%");
    assert!((on[1].0.x - 20.0).abs() < 0.1, "the knob travels 18px");
}

/// A derived theme draws the same well and knob from its palette: the
/// hover tone off, the text at the kit's strength on, the page as knob.
#[test]
fn a_derived_switch_takes_the_palette() {
    let theme = Theme::hoff();
    let c = &theme.colors;
    let off = switch_fills(&Switch::new(false), &theme);
    assert_eq!(off[0].2, c.bg_hover.0);
    assert_eq!(off[1].2, c.bg.0);
    let mut sw = Switch::new(true);
    for _ in 0..300 {
        sw.tick(1.0 / 60.0);
    }
    let on = switch_fills(&sw, &theme)[0].2;
    assert_eq!(&on[..3], &c.text.0[..3]);
}

/// The checkbox is the switch's well: off the kit's `#ECECEC` with an
/// edge that reads, on the ink whole with the knob's paper as the check,
/// under the core kit and a derived theme alike.
#[test]
fn the_checkbox_is_the_switch_well_off_and_the_ink_on() {
    for theme in [Theme::hoff_core(), Theme::hoff()] {
        let s = &theme.selection;
        let fills = |checked: bool| {
            let mut c = Compositor::new();
            c.begin_frame();
            Checkbox::new(checked).render(&mut c, Rect::new(0.0, 0.0, 18.0, 18.0), &theme);
            c.layer(LayerId::DEFAULT)
                .unwrap()
                .nodes()
                .iter()
                .find_map(|n| match *n {
                    SceneNode::GradientRect { color, .. } => Some(color),
                    _ => None,
                })
                .unwrap()
        };
        assert_eq!(fills(false), s.switch.off.fill[0].0);
        let on = fills(true);
        assert_eq!((&on[..3], on[3]), (&s.switch.on.fill[0].0[..3], 1.0));
        assert!(!s.checkbox.off.shadow.is_empty(), "the edge");
    }
}

/// The kit's range: a 36 tall `#f1f1f1` well at radius 8 filled to the
/// value, and a 24 wide block thumb at radius 6 travelling inside it.
#[test]
fn core_slider_is_the_kits_range() {
    let theme = Theme::hoff_core();
    let mut c = Compositor::new();
    c.begin_frame();
    let slider = Slider::new(0.0, 100.0, 50.0);
    slider.render(&mut c, Rect::new(0.0, 0.0, 224.0, 40.0), &theme);
    let rects: Vec<(Rect, f32)> = c
        .layer(LayerId::DEFAULT)
        .unwrap()
        .nodes()
        .iter()
        .filter_map(|n| match *n {
            SceneNode::GradientRect {
                x,
                y,
                w,
                h,
                corner_radius,
                ..
            }
            | SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                corner_radius,
                ..
            } => Some((Rect::new(x, y, w, h), corner_radius)),
            _ => None,
        })
        .collect();
    let well = Rect::new(0.0, 2.0, 224.0, 36.0);
    assert!(rects.contains(&(well, 8.0)), "{rects:?}");
    assert!(
        rects.contains(&(Rect::new(0.0, 2.0, 112.0, 36.0), 8.0)),
        "half"
    );
    assert!(
        rects.contains(&(Rect::new(100.0, 2.0, 24.0, 36.0), 6.0)),
        "thumb"
    );
}

/// The kit's state tag: 11/16 medium in `px-3 py-1` and a border, 26
/// tall at `rounded-xl`; the quiet tag on paper ringed `#ECECEC`, the
/// chosen one `#F1F1F1` with the black text.
#[test]
fn core_chip_is_the_kits_tag() {
    let theme = Theme::hoff_core();
    let t = &theme.selection.chip;
    assert_eq!(Chip::height(&theme), 26.0);
    assert_eq!((t.radius, t.pad_x), (12.0, 12.0));
    let fill = |chip: Chip| {
        let mut c = Compositor::new();
        c.begin_frame();
        chip.render(&mut c, Rect::new(0.0, 0.0, 60.0, 26.0), &theme);
        c.layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .find_map(|n| match *n {
                SceneNode::GradientRect {
                    color,
                    corner_radius,
                    ..
                } => Some((color, corner_radius)),
                _ => None,
            })
            .unwrap()
    };
    let hex = |v| crate::styling::color::Color::hex(v).0;
    assert_eq!(fill(Chip::new("tag")), (hex(0xFCFCFC), 12.0));
    assert_eq!(fill(Chip::new("tag").selected(true)), (hex(0xF1F1F1), 12.0));
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
