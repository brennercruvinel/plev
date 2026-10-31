//! The avatar upload's tokens (`super::super::upload_avatar`): the kit's
//! UploadAvatar (`components/UploadAvatar/index.tsx`), a round photo
//! that, hovered, darkens under a plus to replace it. The kit's photos
//! (`/images/avatars/1.png`, and `/images/no-avatar.png` without one)
//! stand here as two-stop gradients of their colors, top to bottom.

use crate::styling::color::Color;
use crate::styling::theme::{ControlSize, Theme};
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
    /// The kit as written (UploadAvatar :8, the disc; :17, the scrim;
    /// :19-25, the plus); the photos' colors are the images' own, sampled.
    pub fn hoff_core() -> Self {
        Self {
            size: 48.0,
            fill: Color::hex(0xF8F7F7),
            // avatars/1.png: the orange ground at the top, the dark of the
            // figure at the bottom.
            photo: [Color::hex(0xE07430), Color::hex(0x1A0E0C)],
            // no-avatar.png: the pink of its figure, lit at the top.
            empty: [Color::hex(0xF9E1D9), Color::hex(0xEEBFAF)],
            scrim: Color::rgba(0.0, 0.0, 0.0, 0.40),
            icon: 20.0,
            icon_color: Color::hex(0xFCFCFC),
        }
    }

    /// The theme's: one `Lg` control across (a nav link's height, the
    /// kit's 48 on the default ladder), the glass's active surface under
    /// the photo, the accent running into the darker of the page and the
    /// ink for a photo (as the kit's runs into its figure's dark), that
    /// surface into the glass edge without one, the scrim that darker at
    /// the kit's share and the plus in the lighter of the two.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        let g = &theme.glass;
        let (darker, lighter) = if luminance(c.bg.0) >= luminance(c.text.0) {
            (c.text, c.bg)
        } else {
            (c.bg, c.text)
        };
        let share = Self::hoff_core().scrim.0[3];
        Self {
            size: theme.control.height(ControlSize::Lg),
            fill: g.surface_active,
            photo: [c.accent, darker],
            empty: [g.surface_active, g.edge],
            scrim: Color(with_alpha(darker, share)),
            icon: theme.control.icon_for(ControlSize::Md),
            icon_color: lighter,
        }
    }
}
