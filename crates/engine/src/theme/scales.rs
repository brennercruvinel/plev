// ============================================================================
// Scales the widgets used to improvise: shape (radius by role), control
// (heights, paddings, icon sizes, rims), size (container widths), duration,
// layout (breakpoints, gutters, density) and elevation (shadow stacks).
//
// Every number here was a literal somewhere in a widget, an app theme or a
// showcase section (docs/catalog.md, section 3). A widget never restates a
// value from this file; it reads it from `&Theme`.
// ============================================================================

use crate::color::Color;

// ----------------------------------------------------------------------------
// Shape: corner radius by role
// ----------------------------------------------------------------------------

/// Corner radius per role, the HOFF radii table (`styles/variables.sass`).
/// A [`RadiusScale`](super::RadiusScale) is a numeric ladder; this is what
/// a widget actually asks for ("the card radius"), so a non-HOFF theme can
/// remap roles onto its own ladder without every widget knowing.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeTokens {
    /// Pills, buttons, modal sheet, message composer.
    pub pill: f32,
    /// Actions dropdown body, post images.
    pub dropdown: f32,
    /// Tabs container, search, select.
    pub tabs: f32,
    /// Every list card.
    pub card: f32,
    /// Active tab block, counter pill.
    pub block: f32,
    /// Toast, dropdown item, reaction chip.
    pub item: f32,
    /// Nav link, field, switch.
    pub nav: f32,
    /// Message action cluster.
    pub cluster: f32,
    /// Tooltip, tag, topic thumb.
    pub tooltip: f32,
    /// 24px micro action, checkbox.
    pub micro: f32,
}

impl ShapeTokens {
    /// The HOFF table: 32 / 24 / 22 / 20 / 18 / 16 / 12 / 10 / 8 / 6.
    pub fn hoff() -> Self {
        Self {
            pill: 32.0,
            dropdown: 24.0,
            tabs: 22.0,
            card: 20.0,
            block: 18.0,
            item: 16.0,
            nav: 12.0,
            cluster: 10.0,
            tooltip: 8.0,
            micro: 6.0,
        }
    }

    /// Derive the roles from a numeric ladder, so palettes that only
    /// declare `none..full` still give every widget a radius. The tabs
    /// container wraps the active block with the tabs padding, which is
    /// where the 22 = 18 + 4 relation comes from.
    pub fn derive(radius: &super::RadiusScale, tabs_pad: f32) -> Self {
        Self {
            pill: radius.xl,
            dropdown: (radius.lg + radius.xl) / 2.0,
            tabs: radius.lg + tabs_pad,
            card: radius.lg,
            block: radius.lg,
            item: radius.md + radius.sm / 2.0,
            nav: radius.md,
            cluster: radius.md - radius.sm / 4.0,
            tooltip: radius.sm,
            micro: radius.sm * 0.75,
        }
    }
}

// ----------------------------------------------------------------------------
// Controls: heights, paddings, icons, rims, focus ring
// ----------------------------------------------------------------------------

/// The five control heights every interactive widget picks from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ControlSize {
    /// Tab button, micro action row.
    Xs,
    /// Social chip.
    Sm,
    /// Standard pill button, menu item, avatar.
    #[default]
    Md,
    /// Nav link.
    Lg,
    /// Medium pill (the HOFF "button-medium").
    Xl,
}

impl ControlSize {
    pub const ALL: [ControlSize; 5] = [
        ControlSize::Xs,
        ControlSize::Sm,
        ControlSize::Md,
        ControlSize::Lg,
        ControlSize::Xl,
    ];

    fn index(self) -> usize {
        match self {
            ControlSize::Xs => 0,
            ControlSize::Sm => 1,
            ControlSize::Md => 2,
            ControlSize::Lg => 3,
            ControlSize::Xl => 4,
        }
    }
}

