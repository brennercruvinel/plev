//! The select's recipe: the closed control in its looks and its options
//! panel. The core is the kit's Select (`components/Select/index.tsx`, a
//! headless Listbox): a 36px control (`h-9 pl-3 pr-2 gap-1.5`,
//! `rounded-[0.625rem]`) filled `#F1F1F1` under a hairline border, its
//! ghost (`isWhite`), minimal (`isMinimal`) and small (`isSmall`, `h-8`,
//! 12px) looks, the inline one an Invite sets in a field
//! (`components/Invite/index.tsx`), and a flat panel of rows (`p-1`,
//! rows `p-2 rounded-lg`), the selected one on `#F1F1F1`. Every other
//! theme derives the glass select it drew before the kit: the glass pill,
//! the glass popover with its edge light and floating shadow, and a dot
//! on the selected row.

use crate::styling::color::Color;
use crate::styling::theme::{ControlSize, IconSize, ShadowSpec, Theme, TypeRole, TypeRun};

use super::{InputRecipe, InputSurface};

/// How the options panel draws its body.
#[derive(Clone, Debug, PartialEq)]
pub enum PanelBody {
    /// A flat surface: fill, border inside, shadow list (the kit's).
    Flat(InputSurface),
    /// The theme's glass popover: its edge light and the floating shadow.
    Glass,
}

/// The open options panel.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectPanel {
    pub body: PanelBody,
    pub border_width: f32,
    pub radius: f32,
    /// Between the control and the panel (`--anchor-gap`), and below an
    /// inline select (`![--anchor-gap:0.5rem]`).
    pub gap: f32,
    pub inline_gap: f32,
    /// The rows' inset from the panel's edges (`p-1`).
    pub pad: f32,
    /// The panel never grows past this; its rows scroll.
    pub max_h: f32,
    pub row_h: f32,
    pub row_radius: f32,
    /// The text's inset from the panel's edge.
    pub text_inset: f32,
    pub text: TypeRun,
    /// An option at rest, and highlighted or selected.
    pub text_color: Color,
    pub active_color: Color,
    pub selected_fill: Color,
    /// The highlighted row's fill (alpha 0: only its text changes).
    pub hover_fill: Color,
    /// The dot that marks the selected row, its size; `None`: no dot.
    pub marker: Option<f32>,
    pub marker_color: Color,
}

/// Which control a select draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectLook {
    /// Filled, under a hairline border.
    #[default]
    Filled,
    /// Clear, the border only (`isWhite`).
    Ghost,
    /// Nothing until hovered (`isMinimal`).
    Minimal,
    /// Small and raised inside a field's end (the Invite's).
    Inline,
}

/// A look's surface at rest, hovered, open and with the keyboard focus.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectSurfaces {
    pub rest: InputSurface,
    pub hover: InputSurface,
    pub open: InputSurface,
    pub focus: InputSurface,
}

/// The closed control and its panel.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectRecipe {
    pub filled: SelectSurfaces,
    pub ghost: SelectSurfaces,
    pub minimal: SelectSurfaces,
    pub inline: SelectSurfaces,
    pub height: f32,
    /// A select's width where the composition gives it one (the kit's
    /// `w-44`); a small or minimal one is as wide as it measures, and the
    /// `Rect` it gets always wins.
    pub width: f32,
    /// The small control's height and text (`h-8 text-[0.75rem]`), the
    /// inline one's too.
    pub small_height: f32,
    pub small_text: TypeRun,
    /// The text's inset from the start, and the chevron's from the end.
    pub pad_start: f32,
    pub pad_end: f32,
    /// Between the parts of the control: an icon, a label, the value and
    /// the chevron.
    pub gap: f32,
    pub radius: f32,
    pub inline_radius: f32,
    pub border_width: f32,
    pub text: TypeRun,
    pub value_color: Color,
    /// The value after a label (`text-[#7B7B7B]/50`), the label's own.
    pub labelled_color: Color,
    pub label_color: Color,
    pub placeholder: Color,
    /// The icon at the start (`[&_svg]:size-4 [&_svg]:fill-[#7B7B7B]/70`).
    pub icon: f32,
    pub icon_color: Color,
    pub chevron: f32,
    pub chevron_color: Color,
    pub disabled_alpha: f32,
    pub panel: SelectPanel,
}

fn surface(fill: Color, border: Color) -> InputSurface {
    InputSurface {
        fill,
        border,
        shadow: Vec::new(),
    }
}

