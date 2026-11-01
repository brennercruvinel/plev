//! action widgets: button, icon button, chip.

use super::*;
use crate::styling::theme::TypeRole;

// Button

#[test]
fn button_click_fires_on_release_inside() {
    let mut b = Button::new("Save");
    assert!(!b.handle_event(&down(20.0, 20.0), B).clicked);
    assert!(b.is_pressed());
    assert!(b.handle_event(&up(20.0, 20.0), B).clicked);
    assert!(!b.is_pressed());
}

#[test]
fn button_press_cancelled_by_release_outside() {
    let mut b = Button::new("Save");
    b.handle_event(&down(20.0, 20.0), B);
    let r = b.handle_event(&up(500.0, 500.0), B);
    assert!(!r.clicked);
    assert!(r.changed, "visual pressed state must clear");
}

#[test]
fn button_disabled_ignores_clicks() {
    let mut b = Button::new("Save").disabled(true);
    let r = click(|e| b.handle_event(e, B));
    assert!(!r.clicked);
    assert!(!b.is_pressed());
}

#[test]
fn button_hover_state_tracks_pointer() {
    let mut b = Button::new("Save");
    assert!(b.handle_event(&move_to(20.0, 20.0), B).changed);
    assert!(b.is_hovered());
    assert!(b.handle_event(&move_to(500.0, 20.0), B).changed);
    assert!(!b.is_hovered());
    // No change -> no redraw request.
    assert_eq!(
        b.handle_event(&move_to(500.0, 20.0), B),
        EventResult::IGNORED
    );
}

#[test]
fn button_preferred_size_scales_with_size_variant() {
    let sm = Button::new("Commit")
        .size(ButtonSize::Sm)
        .preferred_size(&hoff());
    let md = Button::new("Commit")
        .size(ButtonSize::Md)
        .preferred_size(&hoff());
    assert!(md.0 > sm.0);
    assert!(md.1 > sm.1);
    assert_eq!(sm.1, ButtonSize::Sm.height(&hoff()));
}

#[test]
fn button_renders_nodes_for_all_variants() {
    for theme in [Theme::dark(), Theme::hoff_core()] {
        for variant in [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Warning,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
        ] {
            let b = Button::new("Go").variant(variant).icon("link");
            let mut c = Compositor::new();
            b.render(&mut c, B, &theme);
            // Smoke check: render must not panic and must emit something.
        }
    }
}

#[test]
fn button_destructive_intent_is_the_warning_surface() {
    let b = Button::new("Delete").intent(Intent::Destructive);
    assert_eq!(b.variant, ButtonVariant::Warning);
    let b = Button::new("Save").intent(Intent::Constructive);
    assert_eq!(b.variant, ButtonVariant::Primary);
}

#[test]
fn button_sizes_follow_the_kit_ladder() {
    let t = Theme::hoff_core();
    assert_eq!(ButtonSize::Xs.height(&t), 28.0);
    assert_eq!(ButtonSize::Sm.height(&t), 36.0);
    assert_eq!(ButtonSize::Md.height(&t), 40.0);
    assert_eq!(ButtonSize::Lg.height(&t), 44.0);
    // The label style is the kit's, in Inter.
    let style = ButtonSize::Md.text_style(&t);
    assert_eq!(style.font_size, 14.0);
    assert_eq!(style.line_height, 20.0);
    assert_eq!(style.font_weight, 600);
    assert_eq!(style.font_family.as_deref(), Some("Inter"));
}

#[test]
fn submit_button_turns_dark_on_hover_and_while_active() {
    let t = Theme::hoff_core();
    let mut c = Compositor::new();
    SubmitButton::new().render(&mut c, B, &t);
    let mut s = SubmitButton::new();
    assert!(s.handle_event(&move_to(20.0, 20.0), B).changed);
    assert!(s.is_hovered());
    let mut c = Compositor::new();
    s.render(&mut c, B, &t);
    let mut c = Compositor::new();
    SubmitButton::new()
        .active(true)
        .disabled(true)
        .render(&mut c, B, &t);
}

