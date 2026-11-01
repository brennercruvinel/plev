//! The type scale by role: one style per role of a text (display, the six
//! headings, title, heading, body, label, caption...), each a size, an
//! absolute line height, a weight and a tracking. A recipe keeps the role
//! ([`TypeRun`]), never the numbers, and resolves it against the theme's
//! [`TypographyScale`], so every component of every theme reads the same
//! scale.
//!
//! The core theme writes the roles from the kit's TypographyPage; every
//! other theme derives them from its own ramp ([`TypeRoles::derive`]).
//! The title roles (display to h3) are fluid: they scale with the width of
//! the container they are set in, between a minimum and their full size,
//! the `clamp()` of a css type scale (ADR 0073: ranges, not screens).

use crate::drawing::text::TextStyle;

use super::tokens::TypographyScale;

/// What a text is, not how big it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeRole {
    Display,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    /// The kit's `Title, Large`: the head of a sheet or a modal.
    TitleLg,
    Title,
    Heading,
    BodyLg,
    Body,
    BodyStrong,
    Label,
    Caption,
    Small,
    Paragraph,
    Code,
    /// Figures that line up in a column (a stat, a table cell, a clock).
    Numeric,
    /// A product's name in its logo.
    Wordmark,
}

impl TypeRole {
    pub const COUNT: usize = 20;
    /// Every role, largest first: the order of the Typography page.
    pub const ALL: [TypeRole; Self::COUNT] = [
        Self::Display,
        Self::H1,
        Self::H2,
        Self::H3,
        Self::H4,
        Self::H5,
        Self::H6,
        Self::TitleLg,
        Self::Title,
        Self::Heading,
        Self::BodyLg,
        Self::Body,
        Self::BodyStrong,
        Self::Label,
        Self::Caption,
        Self::Small,
        Self::Paragraph,
        Self::Code,
        Self::Numeric,
        Self::Wordmark,
    ];
    /// The roles that scale with their container: the title roles and
    /// h6, the showcase's group title (the kit's 20/28, 16/24 narrow).
    pub const FLUID: [TypeRole; 5] = [Self::Display, Self::H1, Self::H2, Self::H3, Self::H6];

    const fn index(self) -> usize {
        self as usize
    }

    /// The role's name as a design token (`body-lg`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Display => "display",
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::H5 => "h5",
            Self::H6 => "h6",
            Self::TitleLg => "title-lg",
            Self::Title => "title",
            Self::Heading => "heading",
            Self::BodyLg => "body-lg",
            Self::Body => "body",
            Self::BodyStrong => "body-strong",
            Self::Label => "label",
            Self::Caption => "caption",
            Self::Small => "small",
            Self::Paragraph => "paragraph",
            Self::Code => "code",
            Self::Numeric => "numeric",
            Self::Wordmark => "wordmark",
        }
    }

    /// The run of this role at its own weight.
    pub const fn run(self) -> TypeRun {
        TypeRun {
            role: self,
            weight: None,
        }
    }

    /// The run at regular weight (400).
    pub const fn regular(self) -> TypeRun {
        self.run().weighted(400)
    }

    /// The run at medium weight (500).
    pub const fn medium(self) -> TypeRun {
        self.run().weighted(500)
    }

    /// The run at semibold weight (600), the heaviest face every built-in
    /// family embeds.
    pub const fn strong(self) -> TypeRun {
        self.run().weighted(600)
    }
}

/// A text run of a recipe: its role, and the weight of the run where it
/// is not the role's own (the kit's `Title, Strong`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeRun {
    pub role: TypeRole,
    pub weight: Option<u16>,
}

impl TypeRun {
    const fn weighted(self, weight: u16) -> Self {
        Self {
            weight: Some(weight),
            ..self
        }
    }

    /// The one style the run is measured and drawn with, at full size.
    pub fn style(&self, t: &TypographyScale) -> TextStyle {
        t.run(*self)
    }

    /// The style in a container `width` wide (a fluid role scales).
    pub fn style_at(&self, t: &TypographyScale, width: f32) -> TextStyle {
        t.run_at(*self, width)
    }
}

impl From<TypeRole> for TypeRun {
    fn from(role: TypeRole) -> Self {
        role.run()
    }
}

