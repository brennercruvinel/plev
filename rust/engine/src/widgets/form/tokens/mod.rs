//! Form tokens: what the hoff core kit writes as the tailwind class lists
//! of its Field (`components/Field/index.tsx`), as recipes the form
//! widgets read. The input box is one surface (fill, border, shadow list)
//! at rest, hovered and focused, with its geometry, its text and the
//! colors of the caret, the selection and a composition; the adornments
//! are the icon slots at its two ends; the field's chrome is the label
//! row above it and the message line under it.
//!
//! [`FormTokens::hoff_core`] is the kit as written. Every other theme
//! derives its recipe from the theme it is built into
//! ([`FormTokens::derive`]): the text field each of them drew before the
//! kit, so a theme that is not the core looks as it did.

mod color;
mod edit;
mod select;
mod upload;

pub use color::ColorTokens;
pub use edit::EditTokens;
pub use select::{PanelBody, SelectLook, SelectPanel, SelectRecipe, SelectSurfaces};
pub use upload::UploadTokens;

use crate::styling::color::Color;
use crate::styling::theme::{ControlSize, ShadowSpec, Theme, TypeRole, TypeRun};

/// One state of the input box.
#[derive(Clone, Debug, PartialEq)]
pub struct InputSurface {
    /// Alpha 0 is no fill (the kit's input has none: the card shows).
    pub fill: Color,
    /// The border, drawn inside the box (css `border`, box-sizing
    /// border-box).
    pub border: Color,
    /// The css `box-shadow` list, first entry on top.
    pub shadow: Vec<ShadowSpec>,
}

/// The single-line input box: its surface per state, its geometry, its
/// text and the marks drawn over the text.
#[derive(Clone, Debug, PartialEq)]
pub struct InputRecipe {
    pub rest: InputSurface,
    pub hover: InputSurface,
    /// Focused, hovered or not.
    pub focus: InputSurface,
    /// The box height (`h-12`); the width is always the `Rect`'s.
    pub height: f32,
    /// Inset from either end of the box to the text (`px-5.5`), where no
    /// adornment takes the end.
    pub pad_x: f32,
    pub radius: f32,
    pub border_width: f32,
    /// The one style the text is measured, hit-tested and drawn with.
    pub text: TypeRun,
    pub text_color: Color,
    pub placeholder: Color,
    pub caret: Color,
    /// The selection wash, alpha included.
    pub selection: Color,
    /// The underline of an input method's composition.
    pub composing: Color,
    /// Opacity of the whole field when disabled.
    pub disabled_alpha: f32,
}

impl InputRecipe {
    /// This recipe with `fill` and `border` in every state, its shadows
    /// kept (a css `!bg` and `!border` over the box's classes).
    pub fn pinned(&self, fill: Color, border: Color) -> Self {
        let pin = |s: &InputSurface| InputSurface {
            fill,
            border,
            shadow: s.shadow.clone(),
        };
        Self {
            rest: pin(&self.rest),
            hover: pin(&self.hover),
            focus: pin(&self.focus),
            ..self.clone()
        }
    }
}

/// The icon slots at the ends of the input box: a leading icon, the
/// check of a validated value and the eye that reveals a password.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Adornments {
    /// Width of the leading slot (`w-10`), which is also the text's inset
    /// on that side (`pl-10`).
    pub lead_slot: f32,
    pub lead_icon: f32,
    /// The text's inset on the trailing side when validated (`pr-10`).
    pub check_slot: f32,
    pub check_icon: f32,
    /// The check's inset from the trailing edge (`right-3.5`).
    pub check_inset: f32,
    /// Width of the reveal button (`w-12`), the text's trailing inset on
    /// a password (`pr-12`).
    pub reveal_slot: f32,
    pub reveal_icon: f32,
    pub icon_color: Color,
}

