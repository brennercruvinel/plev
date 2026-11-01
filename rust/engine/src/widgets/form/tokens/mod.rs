//! Form tokens: what the hoff core kit writes as the tailwind class lists
//! of its Field (`components/Field/index.tsx`), as recipes the form
//! widgets read. The input box is one surface (fill, border, shadow list)
//! at rest, hovered and focused, with its geometry, its text and the
//! colors of the caret, the selection and a composition; the adornments
//! are the icon slots at its two ends; the field's chrome is the label
//! row above it and the message line under it.
//!
//! [`FormTokens::hoff_core`] is the kit as written. Every other theme
//! derives its recipe from the theme it is built into
//! ([`FormTokens::derive`]): the text field each of them drew before the
//! kit, so a theme that is not the core looks as it did. The recipes'
//! shapes are here; the kit's values are in `kit.rs`, every other theme's
//! in `derive.rs`.

mod color;
mod derive;
mod edit;
mod kit;
mod select;
mod upload;

pub use color::ColorTokens;
pub use edit::EditTokens;
pub use select::{PanelBody, SelectLook, SelectPanel, SelectRecipe, SelectSurfaces};
pub use upload::UploadTokens;

use crate::styling::color::Color;
use crate::styling::theme::legibility;
use crate::styling::theme::{ShadowSpec, TypeRole, TypeRun};

/// One state of the input box.
#[derive(Clone, Debug, PartialEq)]
pub struct InputSurface {
    /// Alpha 0 is no fill (the kit's input has none: the card shows).
    pub fill: Color,
    /// The border, drawn inside the box (css `border`, box-sizing
    /// border-box).
    pub border: Color,
    /// The css `box-shadow` list, first entry on top.
    pub shadow: Vec<ShadowSpec>,
}

/// The single-line input box: its surface per state, its geometry, its
/// text and the marks drawn over the text.
#[derive(Clone, Debug, PartialEq)]
pub struct InputRecipe {
    pub rest: InputSurface,
    pub hover: InputSurface,
    /// Focused, hovered or not.
    pub focus: InputSurface,
    /// The box height (`h-12`); the width is always the `Rect`'s.
    pub height: f32,
    /// Inset from either end of the box to the text (`px-5.5`), where no
    /// adornment takes the end.
    pub pad_x: f32,
    pub radius: f32,
    pub border_width: f32,
    /// The one style the text is measured, hit-tested and drawn with.
    pub text: TypeRun,
    pub text_color: Color,
    pub placeholder: Color,
    pub caret: Color,
    /// The selection wash, alpha included.
    pub selection: Color,
    /// The underline of an input method's composition.
    pub composing: Color,
    /// Opacity of the whole field when disabled.
    pub disabled_alpha: f32,
}

impl InputRecipe {
    /// This recipe with `fill` and `border` in every state, its shadows
    /// kept (a css `!bg` and `!border` over the box's classes).
    pub fn pinned(&self, fill: Color, border: Color) -> Self {
        let pin = |s: &InputSurface| InputSurface {
            fill,
            border,
            shadow: s.shadow.clone(),
        };
        Self {
            rest: pin(&self.rest),
            hover: pin(&self.hover),
            focus: pin(&self.focus),
            ..self.clone()
        }
    }
}

/// The icon slots at the ends of the input box: a leading icon, the
/// check of a validated value and the eye that reveals a password.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Adornments {
    /// Width of the leading slot (`w-10`), which is also the text's inset
    /// on that side (`pl-10`).
    pub lead_slot: f32,
    pub lead_icon: f32,
    /// The text's inset on the trailing side when validated (`pr-10`).
    pub check_slot: f32,
    pub check_icon: f32,
    /// The check's inset from the trailing edge (`right-3.5`).
    pub check_inset: f32,
    /// Width of the reveal button (`w-12`), the text's trailing inset on
    /// a password (`pr-12`).
    pub reveal_slot: f32,
    pub reveal_icon: f32,
    pub icon_color: Color,
}

