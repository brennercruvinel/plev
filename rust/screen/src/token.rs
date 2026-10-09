//! every token of the IR on its field of the `Theme`, by name, in one
//! exhaustive `match` per kind: a token the `uir` adds without a home in
//! the theme stops the build here, not at a surface on screen.

use engine::widgets::buttons::ButtonSize;
use engine::widgets::content::{LineKind as Row, StatSize};
use engine::drawing::text::TextStyle;
use engine::styling::theme::{self, Intent, Theme};
use plevir::{
    ColorRole, ControlSize, Elevation, LineKind, ShapeRole, SizeToken, SpacingToken, TypographyRole,
};

/// the kind of a `code`'s line, by name in the app's `kinds`, on the
/// kind of row the comps `CodeBlock` draws with its own tokens.
pub(crate) fn line_kind(k: LineKind) -> Row {
    match k {
        LineKind::Context => Row::Context,
        LineKind::Added => Row::Added,
        LineKind::Removed => Row::Removed,
        LineKind::Header => Row::Header,
    }
}

/// `theme.spacing.<token>`.
pub(crate) fn spacing(t: SpacingToken, theme: &Theme) -> f32 {
    let s = &theme.spacing;
    match t {
        SpacingToken::Xs => s.xs,
        SpacingToken::Sm => s.sm,
        SpacingToken::Md => s.md,
        SpacingToken::Lg => s.lg,
        SpacingToken::Xl => s.xl,
        SpacingToken::Xxl => s.xxl,
    }
}

/// `theme.typography.<role>()`.
pub(crate) fn typography(t: TypographyRole, theme: &Theme) -> TextStyle {
    let ty = &theme.typography;
    match t {
        TypographyRole::H4 => ty.h4(),
        TypographyRole::Headline => ty.headline(),
        TypographyRole::Title => ty.title(),
        TypographyRole::BaseR => ty.base_r(),
        TypographyRole::BaseM => ty.base_m(),
        TypographyRole::Base2r => ty.base_2r(),
        TypographyRole::Base2m => ty.base_2m(),
        TypographyRole::Base2sm => ty.base_2sm(),
        TypographyRole::Base2b => ty.base_2b(),
        TypographyRole::CaptionR => ty.caption_r(),
        TypographyRole::CaptionSm => ty.caption_sm(),
        TypographyRole::SmallR => ty.small_r(),
        TypographyRole::SmallSm => ty.small_sm(),
        TypographyRole::Body => ty.body(),
        TypographyRole::Body2r => ty.body_2r(),
        TypographyRole::Body2m => ty.body_2m(),
        TypographyRole::Body2b => ty.body_2b(),
        TypographyRole::Hairline => ty.hairline(),
        TypographyRole::Mono => ty.mono(),
        TypographyRole::Readout => ty.readout(),
    }
}

/// the size of a stat whose value is drawn in `t`: the base ramp and
/// under is the inline readout, `title` the card's (a stat with no
/// style), `headline` and over the hero's.
pub(crate) fn stat_size(t: TypographyRole) -> StatSize {
    match t {
        TypographyRole::H4 | TypographyRole::Headline => StatSize::Lg,
        TypographyRole::Title => StatSize::Md,
        _ => StatSize::Sm,
    }
}

/// the intent a badge's fill takes from its `background`: the accent and
/// the three roles an intent has; none for a role no intent fills with
/// (the badge stays the glass tag).
pub(crate) fn badge_intent(t: ColorRole) -> Option<Intent> {
    match t {
        ColorRole::Accent => Some(Intent::Neutral),
        ColorRole::Success => Some(Intent::Constructive),
        ColorRole::Danger => Some(Intent::Destructive),
        ColorRole::Info => Some(Intent::Informational),
        _ => None,
    }
}

/// `theme.colors.<role>`, as the RGBA a scene node takes.
pub(crate) fn color(t: ColorRole, theme: &Theme) -> [f32; 4] {
    let c = &theme.colors;
    let color = match t {
        ColorRole::Bg => c.bg,
        ColorRole::Surface => c.surface,
        ColorRole::BgPanel => c.bg_panel,
        ColorRole::BgHover => c.bg_hover,
        ColorRole::Text => c.text,
        ColorRole::TextDim => c.text_dim,
        ColorRole::TextMid => c.text_mid,
        ColorRole::Accent => c.accent,
        ColorRole::AccentDim => c.accent_dim,
        ColorRole::Success => c.success,
        ColorRole::Danger => c.danger,
        ColorRole::Warning => c.warning,
        ColorRole::Info => c.info,
        ColorRole::Divider => c.divider,
        ColorRole::BorderActive => c.border_active,
    };
    color.0
}

/// `engine::theme::Elevation`, whose stack is `theme.shadows.stack(e)`.
pub(crate) fn elevation(t: Elevation) -> theme::Elevation {
    match t {
        Elevation::Base => theme::Elevation::Base,
        Elevation::Raised => theme::Elevation::Raised,
        Elevation::Floating => theme::Elevation::Floating,
        Elevation::Overlay => theme::Elevation::Overlay,
        Elevation::Hint => theme::Elevation::Hint,
    }
}