/// A label: the form label that names a control, and a section's
/// caption (the uppercase group heading of the HOFF accordion).
#[derive(Clone, Debug, PartialEq)]
pub struct LabelRecipe {
    pub text: TypeRun,
    pub color: Color,
    pub caption: TypeRun,
    pub caption_color: Color,
    /// Opacity of a disabled label (the control's).
    pub disabled_alpha: f32,
}

/// The field around the box: the label row and the message line.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldChrome {
    /// Between the label row and the box (`mb-2`).
    pub label_gap: f32,
    /// The link at the end of the label row ("Forgot password?").
    pub link: TypeRun,
    pub link_color: Color,
    /// The link's opacity at rest and hovered (`opacity-70`, `hover:opacity-100`).
    pub link_alpha: f32,
    pub link_hover_alpha: f32,
    /// The message line under the box: an error, or a helper.
    pub message: TypeRun,
    /// Between the box and the message (`mt-2`).
    pub message_gap: f32,
    pub error_color: Color,
    pub helper_color: Color,
    /// A field's width where the composition gives it one; the `Rect` it
    /// gets always wins.
    pub width: f32,
}

/// What takes one end of the input box, which sets the text's inset on
/// that end.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Slot {
    /// Nothing: the recipe's padding.
    #[default]
    Pad,
    /// A leading icon.
    Icon,
    /// The check of a validated value.
    Check,
    /// The button that reveals a password.
    Reveal,
    /// An addon of an input group, as wide as it measures.
    Width(f32),
}

/// Which input box a text field draws: the Field's, the search box of an
/// input group, or the inset box an input group with a select draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Look {
    #[default]
    Field,
    Search,
    Inset,
    /// A combobox's filter: the select's control as a text box.
    Combo,
    /// The search of a top bar: the search box on the panel's tone,
    /// edged, whatever the pointer does.
    Header,
    /// An inline edit's box: the text alone ([`EditTokens::bare`]).
    Bare,
}

/// What an input group puts at the ends of its box
/// (`components/Search/index.tsx`): an icon button, a shortcut hint and
/// a text addon (a prefix or a suffix).
#[derive(Clone, Debug, PartialEq)]
pub struct GroupAddons {
    /// The icon button's inset from the box on its three sides
    /// (`top-1 left-1 bottom-1`), its width (`w-8`) and icon (`size-4`).
    pub icon_edge: f32,
    pub icon_slot: f32,
    pub icon: f32,
    /// The icon at rest and with the group hovered (`fill-[#7B7B7B]
    /// group-hover:fill-[#000]`).
    pub icon_color: Color,
    pub icon_hover: Color,
    /// The shortcut hint's inset from the box's end; the hint itself is
    /// the search's key cap, `theme.disclosure.kbd` (`Kbd`, `Md`).
    pub hint_edge: f32,
    /// The least room between an icon or a hint and the text typed
    /// beside it, so a right-to-left text that hangs from the end never
    /// touches the `⌘ K`.
    pub addon_gap: f32,
    /// A text addon's color and its clearance from the box's edge and
    /// from the text typed beside it.
    pub text_color: Color,
    pub text_edge: f32,
    /// The least width of a select at the end (Invite :26 `w-25.5`).
    pub select_w: f32,
    /// A group's width where the composition gives it one: a search, an
    /// invite (a select at its end) and a command palette (the search
    /// over its list). The `Rect` it gets always wins.
    pub width: f32,
    pub invite_width: f32,
    pub palette_width: f32,
    /// The most of the window's height a command palette's dialog takes
    /// (the share of a `cmdk` dialog's `60vh`): its list scrolls past it.
    pub palette_share: f32,
}

/// A one-time code (`super::otp`): its cells are the Field's input box,
/// square, side by side; these are the spaces between them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OtpTokens {
    /// Between two cells of a group.
    pub gap: f32,
    /// The minus between two groups, with a gap on either side of it.
    pub separator: f32,
}

