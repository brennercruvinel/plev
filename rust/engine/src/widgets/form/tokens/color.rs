//! The color picker's tokens (`super::super::color_picker`): the kit's
//! ColorPicker (`components/ColorPicker/index.tsx`), a box with a swatch,
//! the hex typed beside it and the opacity in percent at its end, behind
//! a divider. The two texts are bare text fields ([`super::Look::Bare`]):
//! their text, its color and the caret are the bare box's.

use crate::styling::color::Color;
use crate::styling::theme::{ControlSize, Theme};

use super::InputSurface;

/// The box, the swatch and the opacity's part.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorTokens {
    pub rest: InputSurface,
    pub hover: InputSurface,
    /// While one of its texts has the caret.
    pub focus: InputSurface,
    /// Between the border and what is inside (`p-1`).
    pub pad: f32,
    pub radius: f32,
    pub border_width: f32,
    /// The swatch's side (`size-7`), its radius (`rounded-md`) and the
    /// room after it (`mr-3`).
    pub swatch: f32,
    pub swatch_radius: f32,
    pub swatch_gap: f32,
    /// What the color is painted over, its opacity showing it
    /// (`bg-[#FCFCFC]`), and the swatch's border (`border-[#323232]/10`).
    pub swatch_base: Color,
    pub swatch_border: Color,
    /// The opacity's part: its width (`w-16`; 0, as wide as `100 %`
    /// measures), its inset on either side (`px-3`), the room between
    /// the number and its `%` (`gap-2`) and the divider before it
    /// (`border-l border-[#323232]/10`).
    pub alpha_w: f32,
    pub alpha_pad: f32,
    pub alpha_gap: f32,
    pub divider: Color,
    /// A picker's width where the composition gives it one
    /// (`templates/InputsPage/index.tsx:169`, `w-54.5`); the `Rect` it
    /// gets always wins.
    pub width: f32,
}

/// `#323232` at `a`, the kit's line over the box.
fn line(a: f32) -> Color {
    Color::rgba(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, a)
}

impl ColorTokens {
    /// The kit as written (ColorPicker :13, the box; :17, the swatch;
    /// :24, the opacity's part).
    pub fn hoff_core() -> Self {
        let surface = |fill: u32, border: u32| InputSurface {
            fill: Color::hex(fill),
            border: Color::hex(border),
            shadow: Vec::new(),
        };
        let hover = surface(0xF8F7F7, 0xE2E2E2);
        Self {
            rest: surface(0xF1F1F1, 0xECECEC),
            // Not in the kit (`outline-0`): the hover's, so the box shows
            // where the caret went.
            focus: hover.clone(),
            hover,
            pad: 4.0,
            radius: 10.0,
            border_width: 1.0,
            swatch: 28.0,
            swatch_radius: 6.0,
            swatch_gap: 12.0,
            swatch_base: Color::hex(0xFCFCFC),
            swatch_border: line(0.10),
            alpha_w: 64.0,
            alpha_pad: 12.0,
            alpha_gap: 8.0,
            divider: line(0.10),
            width: 218.0,
        }
    }

    /// The theme's field, as the input group derives its own: the glass
    /// field, its soft edge turning the full edge hovered and
    /// `field_focus_border` focused (the fill stays, as `surface_active`
    /// would take the text under AAA in hoff and dark), the swatch filling one `Md`
    /// control's height less the padding, the theme's spaces between the
    /// parts and the opacity as wide as it measures.
    pub fn derive(theme: &Theme) -> Self {
        let g = &theme.glass;
        let s = &theme.spacing;
        let surface = |fill: Color, border: Color| InputSurface {
            fill,
            border,
            shadow: Vec::new(),
        };
        let pad = s.xs;
        let border_width = theme.control.edge_width;
        let swatch = theme.control.height(ControlSize::Md) - (pad + border_width) * 2.0;
        Self {
            rest: surface(g.field, g.edge_soft),
            hover: surface(g.field, g.edge),
            focus: surface(g.field, g.field_focus_border),
            pad,
            radius: theme.shape.nav,
            border_width,
            swatch,
            swatch_radius: theme.shape.micro,
            swatch_gap: s.md,
            swatch_base: g.surface,
            swatch_border: g.edge_soft,
            alpha_w: 0.0,
            alpha_pad: s.md,
            alpha_gap: s.sm,
            divider: g.edge_soft,
            width: theme.size.menu_w,
        }
    }
}
