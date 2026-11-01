//! Header tokens (`theme.header`): what the hoff core kit writes as the
//! tailwind classes of its top bars (`components/HeaderSimple`,
//! `HeaderAsset`, `HeaderMinimal`) and its `Logo`, as values the header
//! widgets read. `HeaderTokens::hoff_core` is the kit measured; every
//! other theme derives the same shapes from its palette and its button
//! recipes ([`HeaderTokens::derive`]).
//!
//! The kit's pictures (the avatar, the asset thumbnails) are no pixels
//! here: each is a gradient, the kit's picture averaged at its top and
//! its bottom.

use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::legibility::{self, MARK_AA};
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{ButtonRecipe, ButtonSurface, Theme};

/// One look of the header's icon actions: the surface per state and the
/// square it fills.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionLook {
    pub recipe: ButtonRecipe,
    pub size: f32,
    pub radius: f32,
    pub icon: f32,
}

/// The dot of news on an action (`after:size-1 after:top-1
/// after:right-1 after:bg-[#fe5938]`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DotTokens {
    pub size: f32,
    /// From the action's top and end edges.
    pub inset: f32,
    pub color: Color,
}

/// The profile button: a round picture inside a ring that shows on
/// hover (`size-10 p-0.75 border rounded-full`, `size-8` picture,
/// `hover:border-[#121212]/10`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarTokens {
    pub size: f32,
    /// Border and padding between the ring and the picture.
    pub pad: f32,
    pub ring_w: f32,
    pub ring_hover: Color,
    /// The picture, top then bottom.
    pub picture: [Color; 2],
}

/// The asset header's strip of thumbnails (`size-12.5 border rounded-xl`,
/// the current one `size-15 shadow-md`, `gap-3`), with a fade at each end
/// (`w-15 bg-gradient-to-r from-[#FCFCFC]`) that goes while the pointer
/// is on the strip.
#[derive(Clone, Debug, PartialEq)]
pub struct ThumbTokens {
    pub size: f32,
    pub current: f32,
    pub radius: f32,
    pub border: Color,
    pub border_w: f32,
    pub gap: f32,
    pub fade: f32,
    pub shadow: Vec<ShadowSpec>,
    /// One picture per thumbnail, top then bottom; a strip longer than
    /// the list takes them again from the first.
    pub pictures: Vec<[Color; 2]>,
}

