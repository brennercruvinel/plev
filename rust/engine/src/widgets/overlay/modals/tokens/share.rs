//! The share modals' and Variations' recipes: `components/Variations`,
//! `SharePost`, `ShareProfile` and `ShareFile` as their tailwind classes
//! write them.

use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::{Theme, TypographyScale};

use super::{PanelTokens, hex, restyle, rgba};

/// The social links and the copy button a shared post and a shared
/// profile end in (`flex gap-2`, each link `size-10 border
/// border-[#e2e2e2] rounded-xl hover:bg-[#f1f1f1]`, its glyph `size-5`).
#[derive(Clone, Debug, PartialEq)]
pub struct SocialTokens {
    pub size: f32,
    pub radius: f32,
    pub icon: f32,
    pub gap: f32,
}

impl SocialTokens {
    fn hoff_core() -> Self {
        Self {
            size: 40.0,
            radius: 12.0,
            icon: 20.0,
            gap: 8.0,
        }
    }

    fn derive(&self, theme: &Theme) -> Self {
        Self {
            radius: theme.shape.nav,
            ..self.clone()
        }
    }
}

/// `components/Variations`: `w-52 p-4 pt-3 bg-[#FCFCFC]/95
/// rounded-[1.25rem] shadow-2xl`, a head (`mb-3`) with the title
/// (`text-[0.75rem] leading-[1rem] font-semibold`), the count
/// (`mr-2 text-[0.75rem] leading-[1.125rem] text-[#7B7B7B]`) and the
/// close (`size-6 rounded-md hover:bg-[#F1F1F1]`, its glyph `size-4`),
/// then the tiles two to a row (`w-[calc(50%-0.5rem)] mx-1 mt-2
/// rounded-2xl`), the chosen and the hovered one ringed
/// (`shadow-[0_0_0_1.5px_#7b7b7b_inset,0px_0px_0px_3px_#fcfcfc_inset]`).
#[derive(Clone, Debug, PartialEq)]
pub struct VariationsTokens {
    pub panel: PanelTokens,
    pub pad: f32,
    pub pad_top: f32,
    pub head_gap: f32,
    pub title: TextStyle,
    pub count: TextStyle,
    pub count_gap: f32,
    pub close: f32,
    pub close_icon: f32,
    pub close_radius: f32,
    pub columns: usize,
    pub tile_gap: f32,
    pub tile_radius: f32,
    /// The ring: its outer line and the paper band inside it.
    pub ring: Color,
    pub ring_w: f32,
    pub ring_band: f32,
}

impl VariationsTokens {
    pub(super) fn hoff_core(type_: &TypographyScale) -> Self {
        let mut panel = PanelTokens::kit(208.0, 20.0, false);
        panel.fill = rgba(0xFC, 0xFC, 0xFC, 0.95);
        Self {
            panel,
            pad: 16.0,
            pad_top: 12.0,
            head_gap: 12.0,
            title: type_.exact(12.0, 16.0, 600),
            count: type_.exact(12.0, 18.0, 400),
            count_gap: 8.0,
            close: 24.0,
            close_icon: 16.0,
            close_radius: 6.0,
            columns: 2,
            tile_gap: 8.0,
            tile_radius: 16.0,
            ring: hex(0x7B7B7B),
            ring_w: 1.5,
            ring_band: 3.0,
        }
    }

    pub(super) fn derive(&self, theme: &Theme) -> Self {
        let t = &theme.typography;
        Self {
            panel: self.panel.derive(theme, theme.shape.card),
            title: restyle(&self.title, t),
            count: restyle(&self.count, t),
            close_radius: theme.shape.micro,
            tile_radius: theme.shape.item,
            ring: theme.colors.text_mid,
            ..self.clone()
        }
    }
}

/// `components/SharePost`: `w-99 p-2 rounded-[2rem] bg-[#FCFCFC]
/// shadow-2xl`; the render in a box (`p-5 rounded-3xl bg-[#F8F7F7] border
/// border-[#ECECEC]`), then `p-4`: the title (`mb-3 text-[1.125rem]
/// leading-[1.6875rem]`), the meta row (`flex gap-5 mb-6`: the author's
/// `size-5` avatar `gap-2` and name `text-[0.75rem] leading-[1rem]
/// font-semibold`, the date's `size-4` clock `gap-1.5` and date
/// `font-medium text-[#7B7B7B]`), the links and the copy button.
#[derive(Clone, Debug, PartialEq)]
pub struct SharePostTokens {
    pub panel: PanelTokens,
    pub pad: f32,
    pub picture_pad: f32,
    pub picture_radius: f32,
    pub body_pad: f32,
    pub title: TextStyle,
    pub title_gap: f32,
    pub meta_gap: f32,
    pub meta_bottom: f32,
    pub avatar: f32,
    pub author_gap: f32,
    pub author: TextStyle,
    pub clock: f32,
    pub date_gap: f32,
    pub date: TextStyle,
    pub social: SocialTokens,
}