/// One role's measures: size and line height in px, weight, tracking in
/// em of the size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoleSpec {
    pub size: f32,
    pub line_height: f32,
    pub weight: u16,
    pub tracking_em: f32,
}

impl RoleSpec {
    pub const fn new(size: f32, line_height: f32, weight: u16, tracking_em: f32) -> Self {
        Self {
            size,
            line_height,
            weight,
            tracking_em,
        }
    }
}

/// The scale of a theme: a spec per role, the smallest size of each fluid
/// role, and the container widths the fluid roles scale between.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeRoles {
    specs: [RoleSpec; TypeRole::COUNT],
    /// Size and line height of [`TypeRole::FLUID`] at `range[0]` and
    /// below; at `range[1]` and above they are the role's spec.
    fluid_min: [[f32; 2]; 5],
    pub range: [f32; 2],
}

/// The width a fluid role starts growing at (a phone held upright) and
/// the one it stops at (a laptop's content column).
pub const FLUID_RANGE: [f32; 2] = [360.0, 1280.0];
/// A derived fluid role's smallest size, as a fraction of its full one.
const FLUID_MIN: f32 = 0.75;
/// Line heights land on this grid (the kit's 4px).
const LINE_GRID: f32 = 4.0;

fn on_grid(px: f32) -> f32 {
    (px / LINE_GRID).round() * LINE_GRID
}

fn even(px: f32) -> f32 {
    (px / 2.0).round() * 2.0
}

impl TypeRoles {
    pub fn spec(&self, role: TypeRole) -> RoleSpec {
        self.specs[role.index()]
    }

    /// The role's size and line height in a container `width` wide: the
    /// spec for a fixed role, and for a fluid one a straight line from its
    /// minimum at `range[0]` to its spec at `range[1]`, clamped.
    pub fn spec_at(&self, role: TypeRole, width: f32) -> RoleSpec {
        let spec = self.spec(role);
        let Some(i) = TypeRole::FLUID.iter().position(|r| *r == role) else {
            return spec;
        };
        let [from, to] = self.range;
        let k = ((width - from) / (to - from).max(1.0)).clamp(0.0, 1.0);
        let [size, lh] = self.fluid_min[i];
        RoleSpec {
            size: size + (spec.size - size) * k,
            line_height: lh + (spec.line_height - lh) * k,
            ..spec
        }
    }

    /// The fluid minimum of a role, `None` for a fixed one.
    pub fn fluid_min(&self, role: TypeRole) -> Option<[f32; 2]> {
        let i = TypeRole::FLUID.iter().position(|r| *r == role)?;
        Some(self.fluid_min[i])
    }

    /// The hoff core kit's scale (templates/TypographyPage/index.tsx),
    /// line heights on its 4px grid. The UI roles track -0.02em, as the
    /// kit's body does (app/layout.tsx:108) and every component inherits;
    /// the headlines -0.03em, as the page writes them.
    pub fn hoff_core() -> Self {
        let ui = -0.02;
        let head = -0.03;
        let s = RoleSpec::new;
        Self {
            specs: [
                // display: the step above h1 on the 8 grid; the kit stops
                // at h1.
                s(56.0, 64.0, 400, head),
                // h1 48/115% (55.2 on the grid) to h6 20/28, :25-:70.
                s(48.0, 56.0, 400, head),
                s(40.0, 48.0, 400, head),
                s(32.0, 40.0, 400, head),
                s(28.0, 36.0, 400, head),
                s(24.0, 32.0, 500, head),
                s(20.0, 28.0, 500, head),
                // Title, Large 18/150% regular (:223), on the grid.
                s(18.0, 28.0, 400, ui),
                // Title 15/24 medium (:205).
                s(15.0, 24.0, 500, ui),
                // Heading 14/20 medium (:172).
                s(14.0, 20.0, 500, ui),
                // Body Large 13/16 (:148).
                s(13.0, 16.0, 400, ui),
                // Body Medium 12/16 at regular, and Strong (:121).
                s(12.0, 16.0, 400, ui),
                s(12.0, 16.0, 600, ui),
                // Body Medium (:112): the run the kit writes most.
                s(12.0, 16.0, 500, ui),
                // Body Small 11/16 medium (:103).
                s(11.0, 16.0, 500, ui),
                // Body X Small 10/16 (:94).
                s(10.0, 16.0, 400, ui),
                // Paragraph small 13/150% (:247), on the grid.
                s(13.0, 20.0, 400, ui),
                // code: the paragraph's measure in the mono face.
                s(13.0, 20.0, 400, 0.0),
                // numeric: the label's.
                s(12.0, 16.0, 500, ui),
                // wordmark: the logo's 20/20 at the heaviest embedded face.
                s(20.0, 20.0, 600, ui),
            ],
            // h6 narrow: the kit's Group title below 1420 (Group/index.tsx:9).
            fluid_min: [
                [40.0, 48.0],
                [32.0, 40.0],
                [28.0, 36.0],
                [24.0, 32.0],
                [16.0, 24.0],
            ],
            range: FLUID_RANGE,
        }
    }

