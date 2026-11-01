//! The prompt input's tokens: the kit's `components/PromptInput` (the
//! composer with its row of controls), its model menu
//! (`PromptInput/SelectAi`, and the standalone `components/SelectAI`),
//! its add-files menu (`PromptInput/AddFiles`), its voice mark and its
//! premade prompts (`PromptInput/PremadePrompt`).
//!
//! [`PromptTokens::hoff_core`] is the kit as written, the file:line of
//! each value beside it; [`PromptTokens::derive`] lays the same geometry
//! over another theme's palette.

use super::MessageText;
use crate::styling::color::Color;
use crate::styling::theme::hoff_core::{ORANGE, SHADES};
use crate::styling::theme::{ShadowSpec, Theme};

/// A floating panel of the kit: the model menu's listbox and the
/// add-files menu (`w-51 p-2 rounded-[1.25rem] border`).
#[derive(Clone, Debug, PartialEq)]
pub struct PanelTokens {
    pub w: f32,
    pub pad: f32,
    pub radius: f32,
    pub fill: Color,
    pub border: Color,
    pub border_width: f32,
    /// The css `box-shadow` list, the inset paper ring among them.
    pub shadow: Vec<ShadowSpec>,
    /// Between the panel and what opened it (`--anchor-gap`).
    pub gap: f32,
}

/// One model in the model menu: its title over its description.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChoiceTokens {
    pub pad_x: f32,
    pub pad_y: f32,
    pub radius: f32,
    pub title: MessageText,
    pub description: MessageText,
    /// Between the title and the description.
    pub description_gap: f32,
    /// The row under the pointer or the keyboard, and the chosen one.
    pub hover: Color,
    pub selected: Color,
}

/// One row of the add-files menu: an icon in its box, then the label.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuRowTokens {
    pub h: f32,
    pub pad_x: f32,
    pub gap: f32,
    pub radius: f32,
    pub icon: f32,
    pub icon_box: f32,
    pub text: MessageText,
    pub icon_color: Color,
    pub icon_hover: Color,
    pub hover: Color,
}

/// The model menu's trigger: the model's name and a chevron.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriggerTokens {
    pub pad_x: f32,
    pub gap: f32,
    pub radius: f32,
    pub min_w: f32,
    pub text: MessageText,
    pub chevron: f32,
    pub chevron_color: Color,
    /// Hovered or open.
    pub hover: Color,
}

/// The voice input: the recording mark in the box's corner, and how faint
/// the controls voice leaves out get.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceTokens {
    pub ring: f32,
    pub ring_width: f32,
    pub ring_color: Color,
    pub dot: f32,
    /// The dot's fill, top to bottom.
    pub dot_fill: [Color; 2],
    /// From the box's top and end edges to the ring.
    pub inset: f32,
    /// The alpha of a control voice leaves out (`opacity-30`).
    pub dim: f32,
    /// The room the text keeps clear of the mark at its end (`pr-10`).
    pub text_room: f32,
}

/// The premade prompts over the box: a list of suggestions, then a
/// footer of categories on the well.
#[derive(Clone, Debug, PartialEq)]
pub struct PremadeTokens {
    /// Between the panel and the box under it.
    pub gap: f32,
    pub radius: f32,
    pub fill: Color,
    pub border: Color,
    pub border_width: f32,
    pub shadow: Vec<ShadowSpec>,
    pub list_pad: f32,
    pub row_gap: f32,
    pub row_pad: f32,
    pub row_radius: f32,
    pub row: MessageText,
    pub row_hover: Color,
    pub row_hover_border: Color,
    pub footer_pad: f32,
    pub footer_gap: f32,
    pub footer_fill: Color,
}

/// The prompt input of the kit.
#[derive(Clone, Debug, PartialEq)]
pub struct PromptTokens {
    /// The widest the box grows; narrower where its column is.
    pub max_w: f32,
    pub pad: f32,
    pub radius: f32,
    pub fill: Color,
    pub border: Color,
    pub border_hover: Color,
    pub border_width: f32,
    pub shadow: Vec<ShadowSpec>,
    /// The text: its run, its placeholder's color, the space above and
    /// under it and its inset from the start.
    pub text: MessageText,
    pub placeholder: Color,
    pub text_top: f32,
    pub text_bottom: f32,
    pub text_inset: f32,
    /// The row's controls are this tall (and the square ones this wide),
    /// this far apart.
    pub control: f32,
    pub row_gap: f32,
    /// The least the setting's select takes in the row.
    pub setting_min: f32,
    pub trigger: TriggerTokens,
    pub panel: PanelTokens,
    pub choice: ChoiceTokens,
    /// The standalone model menu's drop (`shadow-2xl`).
    pub menu_shadow: Vec<ShadowSpec>,
    pub files: MenuRowTokens,
    pub voice: VoiceTokens,
    pub premade: PremadeTokens,
    /// Between the view controller and the box under it.
    pub view_gap: f32,
}

