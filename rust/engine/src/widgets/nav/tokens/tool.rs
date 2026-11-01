//! The tool half of the navigation slot: the kit's ToolbarPage
//! (`templates/ToolbarPage`), its toolbar and the tool buttons in it
//! (`components/Toolbar`, `Action`, `ZoomControl`), the zoom panel the
//! toolbar opens (`components/Zoom`), the video player
//! (`components/VideoPlayer`) and the view controller
//! (`components/ViewController`). Core copies the kit's class lists;
//! `derive` draws the same parts in another theme's palette and glass.
//!
//! Two of the kit's tones give way where they fall under the bars
//! (`selection::legible`): the player's `8s` (`text-[#7B7B7B]`, 4.2:1 on
//! paper) takes the kit's dark gray `#323232` to read at 7:1, and the
//! marks that carry state (the player's track and what played of it,
//! the view controller's minor ticks, all a faint tint in the kit) take
//! the kit's gray or ink whole, to read at 3:1. A photo the kit lays a
//! player on is a gradient of its dark shades in every theme (an image
//! does not follow the theme), and what sits on it is the kit's paper.

use super::NavText;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::TypeRole;
use crate::styling::theme::hoff_core::SHADES;
use crate::styling::theme::scales::{ShadowSpec, ShadowTokens};
use crate::styling::theme::{ControlSize, IconSize};

/// One tool button (`Toolbar/Action`): a square box with an icon, a wash
/// on hover, a raised-in well while it is the active tool, faded while
/// disabled.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolRecipe {
    pub size: f32,
    pub radius: f32,
    pub border_width: f32,
    pub icon: f32,
    pub icon_color: Color,
    pub hover_fill: Color,
    pub active_fill: Color,
    pub active_border: Color,
    /// The active well's css shadow list (insets).
    pub active_shadow: Vec<ShadowSpec>,
    pub disabled_alpha: f32,
}

/// The toolbar: a pill of tool buttons, the zoom control and the export
/// button after a divider.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarRecipe {
    pub tool: ToolRecipe,
    pub radius: f32,
    pub border_width: f32,
    pub fill: Color,
    pub border: Color,
    pub shadow: Vec<ShadowSpec>,
    /// Around the tools and around the export button.
    pub pad: f32,
    /// Between tools.
    pub gap: f32,
    pub divider: Color,
    /// The zoom control: as wide as `zoom_min_w` at least, its value and
    /// chevron `zoom_gap` apart inside `zoom_pad_x`.
    pub zoom_min_w: f32,
    pub zoom_pad_x: f32,
    pub zoom_gap: f32,
    pub zoom_radius: f32,
    pub zoom_fill: Color,
    pub zoom_border: Color,
    pub zoom_hover_fill: Color,
    pub zoom_hover_border: Color,
    pub zoom_open_border: Color,
    pub zoom_text: NavText,
    pub zoom_text_color: Color,
    pub chevron: f32,
    pub chevron_color: Color,
    /// The export button's width (`w-23`).
    pub export_w: f32,
    /// Between the control and the panel it opens.
    pub panel_gap: f32,
}

/// The zoom panel: a field with the level, the zoom commands with their
/// keys, a rule, and two view toggles; a check marks what is on.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoomRecipe {
    pub width: f32,
    pub radius: f32,
    pub border_width: f32,
    pub fill: Color,
    pub border: Color,
    pub shadow: Vec<ShadowSpec>,
    pub pad: f32,
    pub rule: Color,
    /// The field: its height, its well, its edge, its icon and text.
    pub field_h: f32,
    pub field_radius: f32,
    pub field_fill: Color,
    pub field_border: Color,
    pub field_icon: f32,
    pub field_icon_color: Color,
    /// From the field's edge to its text (the icon centered in it).
    pub field_text_x: f32,
    pub field_text: NavText,
    pub field_text_color: Color,
    /// A command row.
    pub line_h: f32,
    pub line_radius: f32,
    pub line_pad_x: f32,
    pub line_hover: Color,
    pub check_slot: f32,
    pub check: f32,
    pub check_color: Color,
    pub check_gap: f32,
    pub label: NavText,
    pub label_color: Color,
}

/// The tones of a player: on its own pill, or on a photo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerTones {
    pub icon: Color,
    pub icon_hover: Color,
    pub track: Color,
    pub played: Color,
    pub head: Color,
    pub time: Color,
    pub repeat_fill: Color,
    pub repeat_hover: Color,
}