impl SharePostTokens {
    pub(super) fn hoff_core(type_: &TypographyScale) -> Self {
        Self {
            panel: PanelTokens::kit(396.0, 32.0, false),
            pad: 8.0,
            picture_pad: 20.0,
            picture_radius: 24.0,
            body_pad: 16.0,
            title: type_.exact(18.0, 27.0, 400),
            title_gap: 12.0,
            meta_gap: 20.0,
            meta_bottom: 24.0,
            avatar: 20.0,
            author_gap: 8.0,
            author: type_.exact(12.0, 16.0, 600),
            clock: 16.0,
            date_gap: 6.0,
            date: type_.exact(12.0, 16.0, 500),
            social: SocialTokens::hoff_core(),
        }
    }

    pub(super) fn derive(&self, theme: &Theme) -> Self {
        let t = &theme.typography;
        Self {
            panel: self.panel.derive(theme, theme.shape.pill),
            picture_radius: theme.shape.dropdown,
            title: restyle(&self.title, t),
            author: restyle(&self.author, t),
            date: restyle(&self.date, t),
            social: self.social.derive(theme),
            ..self.clone()
        }
    }
}

/// `components/ShareProfile`: `w-105 p-2 rounded-[2rem]`; three covers
/// (`flex gap-1`, each `flex-1 h-36 rounded-lg`, the first
/// `rounded-l-3xl`, the last `rounded-r-3xl`), the avatar over them
/// (`size-22 -mt-11 mb-3 border-4 border-[#FCFCFC] rounded-full`, the
/// photo `size-20`), the name (`text-[1.25rem] leading-[1.75rem]
/// font-medium`), the count (`mt-1 text-[0.8125rem] leading-[1rem]
/// font-semibold text-[#7B7B7B]`, `mb-2`), then `p-4` the links and the
/// copy button, centered.
#[derive(Clone, Debug, PartialEq)]
pub struct ShareProfileTokens {
    pub panel: PanelTokens,
    pub pad: f32,
    pub cover_h: f32,
    pub cover_gap: f32,
    /// The covers' outer corners and the inner ones.
    pub cover_outer: f32,
    pub cover_inner: f32,
    pub avatar: f32,
    pub avatar_ring: f32,
    pub avatar_bottom: f32,
    pub name: TextStyle,
    pub count: TextStyle,
    pub count_gap: f32,
    pub text_bottom: f32,
    pub actions_pad: f32,
    pub social: SocialTokens,
}

impl ShareProfileTokens {
    pub(super) fn hoff_core(type_: &TypographyScale) -> Self {
        Self {
            panel: PanelTokens::kit(420.0, 32.0, false),
            pad: 8.0,
            cover_h: 144.0,
            cover_gap: 4.0,
            cover_outer: 24.0,
            cover_inner: 8.0,
            avatar: 80.0,
            avatar_ring: 4.0,
            avatar_bottom: 12.0,
            name: type_.exact(20.0, 28.0, 500),
            count: type_.exact(13.0, 16.0, 600),
            count_gap: 4.0,
            text_bottom: 8.0,
            actions_pad: 16.0,
            social: SocialTokens::hoff_core(),
        }
    }

    pub(super) fn derive(&self, theme: &Theme) -> Self {
        let t = &theme.typography;
        Self {
            panel: self.panel.derive(theme, theme.shape.pill),
            cover_outer: theme.shape.dropdown,
            cover_inner: theme.shape.tooltip,
            name: restyle(&self.name, t),
            count: restyle(&self.count, t),
            social: self.social.derive(theme),
            ..self.clone()
        }
    }
}

