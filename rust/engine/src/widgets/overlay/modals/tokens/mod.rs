//! The kit's modals' tokens (`templates/ModalPage`, task-104 front B,
//! onda 3): the shades every modal shares ([`ModalInk`]), the surface of
//! each one ([`PanelTokens`]), the side menu Export and Settings share
//! ([`NavTokens`]), the pictures that stand in for the kit's photos
//! ([`PictureTokens`]) and one recipe per modal. `share.rs` holds the
//! three share modals and Variations, `forms.rs` Export and Settings.
//!
//! [`ModalsTokens::hoff_core`] is the kit as its tailwind classes write
//! it. Every other theme derives its recipes from the theme already built
//! ([`ModalsTokens::derive`]): the kit's geometry over the theme's own
//! floating surface, its text, its dividers and its radii, as the
//! confirmation modal does (`ModalTokens::derive`). Every text keeps its
//! role ([`TypeRun`](crate::styling::theme::TypeRun)), the same in every
//! theme, and each theme's scale resolves it.
//!
//! On core the kit's secondary gray (#7B7B7B, and its 70 and 80 percent)
//! is 4.13:1 on the sheet, under AA; it takes shade 7 (#323232), which
//! holds AAA (`styling/theme/tests/contrast.rs`), as the dialog's body
//! and the sheet's hint do. The kit's owner mark (#49BA61, 2.41:1) takes
//! a green that holds 3:1 for a mark.

mod forms;
mod share;

pub use forms::{
    ExportTokens, LineTokens, NavTokens, OptionTokens, PlanTokens, PreviewTokens, SettingsTokens,
};
pub use share::{
    AccessTokens, PersonTokens, ShareFileTokens, SharePostTokens, ShareProfileTokens, SocialTokens,
    VariationsTokens,
};

use crate::styling::color::Color;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{Elevation, Theme};

pub(super) fn hex(v: u32) -> Color {
    Color::hex(v)
}

/// `rgba(r, g, b, a)` written as the kit writes it, in 0 to 255.
pub(super) fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::rgba(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        a,
    )
}

/// tailwind v4's `shadow-2xl`: 0 25px 50px -12px black/25.
pub(super) fn shadow_2xl() -> Vec<ShadowSpec> {
    vec![ShadowSpec::new(
        [0.0, 25.0],
        50.0,
        -12.0,
        rgba(0, 0, 0, 0.25),
    )]
}

/// The shades every modal of the kit draws with.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalInk {
    /// The text (`#121212`).
    pub ink: Color,
    /// The secondary text and the resting icons (the kit's `#7B7B7B`, on
    /// core shade 7: see the module docs).
    pub muted: Color,
    /// A resting icon, a mark the person must see (`fill-[#7B7B7B]`,
    /// 4.13:1, over the 3:1 a mark needs).
    pub icon: Color,
    /// The rules between sections (`border-[#ECECEC]`).
    pub rule: Color,
    /// A control's edge (`border-[#E2E2E2]`).
    pub edge: Color,
    /// The sunk bands and boxes (`bg-[#F8F7F7]`).
    pub well: Color,
    /// Under the pointer and the active item (`bg-[#F1F1F1]`).
    pub hover: Color,
    /// The sheet's own paper (`#FCFCFC`): a raised box in a menu.
    pub paper: Color,
    /// A raised box's shadow (`shadow-[0px_0px_4px_0px_rgba(18,18,18,0.10)]`).
    pub raised: Vec<ShadowSpec>,
    /// Between the hairlines and what they bound (`border`, 1px).
    pub rule_w: f32,
}

impl ModalInk {
    pub fn hoff_core() -> Self {
        Self {
            ink: hex(0x121212),
            muted: hex(0x323232),
            icon: hex(0x7B7B7B),
            rule: hex(0xECECEC),
            edge: hex(0xE2E2E2),
            well: hex(0xF8F7F7),
            hover: hex(0xF1F1F1),
            paper: hex(0xFCFCFC),
            raised: vec![ShadowSpec::new(
                [0.0, 0.0],
                4.0,
                0.0,
                rgba(18, 18, 18, 0.10),
            )],
            rule_w: 1.0,
        }
    }

    /// The theme's text, its dividers, its panel and hover fills, its
    /// floating surface and raised shadow.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        Self {
            ink: c.text,
            muted: c.text_mid,
            icon: c.text_mid,
            rule: c.divider,
            edge: theme.glass.edge,
            well: c.bg_panel,
            hover: c.bg_hover,
            paper: theme.glass.popover,
            raised: theme.shadows.stack(Elevation::Raised).to_vec(),
            rule_w: theme.control.edge_width,
        }
    }
}

