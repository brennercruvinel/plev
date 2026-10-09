//! The values of the theme the builder code was written against, as the
//! plev IR emitter measures them, and the measure itself.
//!
//! The parser does not depend on the engine, so whoever calls the IR
//! emitter hands it this table (`Tokens`), filled from the theme the
//! emitted code reads (`Theme::hoff()`): the value of every IR token and of
//! every named expression the resolvers print (`css_map::NAMED_EXPRS`).
//!
//! The measure: a value goes to the token nearest it. Its tolerance is half
//! the distance from that token to its nearest neighbour of a different
//! value in the same table, so a value within it is closer to that token
//! than to any other one (the triangle inequality). Distances:
//!
//! - color: the largest of the four channel differences in 8-bit units,
//!   alpha included (0 to 255, unrounded), the measure of the ADR
//!   `what-identical-means`;
//! - spacing, radius, width, height: pixels;
//! - typography: the larger of the font size and line height differences in
//!   pixels, among the roles of the same weight when there is one.

use uir::{ColorRole, ShapeRole, SizeToken, SpacingToken, TypographyRole};

use crate::css_map::token_rgba;

/// A text style of the type ramp: what `theme.typography.<role>()` gives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    pub size: f32,
    pub line_height: f32,
    pub weight: u16,
}

/// The value of a named builder expression.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    /// sRGB, 0 to 1, alpha last.
    Color([f32; 4]),
    Length(f32),
}

/// The theme's values, by IR token and by builder expression.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tokens {
    pub spacing: Vec<(SpacingToken, f32)>,
    /// sRGB, 0 to 1, alpha last.
    pub colors: Vec<(ColorRole, [f32; 4])>,
    pub typography: Vec<(TypographyRole, TypeStyle)>,
    pub shapes: Vec<(ShapeRole, f32)>,
    pub sizes: Vec<(SizeToken, f32)>,
    /// `theme.control.edge_width`: the separator's thickness.
    pub edge_width: f32,
    /// The value of each named expression of `css_map::NAMED_EXPRS`.
    pub exprs: Vec<(String, Value)>,
}

/// The token nearest a value, how far the value is from it, and how far it
/// may be (its tolerance).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Near<T> {
    pub token: T,
    pub distance: f32,
    pub tolerance: f32,
}

/// The token of `table` nearest `v` (ties go to the first in table order),
/// with its tolerance: half the distance to its nearest neighbour of a
/// different value, 0 when it has none.
pub fn nearest<T: Copy, V>(
    table: &[(T, V)],
    v: &V,
    dist: impl Fn(&V, &V) -> f32,
) -> Option<Near<T>> {
    let mut best: Option<(usize, f32)> = None;
    for (i, (_, value)) in table.iter().enumerate() {
        let d = dist(v, value);
        if best.is_none_or(|(_, b)| d < b) {
            best = Some((i, d));
        }
    }
    let (i, distance) = best?;
    let (token, value) = table.get(i)?;
    let gap = table
        .iter()
        .map(|(_, other)| dist(value, other))
        .filter(|d| *d > 0.0)
        .fold(f32::INFINITY, f32::min);
    let tolerance = if gap.is_finite() { gap / 2.0 } else { 0.0 };
    Some(Near {
        token: *token,
        distance,
        tolerance,
    })
}

/// The largest channel difference, in 8-bit units.
pub fn color_distance(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs() * 255.0)
        .fold(0.0, f32::max)
}

pub fn px_distance(a: &f32, b: &f32) -> f32 {
    (a - b).abs()
}

/// The larger of the size and line height differences.
pub fn type_distance(a: &TypeStyle, b: &TypeStyle) -> f32 {
    (a.size - b.size)
        .abs()
        .max((a.line_height - b.line_height).abs())
}

/// True for a size token that bounds a height (`dropdown_max_h`), as its
/// name says; every other one bounds a width.
pub fn is_height(t: SizeToken) -> bool {
    t.as_str().ends_with("_h")
}

