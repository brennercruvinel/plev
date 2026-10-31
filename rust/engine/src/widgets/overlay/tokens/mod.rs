//! Overlay tokens (task-104, front B): what the hoff core kit writes as
//! tailwind classes on its anchored overlays, as recipes the widgets
//! read. One field per overlay; the popover is the first. The two edge
//! panels' (the sheet and the drawer) live beside their widget,
//! `sheet/tokens.rs`; the menus' in `menu.rs`.
//!
//! `OverlayTokens::hoff_core` is the kit as written (its Toolbar
//! `ZoomControl` and `ExportFiles` popovers and the header's
//! `NotificationsDropdown`). Every other theme derives its recipes from
//! the theme already built through [`OverlayTokens::derive`], so the
//! overlays render under all of them and stay coherent with the theme's
//! other floating surfaces (the context menu's `glass.popover`).

use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{ButtonSurface, Theme, TypographyScale};

pub use super::sheet::{DrawerTokens, EdgePanelTokens, SheetTokens};

mod menu;
pub use menu::{MenuStyle, MenuTokens, MenubarTokens};

/// The tokens of front B's overlays.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayTokens {
    pub popover: PopoverTokens,
    pub tooltip: TooltipTokens,
    pub hover_card: HoverCardTokens,
    /// The kit's three menus: the profile menu, the default, then the
    /// file and the filter menus ([`OverlayTokens::menu_for`]).
    pub menu: MenuTokens,
    pub file_menu: MenuTokens,
    pub filter_menu: MenuTokens,
    pub menubar: MenubarTokens,
    pub modal: ModalTokens,
    pub sheet: SheetTokens,
    pub drawer: DrawerTokens,
}

impl OverlayTokens {
    /// The hoff core kit, values as written in the tailwind classes.
    pub fn hoff_core() -> Self {
        Self {
            popover: PopoverTokens::hoff_core(),
            tooltip: TooltipTokens::hoff_core(),
            hover_card: HoverCardTokens::hoff_core(),
            menu: MenuTokens::hoff_core(),
            file_menu: MenuTokens::hoff_core_file(),
            filter_menu: MenuTokens::hoff_core_filter(),
            menubar: MenubarTokens::hoff_core(),
            modal: ModalTokens::hoff_core(),
            sheet: SheetTokens::hoff_core(),
            drawer: DrawerTokens::hoff_core(),
        }
    }

    /// Recipes from a theme that is not the core kit, read once the rest
    /// of the theme is built.
    pub fn derive(theme: &Theme) -> Self {
        Self {
            popover: PopoverTokens::derive(theme),
            tooltip: TooltipTokens::derive(theme),
            hover_card: HoverCardTokens::hoff_core(),
            menu: MenuTokens::derive(theme),
            file_menu: MenuTokens::derive(theme),
            filter_menu: MenuTokens::derive(theme),
            menubar: MenubarTokens::derive(theme),
            modal: ModalTokens::derive(theme),
            sheet: SheetTokens::derive(theme),
            drawer: DrawerTokens::derive(theme),
        }
    }

    /// The recipe of a menu drawn as `style`.
    pub fn menu_for(&self, style: MenuStyle) -> &MenuTokens {
        match style {
            MenuStyle::Profile => &self.menu,
            MenuStyle::File => &self.file_menu,
            MenuStyle::Filter => &self.filter_menu,
        }
    }
}

/// Distances an anchored overlay keeps: from its anchor along the main
/// axis, along the cross axis, and from the edge of the container that
/// receives it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clearance {
    /// Between the anchor and the panel (`--anchor-gap`).
    pub gap: f32,
    /// Skid along the cross axis (`--anchor-offset`, floating-ui's
    /// `crossAxis`: positive is toward the end).
    pub offset: f32,
    /// Smallest distance from the container's edge (`--anchor-padding`).
    pub edge: f32,
}

/// The kit's popover trigger (`ZoomControl`'s button): a value and a
/// chevron in a bordered pill, raised while open.
#[derive(Clone, Debug, PartialEq)]
pub struct TriggerTokens {
    pub rest: ButtonSurface,
    pub hover: ButtonSurface,
    /// `data-open`: while its panel is open.
    pub open: ButtonSurface,
    pub height: f32,
    pub min_w: f32,
    pub pad_x: f32,
    /// Between the value and the chevron.
    pub gap: f32,
    pub radius: f32,
    /// Value size and line height in px, and weight.
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    /// The chevron's edge and color; it turns half a turn while open.
    pub chevron: f32,
    pub chevron_color: Color,
}

