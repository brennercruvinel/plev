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
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::tokens::TypographyScale;
use crate::styling::theme::{ControlSize, IconSize, Theme};
use crate::widgets::content::DisclosureTile;

mod data;
pub use data::{CodeRecipe, StatLevel, StatRecipe};

/// A text run: size, line height and weight in px, in the theme's family;
/// tracked as the theme tracks unless the run names its own (in em).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavText {
    pub size: f32,
    pub line_height: f32,
    pub weight: u16,
    pub tracking_em: Option<f32>,
}

impl NavText {
    pub const fn new(size: f32, line_height: f32, weight: u16) -> Self {
        Self {
            size,
            line_height,
            weight,
            tracking_em: None,
        }
    }

    /// The run with its own tracking (the kit's `tracking-[-0.03em]`).
    pub const fn tracked(self, em: f32) -> Self {
        Self {
            tracking_em: Some(em),
            ..self
        }
    }

    /// The one style the run is measured and drawn with.
    pub fn style(&self, typography: &TypographyScale) -> TextStyle {
        let style = typography.exact(self.size, self.line_height, self.weight);
        match self.tracking_em {
            Some(em) => style.with_letter_spacing(em * self.size),
            None => style,
        }
    }

    /// The run of a style the theme already names (what a derived theme
    /// drew with before), at the theme's tracking.
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

/// A link's row in one state: its box (fill and rim, alpha 0 is none)
/// and the tones of its label and its icon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkSurface {
    pub fill: Color,
    pub border: Color,
    pub label: Color,
    pub icon: Color,
}

/// A navigation link: the row, its states, the icon in a slot (or on a
/// tile that rises while the link is current), the label and the
/// trailing hint. Paddings include the rim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavLinkRecipe {
    pub height: f32,
    pub pad_start: f32,
    pub pad_end: f32,
    pub radius: f32,
    pub border_width: f32,
    pub rest: LinkSurface,
    pub hover: LinkSurface,
    pub pressed: LinkSurface,
    /// The link to where the user is (`active`).
    pub active: LinkSurface,
    /// The icon's box: a tile with its own face (the kit), or a bare
    /// slot of `slot` px (`None`). With a tile the link reads the tile's
    /// geometry, its raised fill and its shadow; the icon's tones are the
    /// surfaces'.
    pub tile: Option<DisclosureTile>,
    pub slot: f32,
    pub icon: f32,
    /// Between the icon's box and the label.
    pub icon_gap: f32,
    pub label: NavText,
    pub hint: NavText,
    pub hint_color: Color,
    /// Between the label and a hint or badge, and what the preferred
    /// width keeps for that space.
    pub trailing_gap: f32,
    pub trailing_room: f32,
    /// Narrower than the icon's box, the paddings and this, the row
    /// draws its rail form (the icon alone).
    pub label_min: f32,
    /// Between two links of a list.
    pub link_gap: f32,
}

impl NavLinkRecipe {
    /// The icon's box edge: the tile's, or the bare slot's.
    pub fn icon_box(&self) -> f32 {
        self.tile.map_or(self.slot, |t| t.size)
    }
}

/// The navigation slot of a theme (`theme.navigation`).
#[derive(Clone, Debug, PartialEq)]
pub struct NavigationTokens {
    pub breadcrumb: BreadcrumbRecipe,
    /// The navigation menu's link (`NavLink` on its own).
    pub link: NavLinkRecipe,
    /// The link as the sidebar draws it: the look of before in every
    /// theme until the sidebar moves to the kit.
    pub sidebar_link: NavLinkRecipe,
    pub stat: StatRecipe,
    pub code: CodeRecipe,
}

fn tint(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], c.0[3] * a])
}

