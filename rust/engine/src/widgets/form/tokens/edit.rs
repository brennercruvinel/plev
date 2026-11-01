//! The inline edit's tokens (`super::super::new_field`): the kit's
//! NewField (`components/NewField/index.tsx`), a value with a pencil that
//! turns into a bare box with a confirm. The box is a text field drawn
//! with nothing around its text ([`super::Look::Bare`]).

use crate::styling::color::Color;
use crate::styling::theme::{IconSize, Theme, TypeRole, TypeRun};

use super::{InputRecipe, InputSurface};

/// The value, its pencil, and the bare box with its confirm.
#[derive(Clone, Debug, PartialEq)]
pub struct EditTokens {
    /// The value and the box's text (`text-[0.75rem] font-medium`).
    pub text: TypeRun,
    pub color: Color,
    /// Between the value or the box and its icon (`gap-2`).
    pub gap: f32,
    /// The pencil and the confirm (`size-4`).
    pub icon: f32,
    pub pencil_color: Color,
    /// The confirm with a value to save (`fill-[#49BA61]`), and empty.
    pub confirm: Color,
    pub confirm_empty: Color,
    /// The opacity of the value hovered (`hover:opacity-50`).
    pub hover_alpha: f32,
    /// The box's least width (`w-30`) and its height (`h-5`).
    pub box_w: f32,
    pub box_h: f32,
}

const CLEAR: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

impl EditTokens {
    /// The kit as written (NewField :41, the input; :56, the value; :48
    /// and :63, the icons).
    pub fn hoff_core() -> Self {
        let gray = Color::hex(0x7B7B7B);
        Self {
            text: TypeRole::Label.run(),
            color: Color::hex(0x121212),
            gap: 8.0,
            icon: 16.0,
            // :63 `fill-secondary` names no color of the kit: the gray of
            // its other secondary icons.
            pencil_color: gray,
            confirm: Color::hex(0x49BA61),
            confirm_empty: gray,
            hover_alpha: 0.50,
            box_w: 120.0,
            box_h: 20.0,
        }
    }

    /// The theme's caption text and its colors; the box as wide as its
    /// text measures, one line tall.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        let text = TypeRole::Caption.strong();
        Self {
            box_h: theme.typography.run(text).line_height,
            text,
            color: c.text,
            gap: theme.spacing.sm,
            icon: theme.control.icon(IconSize::Sm),
            pencil_color: c.text_mid,
            confirm: c.success,
            confirm_empty: c.text_mid,
            hover_alpha: 0.50,
            box_w: 0.0,
        }
    }

    /// The bare box: the field's caret, selection and composition, no
    /// surface, no padding, this text.
    pub fn bare(&self, field: &InputRecipe) -> InputRecipe {
        let none = InputSurface {
            fill: CLEAR,
            border: CLEAR,
            shadow: Vec::new(),
        };
        InputRecipe {
            rest: none.clone(),
            hover: none.clone(),
            focus: none,
            height: self.box_h,
            pad_x: 0.0,
            border_width: 0.0,
            text: self.text,
            text_color: self.color,
            ..field.clone()
        }
    }
}
