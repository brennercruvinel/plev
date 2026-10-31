//! The contrast the front's colors keep: a color of the kit or of a
//! theme that falls under the bar gives way to one that passes, measured
//! as WCAG measures it, in linear light over what it sits on
//! (`theme/tests/contrast.rs` checks the pairs).

use crate::styling::color::Color;
use crate::styling::theme::ButtonSurface;

/// WCAG 2.2 AAA for text (1.4.6), the bar the front's text colors keep:
/// a color of the kit or of a theme that falls under it gives way to one
/// that passes (`theme/tests/contrast.rs` measures the pairs).
pub const TEXT_CONTRAST: f32 = 7.0;

/// WCAG 2.2 AA for the parts of a control a person must see (1.4.11):
/// today's ring.
pub const MARK_CONTRAST: f32 = 3.0;

/// `fg` composed over the opaque linear `under`, in linear light.
fn over(fg: Color, under: [f32; 3]) -> [f32; 3] {
    let [r, g, b, a] = fg.to_linear_array();
    let mix = |c: f32, u: f32| c * a + u * (1.0 - a);
    [mix(r, under[0]), mix(g, under[1]), mix(b, under[2])]
}

/// `surface` at its full strength: its fill opaque. The switch's well on
/// is the ink at 70%, where the knob's paper reads 2.8:1; a check must
/// read (1.4.11), so the box takes the ink whole.
pub(super) fn full(surface: &ButtonSurface) -> ButtonSurface {
    let opaque = |c: Color| Color::rgba(c.0[0], c.0[1], c.0[2], 1.0);
    ButtonSurface {
        fill: surface.fill.map(opaque),
        ..surface.clone()
    }
}

/// WCAG contrast of two linear colors, lighter over darker.
fn ratio(a: [f32; 3], b: [f32; 3]) -> f32 {
    let lum = |[r, g, b]: [f32; 3]| 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let (la, lb) = (lum(a), lum(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// `preferred` where it reads at [`TEXT_CONTRAST`] on `fill` laid over
/// `page`, else `fallback`: the kit's or the theme's quiet text where it
/// is legible, the stronger one where it is not.
pub fn legible(preferred: Color, fallback: Color, fill: Color, page: Color) -> Color {
    legible_at(TEXT_CONTRAST, preferred, fallback, fill, page)
}

/// [`legible`] at the contrast `need`.
pub fn legible_at(need: f32, preferred: Color, fallback: Color, fill: Color, page: Color) -> Color {
    let under = over(fill, over(page, [0.0; 3]));
    if ratio(over(preferred, under), under) >= need {
        preferred
    } else {
        fallback
    }
}