/// A modal's surface: its paper, its edge, its corners and its shadow.
#[derive(Clone, Debug, PartialEq)]
pub struct PanelTokens {
    pub fill: Color,
    /// Its border; alpha 0 is none.
    pub border: Color,
    pub border_w: f32,
    pub radius: f32,
    /// Drop shadows under it, first on top.
    pub shadow: Vec<ShadowSpec>,
    /// The hoff glass: a backdrop blur under it and the inset key-light.
    pub glass: bool,
    /// Its width (the kit's `w-*`); the room it is drawn in caps it.
    pub width: f32,
}

impl PanelTokens {
    /// The kit's sheet `w` wide: `bg-[#FCFCFC] shadow-2xl` at `radius`,
    /// with a `border-[#ECECEC]` when `bordered`.
    pub(super) fn kit(width: f32, radius: f32, bordered: bool) -> Self {
        let none = Color::rgba(0.0, 0.0, 0.0, 0.0);
        Self {
            fill: hex(0xFCFCFC),
            border: if bordered { hex(0xECECEC) } else { none },
            border_w: 1.0,
            radius,
            shadow: shadow_2xl(),
            glass: false,
            width,
        }
    }

    /// The kit's width at `radius` on the theme's own sheet: the hoff
    /// glass over the overlay shadow stack, its soft edge.
    pub(super) fn derive(&self, theme: &Theme, radius: f32) -> Self {
        Self {
            fill: theme.glass.popover,
            border: theme.glass.edge_soft,
            border_w: theme.control.edge_width_strong,
            radius,
            shadow: theme.shadows.stack(Elevation::Overlay).to_vec(),
            glass: true,
            width: self.width,
        }
    }
}

/// The pictures that stand in for the kit's photos and renders (the
/// robot variations, the monster, the gallery, the avatars, the export
/// preview): a gradient each, top to bottom. They are content, not
/// chrome, so every theme shows the same.
#[derive(Clone, Debug, PartialEq)]
pub struct PictureTokens {
    /// The four robot variations (`/images/robot/1..4.png`).
    pub variations: [[Color; 2]; 4],
    /// The shared post's render (`/images/3d-objects/17.png`).
    pub post: [Color; 2],
    /// The profile's three covers (`/images/gallery/10, 1, 8.jpg`).
    pub covers: [[Color; 2]; 3],
    /// The people's avatars (`/images/avatars/1..4.png`).
    pub avatars: [[Color; 2]; 4],
    /// The export's preview (`/images/preview-image.png`).
    pub preview: [Color; 2],
}

impl PictureTokens {
    pub fn hoff_core() -> Self {
        Self {
            variations: [
                [hex(0xDCD3F5), hex(0xA99BE0)],
                [hex(0xCDE6F7), hex(0x8FB9E3)],
                [hex(0xF7DCCB), hex(0xE7A57F)],
                [hex(0xD5EEDB), hex(0x93CFA3)],
            ],
            post: [hex(0xF3E7FF), hex(0xC9B5F0)],
            covers: [
                [hex(0xE9E1D6), hex(0xBFAE97)],
                [hex(0xD9E4EE), hex(0x9BB2C8)],
                [hex(0xEEDCE2), hex(0xC99BAA)],
            ],
            avatars: [
                [hex(0xF5C9A8), hex(0xD98E62)],
                [hex(0xB9D7F0), hex(0x6E9FCB)],
                [hex(0xCFE8C4), hex(0x86B874)],
                [hex(0xE3CFF2), hex(0xA77CC9)],
            ],
            preview: [hex(0xE9E6F2), hex(0xBDB5D6)],
        }
    }
}

/// The recipes of the kit's modals.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalsTokens {
    pub ink: ModalInk,
    pub pictures: PictureTokens,
    pub variations: VariationsTokens,
    pub share_post: SharePostTokens,
    pub share_profile: ShareProfileTokens,
    pub share_file: ShareFileTokens,
    pub export: ExportTokens,
    pub settings: SettingsTokens,
}

impl ModalsTokens {
    /// The kit, values as written in the tailwind classes.
    pub fn hoff_core() -> Self {
        Self {
            ink: ModalInk::hoff_core(),
            pictures: PictureTokens::hoff_core(),
            variations: VariationsTokens::hoff_core(),
            share_post: SharePostTokens::hoff_core(),
            share_profile: ShareProfileTokens::hoff_core(),
            share_file: ShareFileTokens::hoff_core(),
            export: ExportTokens::hoff_core(),
            settings: SettingsTokens::hoff_core(),
        }
    }

    /// The kit's geometry over `theme`'s own surfaces, text and radii.
    pub fn derive(theme: &Theme) -> Self {
        let kit = Self::hoff_core();
        Self {
            ink: ModalInk::derive(theme),
            pictures: kit.pictures,
            variations: kit.variations.derive(theme),
            share_post: kit.share_post.derive(theme),
            share_profile: kit.share_profile.derive(theme),
            share_file: kit.share_file.derive(theme),
            export: kit.export.derive(theme),
            settings: kit.settings.derive(theme),
        }
    }
}