impl Tokens {
    /// What the table lacks: an IR token with no value, a named expression
    /// with no value. The emitter reports a construct whose value it cannot
    /// measure as impossible, so a gap here is never silent either.
    pub fn missing(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut check = |kind: &str, name: &str, present: bool| {
            if !present {
                out.push(format!("{kind} {name}"));
            }
        };
        for t in SpacingToken::ALL {
            check(
                "spacing",
                t.as_str(),
                self.spacing.iter().any(|(k, _)| k == t),
            );
        }
        for t in ColorRole::ALL {
            check("color", t.as_str(), self.colors.iter().any(|(k, _)| k == t));
        }
        for t in TypographyRole::ALL {
            let present = self.typography.iter().any(|(k, _)| k == t);
            check("typography", t.as_str(), present);
        }
        for t in ShapeRole::ALL {
            check("shape", t.as_str(), self.shapes.iter().any(|(k, _)| k == t));
        }
        for t in SizeToken::ALL {
            check("size", t.as_str(), self.sizes.iter().any(|(k, _)| k == t));
        }
        for e in crate::css_map::NAMED_EXPRS {
            check("expression", e, self.exprs.iter().any(|(k, _)| k == e));
        }
        out
    }

    fn expr(&self, expr: &str) -> Option<Value> {
        self.exprs.iter().find(|(k, _)| k == expr).map(|(_, v)| *v)
    }

    /// The color a builder expression stands for: a role of the table
    /// (`theme.colors.text_mid`), a named expression, or a parametric one
    /// (`engine::theme::hoff::n2(0.66)`).
    pub fn color_of(&self, expr: &str) -> Option<[f32; 4]> {
        if let Some(name) = expr.strip_prefix("theme.colors.")
            && let Some(role) = role_named(name)
        {
            return self.color(role);
        }
        match self.expr(expr) {
            Some(Value::Color(c)) => Some(c),
            Some(Value::Length(_)) => None,
            None => token_rgba(expr),
        }
    }

    /// The length a builder expression stands for (`theme.radius.xl`).
    pub fn length_of(&self, expr: &str) -> Option<f32> {
        match self.expr(expr)? {
            Value::Length(v) => Some(v),
            Value::Color(_) => None,
        }
    }

    pub fn color(&self, role: ColorRole) -> Option<[f32; 4]> {
        self.colors
            .iter()
            .find(|(k, _)| *k == role)
            .map(|(_, v)| *v)
    }

    pub fn spacing_of(&self, t: SpacingToken) -> Option<f32> {
        self.spacing.iter().find(|(k, _)| *k == t).map(|(_, v)| *v)
    }

    /// The size tokens of one dimension.
    pub fn sizes_of(&self, height: bool) -> Vec<(SizeToken, f32)> {
        self.sizes
            .iter()
            .filter(|(k, _)| is_height(*k) == height)
            .copied()
            .collect()
    }

    /// The role nearest a text style, among the roles of its weight when
    /// the ramp has one, with the tolerance measured over the whole ramp.
    pub fn nearest_type(&self, style: &TypeStyle) -> Option<Near<TypographyRole>> {
        let same: Vec<(TypographyRole, TypeStyle)> = self
            .typography
            .iter()
            .filter(|(_, s)| s.weight == style.weight)
            .copied()
            .collect();
        let pool = if same.is_empty() {
            &self.typography
        } else {
            &same
        };
        let near = nearest(pool, style, type_distance)?;
        let all = nearest(&self.typography, &self.type_of(near.token)?, type_distance)?;
        Some(Near {
            tolerance: all.tolerance,
            ..near
        })
    }

    pub fn type_of(&self, role: TypographyRole) -> Option<TypeStyle> {
        self.typography
            .iter()
            .find(|(k, _)| *k == role)
            .map(|(_, v)| *v)
    }
}

