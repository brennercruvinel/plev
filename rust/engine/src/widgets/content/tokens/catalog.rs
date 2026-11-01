//! The kit's catalog cards (`theme.disclosure.catalog`), the CardsPage:
//! the pricing card, the asset card, the scene card and its skeleton, the
//! explore card, its small tag and the slide item. `hoff_core` is the kit
//! class list by class list; `derive` keeps its geometry and takes the
//! theme's palette, so derived from the core it is the kit again (but
//! the muted text, which takes shade 7 in the core for AAA).
//!
//! A photo of the kit is a rectangle in the `image` gradient: the gallery
//! draws no pixel of a photo.

use super::*;
use crate::styling::theme::TypeRole;

/// The pricing card (PriceCard/index.tsx).
#[derive(Clone, Debug, PartialEq)]
pub struct PriceRecipe {
    pub width: f32,
    pub radius: f32,
    /// The plan's title row, `px-6 py-3` (:35).
    pub title_pad: [f32; 2],
    pub title: TypeRun,
    /// The inner panel, `p-3 border rounded-[1.25rem]` (:42).
    pub pad: f32,
    pub description: TypeRun,
    /// The price box (:46): `p-3 border rounded-[1.25rem]`, its shadow.
    pub price_shadow: Vec<ShadowSpec>,
    pub currency: TypeRun,
    pub amount: TypeRun,
    pub unit: TypeRun,
    /// The `$` cell's padding (`px-0.5 py-1`), the unit's (`p-2`).
    pub currency_pad: [f32; 2],
    pub unit_pad: f32,
    /// `mt-2` before the button, `mt-3` before the features, `gap-2`.
    pub button_gap: f32,
    pub features_gap: f32,
    pub feature_gap: f32,
    pub feature: TypeRun,
    /// The check: `size-4 mr-2.5 ml-0.5`.
    pub check: f32,
    pub check_pad: [f32; 2],
    /// The featured plan's lift (`shadow-2xl`).
    pub lift: Vec<ShadowSpec>,
}

/// The asset card (AssetCard/index.tsx:10) and the scene card (SceneCard/
/// index.tsx:13) with its skeleton (SceneCardSkeleton/index.tsx:2).
#[derive(Clone, Debug, PartialEq)]
pub struct TileRecipe {
    /// Asset: `w-65.5 aspect-square p-6 rounded-[1.25rem]`.
    pub asset_w: f32,
    pub asset_pad: f32,
    pub asset_radius: f32,
    /// Its hover: the scrim `h-24` from the bottom at opacity 30.
    pub scrim_h: f32,
    pub scrim_alpha: f32,
    /// Its buttons: `size-10 rounded-xl`, `top-2 right-2`, the icon
    /// `size-4`; the share label 14/20 semibold, `gap-1.5 px-3`.
    pub button: f32,
    pub button_radius: f32,
    pub button_inset: f32,
    pub icon: f32,
    pub label: TypeRun,
    pub label_gap: f32,
    pub label_pad: f32,
    /// The raised button's border (`border-[#DBDBDB]`) and shadow.
    pub button_border: Color,
    pub raised: Vec<ShadowSpec>,
    /// Scene: `w-65 p-2 rounded-3xl`, the image `rounded-2xl mb-2`
    /// (256 by 196), its text `p-3`, the title `mb-1`.
    pub scene_w: f32,
    pub scene_pad: f32,
    pub scene_radius: f32,
    pub image_radius: f32,
    pub image_ratio: f32,
    pub image_gap: f32,
    pub text_pad: f32,
    pub title_gap: f32,
    pub title: TypeRun,
    pub category: TypeRun,
    /// The type badge: `size-10 rounded-lg top-3 left-3`.
    pub badge: f32,
    pub badge_radius: f32,
    pub badge_inset: f32,
    /// The skeleton's bars: `w-32 h-2` and `w-18 h-2`, `rounded-[2px]`,
    /// in a `p-1`.
    pub bars: [f32; 2],
    pub bar_h: f32,
    pub bar_radius: f32,
    pub bar_pad: f32,
    /// The scene's hover: `shadow-2xl`.
    pub lift: Vec<ShadowSpec>,
}

