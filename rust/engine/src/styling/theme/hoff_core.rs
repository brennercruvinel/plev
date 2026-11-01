//! The hoff core theme: the light kit of the hoff research design system
//! (the private typescript kit, next + tailwind), transcribed token by
//! token. Nine shades of gray from paper (#FCFCFC) to ink (#121212), six
//! accents, Inter tracked -0.02em, radii 8 / 10 / 12 / 16 / 22 and the
//! measured shadow stacks of its depths page.

use super::intent::MotionPhysics;
use super::legibility::{self, MARK_AA, TEXT_AA};
use super::roles::TypeRoles;
use super::scales::{
    ControlTokens, DurationScale, LayoutTokens, ShadowSpec, ShadowTokens, ShapeTokens, SizeTokens,
};
use super::tokens::{
    ColorTokens, EffectTokens, GlassTokens, MONO_FAMILY, RadiusScale, SpacingScale, SwatchGroup,
    TypographyScale,
};
use super::{
    DisclosureTokens, FormTokens, MessageTokens, NavigationTokens, OverlayTokens, SelectionTokens,
    Theme,
};
use crate::styling::color::Color;
use crate::widgets::buttons::tokens::ButtonTokens;
use crate::widgets::header::tokens::HeaderTokens;

/// Shade 1..9 of the colors page, paper to ink.
pub const SHADES: [Color; 9] = [
    Color::hex(0xFCFCFC),
    Color::hex(0xF8F7F7),
    Color::hex(0xF1F1F1),
    Color::hex(0xECECEC),
    Color::hex(0xE2E2E2),
    Color::hex(0x7B7B7B),
    Color::hex(0x323232),
    Color::hex(0x222222),
    Color::hex(0x121212),
];

pub const GREEN: Color = Color::hex(0x49BA61);
pub const BLUE: Color = Color::hex(0x3582FF);
pub const ORANGE: Color = Color::hex(0xE36323);
pub const YELLOW: Color = Color::hex(0xFFB73A);
pub const RED: Color = Color::hex(0xFE5938);
pub const PURPLE: Color = Color::hex(0x8755E9);

/// The kit's secondary gray (Shade 6, #7B7B7B) where it is text: on its
/// paper it reads 4.13:1, under WCAG AA (1.4.3). It moves the least it
/// must (`legibility::reach`) to read at 4.5:1 on paper and on Shade 3,
/// the darkest surface the kit lays text on; the swatch keeps the kit's.
pub fn secondary_text() -> Color {
    legibility::reach(SHADES[5], TEXT_AA, &[&[SHADES[0]], &[SHADES[0], SHADES[2]]])
}

/// The core glass's placeholder, the ink at 40% (1.6:1 on paper): as
/// opaque as it must be to read at 4.5:1 on paper and on Shade 3. It is
/// what `settle` makes of `glass.text_placeholder`, for the kit's tokens
/// that copy it (the field's caption).
pub fn placeholder_text() -> Color {
    legibility::reach(ink(0.40), TEXT_AA, &[&[SHADES[0]], &[SHADES[0], SHADES[2]]])
}

/// The kit's red (#FE5938) where it is text, an error line: 3.05:1 on
/// paper. It deepens the least it must to read at 4.5:1 there.
pub fn red_text() -> Color {
    legibility::reach(RED, TEXT_AA, &[&[SHADES[0]]])
}

/// The kit's hairline (Shade 5, #E2E2E2) where it is the edge a person
/// must see to find a control (1.4.11), the field's border: 1.26:1 on
/// paper. It deepens the least it must to read at 3:1 on paper and on
/// Shade 2, the field's inset well.
pub fn control_edge() -> Color {
    legibility::reach(SHADES[4], MARK_AA, &[&[SHADES[0]], &[SHADES[0], SHADES[1]]])
}

/// The colors page (templates/ColorsPage/content.tsx): its two groups,
/// each color under the kit's name, Shade 8 and the yellow and purple
/// accents included, which no role holds.
pub const PALETTE: [SwatchGroup; 2] = [
    SwatchGroup {
        title: "Shades",
        colors: &[
            ("Shade 1", SHADES[0]),
            ("Shade 2", SHADES[1]),
            ("Shade 3", SHADES[2]),
            ("Shade 4", SHADES[3]),
            ("Shade 5", SHADES[4]),
            ("Shade 6", SHADES[5]),
            ("Shade 7", SHADES[6]),
            ("Shade 8", SHADES[7]),
            ("Shade 9", SHADES[8]),
        ],
    },
    SwatchGroup {
        title: "Colors",
        colors: &[
            ("Green", GREEN),
            ("Blue", BLUE),
            ("Orange", ORANGE),
            ("Yellow", YELLOW),
            ("Red", RED),
            ("Purple", PURPLE),
        ],
    },
];

