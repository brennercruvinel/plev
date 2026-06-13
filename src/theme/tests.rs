#[cfg(test)]
// The inner module only carries the cfg(test) gate for this
// tests.rs file; the same-name nesting is deliberate.
#[allow(clippy::module_inception)]
mod tests {
    use crate::theme::*;

    // -- Theme construction --

    #[test]
    fn dark_theme_creates() {
        let theme = Theme::dark();
        // plev palette: bg is #000000
        assert!(theme.colors.bg.0[0] < 0.01);
        assert_eq!(theme.typography.body, 16.0);
        assert_eq!(theme.spacing.md, 16.0);
        assert_eq!(theme.radius.md, 4.0);
        assert_eq!(theme.motion.mass, 1.0);
    }

    #[test]
    fn light_theme_creates() {
        let theme = Theme::light();
        assert!(theme.colors.bg.0[0] > 0.9);
        assert_eq!(theme.typography.body, 16.0);
        assert_eq!(theme.spacing.md, 16.0);
    }

    #[test]
    fn light_shares_structural_scales_with_dark() {
        let dark = Theme::dark();
        let light = Theme::light();
        assert_eq!(dark.typography, light.typography);
        assert_eq!(dark.spacing, light.spacing);
        assert_eq!(dark.radius, light.radius);
        assert_eq!(dark.motion, light.motion);
    }

    // -- Intent -> Color --

    #[test]
    fn intent_color_destructive_is_red() {
        let theme = Theme::dark();
        let danger = theme.intent_color(Intent::Destructive);
        assert!(danger.0[0] > 0.9);
        assert!(danger.0[1] < 0.4);
    }

    #[test]
    fn intent_color_constructive_is_green() {
        let theme = Theme::dark();
        let success = theme.intent_color(Intent::Constructive);
        assert!(success.0[1] > 0.7);
    }

    #[test]
    fn intent_color_informational_is_cyan() {
        let theme = Theme::dark();
        let info = theme.intent_color(Intent::Informational);
        assert!(info.0[1] > 0.8); // green
        assert!(info.0[2] > 0.9); // blue
    }

    #[test]
    fn all_intents_have_unique_colors() {
        let theme = Theme::dark();
        let intents = [
            Intent::Neutral,
            Intent::Constructive,
            Intent::Destructive,
            Intent::Informational,
        ];
        for i in 0..intents.len() {
            for j in (i + 1)..intents.len() {
                let c1 = theme.intent_color(intents[i]);
                let c2 = theme.intent_color(intents[j]);
                assert_ne!(
                    c1, c2,
                    "{:?} and {:?} should differ",
                    intents[i], intents[j]
                );
            }
        }
    }

    // -- Intent -> Motion --

    #[test]
    fn intent_motion_neutral_unchanged() {
        let theme = Theme::dark();
        let neutral = theme.intent_motion(Intent::Neutral);
        assert_eq!(neutral.stiffness, theme.motion.stiffness);
        assert_eq!(neutral.damping, theme.motion.damping);
        assert_eq!(neutral.mass, theme.motion.mass);
    }

    #[test]
    fn intent_motion_destructive_snappier() {
        let theme = Theme::dark();
        let d = theme.intent_motion(Intent::Destructive);
        assert!(d.stiffness > theme.motion.stiffness);
        assert!(d.mass < theme.motion.mass);
        // Faster settling
        assert!(d.settling_time() < theme.motion.settling_time());
    }

    #[test]
    fn intent_motion_constructive_smoother() {
        let theme = Theme::dark();
        let c = theme.intent_motion(Intent::Constructive);
        assert!(c.stiffness < theme.motion.stiffness);
        assert!(c.damping > theme.motion.damping);
    }

    #[test]
    fn intent_motion_informational_gentle() {
        let theme = Theme::dark();
        let i = theme.intent_motion(Intent::Informational);
        assert!(i.stiffness < theme.motion.stiffness);
        assert!(i.mass > theme.motion.mass);
    }