/// The popover: the panel's surface and geometry, how it keeps clear of
/// its anchor and its container, the kit's width ladder, the trigger.
#[derive(Clone, Debug, PartialEq)]
pub struct PopoverTokens {
    /// The panel: fill, the css shadow list (the border is its ring).
    pub panel: ButtonSurface,
    pub radius: f32,
    /// The smaller radius of the kit's menu-sized panel (`rounded-2xl`),
    /// kept for the Dropdown Menu.
    pub radius_menu: f32,
    pub clearance: Clearance,
    /// Preferred panel widths, small then large (`w-51`, `w-96`); the
    /// container caps them.
    pub widths: [f32; 2],
    /// Space between the panel's edge and its content (`p-2`).
    pub pad: f32,
    /// A line of the panel ([`PopoverLineTokens`]).
    pub line: PopoverLineTokens,
    pub trigger: TriggerTokens,
}

/// A line of the popover's panel (the kit's zoom `Line`): a check slot, a
/// label at the start, its keys in a cap at the end.
#[derive(Clone, Debug, PartialEq)]
pub struct PopoverLineTokens {
    pub height: f32,
    pub pad_x: f32,
    pub radius: f32,
    /// The check's slot and the space after it; the check inside it.
    pub slot: f32,
    pub slot_gap: f32,
    pub check: f32,
    pub check_color: Color,
    pub label: TextStyle,
    pub text: Color,
    /// Under the pointer or the keys.
    pub hover_fill: Color,
    /// The narrowest cap of its keys.
    pub key_min_w: f32,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

fn flat(color: Color) -> [Color; 2] {
    [color, color]
}

/// A 1px ring around the surface (`0 0 0 1px color`, a css border as
/// the button recipes write it).
fn ring(color: Color) -> ShadowSpec {
    ShadowSpec::new([0.0, 0.0], 0.0, 1.0, color)
}

const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

impl PopoverTokens {
    /// The hoff core kit, values as written in the tailwind classes.
    pub fn hoff_core() -> Self {
        let ink = hex(0x121212);
        let paper = hex(0xFCFCFC);
        let shade3 = hex(0xF1F1F1);
        let shade4 = hex(0xECECEC);
        let shade5 = hex(0xE2E2E2);

        // The panel (ZoomControl:19, ExportFiles:15, Notifications:35):
        // bg FCFCFC, border ECECEC, shadow-2xl (0 25px 50px -12px
        // black/25, tailwind v4 theme.css).
        let panel = ButtonSurface {
            fill: flat(paper),
            overlay: NONE,
            shadow: vec![
                ShadowSpec::new([0.0, 25.0], 50.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.25)),
                ring(shade4),
            ],
            text: ink,
        };

        // The trigger (ZoomControl:7): bg F1F1F1 border ECECEC; hover
        // bg FCFCFC border E2E2E2; open border 121212/10 and an inset
        // 0 0 2px 2px white. The text is the body's 121212.
        let rest = ButtonSurface {
            fill: flat(shade3),
            overlay: NONE,
            shadow: vec![ring(shade4)],
            text: ink,
        };
        let hover = ButtonSurface {
            fill: flat(paper),
            shadow: vec![ring(shade5)],
            ..rest.clone()
        };
        let open = ButtonSurface {
            shadow: vec![
                ShadowSpec::inset([0.0, 0.0], 2.0, 2.0, hex(0xFFFFFF)),
                ring(Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.10)),
            ],
            ..rest.clone()
        };