impl NavigationTokens {
    /// The hoff core kit, values as written in its class lists, and the
    /// derivations where it has no such component.
    pub fn hoff_core() -> Self {
        let [paper, _shade2, wash, line, _, gray, _, _, ink] = SHADES;

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

        // HomeSidebar/NavLink/index.tsx: the row `p-0.75 rounded-xl
        // text-[0.75rem] font-semibold border` (:13), `hover:bg-[#f1f1f1]`,
        // `bg-[#f1f1f1] border-[#ececec]` while active, `border-
        // transparent` otherwise (:14-16); the tile `size-8 mr-3
        // rounded-lg`, raised `bg-[#fcfcfc] shadow-[0_0_4px_0_rgba(18,18,
        // 18,0.10)]` while active (:21-24); the icon `20` in `#7b7b7b`,
        // `#121212` hovered or active (:27-31); the label in the body's
        // ink, leading 1rem (`GuidelineSidebar:69`). The row is 3 + 32 + 3
        // inside a 1 px rim: 40. Derived where the kit link has nothing:
        // the trailing pad is the dropdown's `pr-3` (`Dropdown:25`), the
        // hint the group title's text (`text-[0.75rem] leading-[1rem]
        // font-medium text-[#7b7b7b]/70`, `GuidelineSidebar:59`), a press
        // the active rim's tone as a fill.
        let surface = |fill, border, icon| LinkSurface {
            fill,
            border,
            label: ink,
            icon,
        };
        let none = Color::TRANSPARENT;
        let link = NavLinkRecipe {
            height: 40.0,
            pad_start: 4.0,
            pad_end: 12.0,
            radius: 12.0,
            border_width: 1.0,
            rest: surface(none, none, gray),
            hover: surface(wash, none, ink),
            pressed: surface(line, none, ink),
            active: surface(wash, line, ink),
            tile: Some(DisclosureTile {
                size: 32.0,
                radius: 8.0,
                gap: 12.0,
                icon: 20.0,
                fill_current: paper,
                shadow_current: ShadowSpec::new([0.0, 0.0], 4.0, 0.0, tint(ink, 0.10)),
                icon_rest: gray,
                icon_current: ink,
            }),
            slot: 32.0,
            icon: 20.0,
            icon_gap: 12.0,
            label: NavText::new(12.0, 16.0, 600),
            hint: NavText::new(12.0, 16.0, 500),
            hint_color: tint(gray, 0.7),
            trailing_gap: 8.0,
            trailing_room: 20.0,
            label_min: 32.0,
            // The guideline sidebar's list, `flex flex-col gap-0.5`
            // (`GuidelineSidebar:62`).
            link_gap: 2.0,
        };

        // The sidebar is the gallery's chrome: it keeps the look of
        // before until it moves to the kit as a whole (task-104 step 4).
        // That look is read from the finished core theme, which
        // `Theme::hoff_core` writes here last (`sidebar_link_of_before`);
        // the kit link stands in until then.
        Self {
            breadcrumb,
            link,
            sidebar_link: link,
            stat: StatRecipe::hoff_core(),
            code: CodeRecipe::hoff_core(),
        }
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
        let link = Self::sidebar_link_of_before(theme);
        Self {
            breadcrumb,
            link,
            sidebar_link: link,
            stat: StatRecipe::derive(theme),
            code: CodeRecipe::derive(theme),
        }
    }

    /// The link of before, in `theme`: one `Lg` control at the nav
    /// radius, the bare slot of an `Xs` control with the `Md` icon, a
    /// base-2sm label; faint at rest, the default text hovered, the
    /// active text on the glass's active wash inside its edge while
    /// active. Every derived link, and the sidebar's in every theme.
    pub(crate) fn sidebar_link_of_before(theme: &Theme) -> NavLinkRecipe {
        let (glass, control) = (&theme.glass, &theme.control);
        let none = Color::TRANSPARENT;
        let tone = |c: Color| LinkSurface {
            fill: none,
            border: none,
            label: c,
            icon: c,
        };
        NavLinkRecipe {
            height: control.height(ControlSize::Lg),
            pad_start: theme.spacing.sm,
            pad_end: theme.spacing.sm,
            radius: theme.shape.nav,
            border_width: control.edge_width,
            rest: tone(glass.text_faint),
            hover: tone(glass.text_default),
            pressed: tone(glass.text_default),
            active: LinkSurface {
                fill: glass.surface_active,
                border: glass.edge,
                ..tone(glass.text_active)
            },
            tile: None,
            slot: control.height(ControlSize::Xs),
            icon: control.icon(IconSize::Md),
            icon_gap: theme.spacing.xs,
            label: NavText::of(&theme.typography.base_2sm()),
            hint: NavText::of(&theme.typography.caption_r()),
            hint_color: glass.text_placeholder,
            trailing_gap: theme.spacing.sm,
            trailing_room: theme.spacing.lg,
            label_min: theme.spacing.xl,
            link_gap: theme.spacing.xs,
        }
    }
}
