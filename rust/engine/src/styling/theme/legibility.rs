//! Legibility as a formula (task-104 front M, adr-0086): a color that
//! falls under the contrast its role needs moves the least it must to
//! reach it, measured as WCAG measures it (in linear light, over what it
//! sits on, `theme/tests/contrast.rs`). A translucent tone first gains
//! opacity (the kit's "ink at n%" stays ink); an opaque one moves along its
//! OKLCH lightness, away from its ground, keeping its hue and chroma. A
//! color that already reads is returned as it is, so the kit's colors stay
//! wherever they pass.
//!
//! One formula serves two bars: the themes' own (WCAG AA, 4.5:1 for text
//! and 3:1 for the parts of a control and for large text) and the one more
//! contrast asks for (`Contrast::More`, 7:1 for text, AAA).

use super::tokens::{ColorTokens, GlassTokens};
use crate::styling::color::Color;

/// WCAG 2.2 AA for text (1.4.3).
pub const TEXT_AA: f32 = 4.5;
/// WCAG 2.2 AA for large text (1.4.3) and non-text contrast (1.4.11).
pub const MARK_AA: f32 = 3.0;
/// WCAG 2.2 AAA for text (1.4.6).
pub const TEXT_AAA: f32 = 7.0;

/// The bars a theme's colors keep: text, marks (an intent, a ring, an
/// icon, large text), and the edges of controls (the active border, the
/// field's border), `None` where the theme keeps its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub text: f32,
    pub mark: f32,
    pub edge: Option<f32>,
}

impl Bar {
    /// The themes' own bar, WCAG AA.
    ///
    /// The edges keep the theme's: the field's border is not the only
    /// thing that tells a person the field is there (its label and its
    /// place do), so WCAG 1.4.11 does not ask 3:1 of it, and more contrast
    /// strengthens it; focused, a field's border reaches 3:1 in every bar
    /// (`FormTokens::hold_edges`). Brenner's decision B (task-104);
    /// `Some(MARK_AA)` here would hold them at 3:1 at rest too (A).
    pub const AA: Self = Self {
        text: TEXT_AA,
        mark: MARK_AA,
        edge: None,
    };
    /// More contrast: AAA for text, AA's text bar for the marks.
    /// The edges at 3:1.
    pub const MORE: Self = Self {
        text: TEXT_AAA,
        mark: TEXT_AA,
        edge: Some(MARK_AA),
    };
}

/// The least step of contrast between two tones of one ladder (the
/// glass's active, default and faint text): each reads at least 1.15
/// times the one under it (AA's 4.5 becomes about 5.2, then 6.0), so a
/// settled ladder keeps its order. Where the strongest tone has less room
/// (a pale ink at AAA), the steps share it. A chosen step, to review with
/// the design (task-104 front M).
pub const LADDER_STEP: f32 = 1.15;

/// One channel from sRGB to linear light.
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// `fg` composed over the opaque linear `under`, in linear light.
fn over(fg: Color, under: [f32; 3]) -> [f32; 3] {
    let a = fg.0[3];
    let mix = |c: f32, u: f32| linear(c) * a + u * (1.0 - a);
    [
        mix(fg.0[0], under[0]),
        mix(fg.0[1], under[1]),
        mix(fg.0[2], under[2]),
    ]
}

fn luminance([r, g, b]: [f32; 3]) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// `layers` laid from the bottom up over black, in linear light.
fn ground(layers: &[Color]) -> [f32; 3] {
    layers.iter().fold([0.0; 3], |u, c| over(*c, u))
}

/// The WCAG contrast of `fg` laid on `layers` (bottom up) against them.
pub fn contrast(fg: Color, layers: &[Color]) -> f32 {
    let under = ground(layers);
    let (a, b) = (luminance(over(fg, under)), luminance(under));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// `fg` reads at `need` on every one of `grounds`.
pub fn reads(fg: Color, need: f32, grounds: &[&[Color]]) -> bool {
    grounds.iter().all(|layers| contrast(fg, layers) >= need)
}

/// The least of `[lo, hi]` where `ok` holds, `ok` monotone and true at
/// `hi`.
fn least(mut lo: f32, mut hi: f32, ok: impl Fn(f32) -> bool) -> f32 {
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if ok(mid) { hi = mid } else { lo = mid }
    }
    hi
}

/// What a search aims past the bar: the test measures in f64 what this
/// moves in f32, and a color must not land a hair under.
const MARGIN: f32 = 0.005;

