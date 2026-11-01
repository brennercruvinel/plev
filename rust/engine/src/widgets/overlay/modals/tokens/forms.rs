//! Export's and Settings' recipes: `components/Export` and
//! `components/Settings` as their tailwind classes write them. Both are a
//! side menu ([`NavTokens`]) beside the section it picks.

use crate::styling::color::Color;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{Theme, TypeRole, TypeRun};

use super::{PanelTokens, hex, rgba};

/// The side menu (`Export/Menu`, the Settings menu): a column `p-*` with
/// a rule at its end, its items `border rounded-xl text-[0.75rem]
/// font-medium`, the active one `bg-[#F1F1F1] border-[#E2E2E2]`, each
/// with its glyph in a `size-8 rounded-lg` box (raised on paper while
/// active).
#[derive(Clone, Debug, PartialEq)]
pub struct NavTokens {
    pub width: f32,
    pub pad: f32,
    pub item_gap: f32,
    /// An item's padding around its box, and at its end.
    pub item_pad: f32,
    pub item_pad_end: f32,
    pub item_radius: f32,
    /// Between the box and the label.
    pub gap: f32,
    pub label: TypeRun,
    pub box_size: f32,
    pub box_radius: f32,
    /// A resting box's fill (alpha 0 is none).
    pub box_rest: Color,
    pub icon: f32,
    /// The chevron at the active item's end (`size-4`); 0 is none.
    pub chevron: f32,
}

impl NavTokens {
    fn derive(&self, theme: &Theme, box_rest: Color) -> Self {
        Self {
            item_radius: theme.shape.nav,
            box_radius: theme.shape.tooltip,
            box_rest,
            ..self.clone()
        }
    }
}

/// Export's preview (`Export/Preview`: `mt-6 border border-[#323232]/10
/// rounded-xl bg-[#f8f7f7]`, the picture 136 by 96; its button `top-1.5
/// right-1.5 size-6 bg-[#fcfcfc] rounded-md`, raised, the glyph `size-4`
/// or, for the video's play, `size-3`).
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewTokens {
    pub top: f32,
    pub border: Color,
    pub radius: f32,
    /// The picture's height over its width (`96 / 136`).
    pub aspect: f32,
    pub button: f32,
    pub button_inset: f32,
    pub button_radius: f32,
    pub button_shadow: Vec<ShadowSpec>,
    pub icon: f32,
    pub play: f32,
}

/// A labelled line of a section (`Export/Line`: `flex`, the title `pt-2.5
/// pr-3 text-[0.75rem] font-semibold`, the control `shrink-0 w-40`).
#[derive(Clone, Debug, PartialEq)]
pub struct LineTokens {
    pub title: TypeRun,
    pub title_top: f32,
    pub title_gap: f32,
    pub control_w: f32,
    /// Between lines (`gap-1.5`) and after them (`mb-3`).
    pub gap: f32,
    pub bottom: f32,
}

/// `components/Export`: `flex w-137.5 min-h-79.5`, on the ModalPage
/// `bg-[#FCFCFC] border border-[#ECECEC] rounded-[1.25rem] shadow-2xl`;
/// the menu `w-38 p-2 border-r`, the section `p-4`. Images: the head
/// (`h-6 mb-4`, `text-[0.75rem] leading-[1rem] font-medium`, a `size-6`
/// add), the sizes (`gap-1.5`; `w-20` and `w-33` selects, the format
/// tabs, a `size-6` remove `ml-1`), the compression (`p-4 border-t`, its
/// head `mb-2`, the range `h-9` beside a `w-18 h-9 border rounded-[0.625rem]`
/// value), the export button. Video and 3D object: lines, the estimate
/// (`mb-2 text-[0.625rem] text-[#7B7B7B]/80`) and the button.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportTokens {
    pub panel: PanelTokens,
    pub min_h: f32,
    pub nav: NavTokens,
    pub preview: PreviewTokens,
    pub pad: f32,
    pub head_h: f32,
    pub head_bottom: f32,
    pub head: TypeRun,
    /// The small square buttons (add, remove, close): edge, glyph, radius.
    pub tool: f32,
    pub tool_icon: f32,
    pub tool_radius: f32,
    pub row_gap: f32,
    pub size_w: f32,
    pub color_w: f32,
    pub tool_gap: f32,
    pub range_h: f32,
    pub value_w: f32,
    pub value_radius: f32,
    pub value_icon: f32,
    pub value: TypeRun,
    /// The small notes: the dimensions, the estimate.
    pub note: TypeRun,
    pub note_gap: f32,
    /// The dimensions under the resolution (`mt-0.5`).
    pub note_top: f32,
    pub line: LineTokens,
}

