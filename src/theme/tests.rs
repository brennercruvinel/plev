#[cfg(test)]
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
                "{:?} motion should differ from neutral",
                intent,
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
        assert!(freq > 0.0, "freq={}", freq);
    }

    #[test]
    fn settling_time_finite() {
        let theme = Theme::dark();
        let t = theme.motion.settling_time();
        assert!(t > 0.0 && t < 10.0, "settling={}s", t);
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
            assert!(s > 0.0, "{:?}: stiffness must be positive", intent);
            assert!(d > 0.0, "{:?}: damping must be positive", intent);
            assert!(mass > 0.0, "{:?}: mass must be positive", intent);
            let ratio = m.damping_ratio();
            assert!(ratio > 0.0 && ratio < 10.0, "{:?}: ratio={}", intent, ratio);
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
        assert!(t.caption < t.body_sm);
        assert!(t.body_sm < t.body);
        assert!(t.body < t.title_sm);
        assert!(t.title_sm < t.title);
        assert!(t.title < t.display);
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