/// `c` moved the least along its OKLCH lightness (darker or lighter,
/// keeping its hue, chroma and alpha) until `ratio` of it reaches `need`.
/// Where even the end of the range does not, that end.
pub fn shift_until(c: Color, darker: bool, need: f32, ratio: impl Fn(Color) -> f32) -> Color {
    let need = need + MARGIN;
    if ratio(c) >= need {
        return c;
    }
    let l = c.to_oklch().l;
    let room = if darker { l } else { 1.0 - l };
    let moved = |d: f32| {
        let mut m = c.oklch_shift(if darker { -d } else { d }, 0.0, 0.0);
        m.0[3] = c.0[3];
        m
    };
    if ratio(moved(room)) < need {
        return moved(room);
    }
    moved(least(0.0, room, |d| ratio(moved(d)) >= need))
}

/// The least contrast of `fg` over `grounds`.
fn worst(fg: Color, grounds: &[&[Color]]) -> f32 {
    grounds
        .iter()
        .map(|layers| contrast(fg, layers))
        .fold(f32::INFINITY, f32::min)
}

/// `fg` moved the least it must to read at `need` on every one of
/// `grounds` (each its layers from the bottom up): more opaque first,
/// then along its OKLCH lightness, away from the first ground.
pub fn reach(fg: Color, need: f32, grounds: &[&[Color]]) -> Color {
    if grounds.is_empty() || reads(fg, need, grounds) {
        return fg;
    }
    let with_alpha = |a: f32| Color([fg.0[0], fg.0[1], fg.0[2], a]);
    let bar = need + MARGIN;
    if fg.0[3] < 1.0 && reads(with_alpha(1.0), bar, grounds) {
        let a = least(fg.0[3], 1.0, |a| reads(with_alpha(a), bar, grounds));
        return with_alpha(a);
    }
    deepen(with_alpha(1.0), need, grounds)
}

/// `fg` moved along its OKLCH lightness only, its alpha kept, the least
/// it must to read at `need` on every one of `grounds`: a tone the design
/// fades on purpose (a link at 70% that lights to 100% under the pointer).
pub fn deepen(fg: Color, need: f32, grounds: &[&[Color]]) -> Color {
    if grounds.is_empty() {
        return fg;
    }
    let darker = luminance(over(fg, ground(grounds[0]))) <= luminance(ground(grounds[0]));
    shift_until(fg, darker, need, |c| worst(c, grounds))
}

/// A fill moved the least it must for `text` to read on it at `need`,
/// laid on `page` and under each of `washes` (a hover's white wash): away
/// from the text, so a white label keeps its color and its fill deepens.
pub fn fill_under(fill: Color, text: Color, need: f32, page: Color, washes: &[Color]) -> Color {
    let darker = luminance(over(text, [0.0; 3])) > luminance(over(fill, ground(&[page])));
    shift_until(fill, darker, need, |f| {
        std::iter::once(None)
            .chain(washes.iter().map(Some))
            .map(|wash| match wash {
                Some(w) => contrast(text, &[page, f, *w]),
                None => contrast(text, &[page, f]),
            })
            .fold(f32::INFINITY, f32::min)
    })
}

/// The theme's root colors held to `bar`: every text token on every
/// surface text sits on (the page, the card, the panel, the hover and the
/// popover), and on the same surfaces the intents, the accent (the focus
/// ring) and the headline's gradient, which may sit on any of them, and
/// the active border where `bar` holds the edges. A token that reads is
/// left as it is.
pub fn settle(colors: &mut ColorTokens, glass: &mut GlassTokens, bar: Bar) {
    let bg = colors.bg;
    let surfaces: [&[Color]; 5] = [
        &[bg],
        &[bg, colors.surface],
        &[bg, colors.bg_panel],
        &[bg, colors.bg_hover],
        &[bg, glass.popover],
    ];
    for text in [
        &mut colors.text,
        &mut colors.text_mid,
        &mut colors.text_dim,
        &mut glass.text_faint,
        &mut glass.text_placeholder,
    ] {
        *text = reach(*text, bar.text, &surfaces);
    }
    // the glass's ladder keeps its order, from the faintest up: the
    // default a step over the faint, the active a step over the default
    // (a tone that already stands higher keeps its own)
    // where the strongest tone cannot reach two whole steps over the bar
    // (hoff's pale ink at AAA), the steps share what room there is
    let ceiling = worst(
        reach(glass.text_active, f32::INFINITY, &surfaces),
        &surfaces,
    );
    let step = LADDER_STEP.min((ceiling / bar.text).sqrt());
    let above = |below: Color| (worst(below, &surfaces) * step).max(bar.text);
    glass.text_default = reach(glass.text_default, above(glass.text_faint), &surfaces);
    glass.text_active = reach(glass.text_active, above(glass.text_default), &surfaces);
    let [top, foot] = &mut glass.text_gradient;
    for mark in [
        &mut colors.accent,
        &mut colors.success,
        &mut colors.danger,
        &mut colors.warning,
        &mut colors.info,
        top,
        foot,
    ] {
        *mark = reach(*mark, bar.mark, &surfaces);
    }
    if let Some(edge) = bar.edge {
        colors.border_active = reach(colors.border_active, edge, &surfaces);
    }
}
