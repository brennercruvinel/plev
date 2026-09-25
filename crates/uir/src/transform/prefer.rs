//! the person's preferences of accessibility as operations (task-70
//! phase 4): larger text, more contrast, a density. each one is a list of
//! the operations of this module, by token only, over the surface as it
//! is, each applied and verified like any other; the one refused (the top
//! of a ladder, nothing to move) is left out, and said.
//!
//! - **larger text**: `emphasize` on every text and every stat, one step
//!   up the type ramp (a stat's value one of its sizes, its label with
//!   it). a badge, a button, a field, a slider, a table and an image draw
//!   their text at the component's size: none is reached.
//! - **more contrast**: `restyle` to the strongest ink, `text`, of every
//!   text in a secondary one (`text_dim`, `text_mid`) and of every slider,
//!   table and image, whose text the host draws in a secondary ink unless
//!   its `color` says otherwise; and a badge without a fill filled with
//!   the accent, its text drawn in contrast to it. in the four themes
//!   `text` meets WCAG AA on every surface (the engine's contrast test).
//!   a stat's label and a table's header are drawn by the component in
//!   the theme's own secondary ink, which no role of the IR reaches.
//! - **denser** or **sparser**: `density` once, every gap and padding set
//!   one step of the spacing scale.
//!
//! the IR never knows the theme: the same operations serve every theme.

use std::fmt;

use sem::Actor;
use serde::{Deserialize, Serialize};

use super::tree;
use super::{Adapted, Context, Inverse, Op, Step, TransformError};
use crate::ir::IrNode;
use crate::token::ColorRole;
use crate::vocab::Component;

/// one preference of accessibility, as the history names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preference {
    LargerText,
    MoreContrast,
    Denser,
    Sparser,
}

impl Preference {
    /// every preference, in the order they apply.
    pub const ALL: [Preference; 4] = [
        Preference::LargerText,
        Preference::MoreContrast,
        Preference::Denser,
        Preference::Sparser,
    ];

    /// the preference in words: "larger text".
    pub fn name(self) -> &'static str {
        match self {
            Preference::LargerText => "larger text",
            Preference::MoreContrast => "more contrast",
            Preference::Denser => "denser",
            Preference::Sparser => "sparser",
        }
    }

    /// every operation this preference asks of `surface`, in tree order,
    /// before any is tried.
    pub fn ops(self, surface: &Adapted) -> Vec<Op> {
        let nodes = tree::nodes(&surface.ir().surface.root);
        match self {
            Preference::LargerText => nodes
                .into_iter()
                .filter(|n| matches!(n.component, Some(Component::Text | Component::Stat)))
                .map(|n| Op::Emphasize { id: n.sem.clone() })
                .collect(),
            Preference::MoreContrast => nodes.into_iter().filter_map(contrast).collect(),
            Preference::Denser => vec![Op::Density { step: Step::Denser }],
            Preference::Sparser => vec![Op::Density {
                step: Step::Sparser,
            }],
        }
    }
}

impl fmt::Display for Preference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// the inks a text is secondary in: the ones "more contrast" replaces.
const SECONDARY: [ColorRole; 2] = [ColorRole::TextDim, ColorRole::TextMid];

/// the fill a badge takes for "more contrast": its filled shape in the
/// accent, its text in contrast to it.
const FILLED: ColorRole = ColorRole::Accent;

/// the `restyle` "more contrast" asks of `n`, when it asks one.
fn contrast(n: &IrNode) -> Option<Op> {
    let to = |role| Op::Restyle {
        id: n.sem.clone(),
        role,
    };
    let secondary = |c: Option<ColorRole>| c.is_some_and(|c| SECONDARY.contains(&c));
    match n.component? {
        Component::Text if secondary(n.props.color) => Some(to(ColorRole::Text)),
        // the host's own ink for them, unset, is a secondary one
        Component::Slider | Component::Table | Component::Image
            if n.props.color.is_none() || secondary(n.props.color) =>
        {
            Some(to(ColorRole::Text))
        }
        Component::Badge if n.props.background.is_none() => Some(to(FILLED)),
        _ => None,
    }
}

/// the preferences a person turns on for every surface shown: none by
/// default. the density is one of the two, or neither.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(default)]
    pub larger_text: bool,
    #[serde(default)]
    pub more_contrast: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub density: Option<Step>,
}

impl Preferences {
    pub fn is_empty(&self) -> bool {
        *self == Preferences::default()
    }

    /// true when `p` is on.
    pub fn has(&self, p: Preference) -> bool {
        match p {
            Preference::LargerText => self.larger_text,
            Preference::MoreContrast => self.more_contrast,
            Preference::Denser => self.density == Some(Step::Denser),
            Preference::Sparser => self.density == Some(Step::Sparser),
        }
    }

    /// these preferences with `p` on or off; a density on takes the
    /// place of the other.
    pub fn with(mut self, p: Preference, on: bool) -> Preferences {
        match p {
            Preference::LargerText => self.larger_text = on,
            Preference::MoreContrast => self.more_contrast = on,
            Preference::Denser | Preference::Sparser => {
                let step = if p == Preference::Denser {
                    Step::Denser
                } else {
                    Step::Sparser
                };
                if on {
                    self.density = Some(step);
                } else if self.density == Some(step) {
                    self.density = None;
                }
            }
        }
        self
    }

    /// these preferences with `p` flipped.
    pub fn toggled(self, p: Preference) -> Preferences {
        self.with(p, !self.has(p))
    }

    /// the preferences on, in the order they apply.
    pub fn each(&self) -> Vec<Preference> {
        Preference::ALL
            .into_iter()
            .filter(|&p| self.has(p))
            .collect()
    }

    /// `surface` with these preferences over it, asked by `by` (the person
    /// who turned them on), every operation checked and verified again
    /// under `cx`: the surface they make, what applied and what was
    /// refused. nothing on is `surface` as it is.
    pub fn over(&self, surface: &Adapted, by: &Actor, cx: Context<'_>) -> Preferred {
        let mut out = Preferred {
            adapted: surface.clone(),
            applied: Vec::new(),
            refused: Vec::new(),
        };
        for p in self.each() {
            for op in p.ops(&out.adapted) {
                match out.adapted.apply(&op, by, cx) {
                    Ok((next, inverse)) => {
                        out.adapted = next;
                        out.applied.push(Applied {
                            preference: p,
                            op,
                            inverse,
                        });
                    }
                    Err(why) => out.refused.push((p, op, why)),
                }
            }
        }
        out
    }
}

/// one operation a preference made, with its inverse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub preference: Preference,
    pub op: Op,
    pub inverse: Inverse,
}

/// a surface with preferences over it: the verified surface they make,
/// every operation that applied, in order, and every one refused, with
/// why.
#[derive(Clone, Debug)]
pub struct Preferred {
    pub adapted: Adapted,
    pub applied: Vec<Applied>,
    pub refused: Vec<(Preference, Op, TransformError)>,
}
