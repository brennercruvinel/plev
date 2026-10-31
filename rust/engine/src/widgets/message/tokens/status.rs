//! The status pieces' tokens: the badge, the kit's chip label
//! (`components/Card`) and its notification dot (`components/HeaderSimple`),
//! with a count pill derived from the chip's run.
//!
//! The kit gives the geometry and one red; it draws no intent on a chip
//! and no number in a disc. Those colors are derived here from each
//! theme's palette, under the contrast the repository asks for: a mark
//! (the chip's intent disc, the dot) reaches 3:1 on what it sits on, and a
//! count's numeral 7:1 on its fill (AAA text).

use super::{IntentColors, MARK_CONTRAST, MessageText, contrast};
use crate::styling::color::Color;

/// What a count's numeral needs on its fill (WCAG 1.4.6, AAA text).
pub(super) const TEXT_AAA: f32 = 7.0;

/// How finely a color walks toward another until it reads: the first of
/// these equal steps that reaches the contrast wins.
const STEPS: u8 = 20;

/// The chip: a label in a rounded box with a hairline, and, for an
/// intent, a disc before the label.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TagTokens {
    pub pad_x: f32,
    pub pad_y: f32,
    pub radius: f32,
    pub border_width: f32,
    /// The label's run, in the ink.
    pub text: MessageText,
    pub fill: Color,
    /// The outline, 3:1 on the surface and on the well (`BadgeTokens::new`
    /// walks the given tone toward the ink until it is).
    pub border: Color,
    /// The chip on a tinted surface (the kit's `isGray` card): paper with
    /// a hairline, so it still stands out.
    pub raised_fill: Color,
    pub raised_border: Color,
    /// The intent's disc before the label, and the gap after it.
    pub mark: f32,
    pub mark_gap: f32,
}

/// The badge: the chip, the count pill on the chip's run, the dot, and
/// their colors per intent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgeTokens {
    pub tag: TagTokens,
    /// Across a count's numeral; the pill is as tall as the chip's run and
    /// its `pad_y`, and never narrower than tall (a disc for one digit).
    pub count_pad_x: f32,
    /// The notification dot, and its inset from its host's top end corner.
    pub dot: f32,
    pub dot_inset: f32,
    /// The chip's intent disc, 3:1 on the chip's fill.
    pub marks: IntentColors,
    /// The dot, 3:1 on the surface it sits on.
    pub dots: IntentColors,
    /// A count's fill, and its numeral at 7:1 on it.
    pub count_fill: IntentColors,
    pub count_ink: IntentColors,
}

/// What the palette gives a badge: the intents' base colors, the surface
/// the badge sits on and the tinted well it may sit in, and the two tones
/// a numeral may take.
#[derive(Clone, Copy, Debug)]
pub struct BadgePalette {
    pub intents: IntentColors,
    pub surface: Color,
    pub well: Color,
    pub paper: Color,
    pub ink: Color,
}