/// The video player: play, the timeline, the time, and repeat.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerRecipe {
    pub height: f32,
    pub pad_start: f32,
    pub pad_end: f32,
    pub radius: f32,
    pub border_width: f32,
    pub fill: Color,
    pub border: Color,
    pub shadow: Vec<ShadowSpec>,
    pub button: f32,
    pub icon: f32,
    pub button_gap: f32,
    pub track_h: f32,
    pub track_min_w: f32,
    pub head_w: f32,
    pub time: NavText,
    pub time_w: f32,
    pub time_gap: f32,
    pub repeat: f32,
    pub repeat_gap: f32,
    /// How far the repeat button reaches into the end padding (`-mr-2`).
    pub repeat_overhang: f32,
    pub solid: PlayerTones,
    pub on_image: PlayerTones,
    /// The photo's stand-in: a gradient, top to bottom.
    pub image: [Color; 2],
}

/// The view controller: two arrows around a ruler of ticks, across
/// (orbit) or upright (tilt).
#[derive(Clone, Debug, PartialEq)]
pub struct ViewRecipe {
    /// The pill's short side.
    pub thick: f32,
    pub pad: f32,
    pub gap: f32,
    pub border_width: f32,
    pub fill: Color,
    pub border: Color,
    pub shadow: Vec<ShadowSpec>,
    pub arrow: f32,
    pub back: Color,
    pub forward: Color,
    pub tick_gap: f32,
    pub tick_w: f32,
    pub tick_short: f32,
    pub tick_long: f32,
    pub tick: Color,
    pub tick_major: Color,
}

/// The three soft drops the player and the view controller cast
/// (`0 6px 3px rgba(0,0,0,.01), 0 3px 3px .02, 0 1px 1px .02`).
fn soft_drops() -> Vec<ShadowSpec> {
    let black = |a: f32| Color::rgba(0.0, 0.0, 0.0, a);
    vec![
        ShadowSpec::new([0.0, 6.0], 3.0, 0.0, black(0.01)),
        ShadowSpec::new([0.0, 3.0], 3.0, 0.0, black(0.02)),
        ShadowSpec::new([0.0, 1.0], 1.0, 0.0, black(0.02)),
    ]
}

fn alpha(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], c.0[3] * a])
}

/// What sits on a photo: the kit's paper, its track at 20% and what
/// played at 30% (both read at 3:1 on the stand-in's lighter end).
fn on_image() -> PlayerTones {
    let paper = SHADES[0];
    PlayerTones {
        icon: paper,
        icon_hover: alpha(paper, 0.8),
        track: alpha(paper, 0.2),
        played: alpha(paper, 0.3),
        head: paper,
        time: paper,
        repeat_fill: Color::TRANSPARENT,
        repeat_hover: alpha(paper, 0.2),
    }
}

/// The stand-in for a photo: the kit's two darkest grays.
fn image() -> [Color; 2] {
    [SHADES[6], SHADES[8]]
}

/// The tool parts of the slot.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolTokens {
    pub toolbar: ToolbarRecipe,
    pub zoom: ZoomRecipe,
    pub player: PlayerRecipe,
    pub view: ViewRecipe,
}