/// Icon sizes, indexed by the same ladder as [`ControlSize`] where a
/// control carries an icon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IconSize {
    Sm,
    #[default]
    Md,
    Lg,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlTokens {
    /// Heights for Xs..Xl (HOFF: 36 / 40 / 44 / 48 / 52).
    pub heights: [f32; 5],
    /// Horizontal padding for Xs..Xl (HOFF: 16 / 12 / 24 / 16 / 32).
    pub pad_x: [f32; 5],
    /// Icon edge for Sm / Md / Lg (HOFF: 16 / 20 / 24).
    pub icons: [f32; 3],
    /// Smallest hit target on touch (WCAG 2.5.8 / apple HIG: 44).
    pub min_touch_target: f32,
    /// Edge-light rim width on cards and modals.
    pub edge_width: f32,
    /// Edge-light rim width on buttons and active nav items.
    pub edge_width_strong: f32,
    /// Keyboard focus ring stroke.
    pub focus_ring_width: f32,
    /// Gap between a control's edge and its focus ring.
    pub focus_ring_offset: f32,
    /// Padding inside the tabs container around the active block.
    pub tabs_pad: f32,
    /// Padding inside a floating menu around its items.
    pub menu_pad: f32,
    /// Gap between a control and the label or icon beside it.
    pub inline_gap: f32,
    /// Checkbox box edge (HOFF: 18).
    pub box_size: f32,
    /// Switch track width and height (HOFF: 44 by 24).
    pub switch_size: [f32; 2],
    /// Switch knob diameter (HOFF: 16; the pad is derived from the track).
    pub switch_knob: f32,
    /// Slider track thickness and knob diameter (HOFF: 4 and 14).
    pub slider_track: f32,
    pub slider_knob: f32,
    /// Scrollbar thumb width at rest and while hovered or dragged, its
    /// shortest length, and its inset from the region edge.
    pub scrollbar_w: f32,
    pub scrollbar_w_hover: f32,
    pub scrollbar_min_thumb: f32,
    pub scrollbar_inset: f32,
    /// Spinner stroke width.
    pub spinner_stroke: f32,
    /// Progress bar track height and the fill bar inside it.
    pub progress_track: f32,
    pub progress_fill: f32,
    /// Split-pane divider hit width and its visible line width.
    pub divider_hit: f32,
    pub divider_line: f32,
}

impl ControlTokens {
    pub fn hoff() -> Self {
        Self {
            heights: [36.0, 40.0, 44.0, 48.0, 52.0],
            pad_x: [16.0, 12.0, 24.0, 16.0, 32.0],
            icons: [16.0, 20.0, 24.0],
            min_touch_target: 44.0,
            edge_width: 1.0,
            edge_width_strong: 1.5,
            focus_ring_width: 2.0,
            focus_ring_offset: 2.0,
            tabs_pad: 4.0,
            menu_pad: 8.0,
            inline_gap: 8.0,
            box_size: 18.0,
            switch_size: [44.0, 24.0],
            switch_knob: 16.0,
            slider_track: 4.0,
            slider_knob: 14.0,
            scrollbar_w: 6.0,
            scrollbar_w_hover: 10.0,
            scrollbar_min_thumb: 24.0,
            scrollbar_inset: 2.0,
            spinner_stroke: 2.0,
            progress_track: 12.0,
            progress_fill: 4.0,
            divider_hit: 10.0,
            divider_line: 2.0,
        }
    }

    /// Switch knob inset from the track edge, derived so the knob always
    /// sits centered in the track height.
    pub fn switch_knob_pad(&self) -> f32 {
        ((self.switch_size[1] - self.switch_knob) / 2.0).max(0.0)
    }

    pub fn height(&self, size: ControlSize) -> f32 {
        self.heights[size.index()]
    }

    pub fn pad_x(&self, size: ControlSize) -> f32 {
        self.pad_x[size.index()]
    }