fn ink(a: f32) -> Color {
    Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, a)
}

fn black(a: f32) -> Color {
    Color::rgba(0.0, 0.0, 0.0, a)
}

impl TypographyScale {
    /// The core ramp: Inter, tracking -0.02em, 10 / 11 / 13 / 15 / 20 /
    /// 32 / 48 with 16px steps of line height.
    pub fn hoff_core() -> Self {
        Self {
            small: 10.0,
            caption: 11.0,
            body_sm: 13.0,
            body: 15.0,
            title_sm: 20.0,
            title: 32.0,
            display: 48.0,
            family: Some("Inter"),
            mono_family: MONO_FAMILY,
            tracking_em: -0.02,
            roles: TypeRoles::hoff_core(),
            text_scale: super::text_scale::TextScale::IDENTITY,
        }
    }
}

impl ShadowTokens {
    /// The kit's depths (templates/DepthsPage/index.tsx), each stack
    /// listed bottom layer first, the reverse of the css list it copies.
    /// The page's "Subtle" and "Prompt Input" have no elevation here.
    pub fn hoff_core() -> Self {
        Self {
            // Slight hover (:49): 0 2px 8px -4px black/25.
            raised: vec![ShadowSpec::new([0.0, 2.0], 8.0, -4.0, black(0.25))],
            // Toolbar (:29): 0 1px 4px -4px black/8, 0 8px 16px -12px
            // black/13; the select's panel wears it too (Select :90).
            floating: vec![
                ShadowSpec::new([0.0, 8.0], 16.0, -12.0, black(0.13)),
                ShadowSpec::new([0.0, 1.0], 4.0, -4.0, black(0.08)),
            ],
            // Modal (:78), tailwind's `shadow-2xl`: 0 25px 50px -12px
            // black/25, as every modal and menu of the kit wears it
            // (FileMenu :33, DeleteFile :5, Settings :14).
            overlay: vec![ShadowSpec::new([0.0, 25.0], 50.0, -12.0, black(0.25))],
            // components/Tooltip/index.tsx:8: 0 4px 4px -2px black/40.
            hint: vec![ShadowSpec::new([0.0, 4.0], 4.0, -2.0, black(0.40))],
            // Large card hover (:87): 0 12px 11.1px -12px black/15, 0 80px
            // 64px -64px black/20, 0 16px 32px -24px black/62.
            card: vec![
                ShadowSpec::new([0.0, 16.0], 32.0, -24.0, black(0.62)),
                ShadowSpec::new([0.0, 80.0], 64.0, -64.0, black(0.20)),
                ShadowSpec::new([0.0, 12.0], 11.1, -12.0, black(0.15)),
            ],
        }
    }
}

impl ControlTokens {
    /// The kit's control measures, each from the class that writes it
    /// (paths relative to the kit root). What the kit leaves out keeps the
    /// HOFF value, named below.
    pub fn hoff_core() -> Self {
        Self {
            // Xs `h-7 px-2` (templates/ButtonsPage/index.tsx:373), Sm `h-9
            // px-5` (components/Button/index.tsx:59; Select :57, FileMenu
            // :36), Md `h-10 px-6` (Button :60), Lg `h-11 px-3.5`
            // (ButtonsPage :293), Xl the field, `h-12 px-5.5`
            // (components/Field/index.tsx:66).
            heights: [28.0, 36.0, 40.0, 44.0, 48.0],
            pad_x: [8.0, 20.0, 24.0, 14.0, 22.0],
            // `size-4` and `size-5` (Select :81, Field :81); the kit has no
            // 24 icon, so Lg keeps the HOFF one.
            icons: [16.0, 20.0, 24.0],
            // Not the kit's: WCAG 2.5.8 / the apple HIG.
            min_touch_target: 44.0,
            min_target: 24.0,
            // `border` on the card (components/Card/index.tsx:36) and
            // `border-[1.5px]` on the field (Field :66).
            edge_width: 1.0,
            edge_width_strong: 1.5,
            // The kit draws no focus ring (`outline-0`, Field :91); the
            // ring stays, for keyboard users.
            focus_ring_width: 2.0,
            focus_ring_offset: 2.0,
            // `p-0.75` (components/Tabs/index.tsx:26).
            tabs_pad: 3.0,
            // `p-2` (components/FileMenu/index.tsx:33).
            menu_pad: 8.0,
            // `gap-2` (Button :66).
            inline_gap: 8.0,
            // The kit has no checkbox: the HOFF box.
            box_size: 18.0,
            // `w-10 h-5.5 p-0.5` and the knob `size-4.5`
            // (components/Switch/index.tsx:11, :17); the derived pad is
            // the `p-0.5`.
            switch_size: [40.0, 22.0],
            switch_knob: 18.0,
            // The kit's range thumb is a 24 by 36 block (app/globals.css:22),
            // which a track and a knob diameter do not describe: the HOFF
            // slider until a slider recipe exists.
            slider_track: 4.0,
            slider_knob: 14.0,
            // The kit hides its scrollbars (`scrollbar-none`): the HOFF bar.
            scrollbar_w: 6.0,
            scrollbar_w_hover: 10.0,
            scrollbar_min_thumb: 24.0,
            scrollbar_inset: 2.0,
            // No spinner in the kit.
            spinner_stroke: 2.0,
            // The player's trail, `h-0.75`, filled at the same height
            // (components/VideoPlayer/index.tsx:48, :53).
            progress_track: 3.0,
            progress_fill: 3.0,
            // No split pane in the kit.
            divider_hit: 10.0,
            divider_line: 2.0,
        }
    }
}