    #[test]
    fn all_intents_have_different_motion() {
        let theme = Theme::dark();
        let intents = [
            Intent::Constructive,
            Intent::Destructive,
            Intent::Informational,
        ];
        for intent in &intents {
            let m = theme.intent_motion(*intent);
            assert_ne!(
                m.stiffness, theme.motion.stiffness,
                "{intent:?} motion should differ from neutral",
            );
        }
    }

    // -- MotionPhysics --

    #[test]
    fn damping_ratio_critical() {
        let m = MotionPhysics {
            mass: 1.0,
            stiffness: 100.0,
            damping: 20.0,
        };
        assert!((m.damping_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn damping_ratio_underdamped() {
        let m = MotionPhysics {
            mass: 1.0,
            stiffness: 100.0,
            damping: 10.0,
        };
        assert!(m.damping_ratio() < 1.0);
    }

    #[test]
    fn damping_ratio_overdamped() {
        let m = MotionPhysics {
            mass: 1.0,
            stiffness: 100.0,
            damping: 30.0,
        };
        assert!(m.damping_ratio() > 1.0);
    }

    #[test]
    fn natural_frequency_positive() {
        let theme = Theme::dark();
        let freq = theme.motion.natural_frequency();
        assert!(freq > 0.0, "freq={freq}");
    }

    #[test]
    fn settling_time_finite() {
        let theme = Theme::dark();
        let t = theme.motion.settling_time();
        assert!(t > 0.0 && t < 10.0, "settling={t}s");
    }

    #[test]
    fn to_spring_config_matches() {
        let m = MotionPhysics {
            mass: 1.5,
            stiffness: 200.0,
            damping: 25.0,
        };
        let (s, d, mass) = m.to_spring_config();
        assert_eq!(s, 200.0);
        assert_eq!(d, 25.0);
        assert_eq!(mass, 1.5);
    }

    #[test]
    fn spring_config_valid_for_all_intents() {
        let theme = Theme::dark();
        for intent in [
            Intent::Neutral,
            Intent::Constructive,
            Intent::Destructive,
            Intent::Informational,
        ] {
            let m = theme.intent_motion(intent);
            let (s, d, mass) = m.to_spring_config();
            assert!(s > 0.0, "{intent:?}: stiffness must be positive");
            assert!(d > 0.0, "{intent:?}: damping must be positive");
            assert!(mass > 0.0, "{intent:?}: mass must be positive");
            let ratio = m.damping_ratio();
            assert!(ratio > 0.0 && ratio < 10.0, "{intent:?}: ratio={ratio}");
        }
    }

    // -- Typography --

    #[test]
    fn line_height_computed() {
        let theme = Theme::dark();
        let lh = theme.typography.line_height(16.0);
        assert!((lh - 20.8).abs() < 0.01);
    }

    #[test]
    fn typography_scale_ordering() {
        let t = Theme::dark().typography;
        assert!(t.small < t.caption);
        assert!(t.caption < t.body_sm);
        assert!(t.body_sm < t.body);
        assert!(t.body < t.title_sm);
        assert!(t.title_sm < t.title);
        assert!(t.title < t.display);
    }

    /// Every HOFF mixin (variables.sass:39-127) with its exact size,
    /// line-height, weight and letter-spacing (0.025em -> px).
    #[test]
    fn hoff_type_ramp_matches_reference_mixins() {
        let t = Theme::hoff().typography;
        assert_eq!(t, TypographyScale::hoff());
        let cases = [
            ("h4", t.h4(), 36.0, 36.0, 500, 0.0),
            ("title", t.title(), 20.0, 24.0, 500, 0.0),
            ("base-r", t.base_r(), 16.0, 24.0, 400, 0.0),
            ("base-m", t.base_m(), 16.0, 24.0, 500, 0.0),
            ("base-2r", t.base_2r(), 14.0, 19.6, 400, 0.0),
            ("base-2m", t.base_2m(), 14.0, 19.6, 500, 0.0),
            ("base-2sm", t.base_2sm(), 14.0, 19.6, 600, 0.0),
            ("base-2b", t.base_2b(), 14.0, 19.6, 700, 0.0),
            ("caption-r", t.caption_r(), 12.0, 15.96, 400, 0.0),
            ("caption-sm", t.caption_sm(), 12.0, 15.96, 600, 0.0),
            ("small", t.small_r(), 10.0, 12.0, 400, 0.0),
            ("small-sm", t.small_sm(), 10.0, 12.0, 600, 0.0),
            ("body", t.body(), 16.0, 24.0, 400, 0.4),
            ("body-2r", t.body_2r(), 14.0, 23.8, 400, 0.35),
            ("body-2m", t.body_2m(), 14.0, 23.8, 500, 0.35),
            ("body-2b", t.body_2b(), 14.0, 23.8, 500, 0.35),
            ("hairline", t.hairline(), 12.0, 19.8, 500, 0.3),
        ];
        for (name, style, size, lh, weight, spacing) in cases {
            assert_eq!(style.font_size, size, "={name}: font-size");
            assert!(
                (style.line_height - lh).abs() < 1e-4,
                "={name}: line-height {} != {lh}",
                style.line_height
            );
            assert_eq!(style.font_weight, weight, "={name}: weight");
            assert!(
                (style.letter_spacing - spacing).abs() < 1e-4,
                "={name}: letter-spacing {} != {spacing}",
                style.letter_spacing
            );
        }
    }

    /// The Inter body family carries 0.025em tracking; the rest of the
    /// ramp none — exactly like the reference.
    #[test]
    fn hoff_letter_spacing_only_on_body_family() {
        let t = Theme::hoff().typography;
        for style in [
            t.h4(),
            t.title(),
            t.base_r(),
            t.base_2sm(),
            t.caption_r(),
            t.small_r(),
        ] {
            assert_eq!(style.letter_spacing, 0.0);
        }
        for style in [
            t.body(),
            t.body_2r(),
            t.body_2m(),
            t.body_2b(),
            t.hairline(),
        ] {
            assert!(
                (style.letter_spacing - 0.025 * style.font_size).abs() < 1e-4,
                "body-family style must track 0.025em"
            );
        }
    }

    // -- Effects --

    #[test]
    fn effect_tokens_dark() {
        let theme = Theme::dark();
        assert_eq!(theme.effects.shadow_sigma, 4.0);
        assert_eq!(theme.effects.blur_sigma, 8.0);
        assert!(theme.effects.shadow_color.0[3] > 0.0);
    }

    #[test]
    fn effect_tokens_inherited_by_light() {
        let dark = Theme::dark();
        let light = Theme::light();
        assert_eq!(dark.effects.shadow_sigma, light.effects.shadow_sigma);
        assert_eq!(dark.effects.blur_sigma, light.effects.blur_sigma);
    }

    // -- Spacing --

    #[test]
    fn spacing_scale_ordering() {
        let s = Theme::dark().spacing;
        assert!(s.xs < s.sm);
        assert!(s.sm < s.md);
        assert!(s.md < s.lg);
        assert!(s.lg < s.xl);
        assert!(s.xl < s.xxl);
    }

    // -- Radius --

    #[test]
    fn radius_scale_ordering() {
        let r = Theme::dark().radius;
        assert!(r.none < r.sm);
        assert!(r.sm < r.md);
        assert!(r.md < r.lg);
        assert!(r.lg < r.xl);
        assert!(r.xl < r.full);
    }

    // -- HOFF --

    /// rgba(r8, g8, b8, a) as [f32; 4] — mirrors the SASS source values.
    fn rgba8(r: u8, g: u8, b: u8, a: f32) -> [f32; 4] {
        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a]
    }

    #[test]
    fn hoff_is_the_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.colors.bg.0, Theme::hoff().colors.bg.0);
        assert_eq!(
            Theme::named("hoff").unwrap().colors.bg.0,
            Theme::hoff().colors.bg.0
        );
    }