/// A label: the form label that names a control, and a section's
/// caption (the uppercase group heading of the HOFF accordion).
#[derive(Clone, Debug, PartialEq)]
pub struct LabelRecipe {
    pub text: TypeRun,
    pub color: Color,
    pub caption: TypeRun,
    pub caption_color: Color,
    /// Opacity of a disabled label (the control's).
    pub disabled_alpha: f32,
}

/// The field around the box: the label row and the message line.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldChrome {
    /// Between the label row and the box (`mb-2`).
    pub label_gap: f32,
    /// The link at the end of the label row ("Forgot password?").
    pub link: TypeRun,
    pub link_color: Color,
    /// The link's opacity at rest and hovered (`opacity-70`, `hover:opacity-100`).
    pub link_alpha: f32,
    pub link_hover_alpha: f32,
    /// The message line under the box: an error, or a helper.
    pub message: TypeRun,
    /// Between the box and the message (`mt-2`).
    pub message_gap: f32,
    pub error_color: Color,
    pub helper_color: Color,
    /// A field's width where the composition gives it one; the `Rect` it
    /// gets always wins.
    pub width: f32,
}

/// What takes one end of the input box, which sets the text's inset on
/// that end.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Slot {
    /// Nothing: the recipe's padding.
    #[default]
    Pad,
    /// A leading icon.
    Icon,
    /// The check of a validated value.
    Check,
    /// The button that reveals a password.
    Reveal,
    /// An addon of an input group, as wide as it measures.
    Width(f32),
}

/// Which input box a text field draws: the Field's, the search box of an
/// input group, or the inset box an input group with a select draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Look {
    #[default]
    Field,
    Search,
    Inset,
    /// A combobox's filter: the select's control as a text box.
    Combo,
    /// The search of a top bar: the search box on the panel's tone,
    /// edged, whatever the pointer does.
    Header,
    /// An inline edit's box: the text alone ([`EditTokens::bare`]).
    Bare,
}

/// What an input group puts at the ends of its box
/// (`components/Search/index.tsx`): an icon button, a shortcut hint and
/// a text addon (a prefix or a suffix).
#[derive(Clone, Debug, PartialEq)]
pub struct GroupAddons {
    /// The icon button's inset from the box on its three sides
    /// (`top-1 left-1 bottom-1`), its width (`w-8`) and icon (`size-4`).
    pub icon_edge: f32,
    pub icon_slot: f32,
    pub icon: f32,
    /// The icon at rest and with the group hovered (`fill-[#7B7B7B]
    /// group-hover:fill-[#000]`).
    pub icon_color: Color,
    pub icon_hover: Color,
    /// The shortcut hint's inset from the box's end; the hint itself is
    /// the search's key cap, `theme.disclosure.kbd` (`Kbd`, `Md`).
    pub hint_edge: f32,
    /// The least room between an icon or a hint and the text typed
    /// beside it, so a right-to-left text that hangs from the end never
    /// touches the `⌘ K`.
    pub addon_gap: f32,
    /// A text addon's color and its clearance from the box's edge and
    /// from the text typed beside it.
    pub text_color: Color,
    pub text_edge: f32,
    /// The least width of a select at the end (Invite :26 `w-25.5`).
    pub select_w: f32,
    /// A group's width where the composition gives it one: a search, an
    /// invite (a select at its end) and a command palette (the search
    /// over its list). The `Rect` it gets always wins.
    pub width: f32,
    pub invite_width: f32,
    pub palette_width: f32,
    /// The most of the window's height a command palette's dialog takes
    /// (the share of a `cmdk` dialog's `60vh`): its list scrolls past it.
    pub palette_share: f32,
}

/// A one-time code (`super::otp`): its cells are the Field's input box,
/// square, side by side; these are the spaces between them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OtpTokens {
    /// Between two cells of a group.
    pub gap: f32,
    /// The minus between two groups, with a gap on either side of it.
    pub separator: f32,
}