/// The logo (`components/Logo`): the symbol, `size-8`, and the wordmark,
/// `h-4`, `gap-3` apart inside a `p-1` link.
#[derive(Clone, Debug, PartialEq)]
pub struct LogoTokens {
    /// The Lucide icon drawn as the symbol.
    pub symbol: &'static str,
    pub symbol_size: f32,
    pub gap: f32,
    pub pad: f32,
    pub word: TextStyle,
    pub ink: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderTokens {
    /// The bar: `h-20 px-5 bg-[#FCFCFC]`.
    pub height: f32,
    pub pad_x: f32,
    pub fill: Color,
    /// Between items: `gap-4` (simple, minimal), `gap-3` (asset).
    pub gap: f32,
    pub gap_tight: f32,
    /// Between back and forward: `gap-2`.
    pub pair_gap: f32,
    /// Back and forward: quiet grey arrows that press in on hover.
    pub nav: ActionLook,
    /// The filter, the alerts and the search: no frame, a wash on hover.
    pub plain: ActionLook,
    /// The asset header's close, download and share: framed.
    pub framed: ActionLook,
    pub dot: DotTokens,
    /// The search box's width (`w-65`).
    pub search_w: f32,
    pub avatar: AvatarTokens,
    pub thumbs: ThumbTokens,
    pub logo: LogoTokens,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

fn rgba8(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::rgba(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        a,
    )
}

fn flat(c: Color) -> [Color; 2] {
    [c, c]
}

const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

/// `0 0 0 1px color`: a css border as the button painter draws it.
fn ring(color: Color) -> ShadowSpec {
    ShadowSpec::new([0.0, 0.0], 0.0, 1.0, color)
}

/// The same recipe at rest, hovered and pressed apart from `rest`.
fn lit(rest: ButtonSurface, hover: ButtonSurface, disabled_alpha: f32) -> ButtonRecipe {
    ButtonRecipe {
        rest,
        hover: hover.clone(),
        active: hover,
        disabled_alpha,
    }
}

impl HeaderTokens {
    /// The kit, values as its classes write them.
    pub fn hoff_core() -> Self {
        let ink = hex(0x121212);
        let paper = hex(0xFCFCFC);
        let shade3 = hex(0xF1F1F1);
        let shade5 = hex(0xE2E2E2);
        let quiet = hex(0x7B7B7B);
        // HeaderSimple :35, back and forward: `size-10 border
        // border-transparent rounded-[0.625rem] fill-[#7b7b7b]`, hover
        // `bg-[#f1f1f1] fill-[#121212] border-[#E2E2E2]` and the pressed-in
        // pair `0 -1px 3px rgba(18,18,18,.15) inset, 0 1.25px 1px #FFF
        // inset`; disabled `opacity-40`; the arrow `size-5`.
        let nav_hover = ButtonSurface {
            fill: flat(shade3),
            overlay: NONE,
            shadow: vec![
                ShadowSpec::inset([0.0, -1.0], 3.0, 0.0, rgba8(18, 18, 18, 0.15)),
                ShadowSpec::inset([0.0, 1.25], 1.0, 0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
                ring(shade5),
            ],
            text: ink,
        };
        let nav = ActionLook {
            recipe: lit(
                ButtonSurface {
                    fill: flat(NONE),
                    overlay: NONE,
                    shadow: Vec::new(),
                    text: quiet,
                },
                nav_hover,
                0.40,
            ),
            size: 40.0,
            radius: 10.0,
            icon: 20.0,
        };
        // HeaderSimple :71 and :84, HeaderMinimal :11: `size-10 rounded-xl
        // border-transparent hover:bg-[#f1f1f1]`, the icon `size-5
        // fill-primary` (no primary in the kit's css: the svg's black, the
        // kit's ink).
        let plain_rest = ButtonSurface {
            fill: flat(NONE),
            overlay: NONE,
            shadow: Vec::new(),
            text: ink,
        };
        let plain = ActionLook {
            recipe: lit(
                plain_rest.clone(),
                ButtonSurface {
                    fill: flat(shade3),
                    ..plain_rest
                },
                0.40,
            ),
            size: 40.0,
            radius: 12.0,
            icon: 20.0,
        };
        // HeaderAsset :7: `size-10 border border-[#e2e2e2] bg-[#fcfcfc]
        // rounded-xl hover:bg-[#f1f1f1]`, the icon `size-4 fill-[#121212]`.
        let framed_rest = ButtonSurface {
            fill: flat(paper),
            overlay: NONE,
            shadow: vec![ring(shade5)],
            text: ink,
        };
        let framed = ActionLook {
            recipe: lit(
                framed_rest.clone(),
                ButtonSurface {
                    fill: flat(shade3),
                    ..framed_rest
                },
                0.40,
            ),
            size: 40.0,
            radius: 12.0,
            icon: 16.0,
        };
        Self {
            height: 80.0,
            pad_x: 20.0,
            fill: paper,
            gap: 16.0,
            gap_tight: 12.0,
            pair_gap: 8.0,
            nav,
            plain,
            framed,
            // the kit's red deepens the least it must to read as a mark
            // on the bar and on the plain action's #F1F1F1 hover (1.4.11)
            dot: DotTokens {
                size: 4.0,
                inset: 4.0,
                color: legibility::reach(hex(0xFE5938), MARK_AA, &[&[paper], &[paper, shade3]]),
            },
            search_w: 260.0,
            avatar: AvatarTokens {
                size: 40.0,
                pad: 4.0,
                ring_w: 1.0,
                ring_hover: rgba8(18, 18, 18, 0.10),
                // `/images/avatars/1.png`, averaged.
                picture: [hex(0xA95628), hex(0x743C1E)],
            },
            thumbs: ThumbTokens {
                size: 50.0,
                current: 60.0,
                radius: 12.0,
                border: shade5,
                border_w: 1.0,
                gap: 12.0,
                fade: 60.0,
                // tailwind v4 `shadow-md`.
                shadow: vec![
                    ShadowSpec::new([0.0, 4.0], 6.0, -1.0, Color::rgba(0.0, 0.0, 0.0, 0.10)),
                    ShadowSpec::new([0.0, 2.0], 4.0, -2.0, Color::rgba(0.0, 0.0, 0.0, 0.10)),
                ],
                // `/images/3d-objects/5.png` to `9.png`, averaged.
                pictures: vec![
                    [hex(0xAAA197), hex(0xC0B09F)],
                    [hex(0x99BD66), hex(0x86B156)],
                    [hex(0xDECDA6), hex(0xD0BCA5)],
                    [hex(0xE4A867), hex(0xE29F5B)],
                    [hex(0xEAD5B4), hex(0xE9D2AD)],
                ],
            },
            logo: LogoTokens {
                symbol: "hexagon",
                symbol_size: 32.0,
                gap: 12.0,
                pad: 4.0,
                // The wordmark's capitals are 15 of its 16 px: the kit's
                // bold 20 px, in the heaviest face Inter embeds (600).
                word: crate::styling::theme::TypographyScale::hoff_core()
                    .role(crate::styling::theme::TypeRole::Wordmark),
                ink,
            },
        }
    }

    /// Every other theme: the kit's shapes over the theme's own colors.
    /// The bar is the page; the arrows are the toolbar's tool quiet at
    /// rest and selected on hover; the plain actions the toolbar's; the
    /// framed ones the outline button's; the dot the danger color; the
    /// pictures the theme's intents laid over its panel.
    pub fn derive(theme: &Theme) -> Self {
        let kit = Self::hoff_core();
        let c = &theme.colors;
        let b = &theme.button;
        let look = |recipe: &ButtonRecipe, like: &ActionLook| ActionLook {
            recipe: recipe.clone(),
            ..like.clone()
        };
        let mut nav = look(&b.toolbar_active, &kit.nav);
        nav.recipe.rest = ButtonSurface {
            fill: flat(NONE),
            overlay: NONE,
            shadow: Vec::new(),
            text: c.text_mid,
        };
        let tint = |top: Color, bottom: Color| {
            let mix = |x: Color| {
                let a = 0.55;
                Color([0, 1, 2, 3].map(|i| {
                    if i == 3 {
                        1.0
                    } else {
                        x.0[i] * a + c.bg_panel.0[i] * (1.0 - a)
                    }
                }))
            };
            [mix(top), mix(bottom)]
        };
        Self {
            fill: c.bg,
            nav,
            plain: look(&b.toolbar, &kit.plain),
            framed: look(&b.outline, &kit.framed),
            // the danger mark, on the bar and on the plain action's hover
            dot: DotTokens {
                color: legibility::reach(
                    c.danger,
                    MARK_AA,
                    &[&[c.bg], &[c.bg, b.toolbar.hover.fill[0]]],
                ),
                ..kit.dot
            },
            avatar: AvatarTokens {
                ring_hover: c.divider,
                picture: tint(c.warning, c.danger),
                ..kit.avatar
            },
            thumbs: ThumbTokens {
                border: c.divider,
                pictures: vec![
                    tint(c.text_mid, c.text_dim),
                    tint(c.success, c.success),
                    tint(c.warning, c.text_mid),
                    tint(c.warning, c.danger),
                    tint(c.info, c.text_mid),
                ],
                ..kit.thumbs
            },
            logo: LogoTokens {
                word: theme
                    .typography
                    .role(crate::styling::theme::TypeRole::Wordmark),
                ink: c.text,
                ..kit.logo
            },
            ..kit
        }
    }
}