/// The color role a `theme.colors.<name>` names.
pub fn role_named(name: &str) -> Option<ColorRole> {
    ColorRole::ALL.iter().copied().find(|r| r.as_str() == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_nearest_token_and_its_tolerance() {
        let scale = [
            (SpacingToken::Xs, 4.0),
            (SpacingToken::Sm, 8.0),
            (SpacingToken::Xl, 24.0),
        ];
        let near = nearest(&scale, &7.0, px_distance).unwrap();
        assert_eq!(near.token, SpacingToken::Sm);
        assert_eq!((near.distance, near.tolerance), (1.0, 2.0));
        let near = nearest(&scale, &30.0, px_distance).unwrap();
        assert_eq!(near.token, SpacingToken::Xl);
        assert_eq!((near.distance, near.tolerance), (6.0, 8.0));
        // a tie goes to the first token; one token alone has no neighbour
        let near = nearest(&scale, &6.0, px_distance).unwrap();
        assert_eq!(near.token, SpacingToken::Xs);
        let alone = [(SpacingToken::Md, 12.0)];
        assert_eq!(nearest(&alone, &12.0, px_distance).unwrap().tolerance, 0.0);
        assert!(nearest::<SpacingToken, f32>(&[], &1.0, px_distance).is_none());
    }

    #[test]
    fn two_roles_of_one_color_are_no_neighbours() {
        let colors = [
            (ColorRole::Text, [1.0, 1.0, 1.0, 0.95]),
            (ColorRole::Accent, [1.0, 1.0, 1.0, 0.95]),
            (ColorRole::TextMid, [1.0, 1.0, 1.0, 0.70]),
        ];
        let near = nearest(&colors, &[1.0, 1.0, 1.0, 0.94], color_distance).unwrap();
        assert_eq!(near.token, ColorRole::Text);
        assert!((near.distance - 2.55).abs() < 1e-3);
        assert!((near.tolerance - 31.875).abs() < 1e-3);
    }

    #[test]
    fn a_table_says_what_it_lacks_and_reads_every_expression_form() {
        let mut t = Tokens {
            colors: vec![(ColorRole::TextMid, [0.9, 0.9, 0.9, 0.7])],
            exprs: vec![
                ("theme.radius.xl".into(), Value::Length(32.0)),
                (
                    "theme.glass.button".into(),
                    Value::Color([0.1, 0.1, 0.1, 0.7]),
                ),
            ],
            ..Tokens::default()
        };
        assert!(t.missing().contains(&"spacing xs".to_owned()));
        assert!(
            t.missing()
                .contains(&"expression theme.glass.edge".to_owned())
        );
        assert_eq!(
            t.color_of("theme.colors.text_mid"),
            Some([0.9, 0.9, 0.9, 0.7])
        );
        assert_eq!(t.color_of("theme.colors.nothing"), None);
        assert_eq!(t.color_of("theme.glass.button"), Some([0.1, 0.1, 0.1, 0.7]));
        assert_eq!(t.color_of("theme.radius.xl"), None);
        assert_eq!(t.length_of("theme.radius.xl"), Some(32.0));
        let n2 = t.color_of("engine::theme::hoff::n2(0.5)").unwrap();
        assert_eq!(n2[3], 0.5);
        t.sizes = vec![(SizeToken::CardW, 368.0), (SizeToken::DropdownMaxH, 320.0)];
        assert_eq!(t.sizes_of(true), vec![(SizeToken::DropdownMaxH, 320.0)]);
        assert_eq!(t.sizes_of(false), vec![(SizeToken::CardW, 368.0)]);
    }

    #[test]
    fn a_style_goes_to_a_role_of_its_weight() {
        let s = |size, line_height, weight| TypeStyle {
            size,
            line_height,
            weight,
        };
        let t = Tokens {
            typography: vec![
                (TypographyRole::BaseR, s(16.0, 24.0, 400)),
                (TypographyRole::BaseM, s(16.0, 24.0, 500)),
                (TypographyRole::CaptionR, s(12.0, 15.96, 400)),
            ],
            ..Tokens::default()
        };
        let near = t.nearest_type(&s(16.0, 24.0, 500)).unwrap();
        assert_eq!((near.token, near.distance), (TypographyRole::BaseM, 0.0));
        let near = t.nearest_type(&s(12.0, 16.0, 400)).unwrap();
        assert_eq!(near.token, TypographyRole::CaptionR);
        assert!((near.distance - 0.04).abs() < 1e-3);
        assert!((near.tolerance - 4.02).abs() < 1e-3);
        // no role of weight 700: the nearest of any weight
        let near = t.nearest_type(&s(16.0, 24.0, 700)).unwrap();
        assert_eq!(near.token, TypographyRole::BaseR);
    }
}