/// The kit's SubmitButton: size-10 rounded-xl, `isMedium` size-8
/// rounded-[0.625rem], the size-4 arrow in both; the pale isPrimary
/// surface at rest, the dark isSecondary one lit, eased to 90% over the
/// pale when hovered while active; opacity-40 when disabled. Every
/// theme gets the same ladder and a lit surface that is its call to
/// action.
#[test]
fn submit_button_is_the_kit_s() {
    let t = Theme::hoff_core();
    let s = &t.button.submit;
    let at = |size: SubmitButtonSize| {
        let m = size.metrics(&t);
        (m.size, m.radius, m.icon)
    };
    assert_eq!(at(SubmitButtonSize::Lg), (40.0, 12.0, 16.0));
    assert_eq!(at(SubmitButtonSize::Md), (32.0, 10.0, 16.0));
    assert_eq!(SubmitButton::new().preferred_size(&t), (40.0, 40.0));
    assert_eq!(
        SubmitButton::new()
            .size(SubmitButtonSize::Md)
            .preferred_size(&t),
        (32.0, 32.0)
    );
    assert_eq!(s.rest, t.button.primary.rest);
    assert_eq!(s.lit, t.button.secondary.rest);
    assert_eq!(s.disabled_alpha, 0.40);
    // 90% of #323232 over #E5E5E5 at the top stop.
    let top = s.busy_hover.fill[0].0;
    let want = (0.9 * 50.0 + 0.1 * 229.0) / 255.0;
    assert!((top[0] - want).abs() < 1e-4 && top[3] == 1.0, "{top:?}");
    for theme in [Theme::hoff(), Theme::dark()] {
        assert_eq!(theme.button.submit.sizes, s.sizes);
        assert_eq!(theme.button.submit.lit, theme.button.secondary.rest);
    }
}

// IconButton

#[test]
fn icon_button_click_contract_matches_button() {
    let mut b = IconButton::new("copy");
    assert!(!b.handle_event(&down(20.0, 20.0), B).clicked);
    assert!(b.is_pressed());
    assert!(b.handle_event(&up(20.0, 20.0), B).clicked);

    let mut b = IconButton::new("copy").disabled(true);
    let r = click(|e| b.handle_event(e, B));
    assert!(!r.clicked);
}

#[test]
fn icon_button_is_square_and_renders_all_variants() {
    let (w, h) = IconButton::new("x").preferred_size(&hoff());
    assert_eq!(w, h);
    for theme in [Theme::hoff(), Theme::hoff_core()] {
        for variant in [
            IconButtonVariant::Outline,
            IconButtonVariant::Ghost,
            IconButtonVariant::Toolbar,
        ] {
            for size in [IconButtonSize::Sm, IconButtonSize::Md, IconButtonSize::Lg] {
                let mut c = Compositor::new();
                IconButton::new("trash")
                    .variant(variant)
                    .size(size)
                    .active(true)
                    .render(&mut c, B, &theme);
            }
        }
    }
}

// Chip

#[test]
fn chip_preferred_size_uses_real_measurement() {
    let chip = Chip::new("ann");
    let (w, h) = chip.preferred_size(&hoff());
    assert_eq!(h, Chip::height(&hoff()));
    let (tw, _) = crate::drawing::text::TextMeasurer::measure_styled(
        "ann",
        &crate::styling::theme::TypographyScale::hoff().run(TypeRole::Caption.strong()),
        None,
    );
    assert!((w - (tw + 24.0)).abs() < 1.0, "width is text + padding");
}

#[test]
fn chip_is_static_by_default_and_clicks_when_interactive() {
    let mut chip = Chip::new("tag");
    let r = click(|e| chip.handle_event(e, B));
    assert_eq!(r, EventResult::IGNORED, "static chips swallow nothing");

    let mut chip = Chip::new("tag").interactive(true);
    let r = click(|e| chip.handle_event(e, B));
    assert!(r.clicked);
    // Press cancelled by releasing outside.
    chip.handle_event(&down(20.0, 20.0), B);
    let r = chip.handle_event(&up(500.0, 500.0), B);
    assert!(!r.clicked);
    assert!(r.changed);
}

#[test]
fn chip_renders_every_intent_and_variant_without_gpu() {
    let theme = Theme::hoff();
    for intent in [
        Intent::Neutral,
        Intent::Constructive,
        Intent::Destructive,
        Intent::Informational,
    ] {
        for selected in [false, true] {
            let mut c = Compositor::new();
            Chip::new("capability")
                .intent(intent)
                .selected(selected)
                .render(&mut c, B, &theme);
        }
    }
}