        Self {
            panel,
            // rounded-[1.25rem] (ExportFiles:15, Notifications:35).
            radius: 20.0,
            // rounded-2xl (ZoomControl:19).
            radius_menu: 16.0,
            clearance: Clearance {
                // [--anchor-gap:0.75rem] [--anchor-offset:0.5rem]; no
                // --anchor-padding, headlessui's default 0.
                gap: 12.0,
                offset: 8.0,
                edge: 0.0,
            },
            // w-51 (ZoomControl:19), w-96 (Notifications:35).
            widths: [204.0, 384.0],
            // p-2 (Zoom:54).
            pad: 8.0,
            // The zoom line (`Zoom/index.tsx:14-35`): `h-9 px-2
            // rounded-[0.625rem] hover:bg-[#F1F1F1]`, the check's slot
            // `size-6 mr-1` with `size-4 fill-[#7B7B7B]` in it, the title
            // `text-[0.75rem] leading-[1rem] font-medium text-[#000]`, the
            // cap `min-w-8.5`.
            line: PopoverLineTokens {
                height: 36.0,
                pad_x: 8.0,
                radius: 10.0,
                slot: 24.0,
                slot_gap: 4.0,
                check: 16.0,
                check_color: hex(0x7B7B7B),
                label: TypographyScale::hoff_core().exact(12.0, 16.0, 500),
                text: hex(0x000000),
                hover_fill: shade3,
                key_min_w: 34.0,
            },
            trigger: TriggerTokens {
                rest,
                hover,
                open,
                // h-10 min-w-22 px-3 gap-2 rounded-xl.
                height: 40.0,
                min_w: 88.0,
                pad_x: 12.0,
                gap: 8.0,
                radius: 12.0,
                // text-[0.875rem] font-medium; the line height is the
                // preflight's 1.5.
                font_size: 14.0,
                line_height: 21.0,
                font_weight: 500,
                // size-4 fill-[#7b7b7b].
                chevron: 16.0,
                chevron_color: hex(0x7B7B7B),
            },
        }
    }

    /// The kit's geometry over the theme's own floating surface: the
    /// context menu's popover glass with its soft edge and the floating
    /// shadow, the button glass for the trigger, the theme's dropdown
    /// radius.
    pub fn derive(theme: &Theme) -> Self {
        let kit = Self::hoff_core();
        let glass = &theme.glass;
        let colors = &theme.colors;
        let panel = ButtonSurface {
            fill: flat(glass.popover),
            overlay: NONE,
            shadow: [theme.shadows.floating.as_slice(), &[ring(glass.edge_soft)]].concat(),
            text: colors.text,
        };
        let rest = ButtonSurface {
            fill: flat(glass.button),
            overlay: NONE,
            shadow: vec![ring(colors.divider)],
            text: colors.text,
        };
        let hover = ButtonSurface {
            fill: flat(glass.button_hover),
            shadow: vec![ring(colors.border_active)],
            ..rest.clone()
        };
        let open = ButtonSurface {
            shadow: vec![ring(colors.border_active)],
            ..rest.clone()
        };
        // The line as the theme's menu rows: their height, padding, radius,
        // label, colors and icon; the slot the icon and its gap.
        let m = MenuTokens::derive(theme);
        let line = PopoverLineTokens {
            height: m.item_h,
            pad_x: m.item_pad_x,
            radius: m.item_radius,
            slot: m.icon,
            slot_gap: m.gap,
            check: m.icon,
            check_color: m.icon_color,
            label: m.label,
            text: m.text,
            hover_fill: m.hover_fill,
            ..kit.line
        };
        Self {
            panel,
            line,
            radius: theme.shape.dropdown,
            radius_menu: theme.shape.dropdown,
            trigger: TriggerTokens {
                rest,
                hover,
                open,
                chevron_color: colors.text_mid,
                ..kit.trigger
            },
            ..kit
        }
    }
}

/// The tooltip: a dark bubble of one short line, above its anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct TooltipTokens {
    pub fill: Color,
    /// Its shadow stack, bottom layer first.
    pub shadow: Vec<ShadowSpec>,
    pub radius: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub text: TextStyle,
    pub text_color: Color,
    /// From its anchor (main axis), and from the container's edge.
    pub clearance: Clearance,
    /// The widest it gets before its text wraps.
    pub max_w: f32,
    /// A tooltip that names its shortcut: the key beside the text.
    pub key: TooltipKeyTokens,
}

/// The key a tooltip names beside its text (the toolbar's `Action`): a
/// small square with the key in the tooltip's text, and the tooltip's
/// own padding around both.
#[derive(Clone, Debug, PartialEq)]
pub struct TooltipKeyTokens {
    /// The square's edge and corner radius.
    pub size: f32,
    pub radius: f32,
    pub fill: Color,
    /// Its border; alpha 0 is none.
    pub border: Color,
    pub border_w: f32,
    /// The tooltip's padding with a key: top, end, bottom, start.
    pub pad: [f32; 4],
    /// Between the text and the key.
    pub gap: f32,
}