/// `engine::theme::ControlSize`, whose height is `theme.control.height`.
pub(crate) fn control(t: ControlSize) -> theme::ControlSize {
    match t {
        ControlSize::Xs => theme::ControlSize::Xs,
        ControlSize::Sm => theme::ControlSize::Sm,
        ControlSize::Md => theme::ControlSize::Md,
        ControlSize::Lg => theme::ControlSize::Lg,
        ControlSize::Xl => theme::ControlSize::Xl,
    }
}

/// `theme.shape.<role>`, a corner radius.
pub(crate) fn shape(t: ShapeRole, theme: &Theme) -> f32 {
    let s = &theme.shape;
    match t {
        ShapeRole::Pill => s.pill,
        ShapeRole::Dropdown => s.dropdown,
        ShapeRole::Tabs => s.tabs,
        ShapeRole::Card => s.card,
        ShapeRole::Block => s.block,
        ShapeRole::Item => s.item,
        ShapeRole::Nav => s.nav,
        ShapeRole::Cluster => s.cluster,
        ShapeRole::Tooltip => s.tooltip,
        ShapeRole::Micro => s.micro,
    }
}

/// which way a size token measures: every token is a width but
/// `dropdown_max_h`, as its name says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Dim {
    Width,
    Height,
}

/// `theme.size.<token>` and the dimension it bounds.
pub(crate) fn size(t: SizeToken, theme: &Theme) -> (f32, Dim) {
    let s = &theme.size;
    match t {
        SizeToken::MenuW => (s.menu_w, Dim::Width),
        SizeToken::ModalMaxW => (s.modal_max_w, Dim::Width),
        SizeToken::SidebarW => (s.sidebar_w, Dim::Width),
        SizeToken::SidebarRailW => (s.sidebar_rail_w, Dim::Width),
        SizeToken::CardW => (s.card_w, Dim::Width),
        SizeToken::CardMinW => (s.card_min_w, Dim::Width),
        SizeToken::ReadableMaxW => (s.readable_max_w, Dim::Width),
        SizeToken::FieldMinW => (s.field_min_w, Dim::Width),
        SizeToken::ToastW => (s.toast_w, Dim::Width),
        SizeToken::TooltipMaxW => (s.tooltip_max_w, Dim::Width),
        SizeToken::DropdownMaxH => (s.dropdown_max_h, Dim::Height),
    }
}

/// the step of the button's own ladder (`theme.button.sizes`) for a
/// control size: the tallest `ButtonSize` not taller than
/// `theme.control.height` of the token, the shortest when none fits. an
/// approximation: the button does not read `theme.control`.
pub(crate) fn button_size(t: ControlSize, theme: &Theme) -> ButtonSize {
    let height = theme.control.height(control(t));
    [
        ButtonSize::Xs,
        ButtonSize::Sm,
        ButtonSize::Md,
        ButtonSize::Lg,
    ]
    .into_iter()
    .rev()
    .find(|b| b.height(theme) <= height)
    .unwrap_or(ButtonSize::Xs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_token_reads_its_own_field() {
        let theme = Theme::hoff_core();
        assert_eq!(spacing(SpacingToken::Md, &theme), theme.spacing.md);
        assert_eq!(
            typography(TypographyRole::Title, &theme),
            theme.typography.title()
        );
        assert_eq!(color(ColorRole::BgPanel, &theme), theme.colors.bg_panel.0);
        assert_eq!(shape(ShapeRole::Card, &theme), theme.shape.card);
        assert_eq!(
            size(SizeToken::FieldMinW, &theme),
            (theme.size.field_min_w, Dim::Width)
        );
        assert_eq!(size(SizeToken::DropdownMaxH, &theme).1, Dim::Height);
        for t in SizeToken::ALL {
            let (_, dim) = size(*t, &theme);
            let tall = t.as_str().ends_with("_h");
            assert_eq!(dim == Dim::Height, tall, "{}", t.as_str());
        }
    }

    #[test]
    fn a_button_takes_the_tallest_step_that_fits_the_control_height() {
        let theme = Theme::hoff_core();
        for t in ControlSize::ALL {
            let b = button_size(*t, &theme);
            assert!(b.height(&theme) <= theme.control.height(control(*t)));
        }
        // hoff core: controls 36/40/44/48/52, buttons 28/36/40/44.
        assert_eq!(button_size(ControlSize::Xs, &theme), ButtonSize::Sm);
        assert_eq!(button_size(ControlSize::Sm, &theme), ButtonSize::Md);
        assert_eq!(button_size(ControlSize::Md, &theme), ButtonSize::Lg);
        assert_eq!(button_size(ControlSize::Xl, &theme), ButtonSize::Lg);
    }
}
