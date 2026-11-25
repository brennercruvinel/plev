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
/// today's ring, the switch's well and the slider's thumb.
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
    legible_on(need, preferred, fallback, &[&[page, fill]])
}

/// `fg` reads at `need` on `layers`, laid from the bottom up.
fn reads(need: f32, fg: Color, layers: &[Color]) -> bool {
    let under = layers.iter().fold([0.0; 3], |u, c| over(*c, u));
    ratio(over(fg, under), under) >= need
}

/// [`legible_at`] on every one of `grounds`, each its layers from the
/// bottom up: a ring a person must see (1.4.11) against all it borders,
/// the well it holds and what lies around it.
pub fn legible_on(need: f32, preferred: Color, fallback: Color, grounds: &[&[Color]]) -> Color {
    if reads_on(need, preferred, grounds) {
        preferred
    } else {
        fallback
    }
}

/// `fg` reads at `need` on every one of `grounds`.
pub(super) fn reads_on(need: f32, fg: Color, grounds: &[&[Color]]) -> bool {
    grounds.iter().all(|layers| reads(need, fg, layers))
}

/// Which sixth of the color wheel `c` falls in (red, yellow, green, cyan,
/// blue, magenta: the hue's sextant, as HSV cuts it), or `None` for a
/// gray. Two intents a person must tell apart by color stand in
/// different sextants.
pub fn hue_sextant(c: Color) -> Option<u8> {
    let [r, g, b] = [c.0[0], c.0[1], c.0[2]];
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let span = max - min;
    if span <= f32::EPSILON {
        return None;
    }
    // the hue in sixths of a turn, 0 to 6
    let sixths = if max == r {
        ((g - b) / span).rem_euclid(6.0)
    } else if max == g {
        (b - r) / span + 2.0
    } else {
        (r - g) / span + 4.0
    };
    Some(sixths.floor() as u8 % 6)
}