/// A text area (`super::textarea`): the Field's box, taller, its text
/// inset from the top and the bottom as well.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AreaTokens {
    /// Between the text and the box's top and bottom (`py-4`).
    pub pad_y: f32,
    /// The lines the box shows at rest (`h-20`: 80 less the padding is
    /// three 16px lines).
    pub rows: usize,
}

/// The form slot of the theme.
#[derive(Clone, Debug, PartialEq)]
pub struct FormTokens {
    pub label: LabelRecipe,
    pub input: InputRecipe,
    /// The search box of an input group.
    pub search: InputRecipe,
    /// The inset box of an input group with a select at its end.
    pub inset: InputRecipe,
    /// The search box of a top bar ([`Look::Header`]).
    pub header: InputRecipe,
    pub adorn: Adornments,
    pub group: GroupAddons,
    pub field: FieldChrome,
    pub otp: OtpTokens,
    pub area: AreaTokens,
    /// The select and its options panel (`select.rs`).
    pub select: SelectRecipe,
    /// A combobox's filter box, the select's control as a text box
    /// ([`SelectRecipe::as_input`]).
    pub combo: InputRecipe,
    /// The inline edit (`edit.rs`), and its bare box.
    pub edit: EditTokens,
    pub bare: InputRecipe,
    /// The color picker (`color.rs`).
    pub color: ColorTokens,
    /// The avatar upload (`upload.rs`).
    pub upload: UploadTokens,
}

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

/// A section's caption: the scale's strong body (12/16 semibold in the
/// core), in the theme's own face and tracking.
const CAPTION: TypeRun = TypeRole::BodyStrong.run();

impl FormTokens {
    /// The text's inset from an end of the input box that `slot` takes.
    pub fn inset(&self, slot: Slot) -> f32 {
        match slot {
            Slot::Pad => self.input.pad_x,
            Slot::Icon => self.adorn.lead_slot,
            Slot::Check => self.adorn.check_slot,
            Slot::Reveal => self.adorn.reveal_slot,
            Slot::Width(w) => w,
        }
    }

    /// The input box of `look`.
    pub fn recipe(&self, look: Look) -> &InputRecipe {
        match look {
            Look::Field => &self.input,
            Look::Search => &self.search,
            Look::Inset => &self.inset,
            Look::Header => &self.header,
            Look::Combo => &self.combo,
            Look::Bare => &self.bare,
        }
    }