    #[test]
    fn hoff_base_palette_exact() {
        use crate::theme::hoff;
        assert_eq!(hoff::N1.0, rgba8(0xff, 0xff, 0xff, 1.0));
        assert_eq!(hoff::N2.0, rgba8(0xf8, 0xf8, 0xf8, 1.0));
        assert_eq!(hoff::N3.0, rgba8(0x28, 0x28, 0x28, 1.0));
        assert_eq!(hoff::N4.0, rgba8(0x12, 0x12, 0x12, 1.0));
        // Hidden compositing frame (never on screen) vs the graphite tones
        // every screen actually sits on — measured live, not pure black.
        assert_eq!(hoff::BODY_FRAME.0, rgba8(0x44, 0x44, 0x44, 1.0));
        assert_eq!(hoff::PAGE_BG.0, rgba8(0x30, 0x30, 0x30, 1.0));
        assert_eq!(hoff::BG_SURFACE.0, rgba8(0x30, 0x30, 0x30, 1.0));
        assert_eq!(hoff::BG_SIDEBAR.0, rgba8(0x2e, 0x2e, 0x2e, 1.0));
        assert_eq!(hoff::CARD_OVERLAY.0, rgba8(0x2e, 0x2e, 0x2e, 1.0));
        assert_eq!(hoff::SCRIM.0, rgba8(35, 34, 34, 0.9));
        assert_eq!(hoff::POPOVER.0, rgba8(0x3b, 0x3b, 0x3b, 1.0));
        assert_eq!(hoff::TOOLTIP.0, rgba8(0x26, 0x26, 0x26, 1.0));
    }