/// `components/PromptInput/index.tsx:34`, `components/Comment/index.tsx:21`
/// and `components/NewComment/index.tsx:18` share this shadow list.
pub(crate) fn raised_shadow() -> Vec<ShadowSpec> {
    vec![
        ShadowSpec::new([0.0, 18.0], 24.0, -20.0, Color::rgba(0.0, 0.0, 0.0, 0.13)),
        ShadowSpec::inset([0.0, 2.0], 0.0, 0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
        ShadowSpec::new([0.0, 8.0], 16.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.08)),
    ]
}

/// tailwind's `shadow-2xl`: `0 25px 50px -12px rgb(0 0 0 / 0.25)`.
pub(crate) fn shadow_2xl() -> Vec<ShadowSpec> {
    vec![ShadowSpec::new(
        [0.0, 25.0],
        50.0,
        -12.0,
        Color::rgba(0.0, 0.0, 0.0, 0.25),
    )]
}

impl PromptTokens {
    /// The kit as written.
    pub fn hoff_core() -> Self {
        let [
            paper,
            shade2,
            shade3,
            shade4,
            shade5,
            shade6,
            shade7,
            _,
            ink,
        ] = SHADES;
        Self {
            // PromptInput/index.tsx:34 `w-135.5 p-3 border
            // border-[#ececec] bg-[#fcfcfc] rounded-3xl`, the shadow list,
            // `hover:border-[#E2E2E2]`
            max_w: 542.0,
            pad: 12.0,
            radius: 24.0,
            fill: paper,
            border: shade4,
            border_hover: shade5,
            border_width: 1.0,
            shadow: raised_shadow(),
            // :44 `!h-6 mt-2 mb-8 pl-2 text-[0.9375rem] leading-[1.5rem]
            // text-[#121212] placeholder:text-[#7b7b7b]`. The placeholder
            // is 4.1:1 on paper: a hint, kept as the kit's and listed
            // with the theme's placeholders (styling/theme/tests/contrast.rs)
            text: MessageText {
                size: 15.0,
                line_height: 24.0,
                weight: 400,
                color: ink,
            },
            placeholder: shade6,
            text_top: 8.0,
            text_bottom: 32.0,
            text_inset: 8.0,
            // :58 `flex gap-2`; :85 `size-10`; :65 `min-w-38.5`
            control: 40.0,
            row_gap: 8.0,
            setting_min: 154.0,
            // PromptInput/SelectAi/index.tsx:20 `min-w-37`, :25 `gap-2 h-10
            // px-3 rounded-xl text-[0.875rem] font-medium
            // data-[hover]:bg-[#f1f1f1]`, :28 `size-4 fill-[#7b7b7b]`
            trigger: TriggerTokens {
                pad_x: 12.0,
                gap: 8.0,
                radius: 12.0,
                min_w: 148.0,
                text: MessageText {
                    size: 14.0,
                    line_height: 20.0,
                    weight: 500,
                    color: ink,
                },
                chevron: 16.0,
                chevron_color: shade6,
                hover: shade3,
            },
            // SelectAi/index.tsx:37 `w-51 [--anchor-gap:0.5rem] p-2
            // bg-[#fcfcfc] border border-[#e2e2e2] rounded-[1.25rem]` and
            // its shadow list (AddFiles/index.tsx:46 the same)
            panel: PanelTokens {
                w: 204.0,
                pad: 8.0,
                radius: 20.0,
                fill: paper,
                border: shade5,
                border_width: 1.0,
                shadow: vec![
                    ShadowSpec::inset([0.0, 0.0], 0.0, 2.0, Color::WHITE),
                    ShadowSpec::new([0.0, 39.0], 24.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.03)),
                    ShadowSpec::new([0.0, 18.0], 18.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.05)),
                    ShadowSpec::new([0.0, 4.0], 10.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.06)),
                ],
                gap: 8.0,
            },
            // SelectAi/index.tsx:44 `px-3 py-2.5 rounded-xl
            // data-[focus]:bg-[#f8f7f7] data-[selected]:bg-[#f1f1f1]`, :48
            // `text-[0.8125rem] leading-[1rem] font-medium`, :51 `mt-1
            // text-[0.6875rem] leading-[1rem] font-medium
            // text-[#7b7b7b]/70`. That is 2.6:1 on paper; the description
            // takes shade 7 for AAA, as the alert's time
            choice: ChoiceTokens {
                pad_x: 12.0,
                pad_y: 10.0,
                radius: 12.0,
                title: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 500,
                    color: ink,
                },
                description: MessageText {
                    size: 11.0,
                    line_height: 16.0,
                    weight: 500,
                    color: shade7,
                },
                description_gap: 4.0,
                hover: shade2,
                selected: shade3,
            },
            // components/SelectAI/index.tsx:5 `shadow-2xl`
            menu_shadow: shadow_2xl(),
            // AddFiles/index.tsx:53 `gap-2.5 h-10 px-2 rounded-xl
            // text-[0.8125rem] font-medium text-[#121212]
            // hover:bg-[#f1f1f1]`, :58 `size-6`, :60 `size-5
            // fill-[#7b7b7b] group-hover:fill-[#121212]`
            files: MenuRowTokens {
                h: 40.0,
                pad_x: 8.0,
                gap: 10.0,
                radius: 12.0,
                icon: 20.0,
                icon_box: 24.0,
                text: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 500,
                    color: ink,
                },
                icon_color: shade6,
                icon_hover: ink,
                hover: shade3,
            },
            // PromptInput/index.tsx:40 `top-6 right-6 size-4 border
            // border-[#e36323]/10`, `before:size-2 before:from-[#FF732D]
            // before:to-[#E24D03]`; :61 `opacity-30`; :45 `pr-10`
            voice: VoiceTokens {
                ring: 16.0,
                ring_width: 1.0,
                ring_color: Color([ORANGE.0[0], ORANGE.0[1], ORANGE.0[2], 0.1]),
                dot: 8.0,
                dot_fill: [Color::hex(0xFF732D), Color::hex(0xE24D03)],
                inset: 24.0,
                dim: 0.3,
                text_room: 40.0,
            },
            // PremadePrompt/index.tsx:11 `mb-3 bg-[#FCFCFC] border
            // border-[#ECECEC] rounded-3xl` and its shadow list; :12 `gap-0.5
            // p-3`; :15 `p-3 border rounded-[0.625rem] text-[0.8125rem]
            // leading-[1rem] hover:border-[#ECECEC] hover:bg-[#F8F7F7]`;
            // :22 `gap-1.5 p-3 border-t border-[#ECECEC] bg-[#F8F7F7]`
            premade: PremadeTokens {
                gap: 12.0,
                radius: 24.0,
                fill: paper,
                border: shade4,
                border_width: 1.0,
                shadow: vec![
                    ShadowSpec::new([0.0, 6.0], 3.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.01)),
                    ShadowSpec::new([0.0, 3.0], 3.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.02)),
                    ShadowSpec::new([0.0, 1.0], 1.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.02)),
                ],
                list_pad: 12.0,
                row_gap: 2.0,
                row_pad: 12.0,
                row_radius: 10.0,
                row: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 400,
                    color: ink,
                },
                row_hover: shade2,
                row_hover_border: shade4,
                footer_pad: 12.0,
                footer_gap: 6.0,
                footer_fill: shade2,
            },
            // components/ViewController/index.tsx:11 `bottom-full mb-4`
            view_gap: 16.0,
        }
    }

    /// The kit's geometry over `theme`'s palette: the surfaces on its
    /// paper with its divider, the text in its text tone, the hints and the
    /// icons in its muted one, the hover and the chosen rows in its hover
    /// and panel washes. Reads only `theme.colors`.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        let kit = Self::hoff_core();
        let paper = Color([c.bg.0[0], c.bg.0[1], c.bg.0[2], 1.0]);
        let drops = |list: &[ShadowSpec]| list.iter().filter(|s| !s.inset).copied().collect();
        let hairline_hover = Color([c.text_mid.0[0], c.text_mid.0[1], c.text_mid.0[2], 0.35]);
        Self {
            fill: paper,
            border: c.divider,
            border_hover: hairline_hover,
            shadow: drops(&kit.shadow),
            text: MessageText {
                color: c.text,
                ..kit.text
            },
            placeholder: c.text_mid,
            trigger: TriggerTokens {
                text: MessageText {
                    color: c.text,
                    ..kit.trigger.text
                },
                chevron_color: c.text_mid,
                hover: c.bg_hover,
                ..kit.trigger
            },
            panel: PanelTokens {
                fill: paper,
                border: c.divider,
                shadow: drops(&kit.panel.shadow),
                ..kit.panel.clone()
            },
            choice: ChoiceTokens {
                title: MessageText {
                    color: c.text,
                    ..kit.choice.title
                },
                description: MessageText {
                    color: c.text_mid,
                    ..kit.choice.description
                },
                hover: c.bg_panel,
                selected: c.bg_hover,
                ..kit.choice
            },
            files: MenuRowTokens {
                text: MessageText {
                    color: c.text,
                    ..kit.files.text
                },
                icon_color: c.text_mid,
                icon_hover: c.text,
                hover: c.bg_hover,
                ..kit.files
            },
            premade: PremadeTokens {
                fill: paper,
                border: c.divider,
                row: MessageText {
                    color: c.text,
                    ..kit.premade.row
                },
                row_hover: c.bg_panel,
                row_hover_border: c.divider,
                footer_fill: c.bg_panel,
                ..kit.premade.clone()
            },
            ..kit
        }
    }
}
