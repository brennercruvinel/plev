//! The avatar upload's tokens (`super::super::upload_avatar`): the kit's
//! UploadAvatar (`components/UploadAvatar/index.tsx`), a round photo
//! that, hovered, darkens under a plus to replace it. The kit's photos
//! (`/images/avatars/1.png`, and `/images/no-avatar.png` without one)
//! stand here as two-stop gradients of their colors, top to bottom.
//!
//! The scrim's share is not a number: it is the least that lets the plus
//! read as a mark (WCAG 1.4.11, 3:1) over both images, top and bottom, as
//! the window lays them (`theme/tests/contrast.rs` measures the pairs).
//! The kit's `black/40` leaves its plus at 1.95:1 on the no-avatar image.

use crate::styling::color::Color;
use crate::styling::theme::{ControlSize, Theme};
use crate::widgets::selection::tokens::{MARK_CONTRAST, legible_on};
use crate::widgets::widget::{luminance, with_alpha};

/// The disc, the photo standing in it and the plus over it.
#[derive(Clone, Debug, PartialEq)]
pub struct UploadTokens {
    /// The disc's side (`size-12`), and its fill under the photo
    /// (`bg-[#F8F7F7]`).
    pub size: f32,
    pub fill: Color,
    /// A photo, top and bottom, and the stand-in without one.
    pub photo: [Color; 2],
    pub empty: [Color; 2],
    /// Over the photo hovered or focused (`bg-black/40`), and the plus on
    /// it (`size-5 fill-[#FCFCFC]`).
    pub scrim: Color,
    pub icon: f32,
    pub icon_color: Color,
}

impl UploadTokens {
    /// The kit as written (UploadAvatar :8, the disc; :17, the scrim's
    /// black; :19-25, the plus); the photos' colors are the images' own,
    /// sampled. The scrim's share is the least that reads
    /// ([`UploadTokens::reading`]).
    pub fn hoff_core() -> Self {
        Self {
            size: 48.0,
            fill: Color::hex(0xF8F7F7),
            // avatars/1.png: the orange ground at the top, the dark of the
            // figure at the bottom.
            photo: [Color::hex(0xE07430), Color::hex(0x1A0E0C)],
            // no-avatar.png: the pink of its figure, lit at the top.
            empty: [Color::hex(0xF9E1D9), Color::hex(0xEEBFAF)],
            scrim: Color::BLACK,
            icon: 20.0,
            icon_color: Color::hex(0xFCFCFC),
        }
        .reading(&[])
    }

    /// The theme's: one `Lg` control across (a nav link's height, the
    /// kit's 48 on the default ladder), the glass's active surface under
    /// the photo, the accent running into the darker of the page and the
    /// ink for a photo (as the kit's runs into its figure's dark), that
    /// surface into the glass edge without one, the scrim that darker
    /// (at the share that reads, [`UploadTokens::reading`], over the card
    /// the disc sits on) and the plus in the lighter of the two.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        let g = &theme.glass;
        let (darker, lighter) = if luminance(c.bg.0) >= luminance(c.text.0) {
            (c.text, c.bg)
        } else {
            (c.bg, c.text)
        };
        Self {
            size: theme.control.height(ControlSize::Lg),
            fill: g.surface_active,
            photo: [c.accent, darker],
            empty: [g.surface_active, g.edge],
            scrim: darker,
            icon: theme.control.icon_for(ControlSize::Md),
            icon_color: lighter,
        }
        .reading(&[c.bg, c.surface])
    }

    /// The scrim's color at the least share, in steps of an 8-bit alpha,
    /// under which the plus reads at [`MARK_CONTRAST`] over every stop of
    /// both images, each laid on the disc's fill over `under` (from the
    /// bottom up); the whole color where no share reaches it.
    fn reading(mut self, under: &[Color]) -> Self {
        let color = self.scrim;
        let grounds: Vec<Vec<Color>> = self
            .photo
            .iter()
            .chain(&self.empty)
            .map(|stop| under.iter().chain([&self.fill, stop]).copied().collect())
            .collect();
        let reads = |share: f32| {
            let scrim = Color(with_alpha(color, share));
            let layers: Vec<Vec<Color>> = grounds
                .iter()
                .map(|g| g.iter().copied().chain([scrim]).collect())
                .collect();
            let layers: Vec<&[Color]> = layers.iter().map(Vec::as_slice).collect();
            legible_on(MARK_CONTRAST, self.icon_color, scrim, &layers) == self.icon_color
        };
        let share = (0..=u8::MAX)
            .map(|step| f32::from(step) / f32::from(u8::MAX))
            .find(|share| reads(*share))
            .unwrap_or(1.0);
        self.scrim = Color(with_alpha(color, share));
        self
    }
}