    /// Parity contract for the page canvas: the live reference renders on
    /// graphite, never the GOLDEN_SPEC's "#0E0E0E black" nor the hidden
    /// #444444 body. Page == #303030, the rail one notch darker (#2E2E2E),
    /// and a glass card (.02 white over the page) reads a touch LIGHTER.
    #[test]
    fn hoff_page_is_measured_graphite_not_black() {
        let t = Theme::hoff();
        let bg = t.colors.bg.0;
        // Graphite, not black: every channel near 0.19 (#303030 = 48/255).
        assert!((bg[0] - 48.0 / 255.0).abs() < 1e-5, "page bg is #303030");
        assert!(bg[0] > 0.12 && bg[0] < 0.25, "graphite, not #0E0E0E black");
        assert_eq!(bg[3], 1.0, "page bg is opaque");
        // Rail/panel sits one notch darker than the page (measured #2E2E2E).
        assert!(t.colors.surface.0[0] < bg[0], "rail darker than the page");
        // A .02 white card glass composited over the page lands lighter.
        let card_white = t.glass.surface.0;
        let lifted = bg[0] * (1.0 - card_white[3]) + card_white[0] * card_white[3];
        assert!(lifted > bg[0], "a glass card reads lighter than the page");
    }

    #[test]
    fn hoff_text_alphas_exact() {
        let t = Theme::hoff();
        // $text-primary / $text-secondary / $text-tertiary over $n2.
        assert_eq!(t.colors.text.0, rgba8(248, 248, 248, 0.95));
        assert_eq!(t.colors.text_mid.0, rgba8(248, 248, 248, 0.70));
        assert_eq!(t.colors.text_dim.0, rgba8(248, 248, 248, 0.50));
        assert_eq!(t.glass.text_faint.0, rgba8(248, 248, 248, 0.40));
        assert_eq!(t.glass.text_placeholder.0, rgba8(248, 248, 248, 0.25));
    }

    #[test]
    fn hoff_accents_exact() {
        let t = Theme::hoff();
        assert_eq!(t.colors.danger.0, rgba8(0xBD, 0x30, 0x27, 1.0));
        assert_eq!(t.colors.success.0, rgba8(0x55, 0xF0, 0x8B, 1.0));
        assert_eq!(t.colors.info.0, rgba8(124, 255, 176, 0.7));
        assert_eq!(t.colors.warning.0, rgba8(255, 77, 0, 0.9));
    }