/// The explore card (ExploreCard/index.tsx:12), its small tag
/// (ExploreCardSm/index.tsx:12) and the slide item (CardsPage :80-100).
#[derive(Clone, Debug, PartialEq)]
pub struct ExploreRecipe {
    /// `rounded-3xl`, the image 360 by 288.
    pub width: f32,
    pub ratio: f32,
    pub radius: f32,
    /// The hover: scrim `h-24` at opacity 45, the bar `p-3`, the title
    /// `p-2.5` 12/16 semibold, the buttons `size-9 rounded-[0.625rem]
    /// gap-1`.
    pub scrim_alpha: f32,
    pub bar_pad: f32,
    pub title_pad: f32,
    pub button: f32,
    pub button_radius: f32,
    pub button_gap: f32,
    /// The tag: `w-59 p-2 rounded-[1.25rem]`, the thumbnail 64
    /// `rounded-xl`, the text `pl-4`, the description `mt-1`.
    pub tag_w: f32,
    pub tag_pad: f32,
    pub tag_radius: f32,
    pub thumb: f32,
    pub thumb_radius: f32,
    pub text_gap: f32,
    pub line_gap: f32,
    pub tag_title: TypeRun,
    pub tag_line: TypeRun,
    /// The tag's hover: `shadow-xl`.
    pub lift: Vec<ShadowSpec>,
    /// The slide item: a 50 px image in a `border rounded-xl`.
    pub slide: f32,
    pub slide_radius: f32,
}

/// The catalog's palette: the kit's paper, its two washes, its line and
/// ring, the ink, the muted text; a photo's gradient and the featured
/// plan's, the scrim over a photo, the text on it and the hover of a
/// button on it (a glass fill and edge of that text). A photo is content,
/// not chrome: every theme keeps the kit's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CatalogPalette {
    pub paper: Color,
    pub panel: Color,
    pub wash: Color,
    pub line: Color,
    pub ring: Color,
    pub ink: Color,
    pub muted: Color,
    pub image: [Color; 2],
    pub feature: [Color; 2],
    pub scrim: Color,
    pub on_image: Color,
    pub glass: Color,
    pub glass_line: Color,
}

/// The catalog slot.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogTokens {
    pub palette: CatalogPalette,
    pub price: PriceRecipe,
    pub tile: TileRecipe,
    pub explore: ExploreRecipe,
}

/// The kit's five-layer lift of a raised control (`shadow-[0px_16px_4px_0px
/// rgba(0,0,0,0.00),0px_10px_4px...,0px_6px_3px_0px_rgba(0,0,0,0.01),0px_3px
/// _3px_0px_rgba(0,0,0,0.02),0px_1px_1px_0px_rgba(0,0,0,0.02)]`).
fn raised() -> Vec<ShadowSpec> {
    vec![
        ShadowSpec::new([0.0, 6.0], 3.0, 0.0, black(0.01)),
        ShadowSpec::new([0.0, 3.0], 3.0, 0.0, black(0.02)),
        ShadowSpec::new([0.0, 1.0], 1.0, 0.0, black(0.02)),
    ]
}

/// tailwind 4's `shadow-2xl` (`0 25px 50px -12px rgb(0 0 0 / 0.25)`).
fn shadow_2xl() -> Vec<ShadowSpec> {
    vec![ShadowSpec::new([0.0, 25.0], 50.0, -12.0, black(0.25))]
}

/// tailwind 4's `shadow-xl` (`0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px
/// 10px -6px rgb(0 0 0 / 0.1)`).
fn shadow_xl() -> Vec<ShadowSpec> {
    vec![
        ShadowSpec::new([0.0, 20.0], 25.0, -5.0, black(0.10)),
        ShadowSpec::new([0.0, 8.0], 10.0, -6.0, black(0.10)),
    ]
}

