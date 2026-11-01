//! The tokens of the two edge panels (task-104, front B): the sheet and
//! the drawer, which share the surface of a panel held to one edge of
//! its container ([`EdgePanelTokens`]).
//!
//! The kit's one edge panel is its own navigation drawer: the guideline
//! sidebar that `components/Layout` slides in from the left below 1023
//! px. The drawer is that, as written; the sheet takes its surface and
//! its scrim, and its content from the kit's settings panel
//! (`components/Settings`: the title, the option rows), which is a
//! centered panel and not an edge one.
//!
//! The kit's secondary gray (`#7B7B7B` at 70 and 80 %) is 3.0 to 3.6:1
//! on its paper, under WCAG AA; the hint and the group label take the
//! core palette's shade 7 (`#323232`, 12:1, AAA) instead
//! (`styling/theme/tests/contrast.rs`, task-104 front B).

use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::TypeRole;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{Elevation, Theme, TypographyScale};

/// The surface of a panel held to one edge of its container.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgePanelTokens {
    /// Over the container while the panel is open.
    pub scrim: Color,
    pub fill: Color,
    /// The border of the panel's free edges; alpha 0 is none.
    pub rule: Color,
    pub rule_w: f32,
    /// Drop shadows under the panel, first on top.
    pub shadow: Vec<ShadowSpec>,
    /// The two corners away from the edge it is held to.
    pub radius: f32,
    /// The extent it prefers across its edge (a side panel's width); the
    /// container caps it, less a touch target of scrim to close it by.
    pub extent: f32,
}

/// The sheet: an edge panel with a title, a description, option rows
/// and the dialog's two answers.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetTokens {
    pub panel: EdgePanelTokens,
    pub pad_x: f32,
    pub pad_y: f32,
    pub title: TextStyle,
    pub title_color: Color,
    pub description: TextStyle,
    pub description_color: Color,
    /// Between the title and the description, and a row's label and hint.
    pub gap: f32,
    pub row_min_h: f32,
    pub label: TextStyle,
    pub label_color: Color,
    pub hint: TextStyle,
    pub hint_color: Color,
    pub value: TextStyle,
    pub value_color: Color,
    /// Between a row's label and its value.
    pub value_gap: f32,
}

/// The drawer: an edge panel of navigation, groups of links under the
/// app's name.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawerTokens {
    pub panel: EdgePanelTokens,
    /// Around the head (the name and the close button), and at its end
    /// while it is a drawer.
    pub head_pad: f32,
    pub head_pad_end: f32,
    /// The head's content height (the kit's logo).
    pub head_h: f32,
    pub brand: TextStyle,
    pub brand_color: Color,
    /// Left and right of the groups, and under the last one.
    pub body_pad: f32,
    /// Between two groups.
    pub group_gap: f32,
    pub group_label: TextStyle,
    pub group_color: Color,
    pub group_pad: f32,
    /// Between a group's label and its first link, and between links.
    pub item_gap: f32,
    pub item_pad: f32,
    pub item_radius: f32,
    /// Between the icon's box and the label.
    pub item_inline: f32,
    pub item_label: TextStyle,
    pub item_color: Color,
    /// A link hovered or current.
    pub item_fill: Color,
    /// The icon's box; the current link's raised.
    pub icon_box: f32,
    pub icon_box_radius: f32,
    pub icon_box_fill: Color,
    /// The icon box's shadow stack, bottom layer first.
    pub icon_box_shadow: Vec<ShadowSpec>,
    pub icon: f32,
    pub icon_color: Color,
    pub icon_active: Color,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

/// `shadow-2xl` (tailwind v4 theme.css): 0 25px 50px -12px black/25.
fn shadow_2xl() -> ShadowSpec {
    ShadowSpec::new([0.0, 25.0], 50.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.25))
}

impl EdgePanelTokens {
    /// The kit's drawer (`Layout/index.tsx:21-31`, `GuidelineSidebar/
    /// index.tsx:26`): `w-65 bg-[#FCFCFC] border-r border-[#ECECEC]`,
    /// square, flat, over `bg-[#323232]/40`.
    pub fn hoff_core() -> Self {
        Self {
            scrim: Color::rgba(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 0.40),
            fill: hex(0xFCFCFC),
            rule: hex(0xECECEC),
            rule_w: 1.0,
            shadow: Vec::new(),
            radius: 0.0,
            extent: 260.0,
        }
    }

    /// The theme's own floating surface: the popover glass over its
    /// scrim, its soft edge, the overlay shadow stack, the pill radius,
    /// the kit's extent.
    pub fn derive(theme: &Theme) -> Self {
        let glass = &theme.glass;
        Self {
            scrim: glass.scrim,
            fill: glass.popover,
            rule: glass.edge_soft,
            rule_w: theme.control.edge_width,
            shadow: theme.shadows.stack(Elevation::Overlay).to_vec(),
            radius: theme.shape.pill,
            ..Self::hoff_core()
        }
    }
}