impl ExportTokens {
    pub(super) fn hoff_core() -> Self {
        Self {
            panel: PanelTokens::kit(550.0, 20.0, true),
            min_h: 318.0,
            nav: NavTokens {
                width: 152.0,
                pad: 8.0,
                item_gap: 4.0,
                item_pad: 4.0,
                item_pad_end: 8.0,
                item_radius: 12.0,
                gap: 12.0,
                // text-[0.75rem] font-medium (the preflight's 1.5 line).
                label: TypeRole::Label.run(),
                box_size: 32.0,
                box_radius: 8.0,
                box_rest: hex(0xF1F1F1),
                icon: 16.0,
                chevron: 16.0,
            },
            preview: PreviewTokens {
                top: 24.0,
                border: rgba(0x32, 0x32, 0x32, 0.10),
                radius: 12.0,
                aspect: 96.0 / 136.0,
                button: 24.0,
                button_inset: 6.0,
                button_radius: 6.0,
                button_shadow: vec![
                    ShadowSpec::new([0.0, 1.25], 3.0, 0.0, rgba(50, 50, 50, 0.10)),
                    ShadowSpec::inset([0.0, 1.25], 1.0, 0.0, hex(0xFFFFFF)),
                ],
                icon: 16.0,
                play: 12.0,
            },
            pad: 16.0,
            head_h: 24.0,
            head_bottom: 16.0,
            head: TypeRole::Label.run(),
            tool: 24.0,
            tool_icon: 16.0,
            tool_radius: 6.0,
            row_gap: 6.0,
            size_w: 80.0,
            color_w: 132.0,
            tool_gap: 4.0,
            range_h: 36.0,
            value_w: 72.0,
            value_radius: 10.0,
            value_icon: 16.0,
            value: TypeRole::Label.run(),
            // text-[0.625rem] at medium (the preflight's 1.5 line).
            note: TypeRole::Small.medium(),
            note_gap: 8.0,
            note_top: 2.0,
            line: LineTokens {
                title: TypeRole::BodyStrong.run(),
                title_top: 10.0,
                title_gap: 12.0,
                control_w: 160.0,
                gap: 6.0,
                bottom: 12.0,
            },
        }
    }

    pub(super) fn derive(&self, theme: &Theme) -> Self {
        let s = &theme.shape;
        Self {
            panel: self.panel.derive(theme, s.card),
            nav: self.nav.derive(theme, theme.colors.bg_hover),
            preview: PreviewTokens {
                border: theme.colors.divider,
                radius: s.nav,
                button_radius: s.micro,
                button_shadow: theme.shadows.raised.clone(),
                ..self.preview.clone()
            },
            tool_radius: s.micro,
            value_radius: s.cluster,
            ..self.clone()
        }
    }
}

/// A row of a settings section (`Settings/Option`: `flex items-center
/// min-h-14 px-6 py-4 border-t`, the title `text-[0.75rem]
/// leading-[1rem] font-medium` `pr-3`, a description under it `mt-2
/// text-[0.6875rem] leading-[1rem] font-medium text-[#7B7B7B]/80`).
#[derive(Clone, Debug, PartialEq)]
pub struct OptionTokens {
    pub min_h: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub title: TypeRun,
    pub title_gap: f32,
    pub description: TypeRun,
    pub description_gap: f32,
    /// A plain value at the row's end (the email, the display name).
    pub value: TypeRun,
    /// The selects' least width (`min-w-32`).
    pub select_w: f32,
    /// The avatar's upload (`UploadAvatar`: `size-12 rounded-full`, while
    /// hovered `bg-black/40` and a `size-5 fill-[#FCFCFC]` add over it).
    pub upload: f32,
    pub upload_scrim: Color,
    pub upload_icon: f32,
    pub upload_ink: Color,
}

/// The subscription's plan (`Settings/Subscription`: `p-3 border-t`; the
/// renewal `px-3 py-4` with a `size-5` glyph; the plan `border
/// rounded-[1.25rem]`, its head `p-3 bg-[#f8f7f7] border
/// rounded-[1.25rem]` lit from the top, the price `$` `text-[1.5rem]
/// leading-[2rem] text-[#7b7b7b] px-0.5 py-1`, `20` `text-[2.5rem]
/// leading-[3rem]`, the period `p-2 text-[0.75rem] leading-[1rem]
/// font-medium text-[#7b7b7b]`, the buttons `mt-2 gap-2`; the features
/// `p-3 gap-2`, a `size-5` check `ml-0.5 mr-2.5` and `text-[0.8125rem]
/// leading-[1.2188rem]`).
#[derive(Clone, Debug, PartialEq)]
pub struct PlanTokens {
    pub pad: f32,
    pub renew_pad_x: f32,
    pub renew_pad_y: f32,
    pub renew_icon: f32,
    pub radius: f32,
    pub head_pad: f32,
    pub head_shadow: Vec<ShadowSpec>,
    pub currency: TypeRun,
    pub currency_pad: [f32; 2],
    pub price: TypeRun,
    pub period: TypeRun,
    pub period_pad: f32,
    pub buttons_top: f32,
    pub buttons_gap: f32,
    pub features_pad: f32,
    pub features_gap: f32,
    pub check: f32,
    pub check_start: f32,
    pub check_gap: f32,
    pub feature: TypeRun,
}