impl ToolTokens {
    /// The kit's class lists.
    pub fn hoff_core() -> Self {
        let ShadowTokens {
            floating, overlay, ..
        } = ShadowTokens::hoff_core();
        let [paper, well, wash, line, rim, gray, dark, _, ink] = SHADES;
        let black = Color::hex(0x000000);
        // Action: `size-10 border rounded-xl`, `size-5 fill-[#121212]`,
        // `hover:bg-[#f1f1f1]`; active `bg-[#f1f1f1] border-[#e2e2e2]
        // shadow-[0_-1px_3px_0px_rgba(18,18,18,0.15)_inset,_0px_1.25px_1px_0px_#FFF_inset]`;
        // disabled `opacity-40`.
        let tool = ToolRecipe {
            size: 40.0,
            radius: 12.0,
            border_width: 1.0,
            icon: 20.0,
            icon_color: ink,
            hover_fill: wash,
            active_fill: wash,
            active_border: rim,
            active_shadow: vec![
                ShadowSpec::inset([0.0, -1.0], 3.0, 0.0, alpha(ink, 0.15)),
                ShadowSpec::inset([0.0, 1.25], 1.0, 0.0, Color::hex(0xFFFFFF)),
            ],
            disabled_alpha: 0.4,
        };
        // Toolbar: `border border-[#ececec] bg-[#fcfcfc] rounded-[1.25rem]`
        // under the kit's Toolbar depth (theme.shadows.floating), `gap-2
        // p-2`, the export side `p-2 border-l`. ZoomControl: `min-w-22
        // h-10 px-3 gap-2 rounded-xl border-[#ececec] bg-[#f1f1f1]
        // text-[0.875rem] font-medium`, `hover:bg-[#FCFCFC]
        // hover:border-[#E2E2E2]`, open `border-[#121212]/10`, the chevron
        // `size-4 fill-[#7b7b7b]`; the panels `[--anchor-gap:0.75rem]`.
        let toolbar = ToolbarRecipe {
            tool,
            radius: 20.0,
            border_width: 1.0,
            fill: paper,
            border: line,
            shadow: floating,
            pad: 8.0,
            gap: 8.0,
            divider: line,
            zoom_min_w: 88.0,
            zoom_pad_x: 12.0,
            zoom_gap: 8.0,
            zoom_radius: 12.0,
            zoom_fill: wash,
            zoom_border: line,
            zoom_hover_fill: paper,
            zoom_hover_border: rim,
            zoom_open_border: alpha(ink, 0.1),
            zoom_text: NavText::new(14.0, 20.0, 500),
            zoom_text_color: ink,
            chevron: 16.0,
            chevron_color: gray,
            export_w: 92.0,
            panel_gap: 12.0,
        };
        // Zoom: the panel `w-51 rounded-2xl bg-[#FCFCFC] border-[#ECECEC]
        // shadow-2xl` (theme.shadows.overlay), `p-2` and a `border-b`;
        // the field `!h-9 !pl-9 !bg-[#F8F7F7]` in the Field's rim and
        // radius (`rounded-xl`), its icon `size-4 fill-[#7B7B7B]`; a line
        // `h-9 px-2 rounded-[0.625rem] hover:bg-[#F1F1F1]`, the check slot
        // `size-6 mr-1` with a `size-4 fill-[#7B7B7B]` check, the title
        // `text-[0.75rem] leading-[1rem] font-medium text-[#000]`.
        let zoom = ZoomRecipe {
            width: 204.0,
            radius: 16.0,
            border_width: 1.0,
            fill: paper,
            border: line,
            shadow: overlay,
            pad: 8.0,
            rule: line,
            field_h: 36.0,
            field_radius: 12.0,
            field_fill: well,
            field_border: line,
            field_icon: 16.0,
            field_icon_color: gray,
            field_text_x: 36.0,
            field_text: NavText::new(14.0, 20.0, 500),
            field_text_color: ink,
            line_h: 36.0,
            line_radius: 10.0,
            line_pad_x: 8.0,
            line_hover: wash,
            check_slot: 24.0,
            check: 16.0,
            check_color: gray,
            check_gap: 4.0,
            label: NavText::new(12.0, 16.0, 500),
            label_color: black,
        };
        // VideoPlayer: `h-11 pl-2.5 pr-3.5 rounded-3xl border-[#ececec]
        // bg-[#fcfcfc]`; play `size-6 mr-3`, its icon `size-4
        // fill-[#7b7b7b] group-hover:fill-[#121212]`; the track `min-w-45
        // h-0.75`, the head `w-1.25`; the time `w-6 ml-4 text-[0.75rem]
        // font-semibold`; repeat `size-8 ml-1.5 -mr-2 bg-[#f1f1f1]
        // hover:bg-[#7b7b7b]/20`.
        let player = PlayerRecipe {
            height: 44.0,
            pad_start: 10.0,
            pad_end: 14.0,
            radius: 24.0,
            border_width: 1.0,
            fill: paper,
            border: line,
            shadow: soft_drops(),
            button: 24.0,
            icon: 16.0,
            button_gap: 12.0,
            track_h: 3.0,
            track_min_w: 180.0,
            head_w: 5.0,
            time: NavText::new(12.0, 16.0, 600),
            time_w: 24.0,
            time_gap: 16.0,
            repeat: 32.0,
            repeat_gap: 6.0,
            repeat_overhang: 8.0,
            solid: PlayerTones {
                icon: gray,
                icon_hover: ink,
                track: gray,
                played: ink,
                head: ink,
                time: dark,
                repeat_fill: wash,
                repeat_hover: alpha(gray, 0.2),
            },
            on_image: on_image(),
            image: image(),
        };
        // ViewController: `gap-3 bg-[#fcfcfc] border-[#ececec]
        // rounded-full`, `h-11 px-5` across (`w-11 py-5` upright); the
        // arrows `size-4`, the back one `fill-[#7b7b7b]`, the forward one
        // the default ink; the ticks `gap-2`, `w-[0.09375rem]`, `h-2` and
        // `h-3.5` for the two majors.
        let view = ViewRecipe {
            thick: 44.0,
            pad: 20.0,
            gap: 12.0,
            border_width: 1.0,
            fill: paper,
            border: line,
            shadow: soft_drops(),
            arrow: 16.0,
            back: gray,
            forward: ink,
            tick_gap: 8.0,
            tick_w: 1.5,
            tick_short: 8.0,
            tick_long: 14.0,
            tick: gray,
            tick_major: ink,
        };
        Self {
            toolbar,
            zoom,
            player,
            view,
        }
    }

