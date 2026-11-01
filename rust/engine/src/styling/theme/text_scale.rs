//! The person's text size (dynamic type, font scaling; ADR 0086): a
//! curve from the size a role is designed at to the size it is drawn at.
//! Every style of the theme passes through it once
//! ([`super::TypographyScale::exact`]), after the fluid roles have taken
//! their container's width, so a measure and its drawing scale together
//! (one `TextStyle` per run, adr-0006).
//!
//! The curve is a table of `(size, scaled)` points with straight lines
//! between them and a ceiling, the shape of Android's
//! `FontScaleConverter`. [`TextScale::new`] builds the table of a factor
//! from Android 14's own: small text grows by the whole factor, large
//! text by less, so a title still fits a phone at 200% and the hierarchy
//! holds. [`TextScale::linear`] is the two-point table (every size times
//! the factor). A platform that gives a curve of its own (Android's
//! converter, iOS's `UIFontMetrics`) samples it into
//! [`TextScale::from_points`]. Line height follows the size by the same
//! ratio; padding and gaps never scale (the ladders grow by the line
//! instead, `Theme::adapted`).

/// The most points a curve holds.
pub const MAX_POINTS: usize = 9;

/// The sizes Android's tables are written at
/// (`FontScaleConverterFactory`, 8 to 100 sp), every table the same.
const KEYS: [f32; 9] = [8.0, 10.0, 12.0, 14.0, 18.0, 20.0, 24.0, 30.0, 100.0];

/// Android 14's tables (AOSP `FontScaleConverterFactory.java`), one per
/// factor: what each of [`KEYS`] becomes. A factor between two tables
/// takes the straight line between them, as Android does; 1.0 is the
/// identity.
const TABLES: [(f32, [f32; 9]); 6] = [
    (1.0, KEYS),
    (1.15, [9.2, 11.5, 13.8, 16.4, 19.8, 21.8, 25.2, 30.0, 100.0]),
    (1.3, [10.4, 13.0, 15.6, 18.8, 21.6, 23.6, 26.4, 30.0, 100.0]),
    (1.5, [12.0, 15.0, 18.0, 22.0, 24.0, 26.0, 28.0, 30.0, 100.0]),
    (1.8, [14.4, 18.0, 21.6, 24.4, 27.6, 30.8, 32.8, 34.8, 100.0]),
    (2.0, [16.0, 20.0, 24.0, 26.0, 30.0, 34.0, 36.0, 38.0, 100.0]),
];

/// The person's text size: `(size, scaled)` points, sizes rising, joined
/// by straight lines. Under the first point a size grows by the first
/// point's ratio, past the last by the last's; no point grows a size by
/// more than [`TextScale::CEILING`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextScale {
    points: [[f32; 2]; MAX_POINTS],
    len: u8,
    /// What the person asked, for display and logs: the ratio at the
    /// body sizes.
    factor: f32,
}

impl Default for TextScale {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl TextScale {
    /// Every size as designed: the one point `(1, 1)`.
    pub const IDENTITY: Self = Self {
        points: [[1.0, 1.0]; MAX_POINTS],
        len: 1,
        factor: 1.0,
    };
    /// The largest growth, Android 14's maximum (ADR 0086, open question
    /// 2): a larger factor, or a point past it, is held here.
    pub const CEILING: f32 = 2.0;

    /// The factor a platform or a person gave, on Android 14's tables.
    /// Held to `1..=CEILING`: under 1 the kit's smallest sizes would fall
    /// below what it designs for, and the app never lowers a larger one
    /// to make a layout fit (it only stops at the ceiling).
    pub fn new(factor: f32) -> Self {
        let f = Self::held(factor);
        if f == 1.0 {
            return Self::IDENTITY;
        }
        let hi = TABLES
            .iter()
            .position(|(at, _)| *at >= f)
            .unwrap_or(TABLES.len() - 1);
        let ((a, lo), (b, up)) = (TABLES[hi - 1], TABLES[hi]);
        let k = ((f - a) / (b - a)).clamp(0.0, 1.0);
        let mut points = [[0.0; 2]; MAX_POINTS];
        for (i, p) in points.iter_mut().enumerate() {
            *p = [KEYS[i], lo[i] + (up[i] - lo[i]) * k];
        }
        Self {
            points,
            len: MAX_POINTS as u8,
            factor: f,
        }
    }

    /// Every size times `factor` (held like [`Self::new`]): the two
    /// points `(1, f)` and `(100, 100·f)`.
    pub fn linear(factor: f32) -> Self {
        let f = Self::held(factor);
        if f == 1.0 {
            return Self::IDENTITY;
        }
        Self::from_points(&[[1.0, f], [100.0, 100.0 * f]])
    }

    /// A curve a platform sampled, `(size, scaled)` with sizes rising
    /// (the first [`MAX_POINTS`] are kept; a scaled size under its size
    /// or past the ceiling is held). `IDENTITY` for no point.
    pub fn from_points(points: &[[f32; 2]]) -> Self {
        let mut out = Self::IDENTITY;
        let mut len = 0;
        for &[size, scaled] in points.iter().take(MAX_POINTS) {
            let ok = size.is_finite() && scaled.is_finite() && size > 0.0;
            if !ok || (len > 0 && size <= out.points[len - 1][0]) {
                continue;
            }
            out.points[len] = [size, scaled.clamp(size, size * Self::CEILING)];
            len += 1;
        }
        if len == 0 {
            return Self::IDENTITY;
        }
        out.len = len as u8;
        out.factor = 1.0;
        out.factor = out.size(14.0) / 14.0;
        if out.factor == 1.0 && out.points[..len].iter().all(|[s, v]| s == v) {
            return Self::IDENTITY;
        }
        out
    }

    fn held(factor: f32) -> f32 {
        if factor.is_finite() {
            factor.clamp(1.0, Self::CEILING)
        } else {
            1.0
        }
    }

    /// The growth at body sizes (14 px), what the person asked.
    pub fn factor(&self) -> f32 {
        self.factor
    }

    /// The curve's points.
    pub fn points(&self) -> &[[f32; 2]] {
        &self.points[..self.len as usize]
    }

    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// The size `size` is drawn at.
    pub fn size(&self, size: f32) -> f32 {
        if self.is_identity() || size <= 0.0 {
            return size;
        }
        let p = self.points();
        let (first, last) = (p[0], p[p.len() - 1]);
        if size <= first[0] {
            return size * first[1] / first[0];
        }
        if size >= last[0] {
            return size * last[1] / last[0];
        }
        let i = p.iter().rposition(|q| q[0] <= size).unwrap_or(0);
        let ([s0, v0], [s1, v1]) = (p[i], p[i + 1]);
        v0 + (v1 - v0) * (size - s0) / (s1 - s0)
    }

    /// The line height of a `size`/`line_height` style: the same ratio
    /// as its size (Android's line height in sp).
    pub fn line(&self, size: f32, line_height: f32) -> f32 {
        if self.is_identity() || size <= 0.0 {
            return line_height;
        }
        line_height * self.size(size) / size
    }

    /// A measure that follows a text (an icon beside a label, a gap that
    /// must keep its proportion): `value` times the growth of
    /// `relative_to` (`@ScaledMetric(relativeTo:)`).
    pub fn metric(&self, value: f32, relative_to: f32) -> f32 {
        if self.is_identity() || relative_to <= 0.0 {
            return value;
        }
        value * self.size(relative_to) / relative_to
    }
}
