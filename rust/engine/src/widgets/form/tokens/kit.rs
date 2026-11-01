//! The hoff core kit's form recipe, class list by class list
//! (`FormTokens::hoff_core`).

use super::{
    Adornments, AreaTokens, CAPTION, ColorTokens, EditTokens, FieldChrome, FormTokens, GroupAddons,
    InputRecipe, InputSurface, LabelRecipe, OtpTokens, SelectRecipe, UploadTokens,
};
use crate::styling::color::Color;
use crate::styling::theme::hoff_core;
use crate::styling::theme::legibility::{self, TEXT_AA};
use crate::styling::theme::{ShadowSpec, TypeRole};

fn hex(v: u32) -> Color {
    Color::hex(v)
}

fn black(a: f32) -> Color {
    Color::rgba(0.0, 0.0, 0.0, a)
}

/// The kit's ink, `rgba(18,18,18,a)`.
fn ink_a(a: f32) -> Color {
    Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, a)
}

const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

impl FormTokens {
    /// The hoff core kit, values as written in its class lists
    /// (`components/Field/index.tsx`; the body's `text-[#121212]
    /// tracking-[-0.02em]`, `app/layout.tsx:108`).
    pub fn hoff_core() -> Self {
        let gray = hex(0x7B7B7B);
        // The gray where it is text (a placeholder, a hint, a link), and
        // the kit's red and hairline where they must read (WCAG AA, 1.4.3
        // and 1.4.11): each moved the least it must (`hoff_core`).
        let quiet = hoff_core::secondary_text();

        // :66, the input: `border-[1.5px] border-[#E2E2E2] rounded-xl`, no
        // background; hover and focus add `shadow-[0px_1px_4px_-1px_
        // rgba(0,0,0,0.15)]` and keep the border.
        let border = hex(0xE2E2E2);
        let lifted = vec![ShadowSpec::new([0.0, 1.0], 4.0, -1.0, black(0.15))];
        let rest = InputSurface {
            fill: NONE,
            border,
            shadow: Vec::new(),
        };
        let hover = InputSurface {
            shadow: lifted,
            ..rest.clone()
        };
        let ink = hex(0x121212);
        let input = InputRecipe {
            focus: hover.clone(),
            rest,
            hover,
            // :66 `h-12 px-5.5`.
            height: 48.0,
            pad_x: 22.0,
            radius: 12.0,
            border_width: 1.5,
            // :66 `text-[0.8125rem] leading-[1rem] text-[#000]
            // placeholder:text-[#7B7B7B]`.
            text: TypeRole::BodyLg.run(),
            text_color: hex(0x000000),
            placeholder: quiet,
            // Not in the kit (the browser's caret and selection): the
            // body ink and its wash.
            caret: ink,
            selection: Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.10),
            composing: ink,
            // Not in the kit (it has no disabled field): the kit's
            // disabled button, `opacity-30`.
            disabled_alpha: 0.30,
        };

        let adorn = Adornments {
            // :51 `w-10`, :70 `pl-10`; the page's icon is `size-4`.
            lead_slot: 40.0,
            lead_icon: 16.0,
            // :67 `pr-10`; :81 `right-3.5 size-5 fill-[#7B7B7B]`.
            check_slot: 40.0,
            check_icon: 20.0,
            check_inset: 14.0,
            // :91 `w-12`, :68 `pr-12`; :96-98 a 20px eye in `#7B7B7B`.
            reveal_slot: 48.0,
            reveal_icon: 20.0,
            icon_color: gray,
        };