    /// The same parts in another theme: the bar on the panel tone inside
    /// the glass's soft edge, the tools in the text tone on the glass's
    /// washes, the quiet marks in the mid text tone, the steps of the
    /// spacing and the control scale.
    pub fn derive(theme: &Theme) -> Self {
        let (colors, glass, control, space) =
            (&theme.colors, &theme.glass, &theme.control, &theme.spacing);
        let core = Self::hoff_core();
        let tool = ToolRecipe {
            size: control.height(ControlSize::Lg),
            radius: theme.shape.nav,
            border_width: control.edge_width,
            icon: control.icon(IconSize::Md),
            icon_color: colors.text,
            hover_fill: glass.surface_hover,
            active_fill: glass.surface_active,
            active_border: glass.edge,
            active_shadow: Vec::new(),
            disabled_alpha: core.toolbar.tool.disabled_alpha,
        };
        let toolbar = ToolbarRecipe {
            tool,
            radius: theme.radius.lg,
            border_width: control.edge_width,
            fill: colors.bg_panel,
            border: glass.edge_soft,
            shadow: theme.shadows.floating.clone(),
            pad: space.sm,
            gap: space.sm,
            divider: glass.edge_soft,
            zoom_pad_x: space.sm + space.xs,
            zoom_gap: space.sm,
            zoom_radius: theme.shape.nav,
            zoom_fill: glass.field,
            zoom_border: glass.edge_soft,
            zoom_hover_fill: glass.surface_hover,
            zoom_hover_border: glass.edge,
            zoom_open_border: glass.field_focus_border,
            zoom_text: NavText::of(&theme.typography.run(TypeRole::Body.medium())),
            zoom_text_color: colors.text,
            chevron: control.icon(IconSize::Sm),
            chevron_color: colors.text_mid,
            panel_gap: space.sm + space.xs,
            ..core.toolbar
        };
        let zoom = ZoomRecipe {
            radius: theme.radius.lg,
            border_width: control.edge_width,
            fill: colors.bg_panel,
            border: glass.edge_soft,
            shadow: theme.shadows.overlay.clone(),
            pad: space.sm,
            rule: glass.edge_soft,
            field_radius: theme.shape.nav,
            field_fill: glass.field,
            field_border: glass.edge_soft,
            field_icon_color: colors.text_mid,
            field_text: NavText::of(&theme.typography.run(TypeRole::Body.medium())),
            field_text_color: colors.text,
            line_radius: theme.shape.nav,
            line_pad_x: space.sm,
            line_hover: glass.surface_hover,
            check_color: colors.text_mid,
            check_gap: space.xs,
            label: NavText::of(&theme.typography.run(TypeRole::Caption.strong())),
            label_color: colors.text,
            ..core.zoom
        };
        let player = PlayerRecipe {
            border_width: control.edge_width,
            fill: colors.bg_panel,
            border: glass.edge_soft,
            button_gap: space.sm + space.xs,
            time_gap: space.md,
            solid: PlayerTones {
                icon: colors.text_mid,
                icon_hover: colors.text,
                track: colors.text_mid,
                played: colors.text,
                head: colors.text,
                time: colors.text,
                repeat_fill: glass.surface_hover,
                repeat_hover: glass.surface_active,
            },
            ..core.player
        };
        let view = ViewRecipe {
            pad: space.lg - space.xs,
            gap: space.sm + space.xs,
            border_width: control.edge_width,
            fill: colors.bg_panel,
            border: glass.edge_soft,
            back: colors.text_mid,
            forward: colors.text,
            tick_gap: space.sm,
            tick: colors.text_mid,
            tick_major: colors.text,
            ..core.view
        };
        Self {
            toolbar,
            zoom,
            player,
            view,
        }
    }
}