    /// The hoff core kit, values as written in its class lists
    /// (`components/Field/index.tsx`; the body's `text-[#121212]
    /// tracking-[-0.02em]`, `app/layout.tsx:108`).
    pub fn hoff_core() -> Self {
        let gray = hex(0x7B7B7B);

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
            placeholder: gray,
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
            placeholder: Color::rgba(123.0 / 255.0, 123.0 / 255.0, 123.0 / 255.0, 0.50),
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
            text_color: gray,
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
            // caption draws in every other theme.
            caption_color: Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.40),
            disabled_alpha: input.disabled_alpha,
        };
        let field = FieldChrome {
            label_gap: 8.0,
            link: TypeRole::Label.run(),
            link_color: gray,
            link_alpha: 0.70,
            link_hover_alpha: 1.0,
            message: TypeRole::Caption.run(),
            message_gap: 8.0,
            error_color: hex(0xFE5938),
            // Not in the kit (only an error line): the gray of its
            // secondary text.
            helper_color: gray,
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
        Self {
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
        }
    }

    /// The recipe for a theme that is not the core kit, read from
    /// `theme` once it is built: the glass text field every theme drew
    /// before the kit (`glass.field` at rest, `surface_active` hovered or
    /// focused, `edge_soft` turning `field_focus_border` on focus, one
    /// `Md` control tall at the nav radius, body text), so its look does
    /// not change. The adornments and the chrome take the theme's own
    /// sizes and colors.
    pub fn derive(theme: &Theme) -> Self {
        let g = &theme.glass;
        let c = &theme.colors;
        let t = &theme.typography;
        let surface = |fill: Color, border: Color| InputSurface {
            fill,
            border,
            shadow: Vec::new(),
        };
        let accent = c.accent.0;
        let input = InputRecipe {
            rest: surface(g.field, g.edge_soft),
            hover: surface(g.surface_active, g.edge_soft),
            focus: surface(g.surface_active, g.field_focus_border),
            height: theme.control.height(ControlSize::Md),
            pad_x: theme.spacing.md,
            radius: theme.shape.nav,
            border_width: theme.control.edge_width,
            text: TypeRole::BodyLg.run(),
            text_color: c.text,
            placeholder: g.text_placeholder,
            caret: c.accent,
            selection: Color::rgba(accent[0], accent[1], accent[2], g.wash_alpha),
            composing: g.text_active,
            disabled_alpha: g.disabled_alpha,
        };
        let icon = theme.control.icon_for(ControlSize::Md);
        let lead_slot = theme.control.height(ControlSize::Md);
        let adorn = Adornments {
            lead_slot,
            lead_icon: theme.control.icon_for(ControlSize::Sm),
            check_slot: lead_slot,
            check_icon: icon,
            check_inset: (lead_slot - icon) / 2.0,
            reveal_slot: lead_slot,
            reveal_icon: icon,
            icon_color: c.text_mid,
        };
        let group = GroupAddons {
            icon_edge: theme.spacing.xs,
            icon_slot: theme.control.icon_for(ControlSize::Sm) * 2.0,
            icon: theme.control.icon_for(ControlSize::Sm),
            icon_color: c.text_mid,
            icon_hover: c.text,
            hint_edge: theme.spacing.sm,
            addon_gap: theme.spacing.xs,
            text_color: c.text_mid,
            text_edge: theme.spacing.md,
            // As wide as its options measure.
            select_w: 0.0,
            // A menu's width; an invite takes half a readable line, a
            // palette two fifths of it.
            width: theme.size.menu_w,
            invite_width: theme.size.readable_max_w / 2.0,
            palette_width: theme.size.readable_max_w * 0.4,
            palette_share: 0.6,
        };
        let label = LabelRecipe {
            text: TypeRole::Label.run(),
            color: c.text,
            caption: CAPTION,
            caption_color: g.text_placeholder,
            disabled_alpha: g.disabled_alpha,
        };
        let field = FieldChrome {
            label_gap: theme.spacing.sm,
            link: TypeRole::Label.run(),
            link_color: c.text_mid,
            link_alpha: 1.0,
            link_hover_alpha: 1.0,
            message: TypeRole::Caption.run(),
            message_gap: theme.spacing.sm,
            error_color: c.danger,
            helper_color: c.text_mid,
            width: theme.size.menu_w,
        };
        let otp = OtpTokens {
            gap: field.label_gap,
            separator: adorn.lead_icon,
        };
        // The text centered in one box's height, as the field draws it,
        // and three lines at rest, as the kit's.
        let area = AreaTokens {
            pad_y: ((input.height - t.run(input.text).line_height) / 2.0).max(0.0),
            rows: 3,
        };
        let select = SelectRecipe::derive(theme);
        let combo = select.as_input(&input);
        let edit = EditTokens::derive(theme);
        let bare = edit.bare(&input);
        Self {
            label,
            // the search and the header's field write the kit's label
            search: InputRecipe {
                text: TypeRole::Label.run(),
                ..input.clone()
            },
            inset: input.clone(),
            // the panel's tone, edged in the active border, as the kit's
            header: InputRecipe {
                text: TypeRole::Label.run(),
                ..input.pinned(c.bg_panel, c.border_active)
            },
            input,
            adorn,
            group,
            field,
            otp,
            area,
            combo,
            select,
            edit,
            bare,
            color: ColorTokens::derive(theme),
            upload: UploadTokens::derive(theme),
        }
    }
}