        // :35-36 the label `mb-2 text-[0.75rem] leading-[1rem]
        // font-medium`; :41 the link, `text-[#7b7b7b] opacity-70
        // hover:opacity-100`; :114 the error `mt-2 text-[0.6875rem]
        // leading-[1rem] font-medium text-[#FE5938]`.
        // Search :32, the search box: `h-10 px-12 border border-transparent
        // bg-[#FCFCFC] rounded-xl text-[0.75rem] leading-[1rem]
        // font-medium`; `hover:border-[#ECECEC]`; `focus:border-[#F1F1F1]
        // focus:bg-[#F1F1F1] focus:shadow-[0px_1px_3px_0px_rgba(18,18,
        // 18,0.10)_inset]`.
        let well = || vec![ShadowSpec::inset([0.0, 1.0], 3.0, 0.0, ink_a(0.10))];
        let search = InputRecipe {
            rest: InputSurface {
                fill: hex(0xFCFCFC),
                border: NONE,
                shadow: Vec::new(),
            },
            hover: InputSurface {
                fill: hex(0xFCFCFC),
                border: hex(0xECECEC),
                shadow: Vec::new(),
            },
            focus: InputSurface {
                fill: hex(0xF1F1F1),
                border: hex(0xF1F1F1),
                shadow: well(),
            },
            height: 40.0,
            pad_x: 48.0,
            border_width: 1.0,
            text: TypeRole::Label.run(),
            ..input.clone()
        };
        // Invite :22, the Field with `!h-10 !pl-4 !bg-[#F8F7F7]
        // shadow-[0_1px_3px_0_rgba(18,18,18,0.10)_inset]
        // placeholder:opacity-50`; the Field's hover and focus lift stays.
        let inset = InputRecipe {
            rest: InputSurface {
                fill: hex(0xF8F7F7),
                border,
                shadow: well(),
            },
            hover: InputSurface {
                fill: hex(0xF8F7F7),
                ..input.hover.clone()
            },
            focus: InputSurface {
                fill: hex(0xF8F7F7),
                ..input.focus.clone()
            },
            height: 40.0,
            pad_x: 16.0,
            // the kit's gray at 50% on its #F8F7F7 well, a hint as the
            // field's: as opaque as it must be to read (WCAG AA)
            placeholder: legibility::reach(
                Color::rgba(123.0 / 255.0, 123.0 / 255.0, 123.0 / 255.0, 0.50),
                TEXT_AA,
                &[&[hoff_core::SHADES[0], hex(0xF8F7F7)]],
            ),
            ..input.clone()
        };
        // Search :21 the icon button, :41 the `⌘ K` hint (the key cap,
        // `theme.disclosure.kbd`) at `right-2.5`.
        let group = GroupAddons {
            icon_edge: 4.0,
            icon_slot: 32.0,
            icon: 16.0,
            icon_color: gray,
            icon_hover: hex(0x000000),
            hint_edge: 10.0,
            // What Search :34 `px-12` (48) leaves past the hint's
            // `right-2.5` (10) and its `px-1.5` around `⌘ K` (about 34).
            addon_gap: 4.0,
            // Not in the kit (no text addon): the gray of its icons, as
            // far from the edge as its icon button's text is (48 - 36).
            text_color: quiet,
            text_edge: 12.0,
            select_w: 102.0,
            // `templates/InputsPage/index.tsx:176` the Search `w-54`, :161
            // the Invite `w-76.5`.
            width: 216.0,
            invite_width: 306.0,
            // Not in the kit, which has no palette: `w-72`, between its
            // Search and its Invite.
            palette_width: 288.0,
            // Not in the kit: three fifths of the window, a `60vh`.
            palette_share: 0.6,
        };
        let label = LabelRecipe {
            text: TypeRole::Label.run(),
            color: ink,
            caption: CAPTION,
            // Not in the kit: the core glass's placeholder tone, as the
            // caption draws in every other theme
            // (as opaque as it must be to read, WCAG AA).
            caption_color: hoff_core::placeholder_text(),
            disabled_alpha: input.disabled_alpha,
        };
        let field = FieldChrome {
            label_gap: 8.0,
            link: TypeRole::Label.run(),
            // The kit's gray at 70% reads 2.1:1, and in linear light no
            // tone at 70% reaches AA on paper: the ink, at the least share
            // that reads (`legibility::reach`), lit whole under the
            // pointer as the kit's.
            link_color: ink,
            link_alpha: legibility::reach(
                Color::rgba(ink.0[0], ink.0[1], ink.0[2], 0.70),
                TEXT_AA,
                &[&[hoff_core::SHADES[0]]],
            )
            .0[3],
            link_hover_alpha: 1.0,
            message: TypeRole::Caption.run(),
            message_gap: 8.0,
            error_color: hoff_core::red_text(),
            // Not in the kit (only an error line): the gray of its
            // secondary text.
            helper_color: quiet,
            // `templates/InputsPage/index.tsx:36`, `w-60`.
            width: 240.0,
        };

        // Not in the kit (it has no code input): the Field's own spaces,
        // the label's `mb-2` between the cells and its icon's `size-4`
        // for the minus.
        let otp = OtpTokens {
            gap: field.label_gap,
            separator: adorn.lead_icon,
        };
        // :55, the textarea: the input's classes with `h-20 py-4`.
        let area = AreaTokens {
            pad_y: 16.0,
            rows: 3,
        };
        // HeaderSimple :59, the Search with `!bg-[#F8F7F7]
        // !border-[#E2E2E2]`: both kept in every state, the focus well too.
        let header = search.pinned(hex(0xF8F7F7), border);
        let select = SelectRecipe::hoff_core();
        let combo = select.as_input(&input);
        let edit = EditTokens::hoff_core();
        let bare = edit.bare(&input);
        let mut form = Self {
            label,
            input,
            search,
            inset,
            header,
            adorn,
            group,
            field,
            otp,
            area,
            combo,
            select,
            edit,
            bare,
            color: ColorTokens::hoff_core(),
            upload: UploadTokens::hoff_core(),
        };
        // the fields' edges in their states (`hold_edges`)
        form.hold_edges([hoff_core::SHADES[0]; 2], None);
        form
    }
}