/// Every state the same.
fn still(s: InputSurface) -> SelectSurfaces {
    SelectSurfaces {
        rest: s.clone(),
        hover: s.clone(),
        open: s.clone(),
        focus: s,
    }
}

fn gray(a: f32) -> Color {
    Color::rgba(123.0 / 255.0, 123.0 / 255.0, 123.0 / 255.0, a)
}

const CLEAR: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

impl SelectRecipe {
    /// The control's surfaces in `look`.
    pub fn surfaces(&self, look: SelectLook) -> &SelectSurfaces {
        match look {
            SelectLook::Filled => &self.filled,
            SelectLook::Ghost => &self.ghost,
            SelectLook::Minimal => &self.minimal,
            SelectLook::Inline => &self.inline,
        }
    }

    /// The kit as written (`components/Select/index.tsx:54`, the button;
    /// `:90`, the options; `:99`, an option).
    pub fn hoff_core() -> Self {
        let hex = Color::hex;
        let (edge, lit) = (hex(0xECECEC), hex(0xE2E2E2));
        let text = TypeRole::BodyLg.medium();
        let small_text = TypeRole::Label.run();
        // :54 `border border-[#ececec] bg-[#F1F1F1]`; `data-[hover]:
        // border-[#E2E2E2] data-[hover]:bg-[#F8F7F7]`; `data-[open]:
        // border-[#e2e2e2] data-[open]:bg-transparent`; `focus:bg-[#FCFCFC]`.
        let filled = SelectSurfaces {
            rest: surface(hex(0xF1F1F1), edge),
            hover: surface(hex(0xF8F7F7), lit),
            open: surface(CLEAR, lit),
            focus: surface(hex(0xFCFCFC), edge),
        };
        // `isWhite`: `bg-transparent` and the hover fill gone; its
        // `!border-s-02` names no color of the kit, so the border stays.
        let ghost = SelectSurfaces {
            rest: surface(CLEAR, edge),
            hover: surface(CLEAR, lit),
            ..filled.clone()
        };
        // `isMinimal`: `!border-0 !bg-transparent data-[hover]:!bg-[#F1F1F1]`.
        let minimal = SelectSurfaces {
            hover: surface(hex(0xF1F1F1), CLEAR),
            ..still(surface(CLEAR, CLEAR))
        };
        // Invite :28 `!bg-[#FCFCFC] !border-none !rounded-lg
        // shadow-[0_0_4px_0_rgba(18,18,18,0.10)]`.
        let inline = still(InputSurface {
            fill: hex(0xFCFCFC),
            border: CLEAR,
            shadow: vec![ShadowSpec::new(
                [0.0, 0.0],
                4.0,
                0.0,
                Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.10),
            )],
        });
        let panel = SelectPanel {
            // :90 `p-1 bg-[#fcfcfc] border border-[#E2E2E2]
            // shadow-[0px_1px_4px_-4px_rgba(0,0,0,0.075),0px_8px_16px_
            // -12px_rgba(0,0,0,0.125)] rounded-[0.625rem]`, `--anchor-gap:2px`.
            body: PanelBody::Flat(InputSurface {
                fill: hex(0xFCFCFC),
                border: lit,
                shadow: vec![
                    ShadowSpec::new([0.0, 1.0], 4.0, -4.0, Color::rgba(0.0, 0.0, 0.0, 0.075)),
                    ShadowSpec::new([0.0, 8.0], 16.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.125)),
                ],
            }),
            border_width: 1.0,
            radius: 10.0,
            gap: 2.0,
            inline_gap: 8.0,
            pad: 4.0,
            // Not in the kit (its list never scrolls): six rows.
            max_h: 4.0 * 2.0 + 32.0 * 6.0,
            // :99 `p-2 rounded-lg font-medium leading-[1rem]
            // text-[#7B7B7B] data-[focus]:text-[#000]
            // data-[selected]:bg-[#f1f1f1] data-[selected]:text-[#000]`.
            row_h: 32.0,
            row_radius: 8.0,
            text_inset: 4.0 + 8.0,
            text,
            text_color: hex(0x7B7B7B),
            active_color: hex(0x000000),
            selected_fill: hex(0xF1F1F1),
            hover_fill: CLEAR,
            marker: None,
            marker_color: hex(0x000000),
        };
        Self {
            filled,
            ghost,
            minimal,
            inline,
            height: 36.0,
            // `templates/DropdownPage/index.tsx:30`, `w-44`.
            width: 176.0,
            small_height: 32.0,
            small_text,
            pad_start: 12.0,
            pad_end: 8.0,
            gap: 6.0,
            radius: 10.0,
            inline_radius: 8.0,
            border_width: 1.0,
            text,
            // The body's ink (`text-[#121212]`); after a label `text-[#7B7B7B]/50`,
            // the label `text-[#000]`; the placeholder `text-[#7B7B7B]/80`.
            value_color: hex(0x121212),
            labelled_color: gray(0.50),
            label_color: hex(0x000000),
            placeholder: gray(0.80),
            icon: 16.0,
            icon_color: gray(0.70),
            // :79 the chevron, `size-4 fill-[#7B7B7B]`.
            chevron: 16.0,
            chevron_color: hex(0x7B7B7B),
            // Not in the kit (no disabled select): its disabled button's.
            disabled_alpha: 0.30,
            panel,
        }
    }

    /// The glass select every theme but the core drew: the field pill at
    /// rest, the active surface hovered or open, the focus border open;
    /// one `Md` control tall at the tabs radius, the label at `lg` from
    /// either end; the glass popover at the card radius with the menu
    /// padding, rows one `Md` control tall at the nav radius, the
    /// selected one marked by a dot. The looks keep the glass: the ghost
    /// without its fill, the minimal without its fill and edge until
    /// hovered, the inline one `Sm` tall without its edge.
    pub fn derive(theme: &Theme) -> Self {
        let g = &theme.glass;
        let c = &theme.colors;
        let edge = theme.control.edge_width_strong;
        let option_h = theme.control.height(ControlSize::Md);
        let text = TypeRole::Body.medium();
        let pad_x = theme.spacing.lg;
        let filled = SelectSurfaces {
            rest: surface(g.field, g.edge_soft),
            hover: surface(g.surface_active, g.edge_soft),
            open: surface(g.surface_active, g.field_focus_border),
            focus: surface(g.field, g.edge_soft),
        };
        let ghost = SelectSurfaces {
            rest: surface(CLEAR, g.edge_soft),
            ..filled.clone()
        };
        let minimal = SelectSurfaces {
            rest: surface(CLEAR, CLEAR),
            focus: surface(CLEAR, CLEAR),
            hover: surface(g.surface_active, CLEAR),
            open: surface(g.surface_active, CLEAR),
        };
        let inline = still(surface(g.field, CLEAR));
        Self {
            filled,
            ghost,
            minimal,
            inline,
            height: option_h,
            width: theme.size.menu_w,
            small_height: theme.control.height(ControlSize::Sm),
            small_text: TypeRole::Caption.regular(),
            pad_start: pad_x,
            pad_end: pad_x,
            gap: theme.spacing.xs,
            radius: theme.shape.tabs,
            inline_radius: theme.shape.nav,
            border_width: edge,
            text,
            value_color: g.text_active,
            labelled_color: g.text_placeholder,
            label_color: g.text_active,
            placeholder: g.text_placeholder,
            icon: theme.control.icon(IconSize::Sm),
            icon_color: g.text_faint,
            chevron: theme.control.icon(IconSize::Sm),
            chevron_color: g.text_faint,
            disabled_alpha: g.disabled_alpha,
            panel: SelectPanel {
                body: PanelBody::Glass,
                border_width: edge,
                radius: theme.shape.card,
                gap: theme.spacing.xs,
                inline_gap: theme.spacing.sm,
                pad: theme.control.menu_pad,
                max_h: theme.size.dropdown_max_h,
                row_h: option_h,
                row_radius: theme.shape.nav,
                text_inset: pad_x,
                text,
                text_color: g.text_default,
                active_color: g.text_active,
                selected_fill: g.surface_active,
                hover_fill: g.surface_hover,
                marker: Some(theme.spacing.sm),
                marker_color: c.text,
            },
        }
    }

    /// The control as a text box (a combobox's filter): its filled
    /// surfaces, the open one focused, its geometry and its text; the
    /// caret, the selection and a composition are `field`'s.
    pub fn as_input(&self, field: &InputRecipe) -> InputRecipe {
        let f = &self.filled;
        InputRecipe {
            rest: f.rest.clone(),
            hover: f.hover.clone(),
            focus: f.open.clone(),
            height: self.height,
            pad_x: self.pad_start,
            radius: self.radius,
            border_width: self.border_width,
            text: self.text,
            text_color: self.value_color,
            placeholder: self.placeholder,
            disabled_alpha: self.disabled_alpha,
            ..field.clone()
        }
    }
}
