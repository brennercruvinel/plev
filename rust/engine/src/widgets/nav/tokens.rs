//! Navigation tokens: the token slot of task-104 front G, navigation and
//! data, for the breadcrumb, the navigation menu's link, the sidebar's
//! link, the stat, the code block, the charts and the graph.
//!
//! `NavigationTokens::hoff_core` is the hoff core kit (the private
//! typescript design system) where it has the component, and a
//! derivation from its palette, type, radii and shadows where it has
//! not; each value says which. Every other theme gets
//! [`NavigationTokens::derive`], which writes back the values these
//! widgets drew with before the slot existed (the hoff glass and the
//! generic tokens), so those themes look as they did.

use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::hoff_core::SHADES;
use crate::styling::theme::tokens::TypographyScale;
use crate::styling::theme::{IconSize, Theme};

/// A text run: size, line height and weight in px, in the theme's family
/// and tracking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavText {
    pub size: f32,
    pub line_height: f32,
    pub weight: u16,
}

impl NavText {
    pub const fn new(size: f32, line_height: f32, weight: u16) -> Self {
        Self {
            size,
            line_height,
            weight,
        }
    }

    /// The one style the run is measured and drawn with.
    pub fn style(&self, typography: &TypographyScale) -> TextStyle {
        typography.exact(self.size, self.line_height, self.weight)
    }

    /// The run of a style the theme already names (what a derived theme
    /// drew with before).
    pub fn of(style: &TextStyle) -> Self {
        Self::new(style.font_size, style.line_height, style.font_weight)
    }
}

/// The breadcrumb: each crumb a small target with its own box, the
/// chevron between two crumbs, an ellipsis where leading crumbs fold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreadcrumbRecipe {
    pub text: NavText,
    /// A crumb's padding: inline (each side) and block (top and bottom).
    pub pad_x: f32,
    pub pad_y: f32,
    pub radius: f32,
    /// A crumb that leads somewhere, at rest, hovered and pressed.
    pub rest: Color,
    pub hover: Color,
    pub pressed: Color,
    /// The box behind a hovered and a pressed crumb (alpha 0 is none).
    pub hover_fill: Color,
    pub pressed_fill: Color,
    /// The last crumb: where the user is.
    pub current: Color,
    /// The folded crumbs' ellipsis and the chevrons.
    pub faint: Color,
    pub chevron: f32,
    /// Space on each side of a chevron.
    pub gap: f32,
}

/// The navigation slot of a theme (`theme.navigation`).
#[derive(Clone, Debug, PartialEq)]
pub struct NavigationTokens {
    pub breadcrumb: BreadcrumbRecipe,
}

fn tint(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], c.0[3] * a])
}

impl NavigationTokens {
    /// The hoff core kit, values as written in its class lists, and the
    /// derivations where it has no such component.
    pub fn hoff_core() -> Self {
        let [_paper, _shade2, wash, line, _, gray, _, _, ink] = SHADES;

        // No breadcrumb in the kit: a crumb is the home sidebar's link
        // text (`HomeSidebar/NavLink/index.tsx:13`, `text-[0.75rem]
        // font-semibold`, leading 1rem as in `GuidelineSidebar:69`), gray
        // `#7b7b7b` that turns ink on hover (`group-hover:fill-[#121212]`,
        // :28), in the link's hover box (`hover:bg-[#f1f1f1]`) at the
        // icon tile's radius (`rounded-lg`, :21) and `px-2 py-1`; pressed,
        // the box takes the active rim's tone (`border-[#ececec]`). The
        // chevron is the dropdown's (`HomeSidebar/Dropdown:58`, `size-4`)
        // in the group title's faint gray (`text-[#7b7b7b]/70`,
        // `GuidelineSidebar:59`), `gap-0.5` each side.
        let breadcrumb = BreadcrumbRecipe {
            text: NavText::new(12.0, 16.0, 600),
            pad_x: 8.0,
            pad_y: 4.0,
            radius: 8.0,
            rest: gray,
            hover: ink,
            pressed: ink,
            hover_fill: wash,
            pressed_fill: line,
            current: ink,
            faint: tint(gray, 0.7),
            chevron: 16.0,
            gap: 2.0,
        };

        Self { breadcrumb }
    }

    /// What every other theme drew these widgets with before the slot:
    /// the trail in base-2r, crumbs in the mid text tone that turn the
    /// glass's active text on hover, the last one in the text tone, the
    /// chevrons and the ellipsis faint, no box, the `xs` step around a
    /// small icon.
    pub fn derive(theme: &Theme) -> Self {
        let (colors, glass) = (&theme.colors, &theme.glass);
        let breadcrumb = BreadcrumbRecipe {
            text: NavText::of(&theme.typography.base_2r()),
            pad_x: 0.0,
            pad_y: 0.0,
            radius: theme.shape.micro,
            rest: colors.text_mid,
            hover: glass.text_active,
            pressed: glass.text_active,
            hover_fill: Color::TRANSPARENT,
            pressed_fill: Color::TRANSPARENT,
            current: colors.text,
            faint: glass.text_faint,
            chevron: theme.control.icon(IconSize::Sm),
            gap: theme.spacing.xs,
        };
        Self { breadcrumb }
    }
}