impl TooltipTokens {
    /// The kit's `components/Tooltip` (px-1.5 py-1 rounded-md bg-[#121212]
    /// shadow-[0px_4px_4px_-2px_rgba(0,0,0,0.40)] text-[0.6875rem]
    /// leading-[1rem] font-medium text-[#E2E2E2]), `mb-4.5` from its
    /// anchor as the toolbar's `Action` places it. The kit keeps no edge
    /// (it never meets one); the core's `xs` step does.
    pub fn hoff_core() -> Self {
        Self {
            fill: hex(0x121212),
            shadow: vec![ShadowSpec::new(
                [0.0, 4.0],
                4.0,
                -2.0,
                Color::rgba(0.0, 0.0, 0.0, 0.40),
            )],
            radius: 6.0,
            pad_x: 6.0,
            pad_y: 4.0,
            text: TypographyScale::hoff_core().exact(11.0, 16.0, 500),
            text_color: hex(0xE2E2E2),
            clearance: Clearance {
                gap: 18.0,
                offset: 0.0,
                edge: 4.0,
            },
            max_w: 240.0,
            // `Toolbar/Action/index.tsx:35`: `gap-2 !p-0.5 !pl-2`, the
            // key `[&_span]:size-5 rounded-sm bg-[#303030] border
            // border-[#fcfcfc]/5`.
            key: TooltipKeyTokens {
                size: 20.0,
                radius: 4.0,
                fill: hex(0x303030),
                border: Color::rgba(252.0 / 255.0, 252.0 / 255.0, 252.0 / 255.0, 0.05),
                border_w: 1.0,
                pad: [2.0, 2.0, 2.0, 8.0],
                gap: 8.0,
            },
        }
    }

    /// Every other theme keeps the tooltip it had: its tooltip glass and
    /// hint shadow, caption-r in the secondary text, `md` x `xs` padding,
    /// one and a half `xs` from the anchor, `xs` from the edge.
    pub fn derive(theme: &Theme) -> Self {
        let xs = theme.spacing.xs;
        Self {
            fill: theme.glass.tooltip,
            shadow: theme.shadows.hint.clone(),
            radius: theme.shape.tooltip,
            pad_x: theme.spacing.md,
            pad_y: xs,
            text: theme.typography.caption_r(),
            text_color: theme.colors.text_mid,
            clearance: Clearance {
                gap: xs + xs / 2.0,
                offset: 0.0,
                edge: xs,
            },
            max_w: theme.size.tooltip_max_w,
            // The key: one text line and an `xs` tall, outlined by the
            // theme's soft edge on the tooltip's own fill (a lit surface
            // under the text fails AA on dark, `theme/tests/contrast.rs`);
            // half an `xs` around it, `md` before the text, `sm` between.
            key: TooltipKeyTokens {
                size: theme.typography.caption_r().line_height + xs,
                radius: theme.shape.item.min(xs),
                fill: theme.glass.tooltip,
                border: theme.glass.edge_soft,
                border_w: theme.control.edge_width,
                pad: [xs / 2.0, xs / 2.0, xs / 2.0, theme.spacing.md],
                gap: theme.spacing.sm,
            },
        }
    }
}

/// The hover card: the popover's panel, opened by a resting pointer. The
/// kit has none; its timing is the tooltip's convention and its width the
/// kit's large panel, so every theme shares it.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardTokens {
    /// Seconds the pointer rests on the trigger before it opens (the
    /// platforms' tooltip delay, 0.4 to 0.5 s).
    pub open_delay: f32,
    /// Seconds after the pointer leaves trigger and panel before it
    /// closes: room to cross the gap into the panel.
    pub close_delay: f32,
    /// Preferred panel width (`w-96`, the kit's notifications panel);
    /// the container caps it.
    pub width: f32,
}

impl HoverCardTokens {
    pub fn hoff_core() -> Self {
        Self {
            open_delay: 0.45,
            close_delay: 0.3,
            width: 384.0,
        }
    }
}