    pub fn icon(&self, size: IconSize) -> f32 {
        match size {
            IconSize::Sm => self.icons[0],
            IconSize::Md => self.icons[1],
            IconSize::Lg => self.icons[2],
        }
    }

    /// The icon that fits a control of `size`: Sm for Xs/Sm controls, Md
    /// for Md/Lg, Lg for Xl.
    pub fn icon_for(&self, size: ControlSize) -> f32 {
        match size {
            ControlSize::Xs | ControlSize::Sm => self.icon(IconSize::Sm),
            ControlSize::Md | ControlSize::Lg => self.icon(IconSize::Md),
            ControlSize::Xl => self.icon(IconSize::Lg),
        }
    }

    /// Height clamped to the touch minimum, for `Density::Comfortable`
    /// on a touch surface.
    pub fn touch_height(&self, size: ControlSize) -> f32 {
        self.height(size).max(self.min_touch_target)
    }
}

// ----------------------------------------------------------------------------
// Sizes: container widths
// ----------------------------------------------------------------------------

/// Widths of the containers the design fixes rather than derives. Only
/// as min / max: what a container actually takes comes from the space it
/// is given (rul-05, content-driven-layout-not-fixed-constants).
#[derive(Clone, Debug, PartialEq)]
pub struct SizeTokens {
    /// Floating menu body (HOFF Actions: 240).
    pub menu_w: f32,
    /// Modal sheet max width (HOFF Modal: 400).
    pub modal_max_w: f32,
    /// Expanded sidebar (HOFF Sidebar: 248).
    pub sidebar_w: f32,
    /// Collapsed sidebar rail: one nav icon plus its padding.
    pub sidebar_rail_w: f32,
    /// Default card width outside a grid (HOFF deck: 368) and the
    /// narrowest a grid column may shrink one to.
    pub card_w: f32,
    pub card_min_w: f32,
    /// Widest a column of body text should grow.
    pub readable_max_w: f32,
    /// Narrowest useful text field.
    pub field_min_w: f32,
    /// Toast width.
    pub toast_w: f32,
    /// Tooltip max width before wrapping.
    pub tooltip_max_w: f32,
    /// Tallest a select's options panel grows before it scrolls.
    pub dropdown_max_h: f32,
}

impl SizeTokens {
    pub fn hoff() -> Self {
        Self {
            menu_w: 240.0,
            modal_max_w: 400.0,
            sidebar_w: 248.0,
            sidebar_rail_w: 72.0,
            card_w: 368.0,
            card_min_w: 280.0,
            readable_max_w: 720.0,
            field_min_w: 160.0,
            toast_w: 320.0,
            tooltip_max_w: 240.0,
            dropdown_max_h: 320.0,
        }
    }
}

// ----------------------------------------------------------------------------
// Durations
// ----------------------------------------------------------------------------

/// Tween durations in seconds, for the transitions that are timed rather
/// than sprung (fades, slides). Springs keep using
/// [`MotionPhysics`](super::MotionPhysics).
#[derive(Clone, Debug, PartialEq)]
pub struct DurationScale {
    /// Hover tint, focus ring.
    pub fast: f32,
    /// The HOFF global `transition .2s`.
    pub base: f32,
    /// Tabs slide, sidebar collapse, modal fade (`.3s`).
    pub slow: f32,
}

impl DurationScale {
    pub fn hoff() -> Self {
        Self {
            fast: 0.1,
            base: 0.2,
            slow: 0.3,
        }
    }
}

// ----------------------------------------------------------------------------
// Layout: breakpoints, gutters, density
// ----------------------------------------------------------------------------

/// Viewport class, from the logical width. Names follow the material
/// window size classes; the thresholds are tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Breakpoint {
    /// Phone portrait. Single column, sidebar as a drawer.
    Compact,
    /// Phone landscape, small tablet, narrow desktop window. Sidebar as
    /// a rail.
    Medium,
    /// Tablet landscape, desktop. Full sidebar.
    Expanded,
}

