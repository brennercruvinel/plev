//! The scale tokens against the HOFF spec values, and the derived paths
//! for palettes that only declare a radius ladder and a shadow color.
//! The radii / shadow / motion assertions used to live in the ide's own
//! theme copy; they guard the same numbers here, at the single source.

use super::*;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-6
}

fn assert_color(c: crate::color::Color, r: u8, g: u8, b: u8, a: f32) {
    let [cr, cg, cb, ca] = c.0;
    assert!(close(cr, r as f32 / 255.0), "r: {cr} != {r}");
    assert!(close(cg, g as f32 / 255.0), "g: {cg} != {g}");
    assert!(close(cb, b as f32 / 255.0), "b: {cb} != {b}");
    assert!(close(ca, a), "a: {ca} != {a}");
}

#[test]
fn hoff_shape_matches_the_radii_table() {
    let s = Theme::hoff().shape;
    assert_eq!(s.pill, 32.0);
    assert_eq!(s.dropdown, 24.0);
    assert_eq!(s.tabs, 22.0);
    assert_eq!(s.card, 20.0);
    assert_eq!(s.block, 18.0);
    assert_eq!(s.item, 16.0);
    assert_eq!(s.nav, 12.0);
    assert_eq!(s.cluster, 10.0);
    assert_eq!(s.tooltip, 8.0);
    assert_eq!(s.micro, 6.0);
}

#[test]
fn hoff_shape_agrees_with_the_radius_ladder() {
    // The ladder and the role table are two views of the same spec.
    let t = Theme::hoff();
    assert_eq!(t.shape.pill, t.radius.xl);
    assert_eq!(t.shape.card, t.radius.lg);
    assert_eq!(t.shape.nav, t.radius.md);
    assert_eq!(t.shape.tooltip, t.radius.sm);
    assert_eq!(t.shape.tabs, t.shape.block + t.control.tabs_pad);
}

#[test]
fn derived_shape_keeps_role_ordering() {
    // A non-HOFF ladder still yields pill >= dropdown >= card >= nav >=
    // tooltip >= micro, so widgets never get a tighter card than a chip.
    for name in ["plev", "catppuccin", "nord", "github-dark"] {
        let t = Theme::named(name).unwrap();
        let s = &t.shape;
        assert!(s.pill >= s.dropdown, "{name}");
        assert!(s.dropdown >= s.card, "{name}");
        assert!(s.card >= s.nav, "{name}");
        assert!(s.nav >= s.tooltip, "{name}");
        assert!(s.tooltip >= s.micro, "{name}");
        assert!(s.tabs > s.block, "{name}");
    }
}