impl Theme {
    /// The hoff core light theme.
    pub fn hoff_core() -> Self {
        let [
            paper,
            shade2,
            shade3,
            shade4,
            shade5,
            _shade6,
            shade7,
            _shade8,
            ink9,
        ] = SHADES;
        let colors = ColorTokens {
            bg: paper,
            surface: paper,
            bg_panel: shade2,
            bg_hover: shade3,
            text: ink9,
            text_mid: secondary_text(),
            text_dim: secondary_text(),
            accent: ink9,
            accent_dim: shade7,
            success: GREEN,
            danger: RED,
            warning: ORANGE,
            info: BLUE,
            divider: shade4,
            border_active: shade5,
            swatches: &PALETTE,
        };
        let radius = RadiusScale {
            none: 0.0,
            sm: 6.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            full: 9999.0,
        };
        let control = ControlTokens::hoff_core();
        let mut theme = Self {
            glass: GlassTokens {
                surface: paper,
                surface_hover: shade2,
                surface_active: shade3,
                button: shade3,
                tabs: shade2,
                button_hover: shade4,
                field: paper,
                field_focus_border: shade5,
                popover: paper,
                tooltip: ink9,
                scrim: Color::rgba(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 0.40),
                edge: shade4,
                edge_soft: shade4,
                inset_highlight: Color::rgba(1.0, 1.0, 1.0, 0.33),
                knob_gradient: [paper, shade2],
                text_gradient: [ink9, shade7],
                text_faint: secondary_text(),
                text_placeholder: placeholder_text(),
                text_active: ink9,
                text_default: secondary_text(),
                disabled_alpha: 0.30,
                wash_alpha: 0.10,
                wash_hover_alpha: 0.16,
                // The kit's surfaces have a plain `border` (Card :36,
                // Select :90, FileMenu :33) and no inset key-light.
                rim_fade: 1.0,
                keylight: None,
            },
            button: Box::new(ButtonTokens::hoff_core()),
            form: Box::new(FormTokens::hoff_core()),
            overlay: Box::new(OverlayTokens::hoff_core()),
            message: Box::new(MessageTokens::hoff_core()),
            disclosure: Box::new(DisclosureTokens::hoff_core()),
            selection: Box::new(SelectionTokens::hoff_core()),
            navigation: Box::new(NavigationTokens::hoff_core()),
            header: Box::new(HeaderTokens::hoff_core()),
            colors,
            typography: TypographyScale::hoff_core(),
            spacing: SpacingScale {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
                xxl: 48.0,
            },
            radius,
            motion: MotionPhysics {
                mass: 1.0,
                stiffness: 170.0,
                damping: 26.0,
            },
            effects: EffectTokens {
                shadow_sigma: 4.0,
                shadow_color: black(0.15),
                blur_sigma: 8.0,
            },
            // Rounded-xl (12) on controls, 16 on cards, 22 on wide cards,
            // 8 on chips, 6 on tooltips.
            shape: ShapeTokens {
                pill: 12.0,
                dropdown: 16.0,
                tabs: 12.0,
                card: 16.0,
                block: 10.0,
                item: 8.0,
                nav: 12.0,
                cluster: 10.0,
                tooltip: 6.0,
                micro: 6.0,
            },
            control,
            size: SizeTokens::hoff(),
            duration: DurationScale::hoff(),
            layout: LayoutTokens::hoff_core(),
            shadows: ShadowTokens::hoff_core(),
        };
        // The kit's secondary gray (#7B7B7B) as text, its placeholder ink
        // at 40%, its green and its #E2E2E2 active border fall under WCAG
        // AA on its paper: each moves the least it must (`legibility`).
        theme.settle_colors(super::Bar::AA);
        // The sidebar keeps its look of before until it moves to the kit
        // (task-104 front G, step 4); that look reads the finished theme.
        theme.navigation.sidebar_link = NavigationTokens::sidebar_link_of_before(&theme);
        theme
    }
}