/// `components/Settings`: `flex w-148.5 min-h-112 rounded-3xl
/// bg-[#FCFCFC] shadow-2xl`, the menu `w-45 p-3 gap-0.5 border-r` (items
/// `p-0.75 pr-2.5 gap-3`, glyphs `size-5`), the section's title (`px-6
/// py-4 text-[1.125rem] leading-[1.6875rem] font-medium`) over its rows.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsTokens {
    pub panel: PanelTokens,
    pub min_h: f32,
    pub nav: NavTokens,
    pub title: TypeRun,
    pub title_pad_x: f32,
    pub title_pad_y: f32,
    pub option: OptionTokens,
    pub plan: PlanTokens,
}

impl SettingsTokens {
    pub(super) fn hoff_core() -> Self {
        let none = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let clear = rgba(0, 0, 0, 0.0);
        Self {
            panel: PanelTokens::kit(594.0, 24.0, false),
            min_h: 448.0,
            nav: NavTokens {
                width: 180.0,
                pad: 12.0,
                item_gap: 2.0,
                item_pad: 3.0,
                item_pad_end: 10.0,
                item_radius: 12.0,
                gap: 12.0,
                label: TypeRole::Label.run(),
                box_size: 32.0,
                box_radius: 8.0,
                box_rest: none,
                icon: 20.0,
                chevron: 0.0,
            },
            title: TypeRole::TitleLg.medium(),
            title_pad_x: 24.0,
            title_pad_y: 16.0,
            option: OptionTokens {
                min_h: 56.0,
                pad_x: 24.0,
                pad_y: 16.0,
                title: TypeRole::Label.run(),
                title_gap: 12.0,
                description: TypeRole::Caption.run(),
                description_gap: 8.0,
                value: TypeRole::Label.run(),
                select_w: 128.0,
                upload: 48.0,
                upload_scrim: rgba(0, 0, 0, 0.40),
                upload_icon: 20.0,
                upload_ink: hex(0xFCFCFC),
            },
            plan: PlanTokens {
                pad: 12.0,
                renew_pad_x: 12.0,
                renew_pad_y: 16.0,
                renew_icon: 20.0,
                radius: 20.0,
                head_pad: 12.0,
                // shadow-[0px_2px_2px_0px_rgba(255,255,255,0.80)_inset,
                // ...0px_6px_3px_0px_rgba(0,0,0,0.01),
                // 0px_3px_3px_0px_rgba(0,0,0,0.02),
                // 0px_1px_1px_0px_rgba(0,0,0,0.02)]; the two at 0 alpha
                // draw nothing.
                head_shadow: vec![
                    ShadowSpec::inset([0.0, 2.0], 2.0, 0.0, rgba(255, 255, 255, 0.80)),
                    ShadowSpec::new([0.0, 16.0], 4.0, 0.0, clear),
                    ShadowSpec::new([0.0, 6.0], 3.0, 0.0, rgba(0, 0, 0, 0.01)),
                    ShadowSpec::new([0.0, 3.0], 3.0, 0.0, rgba(0, 0, 0, 0.02)),
                    ShadowSpec::new([0.0, 1.0], 1.0, 0.0, rgba(0, 0, 0, 0.02)),
                ],
                currency: TypeRole::H5.regular(),
                currency_pad: [2.0, 4.0],
                price: TypeRole::H2.run(),
                period: TypeRole::Label.run(),
                period_pad: 8.0,
                buttons_top: 8.0,
                buttons_gap: 8.0,
                features_pad: 12.0,
                features_gap: 8.0,
                check: 20.0,
                check_start: 2.0,
                // gap-2 and mr-2.5.
                check_gap: 18.0,
                feature: TypeRole::Paragraph.run(),
            },
        }
    }

    pub(super) fn derive(&self, theme: &Theme) -> Self {
        let p = &self.plan;
        Self {
            panel: self.panel.derive(theme, theme.shape.dropdown),
            nav: self.nav.derive(theme, self.nav.box_rest),
            plan: PlanTokens {
                radius: theme.shape.card,
                head_shadow: theme.shadows.raised.clone(),
                ..p.clone()
            },
            ..self.clone()
        }
    }
}
