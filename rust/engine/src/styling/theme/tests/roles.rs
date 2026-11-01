//! The type scale by role: the core is the kit's TypographyPage on the
//! 4px grid, every theme has every role, and the title roles scale with
//! their container between a minimum and their full size.

use crate::styling::theme::{FLUID_RANGE, Theme, TypeRole};

fn themes() -> [(&'static str, Theme); 4] {
    [
        ("core", Theme::hoff_core()),
        ("hoff", Theme::hoff()),
        ("dark", Theme::dark()),
        ("light", Theme::light()),
    ]
}

#[test]
fn the_core_roles_are_the_kit_typography_page() {
    let t = Theme::hoff_core().typography;
    let cases = [
        (TypeRole::H1, 48.0, 56.0, 400),
        (TypeRole::H2, 40.0, 48.0, 400),
        (TypeRole::H3, 32.0, 40.0, 400),
        (TypeRole::H4, 28.0, 36.0, 400),
        (TypeRole::H5, 24.0, 32.0, 500),
        (TypeRole::H6, 20.0, 28.0, 500),
        (TypeRole::TitleLg, 18.0, 28.0, 400),
        (TypeRole::Title, 15.0, 24.0, 500),
        (TypeRole::Heading, 14.0, 20.0, 500),
        (TypeRole::BodyLg, 13.0, 16.0, 400),
        (TypeRole::Body, 12.0, 16.0, 400),
        (TypeRole::BodyStrong, 12.0, 16.0, 600),
        (TypeRole::Label, 12.0, 16.0, 500),
        (TypeRole::Caption, 11.0, 16.0, 500),
        (TypeRole::Small, 10.0, 16.0, 400),
        (TypeRole::Paragraph, 13.0, 20.0, 400),
        (TypeRole::Wordmark, 20.0, 20.0, 600),
    ];
    for (role, size, lh, weight) in cases {
        let s = t.role(role);
        assert_eq!(
            (s.font_size, s.line_height, s.font_weight),
            (size, lh, weight),
            "{}",
            role.name()
        );
        assert_eq!(s.font_family.as_deref(), Some("Inter"), "{}", role.name());
    }
    // The headlines track -3%, the rest the body's -2%.
    let em = |role: TypeRole| {
        let s = t.role(role);
        s.letter_spacing / s.font_size
    };
    assert!((em(TypeRole::H3) + 0.03).abs() < 1e-6);
    assert!((em(TypeRole::Label) + 0.02).abs() < 1e-6);
    assert_eq!(
        t.role(TypeRole::Code).font_family.as_deref(),
        Some(t.mono_family)
    );
}

/// Every role of every theme: a line at least as tall as its size, on
/// the 4px grid (the wordmark sits on its size), in a weight the
/// embedded faces have, the scale growing from small to display.
#[test]
fn every_theme_has_every_role_on_the_grid() {
    for (name, theme) in themes() {
        let t = &theme.typography;
        for role in TypeRole::ALL {
            let s = t.role(role);
            assert!(s.line_height >= s.font_size, "{name} {}", role.name());
            if role != TypeRole::Wordmark {
                assert_eq!(s.line_height % 4.0, 0.0, "{name} {}", role.name());
            }
            assert!(
                [400, 500, 600].contains(&s.font_weight),
                "{name} {}",
                role.name()
            );
        }
        let size = |r: TypeRole| t.spec(r).size;
        let ladder = [
            TypeRole::Display,
            TypeRole::H1,
            TypeRole::H2,
            TypeRole::H3,
            TypeRole::H4,
            TypeRole::H5,
            TypeRole::H6,
        ];
        for pair in ladder.windows(2) {
            assert!(size(pair[0]) >= size(pair[1]), "{name} {pair:?}");
        }
        assert!(size(TypeRole::H6) >= size(TypeRole::TitleLg), "{name}");
        assert!(size(TypeRole::TitleLg) >= size(TypeRole::Title), "{name}");
        assert!(size(TypeRole::Body) >= size(TypeRole::Caption), "{name}");
        assert!(size(TypeRole::Caption) >= size(TypeRole::Small), "{name}");
    }
}

/// A fluid role is its minimum up to the range's start, its spec from
/// the range's end, and grows in between without a step; a fixed role
/// is the same at every width.
#[test]
fn the_title_roles_scale_between_their_limits() {
    let [from, to] = FLUID_RANGE;
    for (name, theme) in themes() {
        let t = &theme.typography;
        for role in TypeRole::FLUID {
            let [min, min_lh] = t.roles.fluid_min(role).unwrap();
            let full = t.spec(role);
            assert!(min < full.size, "{name} {}", role.name());
            assert_eq!(t.role_at(role, from - 100.0).font_size, min);
            assert_eq!(t.role_at(role, from).line_height, min_lh);
            assert_eq!(t.role_at(role, to + 500.0).font_size, full.size);
            let mut last = 0.0;
            for k in 0..=20 {
                let w = from + (to - from) * k as f32 / 20.0;
                let size = t.role_at(role, w).font_size;
                assert!(size >= last, "{name} {} at {w}", role.name());
                assert!(
                    size - last < 2.0 || k == 0,
                    "{name} {} jumps at {w}",
                    role.name()
                );
                last = size;
            }
        }
        assert_eq!(t.role_at(TypeRole::Label, from), t.role(TypeRole::Label));
        assert_eq!(t.role_at(TypeRole::H4, from), t.role(TypeRole::H4));
    }
    let core = Theme::hoff_core().typography;
    let mid = core.role_at(TypeRole::H1, (from + to) / 2.0);
    assert_eq!((mid.font_size, mid.line_height), (40.0, 48.0));
}

/// A run keeps its role and changes only the weight.
#[test]
fn a_run_weight_overrides_only_the_weight() {
    let t = Theme::hoff_core().typography;
    let strong = TypeRole::Title.strong().style(&t);
    let title = t.role(TypeRole::Title);
    assert_eq!(strong.font_weight, 600);
    assert_eq!(
        (strong.font_size, strong.line_height, strong.letter_spacing),
        (title.font_size, title.line_height, title.letter_spacing)
    );
    assert_eq!(TypeRole::Label.run().style(&t), t.role(TypeRole::Label));
}

/// The numeric role turns on tabular figures (`tnum`), so a column of
/// numbers lines up and a counting value holds still: in the core's Inter
/// every figure is one width. The label it shares its measure with keeps
/// the face's own figures. (The hoff's Inclusive Sans has no `tnum`, so
/// there its figures stay the face's.)
#[test]
fn the_numeric_role_has_tabular_figures() {
    use crate::drawing::text::{TextFeatures, TextMeasurer};
    let t = Theme::hoff_core().typography;
    let numeric = t.role(TypeRole::Numeric);
    assert_eq!(numeric.features, TextFeatures::TABULAR);
    assert_eq!(t.role(TypeRole::Label).features, TextFeatures::NONE);
    let width = |s: &str| TextMeasurer::measure_styled(s, &numeric, None).0;
    let figures: Vec<f32> = (0..10).map(|d| width(&d.to_string().repeat(8))).collect();
    let (lo, hi) = figures
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), w| (lo.min(*w), hi.max(*w)));
    assert!(hi - lo < 0.01, "tabular: {figures:?}");
}