impl SheetTokens {
    /// The drawer's surface with the settings panel's corners and shadow
    /// (`Settings/index.tsx:14`: `rounded-3xl shadow-2xl`) at the kit's
    /// wide panel (`w-96`, `NotificationsDropdown/index.tsx:35`); its
    /// title (`Settings/Title/index.tsx:6`: `px-6 py-4 text-[1.125rem]
    /// leading-[1.6875rem] font-medium`), the dialog's body for the
    /// description (`DeleteFile`: 13 / 19.5), its rows
    /// (`Settings/Option/index.tsx:8-13`: `min-h-14 px-6 py-4 border-t`,
    /// the label `text-[0.75rem] leading-[1rem] font-medium`, the hint
    /// `mt-2 text-[0.6875rem] leading-[1rem] font-medium`, `pr-3` before
    /// the value) and the value of `Settings/Profile/index.tsx:33`
    /// (`text-[0.75rem] font-medium`, the preflight's 1.5).
    pub fn hoff_core() -> Self {
        let type_ = TypographyScale::hoff_core();
        let ink = hex(0x121212);
        // Shade 7: the kit's #7B7B7B fails AA on its paper (module docs).
        let muted = hex(0x323232);
        Self {
            panel: EdgePanelTokens {
                shadow: vec![shadow_2xl()],
                radius: 24.0,
                extent: 384.0,
                ..EdgePanelTokens::hoff_core()
            },
            pad_x: 24.0,
            pad_y: 16.0,
            title: type_.exact(18.0, 27.0, 500),
            title_color: ink,
            description: type_.exact(13.0, 19.5, 400),
            description_color: muted,
            gap: 8.0,
            row_min_h: 56.0,
            label: type_.exact(12.0, 16.0, 500),
            label_color: ink,
            hint: type_.exact(11.0, 16.0, 500),
            hint_color: muted,
            value: type_.exact(12.0, 18.0, 500),
            value_color: ink,
            value_gap: 12.0,
        }
    }

    /// The theme's surface, its title and body ramp, its text tokens, its
    /// spacing; the kit's row height.
    pub fn derive(theme: &Theme) -> Self {
        let s = &theme.spacing;
        let t = &theme.typography;
        let c = &theme.colors;
        Self {
            panel: EdgePanelTokens {
                extent: Self::hoff_core().panel.extent,
                ..EdgePanelTokens::derive(theme)
            },
            pad_x: s.lg,
            pad_y: s.md,
            title: t.role(TypeRole::Title),
            title_color: c.text,
            description: t.role(TypeRole::Body),
            description_color: c.text_mid,
            gap: s.sm,
            label: t.role(TypeRole::BodyStrong),
            label_color: c.text,
            hint: t.run(TypeRole::Caption.regular()),
            hint_color: c.text_mid,
            value: t.role(TypeRole::Body),
            value_color: c.text,
            value_gap: s.md,
            ..Self::hoff_core()
        }
    }
}

impl DrawerTokens {
    /// The kit's drawer as written (`GuidelineSidebar/index.tsx`): the
    /// head `p-6` around the 32 px logo (:31-38), the groups `px-5 pb-5`
    /// (:56) `mb-3` apart (:58), each label `mb-0.5 p-2.5 text-[0.75rem]
    /// leading-[1rem] font-medium` (:59), the links `gap-0.5 p-1
    /// rounded-xl gap-3 text-[0.75rem] leading-[1rem] font-semibold
    /// text-[#121212] hover:bg-[#F1F1F1]` (:62-65), the icon's box
    /// `size-8 rounded-lg`, raised on the current link with
    /// `bg-[#FCFCFC] shadow-[0px_0px_4px_0px_rgba(18,18,18,0.10)]`
    /// (:73-76), the icon `size-5 fill-[#7B7B7B]`, `#121212` hovered or
    /// current (:81). The name takes the sheet's title in the logo's place.
    pub fn hoff_core() -> Self {
        let type_ = TypographyScale::hoff_core();
        let ink = hex(0x121212);
        Self {
            panel: EdgePanelTokens::hoff_core(),
            head_pad: 24.0,
            // max-[1023px]:pr-4 (:31): the drawer's.
            head_pad_end: 16.0,
            head_h: 32.0,
            brand: type_.exact(18.0, 27.0, 500),
            brand_color: ink,
            body_pad: 20.0,
            group_gap: 12.0,
            group_label: type_.exact(12.0, 16.0, 500),
            // Shade 7: the kit's #7B7B7B/70 fails AA (module docs).
            group_color: hex(0x323232),
            group_pad: 10.0,
            item_gap: 2.0,
            item_pad: 4.0,
            item_radius: 12.0,
            item_inline: 12.0,
            item_label: type_.exact(12.0, 16.0, 600),
            item_color: ink,
            item_fill: hex(0xF1F1F1),
            icon_box: 32.0,
            icon_box_radius: 8.0,
            icon_box_fill: hex(0xFCFCFC),
            icon_box_shadow: vec![ShadowSpec::new(
                [0.0, 0.0],
                4.0,
                0.0,
                Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.10),
            )],
            icon: 20.0,
            icon_color: hex(0x7B7B7B),
            icon_active: ink,
        }
    }

    /// The theme's surface, its text and secondary text (which hold AA on
    /// its popover glass, where the glass's nav text does not on every
    /// theme), its active surface for a link hovered or current, the item
    /// radius, its spacing; the kit's geometry.
    pub fn derive(theme: &Theme) -> Self {
        let g = &theme.glass;
        let s = &theme.spacing;
        let t = &theme.typography;
        Self {
            panel: EdgePanelTokens::derive(theme),
            head_pad: s.lg,
            head_pad_end: s.md,
            brand: t.role(TypeRole::Title),
            brand_color: theme.colors.text,
            body_pad: s.lg,
            group_gap: s.md,
            group_label: t.run(TypeRole::Caption.regular()),
            group_color: theme.colors.text_mid,
            group_pad: s.sm,
            item_radius: theme.shape.item,
            item_inline: s.md,
            item_label: t.role(TypeRole::BodyStrong),
            item_color: theme.colors.text,
            item_fill: g.surface_active,
            icon_box_radius: theme.shape.micro,
            icon_box_fill: g.surface_hover,
            icon_box_shadow: theme.shadows.raised.clone(),
            icon_color: theme.colors.text_mid,
            icon_active: theme.colors.text,
            ..Self::hoff_core()
        }
    }
}