/// A row of the general access (`ShareFile/GeneralAccess`: `flex gap-3
/// py-2.5`, the glyph in a `size-8 border border-[#E2E2E2] rounded-lg
/// bg-[#F8F7F7]` box, the title `text-[0.75rem] leading-[1rem]
/// font-medium`, the line under it `text-[0.6875rem] leading-[1rem]
/// text-[#7B7B7B]`).
#[derive(Clone, Debug, PartialEq)]
pub struct AccessTokens {
    pub row_pad_y: f32,
    pub gap: f32,
    pub box_size: f32,
    pub box_radius: f32,
    pub icon: f32,
    pub title: TextStyle,
    pub line: TextStyle,
}

/// A person with access (`ShareFile/Person`: `flex gap-3 px-4 py-2.5`,
/// a `size-8` avatar, the name and the email as an access row's title
/// and line, the access select `min-w-22`, or the owner `px-2` with its
/// `size-4` shield; the remove `w-5.5 bg-[#FE5938] rounded-l-lg` at the
/// row's end while hovered, its glyph `size-3.5 fill-[#fcfcfc]`).
#[derive(Clone, Debug, PartialEq)]
pub struct PersonTokens {
    pub pad_y: f32,
    pub avatar: f32,
    pub select_w: f32,
    pub owner_pad_x: f32,
    pub shield: f32,
    /// The owner's mark (the kit's #49BA61 fails 3:1; see the module
    /// docs).
    pub owner: Color,
    pub remove_w: f32,
    pub remove_radius: f32,
    pub remove_fill: Color,
    pub remove_icon: f32,
    pub remove_ink: Color,
}

/// `components/ShareFile`: `w-105 rounded-[2rem] border
/// border-[#ECECEC]`; the invite row (`flex gap-1.5 p-4`), the general
/// access and the people (`border-t`, `px-4 py-2.5`, each headed by a
/// `py-2 text-[0.6875rem] leading-[1rem] font-medium text-[#7B7B7B]/70`
/// label), the foot (`p-4 gap-2 border-t bg-[#F8F7F7]`, the link
/// `text-[0.8125rem] leading-[1rem] opacity-50` and the copy button
/// `!h-9.5 !px-4`, its link glyph `size-5 fill-[#7B7B7B]`).
#[derive(Clone, Debug, PartialEq)]
pub struct ShareFileTokens {
    pub panel: PanelTokens,
    pub pad: f32,
    pub invite_gap: f32,
    pub section_pad_y: f32,
    pub label: TextStyle,
    pub label_pad_y: f32,
    pub access: AccessTokens,
    pub person: PersonTokens,
    pub foot_gap: f32,
    pub link: TextStyle,
    pub copy_icon: f32,
}

impl ShareFileTokens {
    pub(super) fn hoff_core(type_: &TypographyScale) -> Self {
        Self {
            panel: PanelTokens::kit(420.0, 32.0, true),
            pad: 16.0,
            invite_gap: 6.0,
            section_pad_y: 10.0,
            label: type_.exact(11.0, 16.0, 500),
            label_pad_y: 8.0,
            access: AccessTokens {
                row_pad_y: 10.0,
                gap: 12.0,
                box_size: 32.0,
                box_radius: 8.0,
                icon: 16.0,
                title: type_.exact(12.0, 16.0, 500),
                line: type_.exact(11.0, 16.0, 400),
            },
            person: PersonTokens {
                pad_y: 10.0,
                avatar: 32.0,
                select_w: 88.0,
                owner_pad_x: 8.0,
                shield: 16.0,
                owner: hex(0x3A9A50),
                remove_w: 22.0,
                remove_radius: 8.0,
                remove_fill: hex(0xFE5938),
                remove_icon: 14.0,
                remove_ink: hex(0xFCFCFC),
            },
            foot_gap: 8.0,
            link: type_.exact(13.0, 16.0, 400),
            copy_icon: 20.0,
        }
    }

    pub(super) fn derive(&self, theme: &Theme) -> Self {
        let t = &theme.typography;
        let c = &theme.colors;
        let a = &self.access;
        Self {
            panel: self.panel.derive(theme, theme.shape.pill),
            label: restyle(&self.label, t),
            access: AccessTokens {
                box_radius: theme.shape.tooltip,
                title: restyle(&a.title, t),
                line: restyle(&a.line, t),
                ..a.clone()
            },
            person: PersonTokens {
                owner: c.success,
                remove_radius: theme.shape.tooltip,
                remove_fill: c.danger,
                remove_ink: c.surface,
                ..self.person.clone()
            },
            link: restyle(&self.link, t),
            ..self.clone()
        }
    }
}