/// A text area (`super::textarea`): the Field's box, taller, its text
/// inset from the top and the bottom as well.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AreaTokens {
    /// Between the text and the box's top and bottom (`py-4`).
    pub pad_y: f32,
    /// The lines the box shows at rest (`h-20`: 80 less the padding is
    /// three 16px lines).
    pub rows: usize,
}

/// The form slot of the theme.
#[derive(Clone, Debug, PartialEq)]
pub struct FormTokens {
    pub label: LabelRecipe,
    pub input: InputRecipe,
    /// The search box of an input group.
    pub search: InputRecipe,
    /// The inset box of an input group with a select at its end.
    pub inset: InputRecipe,
    /// The search box of a top bar ([`Look::Header`]).
    pub header: InputRecipe,
    pub adorn: Adornments,
    pub group: GroupAddons,
    pub field: FieldChrome,
    pub otp: OtpTokens,
    pub area: AreaTokens,
    /// The select and its options panel (`select.rs`).
    pub select: SelectRecipe,
    /// A combobox's filter box, the select's control as a text box
    /// ([`SelectRecipe::as_input`]).
    pub combo: InputRecipe,
    /// The inline edit (`edit.rs`), and its bare box.
    pub edit: EditTokens,
    pub bare: InputRecipe,
    /// The color picker (`color.rs`).
    pub color: ColorTokens,
    /// The avatar upload (`upload.rs`).
    pub upload: UploadTokens,
}

/// A section's caption: the scale's strong body (12/16 semibold in the
/// core), in the theme's own face and tracking.
const CAPTION: TypeRun = TypeRole::BodyStrong.run();

impl FormTokens {
    /// The text's inset from an end of the input box that `slot` takes.
    pub fn inset(&self, slot: Slot) -> f32 {
        match slot {
            Slot::Pad => self.input.pad_x,
            Slot::Icon => self.adorn.lead_slot,
            Slot::Check => self.adorn.check_slot,
            Slot::Reveal => self.adorn.reveal_slot,
            Slot::Width(w) => w,
        }
    }

    /// The fields' edges on `ground` (the page, the card): focused,
    /// strong enough to find the field in use (3:1, WCAG 1.4.11); at rest
    /// the design's, or `edge` where a bar holds the edges (more
    /// contrast); hovered, where the design keeps the rest's, halfway
    /// between the two (the kit's step, #E2E2E2 to #C4C4C4). A border the
    /// design leaves out (transparent) stays out. Holding twice changes
    /// nothing.
    pub fn hold_edges(&mut self, ground: [Color; 2], edge: Option<f32>) {
        let hold = |c: Color, need: f32| {
            if c.0[3] > 0.0 {
                legibility::reach(c, need, &[&ground])
            } else {
                c
            }
        };
        for input in [
            &mut self.input,
            &mut self.search,
            &mut self.inset,
            &mut self.header,
            &mut self.combo,
            &mut self.bare,
        ] {
            input.focus.border = hold(input.focus.border, legibility::MARK_AA);
            if let Some(need) = edge {
                input.rest.border = hold(input.rest.border, need);
                input.hover.border = hold(input.hover.border, need);
            }
            let (rest, focus) = (input.rest.border, input.focus.border);
            if input.hover.border == rest && rest.0[3] > 0.0 && rest != focus {
                input.hover.border = Color(crate::widgets::widget::mix(rest.0, focus.0, 0.5));
            }
        }
    }

    /// The input box of `look`.
    pub fn recipe(&self, look: Look) -> &InputRecipe {
        match look {
            Look::Field => &self.input,
            Look::Search => &self.search,
            Look::Inset => &self.inset,
            Look::Header => &self.header,
            Look::Combo => &self.combo,
            Look::Bare => &self.bare,
        }
    }
}
