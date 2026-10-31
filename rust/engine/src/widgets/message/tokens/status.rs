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

/// The avatar: the kit's ladder of discs, the disc that stands in for a
/// photo, the paper ring a stacked avatar wears, and the online dot on
/// its bottom end rim, cut from it by a ring in the paper.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarTokens {
    /// The kit's edges, `AvatarSize::Xs` to `Xl`, at the kit's control
    /// `Md` step (`basis`): a theme draws each as the same share of its
    /// own control `Md`, so a denser theme keeps the ladder's proportions.
    pub ladder: [f32; 5],
    pub basis: f32,
    /// Without a photo: the disc's fill, its hairline, the initial's ink
    /// (7:1 on the fill).
    pub fill: Color,
    pub edge: Color,
    pub edge_width: f32,
    pub initial: Color,
    /// In a stack: the ring each avatar wears, inside its edge, and the
    /// share of an avatar's edge the next one slides under.
    pub stack_ring: f32,
    pub stack_overlap: f32,
    pub online: f32,
    pub online_ring: f32,
    /// The dot, 3:1 on the paper ring.
    pub online_fill: Color,
    pub ring: Color,
}

impl AvatarTokens {
    /// These tokens with their colors made legible: the paper opaque, the
    /// initial moved toward the paper's opposite until it reads at 7:1 on
    /// the fill, the dot toward the initial until it reads at 3:1 on the
    /// paper.
    pub fn legible(self) -> Self {
        let ring = over(self.ring, Color::BLACK);
        let fill = over(self.fill, ring);
        let far = if contrast(Color::BLACK, fill) >= contrast(Color::WHITE, fill) {
            Color::BLACK
        } else {
            Color::WHITE
        };
        let initial = toward(over(self.initial, fill), far, fill, TEXT_AAA);
        Self {
            ring,
            fill,
            initial,
            online_fill: toward(over(self.online_fill, ring), initial, ring, MARK_CONTRAST),
            ..self
        }
    }
}

/// The loading placeholder: the kit's scene card skeleton
/// (`components/SceneCardSkeleton`), an image block and text lines in
/// two grays a step apart, each breathing toward the next step. A
/// placeholder carries no information a reader needs from its color (the
/// widget declares the page busy), so its fills are not held to 3:1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkeletonTokens {
    /// A block (an image, a disc) and the color it breathes toward.
    pub block: Color,
    pub block_pulse: Color,
    pub block_radius: f32,
    /// A text line: its bar, corner, fill and pulse; the padding around
    /// the bar and the gap to the next line's; the last line of a
    /// paragraph as a share of the others.
    pub line: f32,
    pub line_radius: f32,
    pub line_fill: Color,
    pub line_pulse: Color,
    pub line_pad: f32,
    pub line_gap: f32,
    pub last_line: f32,
    /// The scene card: its width, its paper, hairline and corner, its padding, the
    /// image's aspect and the gap under it, the text's padding, and its
    /// two lines as shares of the text column.
    pub card_w: f32,
    pub card_fill: Color,
    pub card_border: Color,
    pub card_border_width: f32,
    pub card_radius: f32,
    pub card_pad: f32,
    pub image_aspect: f32,
    pub image_gap: f32,
    pub text_pad: f32,
    pub card_lines: [f32; 2],
}

impl SkeletonTokens {
    /// From one bar's top to the next's.
    pub fn pitch(&self) -> f32 {
        self.line + self.line_pad * 2.0 + self.line_gap
    }

    /// How tall `n` lines are: their boxes, the gaps between them.
    pub fn lines_h(&self, n: usize) -> f32 {
        if n == 0 {
            0.0
        } else {
            n as f32 * self.pitch() - self.line_gap
        }
    }
}

/// The progress rail: the kit's video player track
/// (`components/VideoPlayer`), a pill as thin as the theme's progress
/// track, its filled part a shade deeper, ending in a head that marks the
/// value. The head is the mark a reader needs, 3:1 on the paper and on
/// the track; the track and the fill are its setting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressTokens {
    pub track: Color,
    /// The filled part per intent, over the track.
    pub fill: IntentColors,
    /// The head at the fill's end: its width, and its color per intent.
    pub head_w: f32,
    pub head: IntentColors,
    /// The narrowest the rail is laid out.
    pub min_w: f32,
}

impl ProgressTokens {
    /// The rail over `paper`, each head moved toward `ink` until it reads
    /// at 3:1 on the paper and on the track over it.
    pub fn new(
        track: Color,
        fill: IntentColors,
        head_w: f32,
        head: IntentColors,
        min_w: f32,
        paper: Color,
        ink: Color,
    ) -> Self {
        let paper = over(paper, Color::BLACK);
        let ink = over(ink, paper);
        let rail = over(track, paper);
        let legible = |c: Color| toward_all(over(c, paper), ink, &[paper, rail], MARK_CONTRAST);
        Self {
            track,
            fill,
            head_w,
            head: IntentColors {
                neutral: legible(head.neutral),
                informational: legible(head.informational),
                constructive: legible(head.constructive),
                destructive: legible(head.destructive),
            },
            min_w,
        }
    }
}

/// The spinner: the kit draws none; it is the progress rail wound into a
/// ring, the track all around and an arc in the rail's neutral head
/// turning over it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpinnerTokens {
    /// The ring's stroke, the rail's thickness.
    pub stroke: f32,
    pub track: Color,
    pub arc: Color,
    /// The arc's share of the ring.
    pub sweep: f32,
}