    /// A theme's scale from its ramp (`small, caption, body_sm, body,
    /// title_sm, title, display`), at `tracking_em`: each role takes the
    /// ramp step it stands for, the core weights and a line height from
    /// its own ratio on the 4px grid. h1 is the ramp's title and h6 its
    /// small title, h2 to h5 the geometric steps between them.
    pub fn derive(ramp: [f32; 7], tracking_em: f32) -> Self {
        let [small, caption, body_sm, body, title_sm, title, display] = ramp;
        let s = |size: f32, ratio: f32, weight: u16| {
            RoleSpec::new(size, on_grid(size * ratio).max(size), weight, tracking_em)
        };
        let step = (title_sm / title).powf(1.0 / 5.0);
        let h = |n: i32| {
            s(
                even(title * step.powi(n)),
                1.3,
                if n < 4 { 400 } else { 500 },
            )
        };
        let specs = [
            s(display, 1.1, 400),
            h(0),
            h(1),
            h(2),
            h(3),
            h(4),
            h(5),
            // title-lg: the small title at regular, between h6 and title.
            s(title_sm, 1.4, 400),
            s(title_sm, 1.2, 500),
            s(body, 1.43, 500),
            s(body, 1.5, 400),
            s(body_sm, 1.43, 400),
            s(body_sm, 1.43, 600),
            s(caption, 1.33, 500),
            s(caption, 1.33, 400),
            s(small, 1.4, 400),
            s(body_sm, 1.6, 400),
            RoleSpec::new(body_sm, on_grid(body_sm * 1.43), 400, 0.0),
            s(caption, 1.33, 500),
            RoleSpec::new(title_sm, title_sm, 600, tracking_em),
        ];
        let min = |spec: RoleSpec| {
            let size = even(spec.size * FLUID_MIN);
            [size, on_grid(size * spec.line_height / spec.size)]
        };
        Self {
            fluid_min: [
                min(specs[0]),
                min(specs[1]),
                min(specs[2]),
                min(specs[3]),
                min(specs[6]),
            ],
            specs,
            range: FLUID_RANGE,
        }
    }
}

impl TypographyScale {
    /// The role's measures at full size.
    pub fn spec(&self, role: TypeRole) -> RoleSpec {
        self.roles.spec(role)
    }

    /// The style of a role at full size, in this scale's family (the mono
    /// family for [`TypeRole::Code`]).
    pub fn role(&self, role: TypeRole) -> TextStyle {
        self.run(role.run())
    }

    /// The style of a run at full size.
    pub fn run(&self, run: TypeRun) -> TextStyle {
        self.styled(run, self.roles.spec(run.role))
    }

    /// The style of a role in a container `width` wide.
    pub fn role_at(&self, role: TypeRole, width: f32) -> TextStyle {
        self.run_at(role.run(), width)
    }

    /// The style of a run in a container `width` wide.
    pub fn run_at(&self, run: TypeRun, width: f32) -> TextStyle {
        self.styled(run, self.roles.spec_at(run.role, width))
    }

    fn styled(&self, run: TypeRun, spec: RoleSpec) -> TextStyle {
        let weight = run.weight.unwrap_or(spec.weight);
        if run.role == TypeRole::Code {
            return self.exact_mono(spec.size, spec.line_height, weight);
        }
        let style = self.exact(spec.size, spec.line_height, weight);
        // the tracking is in em: of the size drawn, the text scale's
        let tracking = spec.tracking_em * style.font_size;
        style.with_letter_spacing(tracking)
    }
}