#[test]
fn control_heights_are_the_five_hoff_controls() {
    let c = Theme::hoff().control;
    assert_eq!(c.height(ControlSize::Xs), 36.0);
    assert_eq!(c.height(ControlSize::Sm), 40.0);
    assert_eq!(c.height(ControlSize::Md), 44.0);
    assert_eq!(c.height(ControlSize::Lg), 48.0);
    assert_eq!(c.height(ControlSize::Xl), 52.0);
    assert_eq!(c.pad_x(ControlSize::Sm), 12.0);
    assert_eq!(c.pad_x(ControlSize::Md), 24.0);
    assert_eq!(c.pad_x(ControlSize::Xl), 32.0);
    // Heights are monotonic in the ladder.
    let hs: Vec<f32> = ControlSize::ALL.iter().map(|s| c.height(*s)).collect();
    assert!(hs.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn touch_height_never_drops_under_the_touch_minimum() {
    let c = Theme::hoff().control;
    for size in ControlSize::ALL {
        assert!(c.touch_height(size) >= c.min_touch_target);
    }
    assert_eq!(c.touch_height(ControlSize::Xs), 44.0);
    assert_eq!(c.touch_height(ControlSize::Xl), 52.0);
}

#[test]
fn icon_sizes_follow_the_control_ladder() {
    let c = Theme::hoff().control;
    assert_eq!(c.icon(IconSize::Sm), 16.0);
    assert_eq!(c.icon(IconSize::Md), 20.0);
    assert_eq!(c.icon(IconSize::Lg), 24.0);
    assert_eq!(c.icon_for(ControlSize::Sm), 16.0);
    assert_eq!(c.icon_for(ControlSize::Md), 20.0);
    assert_eq!(c.icon_for(ControlSize::Xl), 24.0);
}

#[test]
fn container_sizes_match_the_spec() {
    let s = Theme::hoff().size;
    assert_eq!(s.menu_w, 240.0);
    assert_eq!(s.modal_max_w, 400.0);
    assert_eq!(s.sidebar_w, 248.0);
    assert!(s.sidebar_rail_w < s.sidebar_w);
    assert!(s.card_min_w < s.readable_max_w);
}

#[test]
fn durations_match_the_spec_transitions() {
    let d = Theme::hoff().duration;
    assert_eq!(d.base, 0.2);
    assert_eq!(d.slow, 0.3);
    assert!(d.fast < d.base && d.base < d.slow);
}

#[test]
fn breakpoints_split_the_width_axis_in_three() {
    let l = Theme::hoff().layout;
    assert_eq!(l.breakpoint(0.0), Breakpoint::Compact);
    assert_eq!(l.breakpoint(599.0), Breakpoint::Compact);
    assert_eq!(l.breakpoint(600.0), Breakpoint::Medium);
    assert_eq!(l.breakpoint(1023.0), Breakpoint::Medium);
    assert_eq!(l.breakpoint(1024.0), Breakpoint::Expanded);
    assert_eq!(l.breakpoint(2560.0), Breakpoint::Expanded);
}

#[test]
fn gutters_columns_and_sidebar_grow_with_the_breakpoint() {
    let l = Theme::hoff().layout;
    assert!(l.gutter(Breakpoint::Compact) < l.gutter(Breakpoint::Medium));
    assert!(l.gutter(Breakpoint::Medium) < l.gutter(Breakpoint::Expanded));
    assert!(l.columns(Breakpoint::Compact) < l.columns(Breakpoint::Expanded));
    assert_eq!(l.sidebar_mode(Breakpoint::Compact), SidebarMode::Drawer);
    assert_eq!(l.sidebar_mode(Breakpoint::Medium), SidebarMode::Rail);
    assert_eq!(l.sidebar_mode(Breakpoint::Expanded), SidebarMode::Full);
}

#[test]
fn fit_columns_is_the_gallery_grid_formula() {
    let l = Theme::hoff().layout;
    // gap 16, min 280: 280 -> 1, 576 -> 2, 872 -> 3, 871 -> 2.
    assert_eq!(l.fit_columns(280.0, 280.0), 1);
    assert_eq!(l.fit_columns(576.0, 280.0), 2);
    assert_eq!(l.fit_columns(872.0, 280.0), 3);
    assert_eq!(l.fit_columns(871.0, 280.0), 2);
    // Never zero, even when the content is narrower than one cell.
    assert_eq!(l.fit_columns(10.0, 280.0), 1);
}

#[test]
fn density_compact_shrinks_and_comfortable_is_identity() {
    assert_eq!(Density::Comfortable.factor(), 1.0);
    assert!(Density::Compact.factor() < 1.0);
    assert_eq!(Density::default(), Density::Comfortable);
}

#[test]
fn hoff_shadows_match_the_spec_values() {
    let s = Theme::hoff().shadows;
    assert_eq!(s.floating.offset, [0.0, 24.0]);
    assert_eq!(s.floating.blur, 32.0);
    assert_eq!(s.floating.spread, -12.0);
    assert_color(s.floating.color, 18, 18, 18, 0.10);

    assert_eq!(s.overlay[0].offset, [0.0, 24.0]);
    assert_eq!(s.overlay[0].blur, 24.0);
    assert_eq!(s.overlay[0].spread, -16.0);
    assert_color(s.overlay[0].color, 5, 5, 5, 0.09);
    assert_color(s.overlay[3].color, 5, 5, 5, 0.25);

    assert_eq!(s.hint.offset, [0.0, 1.5]);
    assert_color(s.hint.color, 24, 24, 24, 0.15);

    assert_eq!(s.raised.offset, [0.0, 8.0]);
    assert_eq!(s.raised.blur, 16.0);
    assert_eq!(s.raised.spread, -4.0);
    assert_color(s.raised.color, 18, 18, 18, 0.20);
}

#[test]
fn elevation_picks_the_stack_bottom_first() {
    let s = Theme::hoff().shadows;
    assert!(s.stack(Elevation::Base).is_empty());
    assert_eq!(s.stack(Elevation::Raised).len(), 1);
    assert_eq!(s.stack(Elevation::Floating).len(), 1);
    assert_eq!(s.stack(Elevation::Overlay).len(), 4);
    assert_eq!(s.stack(Elevation::Hint).len(), 1);
    // The deepest layer of the overlay stack is pushed first.
    assert_eq!(s.stack(Elevation::Overlay)[0].offset, [0.0, 24.0]);
}

#[test]
fn shadow_spread_folds_into_the_cast_rect() {
    let spec = ShadowSpec::new([0.0, 0.0], 8.0, -4.0, crate::color::Color::hex(0x000000));
    assert_eq!(
        spec.cast_rect(10.0, 10.0, 100.0, 50.0),
        Some((14.0, 14.0, 92.0, 42.0))
    );
    // A spread that eats the surface casts nothing.
    let eaten = ShadowSpec::new([0.0, 0.0], 8.0, -30.0, crate::color::Color::hex(0x000000));
    assert_eq!(eaten.cast_rect(0.0, 0.0, 50.0, 50.0), None);
}

#[test]
fn derived_shadows_keep_the_palette_hue() {
    let t = Theme::named("catppuccin").unwrap();
    let base = t.effects.shadow_color.0;
    for spec in t.shadows.stack(Elevation::Overlay) {
        assert_eq!(&spec.color.0[..3], &base[..3]);
        assert!(spec.color.0[3] > 0.0);
    }
}

#[test]
fn every_named_theme_carries_every_scale() {
    for name in [
        "hoff",
        "hoff-light",
        "plev",
        "catppuccin",
        "dracula",
        "tokyo-night",
        "rose-pine",
        "nord",
        "gruvbox",
        "github-dark",
        "one-dark",
        "kanagawa",
        "moonlight",
    ] {
        let t = Theme::named(name).unwrap();
        assert!(t.control.height(ControlSize::Md) > 0.0, "{name}");
        assert!(t.shape.card > 0.0, "{name}");
        assert!(t.size.modal_max_w > 0.0, "{name}");
        assert!(t.duration.base > 0.0, "{name}");
        assert!(t.layout.medium_max_w > t.layout.compact_max_w, "{name}");
    }
    let light = Theme::light();
    assert_eq!(light.control, Theme::dark().control);
}

#[test]
fn headline_and_mono_come_from_the_ramp() {
    let ty = Theme::hoff().typography;
    let h = ty.headline();
    assert_eq!(h.font_size, 32.0);
    assert_eq!(h.line_height, 40.0);
    assert_eq!(h.font_weight, 500);
    let m = ty.mono();
    assert_eq!(m.font_size, 14.0);
    assert_eq!(m.font_weight, 400);
    let r = ty.readout();
    assert_eq!(r.font_size, 10.0);
}

#[test]
fn hoff_light_inverts_the_ink_and_keeps_every_scale() {
    let dark = Theme::hoff();
    let light = Theme::hoff_light();
    let [r, g, b, a] = light.colors.text.0;
    assert!(r < 0.1 && g < 0.1 && b < 0.1, "light text is ink");
    assert!(close(a, 0.95));
    assert!(light.colors.bg.0[0] > 0.7, "the page is pale");
    assert!(close(light.glass.surface.0[3], 0.02));
    assert!(close(light.glass.surface_active.0[3], 0.10));
    assert_eq!(light.shape, dark.shape);
    assert_eq!(light.control, dark.control);
    assert_eq!(light.typography, dark.typography);
    assert_eq!(light.spacing, dark.spacing);
    assert_eq!(light.layout, dark.layout);
    assert_eq!(light.duration, dark.duration);
    assert_eq!(
        Theme::named("hoff-light").unwrap().colors.bg.0,
        light.colors.bg.0
    );
}