impl CatalogTokens {
    /// The kit, class list by class list.
    pub fn hoff_core() -> Self {
        let on_image = hex(0xFCFCFC);
        let palette = CatalogPalette {
            paper: hex(0xFCFCFC),
            panel: hex(0xF8F7F7),
            wash: hex(0xF1F1F1),
            line: hex(0xECECEC),
            ring: hex(0xE2E2E2),
            ink: hex(0x121212),
            // The kit's #7B7B7B text fails AAA on its paper: shade 7.
            muted: crate::styling::theme::hoff_core::SHADES[6],
            // A photo: the ring darkening to shade 7, so the text the
            // hover lays at its foot reads at AAA (contrast.rs).
            image: [hex(0xE2E2E2), hex(0x323232)],
            // The featured plan's image (`/images/bg-pricing.png`, under
            // a `text-[#FCFCFC]` title): shade 7 to the ink.
            feature: [hex(0x323232), hex(0x121212)],
            // `from-[#171717]/100`, `text-[#FCFCFC]`; a button on a photo
            // hovers `hover:bg-[#FCFCFC]/15 hover:border-[#FCFCFC]/25`
            // (AssetCard :30-40, ExploreCard :28-58).
            scrim: hex(0x171717),
            on_image,
            glass: tint(on_image, 0.15),
            glass_line: tint(on_image, 0.25),
        };
        let price = PriceRecipe {
            width: 320.0,
            radius: 20.0,
            title_pad: [24.0, 12.0],
            title: TypeRole::H6.run(),
            pad: 12.0,
            // `text-[0.9375rem] leading-[1.41rem] font-medium` (:43).
            description: TypeRole::Title.run(),
            // `shadow-[0px_2px_2px_0px_rgba(255,255,255,0.80)_inset, ...]`.
            price_shadow: {
                let mut s = vec![ShadowSpec::inset([0.0, 2.0], 2.0, 0.0, white(0.80))];
                s.extend(raised());
                s
            },
            currency: TypeRole::H5.run(),
            amount: TypeRole::H2.run(),
            unit: TypeRole::Body.run(),
            currency_pad: [2.0, 4.0],
            unit_pad: 8.0,
            button_gap: 8.0,
            features_gap: 12.0,
            feature_gap: 8.0,
            // `text-[0.8125rem] leading-[1.22rem]` (:79).
            feature: TypeRole::Paragraph.run(),
            check: 16.0,
            check_pad: [2.0, 10.0],
            lift: shadow_2xl(),
        };
        let tile = TileRecipe {
            asset_w: 262.0,
            asset_pad: 24.0,
            asset_radius: 20.0,
            scrim_h: 96.0,
            scrim_alpha: 0.30,
            button: 40.0,
            button_radius: 12.0,
            button_inset: 8.0,
            icon: 16.0,
            label: TypeRole::Heading.strong(),
            label_gap: 6.0,
            label_pad: 12.0,
            button_border: hex(0xDBDBDB),
            raised: raised(),
            scene_w: 260.0,
            scene_pad: 8.0,
            scene_radius: 24.0,
            image_radius: 16.0,
            image_ratio: 256.0 / 196.0,
            image_gap: 8.0,
            text_pad: 12.0,
            title_gap: 4.0,
            title: TypeRole::BodyStrong.run(),
            category: TypeRole::Caption.run(),
            badge: 40.0,
            badge_radius: 8.0,
            badge_inset: 12.0,
            bars: [128.0, 72.0],
            bar_h: 8.0,
            bar_radius: 2.0,
            bar_pad: 4.0,
            lift: shadow_2xl(),
        };
        let explore = ExploreRecipe {
            width: 360.0,
            ratio: 360.0 / 288.0,
            radius: 24.0,
            scrim_alpha: 0.45,
            bar_pad: 12.0,
            title_pad: 10.0,
            button: 36.0,
            button_radius: 10.0,
            button_gap: 4.0,
            tag_w: 236.0,
            tag_pad: 8.0,
            tag_radius: 20.0,
            thumb: 64.0,
            thumb_radius: 12.0,
            text_gap: 16.0,
            line_gap: 4.0,
            tag_title: TypeRole::BodyStrong.run(),
            tag_line: TypeRole::Label.run(),
            lift: shadow_xl(),
            slide: 50.0,
            slide_radius: 12.0,
        };
        Self {
            palette,
            price,
            tile,
            explore,
        }
    }

    /// The kit's geometry in the theme's palette: its surface for the
    /// paper, its panel and hover washes, its divider and active border,
    /// its text, the mid text muted; the kit's photos.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        let kit = Self::hoff_core();
        Self {
            palette: CatalogPalette {
                paper: c.surface,
                panel: c.bg_panel,
                wash: c.bg_hover,
                line: c.divider,
                ring: c.border_active,
                ink: c.text,
                muted: c.text_mid,
                ..kit.palette
            },
            ..kit
        }
    }
}