impl Breakpoint {
    pub const ALL: [Breakpoint; 3] = [
        Breakpoint::Compact,
        Breakpoint::Medium,
        Breakpoint::Expanded,
    ];
}

/// How the app sidebar shows at a breakpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SidebarMode {
    /// Hidden until toggled; slides over the content.
    Drawer,
    /// Icons only, [`SizeTokens::sidebar_rail_w`] wide.
    Rail,
    /// Icons and labels, [`SizeTokens::sidebar_w`] wide.
    Full,
}

/// Spacing multiplier. Touch surfaces want Comfortable; dense desktop
/// tools may pick Compact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

impl Density {
    /// Factor applied to spacing and control heights.
    pub fn factor(self) -> f32 {
        match self {
            Density::Comfortable => 1.0,
            Density::Compact => 0.85,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutTokens {
    /// Widths below this are Compact.
    pub compact_max_w: f32,
    /// Widths below this (and at least `compact_max_w`) are Medium.
    pub medium_max_w: f32,
    /// Page gutter per breakpoint (Compact / Medium / Expanded).
    pub gutters: [f32; 3],
    /// Grid columns per breakpoint.
    pub columns: [u8; 3],
    /// Gap between grid cells.
    pub grid_gap: f32,
}

impl LayoutTokens {
    pub fn hoff() -> Self {
        Self {
            compact_max_w: 600.0,
            medium_max_w: 1024.0,
            gutters: [16.0, 24.0, 40.0],
            columns: [4, 8, 12],
            grid_gap: 16.0,
        }
    }

    pub fn breakpoint(&self, width: f32) -> Breakpoint {
        if width < self.compact_max_w {
            Breakpoint::Compact
        } else if width < self.medium_max_w {
            Breakpoint::Medium
        } else {
            Breakpoint::Expanded
        }
    }

    fn index(bp: Breakpoint) -> usize {
        match bp {
            Breakpoint::Compact => 0,
            Breakpoint::Medium => 1,
            Breakpoint::Expanded => 2,
        }
    }

    pub fn gutter(&self, bp: Breakpoint) -> f32 {
        self.gutters[Self::index(bp)]
    }

    pub fn columns(&self, bp: Breakpoint) -> u8 {
        self.columns[Self::index(bp)]
    }

    pub fn sidebar_mode(&self, bp: Breakpoint) -> SidebarMode {
        match bp {
            Breakpoint::Compact => SidebarMode::Drawer,
            Breakpoint::Medium => SidebarMode::Rail,
            Breakpoint::Expanded => SidebarMode::Full,
        }
    }

    /// Column count for a grid of cells at least `min_w` wide inside
    /// `content_w`, the formula every gallery grid already used
    /// (`floor((w + gap) / (min + gap)).max(1)`).
    pub fn fit_columns(&self, content_w: f32, min_w: f32) -> usize {
        (((content_w + self.grid_gap) / (min_w + self.grid_gap)).floor() as usize).max(1)
    }
}

// ----------------------------------------------------------------------------
// Elevation: shadow stacks
// ----------------------------------------------------------------------------

/// One css `box-shadow` layer. `spread` grows or shrinks the casting rect
/// (the compositor's analytic shadow has no spread of its own).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowSpec {
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl ShadowSpec {
    pub const fn new(offset: [f32; 2], blur: f32, spread: f32, color: Color) -> Self {
        Self {
            offset,
            blur,
            spread,
            color,
        }
    }

    /// The rect a compositor shadow node casts for a surface of `w` by
    /// `h` at `(x, y)`, with spread folded in. `None` when spread ate the
    /// whole surface (nothing to draw).
    pub fn cast_rect(&self, x: f32, y: f32, w: f32, h: f32) -> Option<(f32, f32, f32, f32)> {
        let s = self.spread;
        let (w2, h2) = (w + 2.0 * s, h + 2.0 * s);
        if w2 <= 0.0 || h2 <= 0.0 {
            return None;
        }
        Some((x - s, y - s, w2, h2))
    }
}

/// Which shadow stack a surface gets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Elevation {
    /// Flat: cards on the page, no shadow.
    #[default]
    Base,
    /// Active tab block.
    Raised,
    /// Floating menu, dropdown.
    Floating,
    /// Modal sheet, options panel.
    Overlay,
    /// Tooltip.
    Hint,
}

/// The HOFF shadow stacks (Modal.module.sass, Actions, Tabs, Tooltip).
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowTokens {
    /// Active tabs block: `0 8px 16px -4px rgba(18,18,18,.20)`.
    pub raised: ShadowSpec,
    /// Floating menu: `0 24px 32px -12px rgba(18,18,18,.10)`.
    pub floating: ShadowSpec,
    /// Deep stack under modal / nav / options, four layers.
    pub overlay: [ShadowSpec; 4],
    /// Tooltip: `0 1.5px 2px rgba(24,24,24,.15)`.
    pub hint: ShadowSpec,
    /// Deck card lift: `0 32px 24px -16px rgba(0,0,0,.40)`.
    pub card: ShadowSpec,
}

const fn shade(v: f32, alpha: f32) -> Color {
    Color::rgba(v / 255.0, v / 255.0, v / 255.0, alpha)
}

impl ShadowTokens {
    pub fn hoff() -> Self {
        Self {
            raised: ShadowSpec::new([0.0, 8.0], 16.0, -4.0, shade(18.0, 0.20)),
            floating: ShadowSpec::new([0.0, 24.0], 32.0, -12.0, shade(18.0, 0.10)),
            overlay: [
                ShadowSpec::new([0.0, 24.0], 24.0, -16.0, shade(5.0, 0.09)),
                ShadowSpec::new([0.0, 6.0], 13.0, 0.0, shade(5.0, 0.10)),
                ShadowSpec::new([0.0, 6.0], 4.0, -4.0, shade(5.0, 0.10)),
                ShadowSpec::new([0.0, 5.0], 2.0, -4.0, shade(5.0, 0.25)),
            ],
            hint: ShadowSpec::new([0.0, 1.5], 2.0, 0.0, shade(24.0, 0.15)),
            card: ShadowSpec::new([0.0, 32.0], 24.0, -16.0, shade(0.0, 0.40)),
        }
    }

    /// Derive a stack family from a single shadow color, for palettes
    /// that only declare `effects.shadow_color`.
    pub fn derive(color: Color) -> Self {
        let tint = |a: f32| Color([color.0[0], color.0[1], color.0[2], a]);
        Self {
            raised: ShadowSpec::new([0.0, 8.0], 16.0, -4.0, tint(0.20)),
            floating: ShadowSpec::new([0.0, 24.0], 32.0, -12.0, tint(0.25)),
            overlay: [
                ShadowSpec::new([0.0, 24.0], 24.0, -16.0, tint(0.20)),
                ShadowSpec::new([0.0, 6.0], 13.0, 0.0, tint(0.20)),
                ShadowSpec::new([0.0, 6.0], 4.0, -4.0, tint(0.20)),
                ShadowSpec::new([0.0, 5.0], 2.0, -4.0, tint(0.35)),
            ],
            hint: ShadowSpec::new([0.0, 1.5], 2.0, 0.0, tint(0.30)),
            card: ShadowSpec::new([0.0, 32.0], 24.0, -16.0, tint(0.40)),
        }
    }

    /// The layers a surface at `elevation` casts, bottom first.
    pub fn stack(&self, elevation: Elevation) -> &[ShadowSpec] {
        match elevation {
            Elevation::Base => &[],
            Elevation::Raised => std::slice::from_ref(&self.raised),
            Elevation::Floating => std::slice::from_ref(&self.floating),
            Elevation::Overlay => &self.overlay,
            Elevation::Hint => std::slice::from_ref(&self.hint),
        }
    }
}