    #[test]
    fn hoff_glass_surfaces_exact() {
        let g = Theme::hoff().glass;
        assert_eq!(g.surface.0, rgba8(248, 248, 248, 0.02));
        assert_eq!(g.surface_hover.0, rgba8(248, 248, 248, 0.05));
        assert_eq!(g.surface_active.0, rgba8(248, 248, 248, 0.10));
        assert_eq!(g.button.0, rgba8(40, 40, 40, 0.70));
        assert_eq!(g.button_hover.0, rgba8(248, 248, 248, 0.10));
        assert_eq!(g.field.0, rgba8(248, 248, 248, 0.05));
        assert_eq!(g.field_focus_border.0, rgba8(248, 248, 248, 0.25));
        assert_eq!(g.edge.0, [1.0, 1.0, 1.0, 0.10]);
        assert_eq!(g.edge_soft.0, [1.0, 1.0, 1.0, 0.05]);
        assert_eq!(g.inset_highlight.0, rgba8(248, 248, 248, 0.06));
        assert_eq!(g.knob_gradient[0].0, rgba8(248, 248, 248, 0.90));
        assert_eq!(g.knob_gradient[1].0, rgba8(248, 248, 248, 0.30));
        assert_eq!(g.text_gradient[0].0, rgba8(248, 248, 248, 0.90));
        assert_eq!(g.text_gradient[1].0, rgba8(248, 248, 248, 0.50));
    }

    #[test]
    fn hoff_scales_exact() {
        let t = Theme::hoff();
        // Type ramp: caption 12 / base-2 14 / base 16 / title 20 /
        // headline 32 / h4 36.
        assert_eq!(t.typography.caption, 12.0);
        assert_eq!(t.typography.body_sm, 14.0);
        assert_eq!(t.typography.body, 16.0);
        assert_eq!(t.typography.title_sm, 20.0);
        assert_eq!(t.typography.title, 32.0);
        assert_eq!(t.typography.display, 36.0);
        // Radii: 8 tooltip / 12 nav / 20 card / 32 pill.
        assert_eq!(t.radius.sm, 8.0);
        assert_eq!(t.radius.md, 12.0);
        assert_eq!(t.radius.lg, 20.0);
        assert_eq!(t.radius.xl, 32.0);
        // Spacing: 4 / 8 / 12 / 16 / 24 / 32.
        assert_eq!(t.spacing.md, 12.0);
        assert_eq!(t.spacing.xl, 24.0);
        assert_eq!(t.spacing.xxl, 32.0);
    }

    #[test]
    fn hoff_motion_settles_in_200ms() {
        let t = Theme::hoff();
        let settle = t.motion.settling_time();
        assert!(
            (0.15..=0.25).contains(&settle),
            "HOFF global transition is .2s, spring settles in {settle}s"
        );
        // No overshoot: damping at or above critical.
        assert!(t.motion.damping_ratio() >= 0.95);
    }

    #[test]
    fn non_hoff_themes_carry_derived_glass_tokens() {
        for name in ["dark", "light", "dracula", "nord"] {
            let t = Theme::named(name)
                .or_else(|| (name == "dark").then(Theme::dark))
                .or_else(|| (name == "light").then(Theme::light))
                .unwrap();
            assert!(t.glass.surface.0[3] > 0.0, "{name}: glass derived");
            assert!(t.glass.edge.0[3] > 0.0, "{name}: edge derived");
        }
    }

    // -- Accessibility --

    #[cfg(feature = "accessibility")]
    #[test]
    fn intent_role_mapping() {
        let theme = Theme::dark();
        assert_eq!(
            theme.intent_role(Intent::Destructive),
            accesskit::Role::Button
        );
        assert_eq!(
            theme.intent_role(Intent::Informational),
            accesskit::Role::Label
        );
        assert_eq!(
            theme.intent_role(Intent::Neutral),
            accesskit::Role::GenericContainer
        );
    }
}
