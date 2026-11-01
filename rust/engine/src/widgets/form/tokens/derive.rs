//! Every other theme's form recipe: the glass text field each drew
//! before the kit, read from the theme (`FormTokens::derive`).

use super::{
    Adornments, AreaTokens, CAPTION, ColorTokens, EditTokens, FieldChrome, FormTokens, GroupAddons,
    InputRecipe, InputSurface, LabelRecipe, OtpTokens, SelectRecipe, UploadTokens,
};
use crate::styling::color::Color;
use crate::styling::theme::legibility::{self, TEXT_AA};
use crate::styling::theme::{ControlSize, Theme, TypeRole};

impl FormTokens {
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
        // The field's placeholder and its error line on the card read as
        // WCAG AA asks (1.4.3): the glass's placeholder tone and the danger
        // mark moved the least they must. Its edge is the glass's
        // (`legibility::Bar::AA`).
        let card: &[Color] = &[c.bg, c.surface];
        let edge = g.edge_soft;
        let placeholder =
            legibility::reach(g.text_placeholder, TEXT_AA, &[&[c.bg, c.surface, g.field]]);
        let input = InputRecipe {
            rest: surface(g.field, edge),
            hover: surface(g.surface_active, edge),
            focus: surface(g.surface_active, g.field_focus_border),
            height: theme.control.height(ControlSize::Md),
            pad_x: theme.spacing.md,
            radius: theme.shape.nav,
            border_width: theme.control.edge_width,
            text: TypeRole::BodyLg.run(),
            text_color: c.text,
            placeholder,
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
            error_color: legibility::reach(c.danger, TEXT_AA, &[card]),
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
        let mut form = Self {
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
        };
        // the fields' edges in their states (`hold_edges`)
        form.hold_edges([theme.colors.bg, theme.colors.surface], None);
        form
    }
}