/// The modal dialog: the scrim over the window, the sheet, its text and
/// its two buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalTokens {
    pub scrim: Color,
    pub fill: Color,
    /// Drop shadows under the sheet, first on top.
    pub shadow: Vec<ShadowSpec>,
    pub border: Color,
    pub border_w: f32,
    /// The hoff glass: a backdrop blur under the sheet and the inset
    /// key-light on it.
    pub glass: bool,
    pub radius: f32,
    /// The widest the sheet gets; the window less its gutter caps it.
    pub max_w: f32,
    pub pad: f32,
    pub title: TextStyle,
    pub body: TextStyle,
    pub title_color: Color,
    pub body_color: Color,
    /// Between the title and the body, and between the body and the
    /// buttons when they share the sheet's body.
    pub title_gap: f32,
    pub actions_gap: f32,
    /// The text centered (the kit's `text-center`), or at the start.
    pub centered: bool,
    /// A footer band for the buttons (its fill and its top rule; alpha 0
    /// is none, the buttons then sit at the body's end).
    pub footer: Color,
    pub footer_rule: Color,
    pub button_gap: f32,
    /// A dialog with an illustration (`DeleteFile`): the art's bottom,
    /// from the sheet's top (it rises above the sheet by the rest of its
    /// height), and the text's top padding under it.
    pub art_bottom: f32,
    pub art_pad_top: f32,
}

impl ModalTokens {
    /// The kit's dialog (`components/DeleteFile`, the ModalPage): `w-89
    /// bg-[#FCFCFC] rounded-[2rem] border border-[#ECECEC] shadow-2xl`,
    /// a centered body `p-6` with the title `text-[1.5rem] leading-[2rem]
    /// font-medium mb-2` and the text `text-[0.8125rem]
    /// leading-[1.2188rem] text-[#7b7b7b]` (in shade 7: the kit's gray
    /// fails AA), a footer `flex gap-3 p-6
    /// bg-[#F8F7F7] border-t border-[#E2E2E2]` whose buttons share its
    /// width (`flex-1`), over the scrim `#323232/40`.
    pub fn hoff_core() -> Self {
        let type_ = TypographyScale::hoff_core();
        Self {
            scrim: Color::rgba(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 0.40),
            fill: hex(0xFCFCFC),
            shadow: vec![ShadowSpec::new(
                [0.0, 25.0],
                50.0,
                -12.0,
                Color::rgba(0.0, 0.0, 0.0, 0.25),
            )],
            border: hex(0xECECEC),
            border_w: 1.0,
            glass: false,
            radius: 32.0,
            max_w: 356.0,
            pad: 24.0,
            title: type_.exact(24.0, 32.0, 500),
            body: type_.exact(13.0, 19.5, 400),
            title_color: hex(0x121212),
            // The kit's #7b7b7b is 4.13:1 on its paper, under AA; shade 7
            // (#323232) holds AAA (`styling/theme/tests/contrast.rs`).
            body_color: hex(0x323232),
            title_gap: 8.0,
            actions_gap: 24.0,
            centered: true,
            footer: hex(0xF8F7F7),
            footer_rule: hex(0xE2E2E2),
            button_gap: 12.0,
            // `DeleteFile/index.tsx:6`, `:15`: bottom-[calc(100%-11.25rem)],
            // pt-39.
            art_bottom: 180.0,
            art_pad_top: 156.0,
        }
    }

    /// Every other theme keeps the dialog it had: the hoff glass sheet at
    /// the pill radius over the overlay shadow stack, the title and the
    /// base-2r body at the start, the buttons at the body's end.
    pub fn derive(theme: &Theme) -> Self {
        let glass = &theme.glass;
        let none = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let kit = Self::hoff_core();
        Self {
            scrim: glass.scrim,
            fill: glass.popover,
            shadow: theme
                .shadows
                .stack(crate::styling::theme::Elevation::Overlay)
                .to_vec(),
            border: glass.edge_soft,
            border_w: theme.control.edge_width_strong,
            glass: true,
            radius: theme.shape.pill,
            max_w: theme.size.modal_max_w,
            pad: theme.spacing.xxl,
            title: theme.typography.title(),
            body: theme.typography.base_2r(),
            title_color: theme.colors.text,
            body_color: theme.colors.text_mid,
            title_gap: theme.spacing.sm,
            actions_gap: theme.spacing.xl,
            centered: false,
            footer: none,
            footer_rule: none,
            button_gap: theme.spacing.sm,
            // The kit's art, as the kit places it.
            art_bottom: kit.art_bottom,
            art_pad_top: kit.art_pad_top,
        }
    }
}