impl BadgeTokens {
    /// The geometry, with every intent's colors derived from `palette`.
    pub fn new(
        tag: TagTokens,
        count_pad_x: f32,
        dot: f32,
        dot_inset: f32,
        palette: BadgePalette,
    ) -> Self {
        let surface = over(palette.surface, Color::BLACK);
        let chip = over(tag.fill, surface);
        let base = palette.intents;
        let each = |f: &dyn Fn(Color) -> Color| IntentColors {
            neutral: f(base.neutral),
            informational: f(base.informational),
            constructive: f(base.constructive),
            destructive: f(base.destructive),
        };
        let numeral = |fill: Color| {
            let fill = over(fill, surface);
            let (paper, ink) = (over(palette.paper, surface), over(palette.ink, surface));
            if contrast(paper, fill) >= contrast(ink, fill) {
                paper
            } else {
                ink
            }
        };
        // a neutral count is filled with the ink, as the kit's primary
        // button; the others with their intent
        let counts = IntentColors {
            neutral: palette.ink,
            ..base
        };
        let count_ink = IntentColors {
            neutral: numeral(counts.neutral),
            informational: numeral(counts.informational),
            constructive: numeral(counts.constructive),
            destructive: numeral(counts.destructive),
        };
        let count_fill = IntentColors {
            neutral: deepen(over(counts.neutral, surface), count_ink.neutral),
            informational: deepen(over(base.informational, surface), count_ink.informational),
            constructive: deepen(over(base.constructive, surface), count_ink.constructive),
            destructive: deepen(over(base.destructive, surface), count_ink.destructive),
        };
        let ink = over(palette.ink, surface);
        // the chip's outline holds 3:1 on the surface and on the well, so
        // the chip shows wherever it sits, not only its label
        let grounds = [surface, over(palette.well, surface)];
        let outline = |c: Color| toward_all(over(c, surface), ink, &grounds, MARK_CONTRAST);
        let tag = TagTokens {
            border: outline(tag.border),
            raised_border: outline(tag.raised_border),
            ..tag
        };
        Self {
            tag,
            count_pad_x,
            dot,
            dot_inset,
            marks: each(&|c| toward(over(c, chip), ink, chip, MARK_CONTRAST)),
            dots: each(&|c| toward(over(c, surface), ink, surface, MARK_CONTRAST)),
            count_fill,
            count_ink,
        }
    }
}

/// A fill moved away from its numeral until the numeral reads at AAA.
fn deepen(fill: Color, numeral: Color) -> Color {
    let away = if contrast(Color::WHITE, numeral) > contrast(Color::BLACK, numeral) {
        Color::WHITE
    } else {
        Color::BLACK
    };
    toward(fill, away, numeral, TEXT_AAA)
}

/// `c` walked toward `to` by the fewest [`STEPS`] that make it reach `need`
/// on `under`; `to` when none does. Opaque colors in, opaque out.
pub(super) fn toward(c: Color, to: Color, under: Color, need: f32) -> Color {
    (0..=STEPS)
        .map(|i| mix(c, to, f32::from(i) / f32::from(STEPS)))
        .find(|m| contrast(*m, under) >= need)
        .unwrap_or(to)
}

/// [`toward`] until `c` reaches `need` on every one of `unders`.
fn toward_all(c: Color, to: Color, unders: &[Color], need: f32) -> Color {
    (0..=STEPS)
        .map(|i| mix(c, to, f32::from(i) / f32::from(STEPS)))
        .find(|m| unders.iter().all(|u| contrast(*m, *u) >= need))
        .unwrap_or(to)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let at = |i: usize| a.0[i] + (b.0[i] - a.0[i]) * t;
    Color([at(0), at(1), at(2), 1.0])
}

/// `c` composed over the opaque `under` in linear light, as the renderer
/// blends: a translucent glass lightens a dark page more than an sRGB mix
/// says.
pub(super) fn over(c: Color, under: Color) -> Color {
    let lin = |v: f32| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let gamma = |v: f32| {
        if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    };
    let a = c.0[3];
    if a >= 1.0 {
        // opaque: itself, exactly, not a round trip through linear light
        return c;
    }
    let at = |i: usize| gamma(lin(c.0[i]) * a + lin(under.0[i]) * (1.0 - a));
    Color([at(0), at(1), at(2), 1.0])
}

/// The avatar's online dot: a disc on the avatar's bottom end rim, cut
/// from it by a ring in the paper, in a green that reads on the paper.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarTokens {
    pub online: f32,
    pub online_ring: f32,
    pub online_fill: Color,
    pub ring: Color,
}

impl AvatarTokens {
    /// The geometry, the dot's `green` moved toward `ink` until it reads
    /// on `paper` at 3:1.
    pub fn new(online: f32, online_ring: f32, green: Color, paper: Color, ink: Color) -> Self {
        let paper = over(paper, Color::BLACK);
        Self {
            online,
            online_ring,
            online_fill: toward(over(green, paper), over(ink, paper), paper, MARK_CONTRAST),
            ring: paper,
        }
    }
}
